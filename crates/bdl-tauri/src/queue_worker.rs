use std::sync::Arc;

use bdl_core::fetcher::{BandwidthLimiter, FetchCancelToken, FetchConfig, ReqwestFetcher};
use bdl_core::queue::{DownloadTask, QueueLogLevel, TaskStatus};
use futures::stream::{FuturesUnordered, StreamExt};
use tauri::{AppHandle, Manager};

use crate::commands::{
    CommandResult, DownloadRuntimeOptions, concurrent_tasks, emit_queue_log,
    queue_task_launch_context, run_download_task,
};
use crate::events;
use crate::state::{AppState, SettingsSnapshot};
use crate::task_failure::{has_auto_refresh_attempt, is_expired_url_error};

pub(crate) fn start(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        state.notify_queue_changed();
        if !state.try_start_queue_worker() {
            return;
        }

        let result = run(&app, state.inner()).await;
        if let Err(error) = state.task_execution().set_active(false) {
            tracing::warn!("failed to stop platform task execution: {error}");
        }
        state.finish_queue_worker();

        if let Err(error) = result {
            tracing::error!("queue worker failed: {}", error.message);
        }

        let should_restart = matches!(state.has_startable_task(), Ok(true))
            || matches!(state.next_scheduled_at(), Ok(Some(_)));
        if should_restart {
            start(&app);
        }
    });
}

async fn run(app: &AppHandle, state: &AppState) -> CommandResult<()> {
    let global_limiter = Arc::new(BandwidthLimiter::new(
        state.settings()?.global_speed_limit_bytes_per_second,
    ));
    let mut running = FuturesUnordered::new();
    let mut execution_active = false;

    loop {
        let current_settings = state.settings()?;
        global_limiter.set_limit(current_settings.global_speed_limit_bytes_per_second);
        let concurrent_tasks = concurrent_tasks(&current_settings);
        while running.len() < concurrent_tasks {
            let Some(task) = state.take_next_startable_task()? else {
                break;
            };
            if !execution_active {
                match state.task_execution().set_active(true) {
                    Ok(()) => execution_active = true,
                    Err(error) => {
                        tracing::warn!("failed to activate platform task execution: {error}");
                    }
                }
            }
            let launch =
                queue_task_launch_context(state.settings()?, task.speed_limit_bytes_per_second);

            events::emit(app, events::QUEUE_TASK_UPDATED, &task)?;
            emit_queue_log(app, state, &task.id, QueueLogLevel::Info, "开始下载任务")?;
            running.push(run_download_attempt(
                app,
                state,
                global_limiter.clone(),
                launch.fetch_config,
                task,
                launch.runtime_options,
                launch.settings,
            ));
        }

        let scheduled_wakeups = state.scheduled_wakeups()?;
        let next_scheduled_at = scheduled_wakeups.first().copied();
        let scheduled_wakeup_millis = scheduled_wakeups
            .iter()
            .map(|scheduled_at| scheduled_at.timestamp_millis())
            .collect::<Vec<_>>();
        if let Err(error) = state
            .task_execution()
            .sync_scheduled_wakeups(&scheduled_wakeup_millis)
        {
            tracing::warn!("failed to sync platform scheduled wakeups: {error}");
        }

        if running.is_empty() {
            if execution_active {
                if let Err(error) = state.task_execution().set_active(false) {
                    tracing::warn!("failed to stop platform task execution: {error}");
                } else {
                    execution_active = false;
                }
            }
            let Some(scheduled_at) = next_scheduled_at else {
                break;
            };
            wait_for_schedule_or_queue_change(state, scheduled_at).await;
            continue;
        }

        let completed = if let Some(scheduled_at) = next_scheduled_at {
            tokio::select! {
                completed = running.next() => completed,
                () = wait_for_schedule_or_queue_change(state, scheduled_at) => None,
            }
        } else {
            tokio::select! {
                completed = running.next() => completed,
                () = state.wait_for_queue_change() => None,
            }
        };

        if let Some(outcome) = completed {
            outcome?;
        }
    }

    Ok(())
}

async fn wait_for_schedule_or_queue_change(
    state: &AppState,
    scheduled_at: chrono::DateTime<chrono::Utc>,
) {
    let delay = (scheduled_at - chrono::Utc::now())
        .to_std()
        .unwrap_or(std::time::Duration::ZERO);
    tokio::select! {
        () = tokio::time::sleep(delay) => {}
        () = state.wait_for_queue_change() => {}
    }
}

async fn run_download_attempt(
    app: &AppHandle,
    state: &AppState,
    global_limiter: Arc<BandwidthLimiter>,
    fetch_config: FetchConfig,
    task: DownloadTask,
    runtime_options: DownloadRuntimeOptions,
    settings: SettingsSnapshot,
) -> CommandResult<()> {
    let cancel_token = state.register_task_cancel_token(&task.id)?;
    // Pause/remove can arrive after pickup, before this future is first polled.
    if !matches!(state.task_status(&task.id), Ok(TaskStatus::Downloading)) {
        state.clear_task_cancel_token(&task.id)?;
        return Ok(());
    }
    let outcome = match ReqwestFetcher::with_global_limiter(fetch_config, global_limiter) {
        Ok(fetcher) => {
            run_download_task(
                app,
                state,
                &fetcher,
                task.clone(),
                runtime_options,
                cancel_token.clone(),
            )
            .await
        }
        Err(error) => Err(error.into()),
    };
    // Refresh is part of this task's future: the scheduler keeps polling other
    // downloads while it waits for the parser, and the same token remains live.
    let result =
        handle_download_outcome(app, state, &settings, &task, outcome, &cancel_token).await;
    state.clear_task_cancel_token(&task.id)?;
    result
}

async fn handle_download_outcome(
    app: &AppHandle,
    state: &AppState,
    settings: &SettingsSnapshot,
    task: &DownloadTask,
    outcome: CommandResult<()>,
    cancel_token: &FetchCancelToken,
) -> CommandResult<()> {
    let Err(error) = outcome else {
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
                &format!("成品已生成，但收尾处理未完全完成：{}", error.message),
            )?;
        }
        _ => {
            let already_refreshed = state
                .task_logs(&task.id, 50)
                .map(|logs| has_auto_refresh_attempt(&logs))
                .unwrap_or(false);
            if settings.auto_refresh_expired_urls
                && matches!(state.task_status(&task.id), Ok(TaskStatus::Downloading))
                && is_expired_url_error(&error.message)
                && !already_refreshed
            {
                emit_queue_log(
                    app,
                    state,
                    &task.id,
                    QueueLogLevel::Warning,
                    "正在刷新过期链接",
                )?;

                let Some(refreshed) =
                    cancellable_refresh(cancel_token, state.refresh_task_media_urls(&task.id))
                        .await
                else {
                    return Ok(());
                };
                // A cancelled wait does not consume the automatic refresh attempt.
                emit_queue_log(
                    app,
                    state,
                    &task.id,
                    QueueLogLevel::Warning,
                    "自动刷新过期链接",
                )?;
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
                        emit_queue_log(
                            app,
                            state,
                            &task.id,
                            QueueLogLevel::Error,
                            &format!("自动刷新过期链接失败：{refresh_error}"),
                        )?;
                    }
                }
            }

            if let Some(failed) =
                state.finish_task_attempt(&task.id, TaskStatus::Failed, cancel_token)?
            {
                events::emit(app, events::QUEUE_TASK_UPDATED, &failed)?;
                emit_queue_log(app, state, &task.id, QueueLogLevel::Error, &error.message)?;
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
    use std::time::Duration;
    use tokio::time::Instant;

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
