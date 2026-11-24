use std::sync::Arc;
use std::time::Duration;

use bdl_core::fetcher::{BandwidthLimiter, FetchCancelToken, FetchConfig, ReqwestFetcher};
use bdl_core::queue::{
    DownloadResourceIntent, DownloadTask, QueueLogLevel, ResourceStatus, TaskStatus,
};
use futures::stream::{FuturesUnordered, StreamExt};
use tauri::{AppHandle, Manager};
use tokio::time::Instant;

use crate::commands::{
    CommandResult, DownloadRuntimeOptions, concurrent_tasks, emit_queue_log,
    publish_download_result, queue_task_launch_context, run_download_task,
};
use crate::events;
use crate::state::{AppState, SettingsSnapshot};
use crate::task_failure::{recent_auto_refresh_attempt_count, should_refresh_media_url};

const MAX_AUTO_REFRESH_ATTEMPTS: usize = 3;

pub(crate) fn start(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        state.notify_queue_changed();
        if !state.try_start_queue_worker() {
            return;
        }

        let result = run(&app, state.inner()).await;
        if let Err(error) = state.sync_download_activity() {
            tracing::warn!("failed to stop platform task execution: {error}");
            crate::commands::report_runtime_error(
                &app,
                &state,
                "stop_task_execution",
                error.into(),
            )
            .await;
        }
        state.finish_queue_worker();

        if let Err(error) = result {
            tracing::error!("queue worker failed: {}", error.diagnostic_message());
            crate::commands::report_runtime_error(&app, &state, "queue_worker", error).await;
        }

        let should_restart = matches!(state.has_startable_task(), Ok(true))
            || matches!(state.next_scheduled_at(), Ok(Some(_)));
        if should_restart {
            start(&app);
        }
    });
}

async fn run(app: &AppHandle, state: &AppState) -> CommandResult<()> {
    let global_limiter = state.global_limiter();
    let mut running = FuturesUnordered::new();
    let mut execution_active = false;

    let result = async {
        loop {
            let current_settings = state.settings()?;
            let concurrent_tasks = concurrent_tasks(&current_settings);
            let mut start_wakeup = state
                .queue_start_deadline()?
                .filter(|deadline| *deadline > Instant::now());
            if running.len() < concurrent_tasks
                && start_wakeup.is_none()
                && let Some(task) = state.take_next_startable_task()?
            {
                start_wakeup = state.queue_start_deadline()?;
                let attempt = AttemptGuard {
                    state,
                    task_id: task.id.clone(),
                };
                if !execution_active {
                    match state.task_execution().set_active(true) {
                        Ok(()) => execution_active = true,
                        Err(error) => {
                            tracing::warn!("failed to activate platform task execution: {error}");
                            crate::commands::report_runtime_error(
                                app,
                                state,
                                "start_task_execution",
                                error.into(),
                            )
                            .await;
                        }
                    }
                }
                let launch =
                    queue_task_launch_context(state.settings()?, task.speed_limit_bytes_per_second);

                events::emit(app, events::QUEUE_TASK_UPDATED, &task)?;
                let message = if task.status == TaskStatus::Parsing {
                    "正在获取下载地址"
                } else {
                    "开始下载任务"
                };
                emit_queue_log(app, state, &task.id, QueueLogLevel::Info, message)?;
                running.push(run_download_attempt(
                    app,
                    state,
                    global_limiter.clone(),
                    launch.fetch_config,
                    (task, attempt),
                    launch.runtime_options,
                    launch.settings,
                ));
            }

            let scheduled_wakeups = state.scheduled_wakeups()?;
            let schedule_wakeup = scheduled_wakeups.first().map(|scheduled_at| {
                let delay = (*scheduled_at - chrono::Utc::now())
                    .to_std()
                    .unwrap_or(Duration::ZERO);
                Instant::now() + delay
            });
            if start_wakeup.is_some()
                && (running.len() >= concurrent_tasks || !state.has_startable_task()?)
            {
                start_wakeup = None;
            }
            let next_wakeup = [schedule_wakeup, start_wakeup].into_iter().flatten().min();
            let scheduled_wakeup_millis = scheduled_wakeups
                .iter()
                .map(|scheduled_at| scheduled_at.timestamp_millis())
                .collect::<Vec<_>>();
            if let Err(error) = state
                .task_execution()
                .sync_scheduled_wakeups(&scheduled_wakeup_millis)
            {
                tracing::warn!("failed to sync platform scheduled wakeups: {error}");
                crate::commands::report_runtime_error(
                    app,
                    state,
                    "sync_scheduled_wakeups",
                    error.into(),
                )
                .await;
            }

            if running.is_empty() {
                if execution_active {
                    match state.sync_download_activity() {
                        Ok(active) => execution_active = active,
                        Err(error) => {
                            tracing::warn!("failed to stop platform task execution: {error}");
                            crate::commands::report_runtime_error(
                                app,
                                state,
                                "stop_task_execution",
                                error.into(),
                            )
                            .await;
                        }
                    }
                }
                let Some(wakeup) = next_wakeup else {
                    if state
                        .file_cleanup_active
                        .load(std::sync::atomic::Ordering::SeqCst)
                    {
                        wait_for_queue_wakeup(state, None).await;
                        continue;
                    }
                    break;
                };
                wait_for_queue_wakeup(state, Some(wakeup)).await;
                continue;
            }

            let completed = tokio::select! {
                completed = running.next() => completed,
                () = wait_for_queue_wakeup(state, next_wakeup) => None,
            };

            if let Some(outcome) = completed {
                outcome?;
            }
        }

        Ok(())
    }
    .await;
    if result.is_err() {
        // Native exports and muxers may keep writing after their awaiter is
        // dropped. Stop new pickups, then drain all attempts before exit.
        if let Ok(tasks) = state.queue_snapshot() {
            for task in tasks {
                if matches!(state.cancel_running_task(&task.id), Ok(true))
                    || task.status == TaskStatus::Downloading
                {
                    let _ = state.update_task_status(&task.id, TaskStatus::Paused);
                }
            }
        }
        while running.next().await.is_some() {}
    }
    result
}

