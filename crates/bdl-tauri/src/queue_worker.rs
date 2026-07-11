use std::sync::Arc;

use bdl_core::fetcher::{BandwidthLimiter, FetchConfig, ReqwestFetcher};
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

    loop {
        let current_settings = state.settings()?;
        global_limiter.set_limit(current_settings.global_speed_limit_bytes_per_second);
        let concurrent_tasks = concurrent_tasks(&current_settings);
        while running.len() < concurrent_tasks {
            let Some(task) = state.take_next_startable_task()? else {
                break;
            };
            let launch =
                queue_task_launch_context(state.settings()?, task.speed_limit_bytes_per_second);

            events::emit(app, events::QUEUE_TASK_UPDATED, &task)?;
            emit_queue_log(app, state, &task.id, QueueLogLevel::Info, "开始下载任务")?;
            running.push(run_download_task_with_identity(
                app,
                state,
                global_limiter.clone(),
                launch.fetch_config,
                task,
                launch.runtime_options,
                launch.settings,
            ));
        }

        if running.is_empty() {
            let Some(scheduled_at) = state.next_scheduled_at()? else {
                break;
            };
            wait_for_schedule_or_queue_change(state, scheduled_at).await;
            continue;
        }

        let completed = if let Some(scheduled_at) = state.next_scheduled_at()? {
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

        if let Some((task, settings, outcome)) = completed {
            handle_download_outcome(app, state, &settings, &task, outcome).await?;
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

async fn run_download_task_with_identity(
    app: &AppHandle,
    state: &AppState,
    global_limiter: Arc<BandwidthLimiter>,
    fetch_config: FetchConfig,
    task: DownloadTask,
    runtime_options: DownloadRuntimeOptions,
    settings: SettingsSnapshot,
) -> (DownloadTask, SettingsSnapshot, CommandResult<()>) {
    let outcome = match ReqwestFetcher::with_global_limiter(fetch_config, global_limiter) {
        Ok(fetcher) => run_download_task(app, state, &fetcher, task.clone(), runtime_options).await,
        Err(error) => Err(error.into()),
    };
    (task, settings, outcome)
}

async fn handle_download_outcome(
    app: &AppHandle,
    state: &AppState,
    settings: &SettingsSnapshot,
    task: &DownloadTask,
    outcome: CommandResult<()>,
) -> CommandResult<()> {
    let Err(error) = outcome else {
        return Ok(());
    };
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
                && is_expired_url_error(&error.message)
                && !already_refreshed
            {
                emit_queue_log(
                    app,
                    state,
                    &task.id,
                    QueueLogLevel::Warning,
                    "自动刷新过期链接",
                )?;

                match state.refresh_task_media_urls(&task.id).await {
                    Ok(_) => {
                        let retried = state.retry_task(&task.id)?;
                        events::emit(app, events::QUEUE_TASK_UPDATED, &retried)?;
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

            let failed = state.update_task_status(&task.id, TaskStatus::Failed)?;
            events::emit(app, events::QUEUE_TASK_UPDATED, &failed)?;
            emit_queue_log(app, state, &task.id, QueueLogLevel::Error, &error.message)?;
        }
    }
    Ok(())
}
