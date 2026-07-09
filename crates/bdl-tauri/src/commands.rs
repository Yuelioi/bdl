use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use bdl_core::account::{QrLoginSession, QrLoginStatus, poll_qr_login, start_qr_login};
use bdl_core::fetcher::{FetchConfig, Fetcher, ReqwestFetcher, state_path_for};
use bdl_core::ids::{PartId, SourceId};
use bdl_core::model::NormalizedSourceTree;
use bdl_core::muxer::{
    MediaMuxer, MediaMuxerConfig, MuxRequest, supports_cover_embedding, supports_subtitle_embedding,
};
use bdl_core::planner::{
    ArchiveMode, DownloadOptions, MissingQualityPolicy, StreamPreference, parse_stream_codec,
    plan_selected_parts,
};
use bdl_core::queue::{
    DownloadResource, DownloadResourceIntent, DownloadTask, QueueLogEntry, QueueLogLevel,
    ResourceStatus, TaskStatus,
};
use bdl_core::{BdlError, BdlResult};
use chrono::Utc;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;
use tokio::fs;

use crate::events;
use crate::state::{AccountSnapshot, AppState, SettingsSnapshot};

pub type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl From<BdlError> for CommandError {
    fn from(error: BdlError) -> Self {
        Self {
            code: "core_error".to_owned(),
            message: error.to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParseCreateSourceRequest {
    pub input: String,
    #[serde(default)]
    pub fetch_streams: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParseCloseSourceResponse {
    pub removed: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueueRemoveResponse {
    pub removed: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulkQueueRequest {
    pub task_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BulkQueueFailure {
    pub task_id: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct BulkQueueResult {
    pub updated: Vec<DownloadTask>,
    pub removed: Vec<String>,
    pub failed: Vec<BulkQueueFailure>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParseSourcePageRequest {
    pub source_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParseLoadAllRequest {
    pub source_id: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SelectionCreateTasksRequest {
    pub source_id: String,
    pub part_ids: Vec<String>,
    pub output_dir: Option<String>,
    pub archive_mode: Option<String>,
    pub output_extension: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountImportCookieRequest {
    pub cookie: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountLoginQrPollRequest {
    pub qrcode_key: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccountLoginQrPollResponse {
    pub status: QrLoginStatus,
    pub message: String,
    pub account: Option<AccountSnapshot>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MaintenanceResult {
    pub removed_files: usize,
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticsExportResponse {
    pub path: String,
}

#[derive(Debug, Clone)]
struct DownloadRuntimeOptions {
    ffmpeg_path: Option<PathBuf>,
    retain_raw_streams: bool,
    embed_cover: bool,
    embed_subtitles: bool,
}

impl From<&SettingsSnapshot> for DownloadRuntimeOptions {
    fn from(settings: &SettingsSnapshot) -> Self {
        Self {
            ffmpeg_path: settings.ffmpeg_path.as_deref().map(PathBuf::from),
            retain_raw_streams: settings.retain_raw_streams,
            embed_cover: settings.embed_cover,
            embed_subtitles: settings.embed_subtitles,
        }
    }
}

#[tauri::command]
pub async fn parse_create_source(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ParseCreateSourceRequest,
) -> CommandResult<NormalizedSourceTree> {
    let tree = state
        .parse_source(&request.input, request.fetch_streams)
        .await?;
    events::emit(&app, events::PARSE_SOURCE_UPDATED, &tree)?;
    Ok(tree)
}

#[tauri::command]
pub async fn parse_load_more(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ParseSourcePageRequest,
) -> CommandResult<NormalizedSourceTree> {
    let tree = state.load_more(&SourceId(request.source_id)).await?;
    events::emit(&app, events::PARSE_SOURCE_UPDATED, &tree)?;
    Ok(tree)
}

#[tauri::command]
pub async fn parse_load_all(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ParseLoadAllRequest,
) -> CommandResult<NormalizedSourceTree> {
    let tree = state
        .load_all(&SourceId(request.source_id), request.limit)
        .await?;
    events::emit(&app, events::PARSE_SOURCE_UPDATED, &tree)?;
    Ok(tree)
}

#[tauri::command]
pub fn parse_close_source(
    state: State<'_, AppState>,
    source_id: String,
) -> CommandResult<ParseCloseSourceResponse> {
    let removed = state.close_source(&SourceId(source_id))?;
    Ok(ParseCloseSourceResponse { removed })
}

#[tauri::command]
pub async fn parse_refresh_source(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ParseSourcePageRequest,
) -> CommandResult<NormalizedSourceTree> {
    let tree = state.refresh_source(&SourceId(request.source_id)).await?;
    events::emit(&app, events::PARSE_SOURCE_UPDATED, &tree)?;
    Ok(tree)
}

#[tauri::command]
pub async fn selection_create_tasks(
    app: AppHandle,
    state: State<'_, AppState>,
    request: SelectionCreateTasksRequest,
) -> CommandResult<Vec<DownloadTask>> {
    let source_id = SourceId(request.source_id);
    let settings = state.settings()?;
    let selected_part_ids = request.part_ids.into_iter().map(PartId).collect::<Vec<_>>();

    let output_dir = request
        .output_dir
        .or(settings.download_dir)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("downloads"));

    let archive_mode = parse_archive_mode(
        request
            .archive_mode
            .as_deref()
            .unwrap_or(&settings.archive_mode),
    )?;

    let mut options = DownloadOptions::new(output_dir).with_archive_mode(archive_mode);
    options.output_extension = request
        .output_extension
        .unwrap_or(settings.output_extension);
    options.naming_template = settings.naming_template;
    options.duplicate_naming_strategy = settings.duplicate_naming_strategy;
    options.video_quality = StreamPreference::parse(&settings.quality, "视频清晰度")?;
    options.audio_quality = StreamPreference::parse(&settings.audio_quality, "音频质量")?;
    options.video_codec = parse_stream_codec(&settings.codec)?;
    options.missing_quality_policy = MissingQualityPolicy::parse(&settings.missing_quality_policy)?;
    options.archive_assets = settings.archive_assets;

    let prepared = state
        .prepare_selection(&source_id, &selected_part_ids)
        .await?;
    let planned_tasks = plan_selected_parts(&prepared.tree, &prepared.part_ids, &options)?;
    let tasks = state.enqueue_tasks(planned_tasks)?;

    if prepared.tree_updated {
        events::emit(&app, events::PARSE_SOURCE_UPDATED, &prepared.tree)?;
    }

    for task in &tasks {
        events::emit(&app, events::QUEUE_TASK_UPDATED, task)?;
    }
    if !tasks.is_empty() {
        start_queue_worker(&app);
    }

    Ok(tasks)
}

#[tauri::command]
pub fn queue_list(app: AppHandle, state: State<'_, AppState>) -> CommandResult<Vec<DownloadTask>> {
    let tasks = state.queue_snapshot()?;
    start_queue_worker(&app);
    Ok(tasks)
}

#[tauri::command]
pub fn queue_logs(
    state: State<'_, AppState>,
    task_id: String,
    limit: Option<usize>,
) -> CommandResult<Vec<QueueLogEntry>> {
    let limit = limit.unwrap_or(200).clamp(1, 500);
    Ok(state.task_logs(&task_id, limit)?)
}

#[tauri::command]
pub fn queue_pause(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: String,
) -> CommandResult<DownloadTask> {
    update_task_status(app, state, &task_id, TaskStatus::Paused)
}

#[tauri::command]
pub fn queue_resume(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: String,
) -> CommandResult<DownloadTask> {
    update_task_status(app, state, &task_id, TaskStatus::Waiting)
}

#[tauri::command]
pub fn queue_cancel(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: String,
) -> CommandResult<DownloadTask> {
    update_task_status(app, state, &task_id, TaskStatus::Cancelled)
}

#[tauri::command]
pub fn queue_retry(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: String,
) -> CommandResult<DownloadTask> {
    retry_task(app, state.inner(), &task_id)
}

#[tauri::command]
pub async fn queue_refresh_urls_and_retry(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: String,
) -> CommandResult<DownloadTask> {
    refresh_urls_and_retry(app, state.inner(), &task_id).await
}

#[tauri::command]
pub fn queue_bulk_pause(
    app: AppHandle,
    state: State<'_, AppState>,
    request: BulkQueueRequest,
) -> CommandResult<BulkQueueResult> {
    Ok(bulk_update_task_status(
        &app,
        state.inner(),
        request.task_ids,
        TaskStatus::Paused,
    ))
}

#[tauri::command]
pub fn queue_bulk_resume(
    app: AppHandle,
    state: State<'_, AppState>,
    request: BulkQueueRequest,
) -> CommandResult<BulkQueueResult> {
    let result =
        bulk_update_task_status(&app, state.inner(), request.task_ids, TaskStatus::Waiting);
    if !result.updated.is_empty() {
        start_queue_worker(&app);
    }

    Ok(result)
}

#[tauri::command]
pub fn queue_bulk_retry(
    app: AppHandle,
    state: State<'_, AppState>,
    request: BulkQueueRequest,
) -> CommandResult<BulkQueueResult> {
    let mut result = BulkQueueResult::default();

    for task_id in request.task_ids {
        match retry_task(app.clone(), state.inner(), &task_id) {
            Ok(task) => result.updated.push(task),
            Err(error) => result.failed.push(BulkQueueFailure {
                task_id,
                message: error.message,
            }),
        }
    }

    Ok(result)
}

#[tauri::command]
pub async fn queue_bulk_refresh_urls_and_retry(
    app: AppHandle,
    state: State<'_, AppState>,
    request: BulkQueueRequest,
) -> CommandResult<BulkQueueResult> {
    let mut result = BulkQueueResult::default();

    for task_id in request.task_ids {
        let outcome = refresh_urls_and_retry(app.clone(), state.inner(), &task_id).await;
        match outcome {
            Ok(task) => result.updated.push(task),
            Err(error) => result.failed.push(BulkQueueFailure {
                task_id,
                message: error.message,
            }),
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn queue_bulk_remove(
    state: State<'_, AppState>,
    request: BulkQueueRequest,
) -> CommandResult<BulkQueueResult> {
    let mut result = BulkQueueResult::default();

    for task_id in request.task_ids {
        match state.remove_task(&task_id) {
            Ok(true) => result.removed.push(task_id),
            Ok(false) => result.failed.push(BulkQueueFailure {
                task_id,
                message: "任务不存在。".to_owned(),
            }),
            Err(error) => result.failed.push(BulkQueueFailure {
                task_id,
                message: error.to_string(),
            }),
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn queue_clear_completed(state: State<'_, AppState>) -> CommandResult<BulkQueueResult> {
    let task_ids = state
        .queue_snapshot()?
        .into_iter()
        .filter(|task| task.status == TaskStatus::Completed)
        .map(|task| task.id)
        .collect::<Vec<_>>();
    let request = BulkQueueRequest { task_ids };

    queue_bulk_remove(state, request)
}

async fn refresh_urls_and_retry(
    app: AppHandle,
    state: &AppState,
    task_id: &str,
) -> CommandResult<DownloadTask> {
    emit_queue_log(&app, state, task_id, QueueLogLevel::Info, "刷新下载地址")?;
    if let Err(error) = state.refresh_task_media_urls(task_id).await {
        emit_queue_log(
            &app,
            state,
            task_id,
            QueueLogLevel::Error,
            &format!("刷新下载地址失败：{error}"),
        )?;
        return Err(error.into());
    }

    retry_task(app, state, task_id)
}

fn retry_task(app: AppHandle, state: &AppState, task_id: &str) -> CommandResult<DownloadTask> {
    let task = state.retry_task(task_id)?;
    events::emit(&app, events::QUEUE_TASK_UPDATED, &task)?;
    emit_queue_log(&app, state, task_id, QueueLogLevel::Info, "重试任务")?;
    start_queue_worker(&app);
    Ok(task)
}

fn bulk_update_task_status(
    app: &AppHandle,
    state: &AppState,
    task_ids: Vec<String>,
    status: TaskStatus,
) -> BulkQueueResult {
    let mut result = BulkQueueResult::default();

    for task_id in task_ids {
        match state.update_task_status(&task_id, status) {
            Ok(task) => {
                if let Err(error) = events::emit(app, events::QUEUE_TASK_UPDATED, &task) {
                    result.failed.push(BulkQueueFailure {
                        task_id,
                        message: error.message,
                    });
                } else {
                    result.updated.push(task);
                }
            }
            Err(error) => result.failed.push(BulkQueueFailure {
                task_id,
                message: error.to_string(),
            }),
        }
    }

    result
}

#[tauri::command]
pub fn queue_remove(
    state: State<'_, AppState>,
    task_id: String,
) -> CommandResult<QueueRemoveResponse> {
    Ok(QueueRemoveResponse {
        removed: state.remove_task(&task_id)?,
    })
}

#[tauri::command]
pub fn queue_open_file(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: String,
) -> CommandResult<()> {
    let task = state.task_snapshot(&task_id)?;
    if task.output_path.exists() {
        open_path(&app, &task.output_path)
    } else {
        open_task_output_dir(&app, &task)
    }
}

#[tauri::command]
pub fn queue_open_dir(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: String,
) -> CommandResult<()> {
    let task = state.task_snapshot(&task_id)?;
    open_task_output_dir(&app, &task)
}

#[tauri::command]
pub fn settings_get(state: State<'_, AppState>) -> CommandResult<SettingsSnapshot> {
    Ok(state.settings()?)
}

#[tauri::command]
pub fn settings_update(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: SettingsSnapshot,
) -> CommandResult<SettingsSnapshot> {
    let settings = state.update_settings(settings)?;
    events::emit(&app, events::SETTINGS_UPDATED, &settings)?;
    Ok(settings)
}

#[tauri::command]
pub async fn maintenance_cleanup_cache(
    state: State<'_, AppState>,
) -> CommandResult<MaintenanceResult> {
    let cache_dir = state.data_dir().join("cache");
    let removed_files = remove_dir_contents(&cache_dir).await?;
    Ok(MaintenanceResult {
        removed_files,
        path: cache_dir.to_string_lossy().into_owned(),
    })
}

#[tauri::command]
pub async fn maintenance_cleanup_temp(
    state: State<'_, AppState>,
) -> CommandResult<MaintenanceResult> {
    let data_temp_dir = state.data_dir().join("temp");
    let mut removed_files = remove_dir_contents(&data_temp_dir).await?;
    for task in state.queue_snapshot()? {
        for resource in task.resources {
            removed_files += remove_file_if_exists(&resource.temp_path).await? as usize;
            removed_files +=
                remove_file_if_exists(&state_path_for(&resource.temp_path)).await? as usize;
        }
    }

    Ok(MaintenanceResult {
        removed_files,
        path: state.data_dir().to_string_lossy().into_owned(),
    })
}

#[tauri::command]
pub async fn diagnostics_export(
    state: State<'_, AppState>,
) -> CommandResult<DiagnosticsExportResponse> {
    let diagnostics_dir = state.data_dir().join("diagnostics");
    fs::create_dir_all(&diagnostics_dir)
        .await
        .map_err(BdlError::from)?;

    let settings = state.settings()?;
    let mut settings_json = serde_json::to_value(&settings).map_err(BdlError::from)?;
    if let Some(proxy_url) = settings_json
        .get("proxy_url")
        .and_then(serde_json::Value::as_str)
        .map(redact_url)
    {
        settings_json["proxy_url"] = serde_json::Value::String(proxy_url);
    }

    let tasks = state.queue_snapshot()?;
    let task_logs = tasks
        .iter()
        .map(|task| {
            Ok(serde_json::json!({
                "task_id": task.id,
                "logs": state.task_logs(&task.id, 50)?,
            }))
        })
        .collect::<BdlResult<Vec<_>>>()?;

    let report = serde_json::json!({
        "generated_at": Utc::now().to_rfc3339(),
        "version": crate::version(),
        "data_dir": state.data_dir(),
        "account": state.account()?,
        "settings": settings_json,
        "tasks": tasks,
        "task_logs": task_logs,
    });
    let file_name = format!(
        "bdl-diagnostics-{}.json",
        Utc::now().format("%Y%m%d-%H%M%S")
    );
    let path = diagnostics_dir.join(file_name);
    fs::write(
        &path,
        serde_json::to_vec_pretty(&report).map_err(BdlError::from)?,
    )
    .await
    .map_err(BdlError::from)?;

    Ok(DiagnosticsExportResponse {
        path: path.to_string_lossy().into_owned(),
    })
}

#[tauri::command]
pub fn account_get(state: State<'_, AppState>) -> CommandResult<AccountSnapshot> {
    Ok(state.account()?)
}

#[tauri::command]
pub async fn account_login_qr_start() -> CommandResult<QrLoginSession> {
    Ok(start_qr_login().await?)
}

#[tauri::command]
pub async fn account_login_qr_poll(
    app: AppHandle,
    state: State<'_, AppState>,
    request: AccountLoginQrPollRequest,
) -> CommandResult<AccountLoginQrPollResponse> {
    let outcome = poll_qr_login(&request.qrcode_key).await?;
    let account = match outcome.cookie_header {
        Some(cookie_header) => {
            let account = state.import_cookie(&cookie_header)?;
            events::emit(&app, events::ACCOUNT_UPDATED, &account)?;
            Some(account)
        }
        None => None,
    };

    Ok(AccountLoginQrPollResponse {
        status: outcome.status,
        message: outcome.message,
        account,
    })
}

#[tauri::command]
pub fn account_import_cookie(
    app: AppHandle,
    state: State<'_, AppState>,
    request: AccountImportCookieRequest,
) -> CommandResult<AccountSnapshot> {
    let account = state.import_cookie(&request.cookie)?;
    events::emit(&app, events::ACCOUNT_UPDATED, &account)?;
    Ok(account)
}

#[tauri::command]
pub fn account_logout(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<AccountSnapshot> {
    let account = state.logout()?;
    events::emit(&app, events::ACCOUNT_UPDATED, &account)?;
    Ok(account)
}

#[tauri::command]
pub fn account_verify(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<AccountSnapshot> {
    let account = state.verify_account()?;
    events::emit(&app, events::ACCOUNT_UPDATED, &account)?;
    Ok(account)
}

fn update_task_status(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: &str,
    status: TaskStatus,
) -> CommandResult<DownloadTask> {
    let task = state.update_task_status(task_id, status)?;
    events::emit(&app, events::QUEUE_TASK_UPDATED, &task)?;
    if status.can_start() {
        start_queue_worker(&app);
    }
    Ok(task)
}

fn start_queue_worker(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        if !state.try_start_queue_worker() {
            return;
        }

        let result = run_queue_worker(&app, state.inner()).await;
        state.finish_queue_worker();

        if let Err(error) = result {
            tracing::error!("queue worker failed: {}", error.message);
        }

        if matches!(state.has_startable_task(), Ok(true)) {
            start_queue_worker(&app);
        }
    });
}

async fn run_queue_worker(app: &AppHandle, state: &AppState) -> CommandResult<()> {
    loop {
        let settings = state.settings()?;
        let fetcher = ReqwestFetcher::with_config(FetchConfig {
            max_retries: retry_count(&settings),
            proxy_url: settings.proxy_url.clone(),
        })?;
        let concurrent_tasks = concurrent_tasks(&settings);
        let runtime_options = DownloadRuntimeOptions::from(&settings);
        let mut tasks = Vec::with_capacity(concurrent_tasks);

        for _ in 0..concurrent_tasks {
            let Some(task) = state.take_next_startable_task()? else {
                break;
            };

            events::emit(app, events::QUEUE_TASK_UPDATED, &task)?;
            emit_queue_log(app, state, &task.id, QueueLogLevel::Info, "开始下载任务")?;
            tasks.push(task);
        }

        if tasks.is_empty() {
            break;
        }

        let outcomes =
            join_all(tasks.iter().cloned().map(|task| {
                run_download_task(app, state, &fetcher, task, runtime_options.clone())
            }))
            .await;

        for (task, outcome) in tasks.into_iter().zip(outcomes) {
            if let Err(error) = outcome {
                match state.task_status(&task.id) {
                    Ok(TaskStatus::Paused | TaskStatus::Cancelled) => {
                        emit_queue_log(app, state, &task.id, QueueLogLevel::Warning, "任务已停止")?;
                    }
                    _ => {
                        if settings.auto_refresh_expired_urls
                            && is_expired_url_error(&error.message)
                            && !already_auto_refreshed(state, &task.id)
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
                                    continue;
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
            }
        }
    }

    Ok(())
}

async fn run_download_task(
    app: &AppHandle,
    state: &AppState,
    fetcher: &ReqwestFetcher,
    task: DownloadTask,
    runtime_options: DownloadRuntimeOptions,
) -> CommandResult<()> {
    for resource in task
        .resources
        .iter()
        .filter(|resource| should_fetch(resource))
    {
        if !task_should_continue(state, &task.id)? {
            emit_queue_log(
                app,
                state,
                &task.id,
                QueueLogLevel::Warning,
                "任务已暂停或取消",
            )?;
            return Ok(());
        }
        if !resource.status.can_start() {
            continue;
        }

        let updated =
            state.update_resource_status(&task.id, &resource.id, ResourceStatus::Downloading)?;
        events::emit(app, events::QUEUE_TASK_UPDATED, &updated)?;
        emit_queue_log(
            app,
            state,
            &task.id,
            QueueLogLevel::Info,
            &format!("下载资源 {}", resource_label(resource)),
        )?;

        if let Err(error) = fetcher.fetch(resource, None).await {
            let updated =
                state.update_resource_status(&task.id, &resource.id, ResourceStatus::Failed)?;
            events::emit(app, events::QUEUE_TASK_UPDATED, &updated)?;
            return Err(error.into());
        }

        let updated =
            state.update_resource_status(&task.id, &resource.id, ResourceStatus::Completed)?;
        events::emit(app, events::QUEUE_TASK_UPDATED, &updated)?;
    }

    if !task_should_continue(state, &task.id)? {
        emit_queue_log(
            app,
            state,
            &task.id,
            QueueLogLevel::Warning,
            "任务已暂停或取消",
        )?;
        return Ok(());
    }

    let muxing = state.update_task_status(&task.id, TaskStatus::Muxing)?;
    events::emit(app, events::QUEUE_TASK_UPDATED, &muxing)?;
    emit_queue_log(app, state, &task.id, QueueLogLevel::Info, "合并音视频")?;

    let latest = state.task_snapshot(&task.id)?;
    let attachments = mux_attachments(&latest, &runtime_options);
    for warning in &attachments.warnings {
        emit_queue_log(app, state, &latest.id, QueueLogLevel::Warning, warning)?;
    }
    if attachments.cover_path.is_some() || !attachments.subtitle_paths.is_empty() {
        emit_queue_log(app, state, &latest.id, QueueLogLevel::Info, "嵌入归档素材")?;
    }

    let video = resource_by_intent(&latest, DownloadResourceIntent::Video)?;
    let audio = resource_by_intent(&latest, DownloadResourceIntent::Audio)?;
    let muxer = MediaMuxer::new(MediaMuxerConfig {
        ffmpeg_path: runtime_options.ffmpeg_path.clone(),
    })
    .map_err(BdlError::from)?;
    muxer
        .mux(&MuxRequest {
            video_path: video.target_path.clone(),
            audio_path: audio.target_path.clone(),
            output_path: latest.output_path.clone(),
            cover_path: attachments.cover_path,
            subtitle_paths: attachments.subtitle_paths,
        })
        .await
        .map_err(BdlError::from)?;

    if !runtime_options.retain_raw_streams {
        cleanup_raw_streams(app, state, &task).await?;
    }

    finalize_archive_assets(app, state, &task).await?;

    if !task_should_continue(state, &task.id)? {
        emit_queue_log(
            app,
            state,
            &task.id,
            QueueLogLevel::Warning,
            "任务已暂停或取消",
        )?;
        return Ok(());
    }

    let completed = state.update_task_status(&task.id, TaskStatus::Completed)?;
    events::emit(app, events::QUEUE_TASK_UPDATED, &completed)?;
    emit_queue_log(app, state, &task.id, QueueLogLevel::Info, "下载完成")?;

    Ok(())
}

fn should_fetch(resource: &DownloadResource) -> bool {
    matches!(
        resource.intent,
        DownloadResourceIntent::Video
            | DownloadResourceIntent::Audio
            | DownloadResourceIntent::Cover
            | DownloadResourceIntent::Subtitle
            | DownloadResourceIntent::Danmaku
    ) && !resource.current_urls.is_empty()
}

struct MuxAttachmentSelection {
    cover_path: Option<PathBuf>,
    subtitle_paths: Vec<PathBuf>,
    warnings: Vec<String>,
}

fn mux_attachments(
    task: &DownloadTask,
    runtime_options: &DownloadRuntimeOptions,
) -> MuxAttachmentSelection {
    let mut selection = MuxAttachmentSelection {
        cover_path: None,
        subtitle_paths: Vec::new(),
        warnings: Vec::new(),
    };

    if runtime_options.embed_cover && task_has_resource_intent(task, DownloadResourceIntent::Cover)
    {
        match completed_resource_by_intent(task, DownloadResourceIntent::Cover) {
            Some(resource)
                if supports_cover_embedding(&task.output_path, &resource.target_path) =>
            {
                selection.cover_path = Some(resource.target_path.clone());
            }
            Some(_) => selection
                .warnings
                .push("跳过封面嵌入：当前封面格式或封装格式不支持。".to_owned()),
            None => selection
                .warnings
                .push("跳过封面嵌入：没有已下载的封面文件。".to_owned()),
        }
    }

    if runtime_options.embed_subtitles
        && task_has_resource_intent(task, DownloadResourceIntent::Subtitle)
    {
        match completed_resource_by_intent(task, DownloadResourceIntent::Subtitle) {
            Some(resource)
                if supports_subtitle_embedding(&task.output_path, &resource.target_path) =>
            {
                selection.subtitle_paths.push(resource.target_path.clone());
            }
            Some(_) => selection
                .warnings
                .push("跳过字幕嵌入：当前字幕格式或封装格式不支持。".to_owned()),
            None => selection
                .warnings
                .push("跳过字幕嵌入：没有已下载的字幕文件。".to_owned()),
        }
    }

    selection
}

fn task_has_resource_intent(task: &DownloadTask, intent: DownloadResourceIntent) -> bool {
    task.resources
        .iter()
        .any(|resource| resource.intent == intent)
}

fn completed_resource_by_intent(
    task: &DownloadTask,
    intent: DownloadResourceIntent,
) -> Option<&DownloadResource> {
    task.resources.iter().find(|resource| {
        resource.intent == intent
            && resource.status == ResourceStatus::Completed
            && resource.target_path.is_file()
    })
}

async fn cleanup_raw_streams(
    app: &AppHandle,
    state: &AppState,
    task: &DownloadTask,
) -> CommandResult<()> {
    for resource in task.resources.iter().filter(|resource| {
        matches!(
            resource.intent,
            DownloadResourceIntent::Video | DownloadResourceIntent::Audio
        )
    }) {
        match fs::remove_file(&resource.target_path).await {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(BdlError::from(error).into()),
        }
    }

    emit_queue_log(
        app,
        state,
        &task.id,
        QueueLogLevel::Info,
        "清理原始音视频轨道",
    )
}

async fn finalize_archive_assets(
    app: &AppHandle,
    state: &AppState,
    task: &DownloadTask,
) -> CommandResult<()> {
    let latest = state.task_snapshot(&task.id)?;
    for resource in latest
        .resources
        .iter()
        .filter(|resource| resource.kind == bdl_core::queue::DownloadResourceKind::Asset)
        .filter(|resource| resource.status.can_start())
    {
        match resource.intent {
            DownloadResourceIntent::Nfo => {
                write_nfo(&latest, resource).await?;
                let updated = state.update_resource_status(
                    &latest.id,
                    &resource.id,
                    ResourceStatus::Completed,
                )?;
                events::emit(app, events::QUEUE_TASK_UPDATED, &updated)?;
                emit_queue_log(app, state, &latest.id, QueueLogLevel::Info, "生成 NFO")?;
            }
            DownloadResourceIntent::Cover
            | DownloadResourceIntent::Subtitle
            | DownloadResourceIntent::Danmaku
                if resource.current_urls.is_empty() =>
            {
                let updated = state.update_resource_status(
                    &latest.id,
                    &resource.id,
                    ResourceStatus::Completed,
                )?;
                events::emit(app, events::QUEUE_TASK_UPDATED, &updated)?;
                emit_queue_log(
                    app,
                    state,
                    &latest.id,
                    QueueLogLevel::Warning,
                    &format!("跳过{}：暂无可用地址", resource_label(resource)),
                )?;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn write_nfo(task: &DownloadTask, resource: &DownloadResource) -> CommandResult<()> {
    if let Some(parent) = resource.target_path.parent() {
        fs::create_dir_all(parent).await.map_err(BdlError::from)?;
    }
    fs::write(&resource.target_path, nfo_content(task))
        .await
        .map_err(BdlError::from)?;
    Ok(())
}

fn nfo_content(task: &DownloadTask) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<movie>\n\
  <title>{}</title>\n\
  <source>{}</source>\n\
  <filename>{}</filename>\n\
</movie>\n",
        escape_xml(&task.title),
        escape_xml(&task.source_id),
        escape_xml(&task.output_path.to_string_lossy())
    )
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn resource_by_intent(
    task: &DownloadTask,
    intent: DownloadResourceIntent,
) -> CommandResult<&DownloadResource> {
    task.resources
        .iter()
        .find(|resource| resource.intent == intent)
        .ok_or_else(|| CommandError {
            code: "missing_resource".to_owned(),
            message: format!("任务 `{}` 缺少 {:?} 资源。", task.title, intent),
        })
}

fn task_should_continue(state: &AppState, task_id: &str) -> CommandResult<bool> {
    Ok(!matches!(
        state.task_status(task_id)?,
        TaskStatus::Paused | TaskStatus::Cancelled
    ))
}

fn resource_label(resource: &DownloadResource) -> &'static str {
    match resource.intent {
        DownloadResourceIntent::Video => "视频",
        DownloadResourceIntent::Audio => "音频",
        DownloadResourceIntent::Cover => "封面",
        DownloadResourceIntent::Subtitle => "字幕",
        DownloadResourceIntent::Danmaku => "弹幕",
        DownloadResourceIntent::Nfo => "NFO",
    }
}

fn open_task_output_dir(app: &AppHandle, task: &DownloadTask) -> CommandResult<()> {
    let dir = task.output_path.parent().ok_or_else(|| CommandError {
        code: "invalid_output_path".to_owned(),
        message: format!("任务 `{}` 没有有效输出目录。", task.title),
    })?;
    std::fs::create_dir_all(dir).map_err(BdlError::from)?;
    open_path(app, dir)
}

fn open_path(app: &AppHandle, path: &Path) -> CommandResult<()> {
    app.opener()
        .open_path(path.to_string_lossy().into_owned(), None::<String>)
        .map_err(|error| CommandError {
            code: "open_path_failed".to_owned(),
            message: format!("打开路径失败：{error}"),
        })
}

async fn remove_file_if_exists(path: &Path) -> CommandResult<bool> {
    match fs::remove_file(path).await {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(BdlError::from(error).into()),
    }
}

async fn remove_dir_contents(path: &Path) -> CommandResult<usize> {
    remove_dir_contents_sync(path).map_err(Into::into)
}

fn remove_dir_contents_sync(path: &Path) -> BdlResult<usize> {
    if !path.exists() {
        return Ok(0);
    }

    let mut removed_files = 0;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            removed_files += count_files_sync(&entry_path)?;
            std::fs::remove_dir_all(&entry_path)?;
        } else {
            std::fs::remove_file(&entry_path)?;
            removed_files += 1;
        }
    }

    Ok(removed_files)
}

fn count_files_sync(path: &Path) -> BdlResult<usize> {
    let mut count = 0;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            count += count_files_sync(&entry.path())?;
        } else {
            count += 1;
        }
    }
    Ok(count)
}

fn redact_url(raw: &str) -> String {
    let Some(scheme_end) = raw.find("://") else {
        return raw.to_owned();
    };
    let Some(credentials_end) = raw[scheme_end + 3..].find('@') else {
        return raw.to_owned();
    };
    let host_start = scheme_end + 3 + credentials_end + 1;
    format!("{}://<redacted>@{}", &raw[..scheme_end], &raw[host_start..])
}

fn emit_queue_log(
    app: &AppHandle,
    state: &AppState,
    task_id: &str,
    level: QueueLogLevel,
    message: &str,
) -> CommandResult<()> {
    if !queue_log_enabled(state, level)? {
        return Ok(());
    }

    let entry = state.append_task_log(QueueLogEntry {
        task_id: task_id.to_owned(),
        level,
        message: message.to_owned(),
        created_at: Utc::now().to_rfc3339(),
    })?;
    events::emit(app, events::QUEUE_LOG_APPENDED, &entry)
}

fn queue_log_enabled(state: &AppState, level: QueueLogLevel) -> CommandResult<bool> {
    let settings = state.settings()?;
    let threshold = match settings.log_level.as_str() {
        "debug" | "info" => 1,
        "warning" => 2,
        "error" => 3,
        _ => 1,
    };
    Ok(queue_log_level_rank(level) >= threshold)
}

fn queue_log_level_rank(level: QueueLogLevel) -> u8 {
    match level {
        QueueLogLevel::Info => 1,
        QueueLogLevel::Warning => 2,
        QueueLogLevel::Error => 3,
    }
}

fn concurrent_tasks(settings: &SettingsSnapshot) -> usize {
    settings.concurrent_tasks.clamp(1, 5)
}

fn retry_count(settings: &SettingsSnapshot) -> usize {
    settings.retry_count.min(5)
}

fn is_expired_url_error(message: &str) -> bool {
    message.contains("HTTP 404") || message.contains("资源长度失败")
}

fn already_auto_refreshed(state: &AppState, task_id: &str) -> bool {
    state
        .task_logs(task_id, 50)
        .map(|logs| logs.iter().any(|log| log.message == "自动刷新过期链接"))
        .unwrap_or(false)
}

fn parse_archive_mode(value: &str) -> CommandResult<ArchiveMode> {
    match value {
        "fast" => Ok(ArchiveMode::Fast),
        "complete_archive" => Ok(ArchiveMode::CompleteArchive),
        "custom" => Ok(ArchiveMode::Custom),
        other => Err(CommandError {
            code: "invalid_archive_mode".to_owned(),
            message: format!("unknown archive mode `{other}`"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::{nfo_content, should_fetch};
    use std::path::PathBuf;

    use bdl_core::model::HeaderPair;
    use bdl_core::queue::{
        DownloadResource, DownloadResourceIntent, DownloadResourceKind, DownloadTask,
        ResourceStatus, TaskStatus,
    };

    #[test]
    fn nfo_content_escapes_xml_sensitive_fields() {
        let task = DownloadTask {
            id: "task:fixture".to_owned(),
            title: "A&B <C>".to_owned(),
            source_id: "video:\"source\"".to_owned(),
            status: TaskStatus::Completed,
            resources: Vec::new(),
            output_path: PathBuf::from("downloads/A&B <C>.mp4"),
        };

        let nfo = nfo_content(&task);

        assert!(nfo.contains("A&amp;B &lt;C&gt;"));
        assert!(nfo.contains("video:&quot;source&quot;"));
    }

    #[test]
    fn should_fetch_skips_empty_cover_asset() {
        let resource = resource(DownloadResourceIntent::Cover, Vec::new());

        assert!(!should_fetch(&resource));
    }

    #[test]
    fn should_fetch_downloads_subtitle_and_danmaku_assets_with_urls() {
        let subtitle = resource(
            DownloadResourceIntent::Subtitle,
            vec!["https://example.invalid/subtitle.json".to_owned()],
        );
        let danmaku = resource(
            DownloadResourceIntent::Danmaku,
            vec!["https://example.invalid/danmaku.xml".to_owned()],
        );

        assert!(should_fetch(&subtitle));
        assert!(should_fetch(&danmaku));
    }

    fn resource(intent: DownloadResourceIntent, current_urls: Vec<String>) -> DownloadResource {
        let suffix = format!("{intent:?}").to_ascii_lowercase();
        DownloadResource {
            id: format!("resource:{suffix}"),
            kind: DownloadResourceKind::Asset,
            intent,
            current_urls,
            headers: Vec::<HeaderPair>::new(),
            target_path: PathBuf::from(&suffix),
            temp_path: PathBuf::from(format!("{suffix}.bdlpart")),
            status: ResourceStatus::Pending,
        }
    }
}