async fn wait_for_queue_wakeup(state: &AppState, deadline: Option<Instant>) {
    if let Some(deadline) = deadline {
        tokio::select! {
            () = tokio::time::sleep_until(deadline) => {}
            () = state.wait_for_queue_change() => {}
        }
    } else {
        state.wait_for_queue_change().await;
    }
}

async fn run_download_attempt(
    app: &AppHandle,
    state: &AppState,
    global_limiter: Arc<BandwidthLimiter>,
    fetch_config: FetchConfig,
    picked: (DownloadTask, AttemptGuard<'_>),
    runtime_options: DownloadRuntimeOptions,
    settings: SettingsSnapshot,
) -> CommandResult<()> {
    let (task, _attempt) = picked;
    let cancel_token = state.register_task_cancel_token(&task.id)?;
    // Pause/remove can arrive after pickup, before this future is first polled.
    if !matches!(
        state.task_status(&task.id),
        Ok(TaskStatus::Parsing | TaskStatus::Downloading)
    ) {
        return Ok(());
    }
    let outcome = match ReqwestFetcher::with_global_limiter(fetch_config, global_limiter) {
        Ok(fetcher) => {
            async {
                let mut current = task.clone();
                loop {
                    if current.media_selection.deferred.is_some() {
                        let Some(resolved) = cancellable_refresh(
                            &cancel_token,
                            crate::preparation::materialize_queued_download(
                                state,
                                &current,
                                &cancel_token,
                            ),
                        )
                        .await
                        else {
                            return Ok(());
                        };
                        let Some(resolved) = resolved? else {
                            return Ok(());
                        };
                        publish_download_result(app, state, &resolved, "下载完成")?;
                        match resolved.status {
                            TaskStatus::Parsing => {
                                current = resolved;
                                continue;
                            }
                            TaskStatus::Cancelled => {
                                emit_queue_log(
                                    app,
                                    state,
                                    &task.id,
                                    QueueLogLevel::Info,
                                    "已跳过已有文件",
                                )?;
                                return Ok(());
                            }
                            TaskStatus::Downloading => {}
                            _ => return Ok(()),
                        }
                        emit_queue_log(app, state, &task.id, QueueLogLevel::Info, "开始下载任务")?;
                        current = resolved;
                    }
                    if current.media_selection.post.is_some()
                        && current.resources.iter().any(|resource| {
                            matches!(
                                resource.intent,
                                DownloadResourceIntent::Video | DownloadResourceIntent::Audio
                            ) && resource.status != ResourceStatus::Completed
                                && resource.current_urls.is_empty()
                        })
                    {
                        let Some(refreshed) = cancellable_refresh(
                            &cancel_token,
                            state.refresh_task_media_urls(
                                &current.id,
                                crate::parse_pacing::CooldownBehavior::Wait,
                            ),
                        )
                        .await
                        else {
                            return Ok(());
                        };
                        current = refreshed?;
                    }
                    run_download_task(
                        app,
                        state,
                        &fetcher,
                        current,
                        runtime_options.clone(),
                        cancel_token.clone(),
                    )
                    .await?;
                    current = state.task_snapshot(&task.id)?;
                    if !matches!(
                        current.status,
                        TaskStatus::Parsing | TaskStatus::Downloading
                    ) || cancel_token.is_cancelled()
                    {
                        return Ok(());
                    }
                }
            }
            .await
        }
        Err(error) => Err(error.into()),
    };
    // Refresh is part of this task's future: the scheduler keeps polling other
    // downloads while it waits for the parser, and the same token remains live.
    handle_download_outcome(app, state, &settings, &task, outcome, &cancel_token).await
}

// Created at pickup, before the future is polled. Dropping a queued or failed
// future must release its cancellation slot as well as a normal completion.
struct AttemptGuard<'a> {
    state: &'a AppState,
    task_id: String,
}

impl Drop for AttemptGuard<'_> {
    fn drop(&mut self) {
        let _ = self.state.clear_task_cancel_token(&self.task_id);
    }
}

async fn handle_download_outcome(
    app: &AppHandle,
    state: &AppState,
    settings: &SettingsSnapshot,
    task: &DownloadTask,
    outcome: CommandResult<()>,
    cancel_token: &FetchCancelToken,
) -> CommandResult<()> {
    let Err(mut error) = outcome else {
        return Ok(());
    };
    if cancel_token.is_cancelled() && !matches!(state.task_status(&task.id), Ok(TaskStatus::Muxing))
    {
        return Ok(());
    }
    match state.task_status(&task.id) {
        Ok(TaskStatus::Paused | TaskStatus::Cancelled) => {
            emit_queue_log(app, state, &task.id, QueueLogLevel::Warning, "任务已停止")?;
        }
        Ok(TaskStatus::Completed) => {
            let completed = state.task_snapshot(&task.id)?;
            events::emit(app, events::QUEUE_TASK_UPDATED, &completed)?;
            emit_queue_log(
                app,
                state,
                &task.id,
                QueueLogLevel::Warning,
                &format!(
                    "成品已生成，但收尾处理未完全完成：{}",
                    error.diagnostic_message()
                ),
            )?;
        }
        _ => {
            let auto_refresh_attempts = state
                .task_logs(&task.id, 50)
                .map(|logs| recent_auto_refresh_attempt_count(&logs))
                .unwrap_or_default();
            if settings.auto_refresh_expired_urls
                && matches!(state.task_status(&task.id), Ok(TaskStatus::Downloading))
                && should_refresh_media_url(&error.diagnostic_message())
                && auto_refresh_attempts < MAX_AUTO_REFRESH_ATTEMPTS
                && state.task_snapshot(&task.id)?.can_resolve_media_urls()
            {
                let attempt = auto_refresh_attempts + 1;
                emit_queue_log(
                    app,
                    state,
                    &task.id,
                    QueueLogLevel::Warning,
                    &format!("自动刷新过期链接（{attempt}/{MAX_AUTO_REFRESH_ATTEMPTS}）"),
                )?;

                let Some(refreshed) = cancellable_refresh(
                    cancel_token,
                    state.refresh_task_media_urls(
                        &task.id,
                        crate::parse_pacing::CooldownBehavior::Wait,
                    ),
                )
                .await
                else {
                    return Ok(());
                };
                match refreshed {
                    Ok(_) => {
                        if let Some(retried) = state.finish_task_attempt(
                            &task.id,
                            TaskStatus::Waiting,
                            cancel_token,
                        )? {
                            events::emit(app, events::QUEUE_TASK_UPDATED, &retried)?;
                        }
                        return Ok(());
                    }
                    Err(refresh_error) => {
                        let failure = crate::commands::CommandError::from(refresh_error);
                        emit_queue_log(
                            app,
                            state,
                            &task.id,
                            QueueLogLevel::Error,
                            &format!(
                                "[stage=refresh_urls code={}] 自动刷新过期链接失败：{}",
                                failure.code,
                                failure.diagnostic_message()
                            ),
                        )?;
                        error = failure;
                    }
                }
            }

            if let Some(failed) =
                state.finish_task_attempt(&task.id, TaskStatus::Failed, cancel_token)?
            {
                events::emit(app, events::QUEUE_TASK_UPDATED, &failed)?;
                let message = if error.diagnostic_message().starts_with("[stage=") {
                    error.diagnostic_message()
                } else {
                    format!(
                        "[stage=task code={}] {}",
                        error.code,
                        error.diagnostic_message()
                    )
                };
                emit_queue_log(app, state, &task.id, QueueLogLevel::Error, &message)?;
            }
        }
    }
    Ok(())
}

async fn cancellable_refresh<T>(
    cancel_token: &FetchCancelToken,
    refresh: impl std::future::Future<Output = T>,
) -> Option<T> {
    tokio::select! {
        biased;
        () = cancel_token.cancelled() => None,
        result = refresh => Some(result),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_pacing::ParsePacer;
    use bdl_core::settings::ParseRules;
    use futures::FutureExt;

    #[test]
    fn dropping_an_unpolled_attempt_releases_its_pickup_slot() {
        let root = std::env::temp_dir().join(format!("bdl-attempt-{}", uuid::Uuid::new_v4()));
        drop(bdl_core::storage::TaskStorage::open(root.join("state/tasks.sqlite")).unwrap());
        let state = AppState::new(root.join("state"), root.join("downloads")).unwrap();
        let token = state.register_task_cancel_token("picked").unwrap();
        state.cancel_running_task("picked").unwrap();
        assert!(token.is_cancelled());
        // Re-registration at first poll must not replace the cancelled token.
        assert!(
            state
                .register_task_cancel_token("picked")
                .unwrap()
                .is_cancelled()
        );
        let guard = AttemptGuard {
            state: &state,
            task_id: "picked".into(),
        };
        let unpolled = async move {
            let _attempt = guard;
        };
        drop(unpolled);
        assert!(!state.cancel_running_task("picked").unwrap());
        drop(state);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[tokio::test(start_paused = true)]
    async fn refresh_cooldown_does_not_block_sibling_and_can_be_cancelled() {
        let pacer = ParsePacer::default();
        let rules = ParseRules {
            pages_per_round: 1,
            interval_seconds: 10,
            rest_seconds: 10,
        };
        pacer.run(rules, async { Ok(()) }, |_| 1).await.unwrap();
        let token = FetchCancelToken::new();
        let mut running = FuturesUnordered::new();
        let started = Instant::now();
        running.push(
            async {
                cancellable_refresh(
                    &token,
                    pacer.run(
                        rules,
                        async {
                            panic!("cancelled refresh must not send a request");
                            #[allow(unreachable_code)]
                            Ok(())
                        },
                        |_| 0,
                    ),
                )
                .await
                .map(|result| result.unwrap())
            }
            .boxed(),
        );
        running.push(
            async {
                tokio::time::sleep(Duration::from_secs(1)).await;
                Some(())
            }
            .boxed(),
        );
        assert_eq!(running.next().await, Some(Some(())));
        assert_eq!(started.elapsed(), Duration::from_secs(1));
        token.cancel();
        assert_eq!(running.next().await, Some(None));
        assert_eq!(started.elapsed(), Duration::from_secs(1));
        // Cancellation drops the resolver future and releases the shared gate.
        pacer.run(rules, async { Ok(()) }, |_| 0).await.unwrap();
        assert_eq!(started.elapsed(), Duration::from_secs(10));
    }

    #[tokio::test]
    async fn cancelled_refresh_wins_over_ready_result() {
        let token = FetchCancelToken::new();
        token.cancel();
        assert_eq!(cancellable_refresh(&token, async { 42 }).await, None);
    }
}
