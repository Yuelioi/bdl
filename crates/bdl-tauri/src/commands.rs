use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bdl_core::account::{
    AccountLibraryFolderKind, AccountLibraryPage, QrLoginSession, QrLoginStatus, poll_qr_login,
    start_qr_login,
};
use bdl_core::fetcher::{
    BandwidthLimiter, FetchCancelToken, FetchConfig, FetchProgress, ProgressSender, ReqwestFetcher,
    state_path_for,
};
use bdl_core::ids::{PartId, SourceId};
use bdl_core::model::NormalizedSourceTree;
use bdl_core::muxer::{MediaMuxer, MediaMuxerConfig, MuxError, MuxRequest};
use bdl_core::planner::{
    ArchiveMode, DownloadMediaMode, DownloadOptions, MissingQualityPolicy, StreamPreference,
    parse_stream_codec, plan_selected_parts,
};
use bdl_core::queue::{
    DownloadResource, DownloadResourceIntent, DownloadTask, DuplicateTaskPolicy, QueueLogEntry,
    QueueLogLevel, ResourceStatus, TaskStatus,
};
use bdl_core::settings::{validate_embedding_container, validate_speed_limit};
use bdl_core::{BdlError, BdlResult};
use chrono::{DateTime, Utc};
use futures::stream::{FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::unbounded_channel;

use crate::diagnostic_export::{
    redact_log as redact_log_for_diagnostics, redact_task as redact_task_for_diagnostics,
    redact_url,
};
use crate::events;
use crate::media_finalize::{
    completed_resource_by_intent, select_mux_attachments, task_has_resource_intent, write_nfo,
};
use crate::state::{AccountSnapshot, AppState, SettingsSnapshot, StartupRecoverySnapshot};

pub type CommandResult<T> = Result<T, CommandError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}

impl From<BdlError> for CommandError {
    fn from(error: BdlError) -> Self {
        Self {
            code: command_error_code(&error).to_owned(),
            message: command_error_message(&error).unwrap_or_else(|| error.to_string()),
        }
    }
}

fn command_error_code(error: &BdlError) -> &'static str {
    match error {
        BdlError::InvalidInput { .. } => "parse_unrecognized",
        BdlError::UnsupportedSource { .. } => "unsupported_source",
        BdlError::Mux(MuxError::FfmpegNotFound { .. }) => "missing_ffmpeg",
        BdlError::Io(error) if error.kind() == ErrorKind::PermissionDenied => {
            "unwritable_save_directory"
        }
        BdlError::Fetch { message } | BdlError::Bpi(message)
            if is_private_resource_error(message) =>
        {
            "private_resource"
        }
        BdlError::Fetch { message } | BdlError::Bpi(message) if is_login_expired_error(message) => {
            "login_required"
        }
        _ => "core_error",
    }
}

fn command_error_message(error: &BdlError) -> Option<String> {
    match error {
        BdlError::InvalidInput { message } => Some(message.clone()),
        BdlError::UnsupportedSource { kind } => Some(format!("暂不支持 `{kind}` 类型的来源。")),
        BdlError::Mux(MuxError::FfmpegNotFound { .. }) => {
            Some("未找到 FFmpeg，请在设置中配置 FFmpeg 路径。".to_owned())
        }
        BdlError::Io(error) if error.kind() == ErrorKind::PermissionDenied => {
            Some("保存目录不可写，请检查权限或更换保存目录。".to_owned())
        }
        BdlError::Fetch { message } | BdlError::Bpi(message)
            if is_private_resource_error(message) =>
        {
            Some("资源不可访问，可能是私密稿件、已失效或当前账号无权访问。".to_owned())
        }
        BdlError::Fetch { message } | BdlError::Bpi(message) if is_login_expired_error(message) => {
            Some("登录状态可能已失效，请重新登录后重试。".to_owned())
        }
        _ => None,
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

#[derive(Debug, Clone, Deserialize)]
pub struct ScheduleTaskRequest {
    pub task_id: String,
    pub scheduled_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SpeedLimitTaskRequest {
    pub task_id: String,
    pub speed_limit_bytes_per_second: Option<u64>,
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

#[derive(Debug, Clone, Serialize)]
pub struct QueueProgressEntry {
    pub task_id: String,
    pub resource_id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub created_at: String,
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
    pub naming_template: Option<String>,
    pub media_mode: Option<String>,
    pub quality: Option<String>,
    pub audio_quality: Option<String>,
    pub codec: Option<String>,
    pub scheduled_at: Option<String>,
    pub speed_limit_bytes_per_second: Option<u64>,
    #[serde(default)]
    pub duplicate_policy: DuplicateTaskPolicy,
}

#[derive(Debug, Clone, Serialize)]
pub struct DuplicateTaskMatch {
    pub proposed_task_id: String,
    pub title: String,
    pub existing_task_id: String,
    pub existing_status: TaskStatus,
}

#[derive(Debug, Clone, Serialize)]
pub struct SelectionCreateTasksResult {
    pub created: Vec<DownloadTask>,
    pub duplicates: Vec<DuplicateTaskMatch>,
    pub requires_confirmation: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountImportCookieRequest {
    pub cookie: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountLoginQrPollRequest {
    pub qrcode_key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountLibraryRequest {
    pub kind: AccountLibraryFolderKind,
    pub page: u32,
    pub page_size: u32,
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

#[derive(Debug, Clone, Deserialize)]
pub struct EnvironmentHealthRequest {
    pub download_dir: Option<String>,
    pub ffmpeg_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateDownloadDirectoryRequest {
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadDirectoryStatus {
    Ready,
    Missing,
    NotDirectory,
    Unwritable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DownloadDirectoryHealth {
    pub status: DownloadDirectoryStatus,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FfmpegStatus {
    Ready,
    Missing,
    Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FfmpegSource {
    Configured,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FfmpegHealth {
    pub status: FfmpegStatus,
    pub source: FfmpegSource,
    pub path: Option<String>,
    pub version: Option<String>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EnvironmentHealthSnapshot {
    pub ready: bool,
    pub download_directory: DownloadDirectoryHealth,
    pub ffmpeg: FfmpegHealth,
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

struct QueueTaskLaunchContext {
    settings: SettingsSnapshot,
    fetch_config: FetchConfig,
    runtime_options: DownloadRuntimeOptions,
}

fn queue_task_launch_context(
    settings: SettingsSnapshot,
    task_speed_limit_bytes_per_second: Option<u64>,
) -> QueueTaskLaunchContext {
    let fetch_config = FetchConfig {
        max_retries: retry_count(&settings),
        proxy_url: settings.proxy_url.clone(),
        segment_count: segment_count(&settings),
        speed_limit_bytes_per_second: task_speed_limit_bytes_per_second,
    };
    let runtime_options = DownloadRuntimeOptions::from(&settings);
    QueueTaskLaunchContext {
        settings,
        fetch_config,
        runtime_options,
    }
}

#[tauri::command]
pub async fn parse_create_source(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ParseCreateSourceRequest,
) -> CommandResult<NormalizedSourceTree> {
    let tree = state
        .parse_source_for_workspace(&request.input, request.fetch_streams)
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
) -> CommandResult<SelectionCreateTasksResult> {
    let source_id = SourceId(request.source_id);
    let duplicate_policy = request.duplicate_policy;
    let scheduled_at = parse_future_schedule(request.scheduled_at.as_deref(), Utc::now())?;
    let speed_limit_bytes_per_second =
        parse_speed_limit(request.speed_limit_bytes_per_second, "单任务下载限速")?;
    let settings = state.settings()?;
    let selected_part_ids = request.part_ids.into_iter().map(PartId).collect::<Vec<_>>();

    let output_dir = request
        .output_dir
        .as_deref()
        .or(settings.download_dir.as_deref())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("downloads"));

    let archive_mode = parse_archive_mode(
        request
            .archive_mode
            .as_deref()
            .unwrap_or(&settings.archive_mode),
    )?;

    let mut options = DownloadOptions::new(output_dir).with_archive_mode(archive_mode);
    let output_extension = request
        .output_extension
        .as_deref()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| settings.output_extension.clone());
    validate_embedding_container(
        &output_extension,
        settings.embed_cover,
        settings.embed_subtitles,
    )?;
    options.output_extension = output_extension;
    options.media_mode =
        DownloadMediaMode::parse(request.media_mode.as_deref().unwrap_or("audio_video"))?;
    options.naming_template = request.naming_template.unwrap_or(settings.naming_template);
    options.duplicate_naming_strategy = settings.duplicate_naming_strategy;
    options.video_quality = StreamPreference::parse(
        request.quality.as_deref().unwrap_or(&settings.quality),
        "视频清晰度",
    )?;
    options.audio_quality = StreamPreference::parse(
        request
            .audio_quality
            .as_deref()
            .unwrap_or(&settings.audio_quality),
        "音频质量",
    )?;
    options.video_codec = parse_stream_codec(request.codec.as_deref().unwrap_or(&settings.codec))?;
    options.missing_quality_policy = MissingQualityPolicy::parse(&settings.missing_quality_policy)?;
    options.archive_assets = settings.archive_assets;

    let prepared = state
        .prepare_selection(&source_id, &selected_part_ids)
        .await?;
    let mut planned_tasks = plan_selected_parts(&prepared.tree, &prepared.part_ids, &options)?;
    for task in &mut planned_tasks {
        task.scheduled_at = scheduled_at;
        task.speed_limit_bytes_per_second = speed_limit_bytes_per_second;
    }

    if prepared.tree_updated {
        events::emit(&app, events::PARSE_SOURCE_UPDATED, &prepared.tree)?;
    }

    let outcome = state.enqueue_tasks_with_duplicate_policy(planned_tasks, duplicate_policy)?;
    let tasks = outcome.inserted;
    let duplicates = outcome
        .duplicates
        .into_iter()
        .map(|duplicate| DuplicateTaskMatch {
            proposed_task_id: duplicate.proposed_task_id,
            title: duplicate.title,
            existing_task_id: duplicate.existing_task_id,
            existing_status: duplicate.existing_status,
        })
        .collect();

    for task in &tasks {
        events::emit(&app, events::QUEUE_TASK_UPDATED, task)?;
    }
    if !tasks.is_empty() {
        start_queue_worker(&app);
    }

    Ok(SelectionCreateTasksResult {
        created: tasks,
        duplicates,
        requires_confirmation: outcome.requires_confirmation,
    })
}

#[tauri::command]
pub fn queue_list(app: AppHandle, state: State<'_, AppState>) -> CommandResult<Vec<DownloadTask>> {
    let tasks = state.queue_snapshot()?;
    start_queue_worker(&app);
    Ok(tasks)
}

#[tauri::command]
pub fn queue_startup_recovery(
    state: State<'_, AppState>,
) -> CommandResult<StartupRecoverySnapshot> {
    Ok(state.startup_recovery()?)
}

#[tauri::command]
pub fn queue_dismiss_startup_recovery(
    state: State<'_, AppState>,
) -> CommandResult<StartupRecoverySnapshot> {
    Ok(state.clear_startup_recovery()?)
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
pub fn queue_schedule(
    app: AppHandle,
    state: State<'_, AppState>,
    request: ScheduleTaskRequest,
) -> CommandResult<DownloadTask> {
    let scheduled_at =
        parse_future_schedule(Some(&request.scheduled_at), Utc::now())?.ok_or_else(|| {
            BdlError::Planning {
                message: "请选择任务开始时间。".to_owned(),
            }
        })?;
    let task = state.set_task_schedule(&request.task_id, Some(scheduled_at))?;
    events::emit(&app, events::QUEUE_TASK_UPDATED, &task)?;
    emit_queue_log(
        &app,
        state.inner(),
        &task.id,
        QueueLogLevel::Info,
        &format!("任务已定时至 {}", scheduled_at.to_rfc3339()),
    )?;
    start_queue_worker(&app);
    Ok(task)
}

#[tauri::command]
pub fn queue_unschedule(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: String,
) -> CommandResult<DownloadTask> {
    let task = state.set_task_schedule(&task_id, None)?;
    events::emit(&app, events::QUEUE_TASK_UPDATED, &task)?;
    emit_queue_log(
        &app,
        state.inner(),
        &task.id,
        QueueLogLevel::Info,
        "已取消定时，任务恢复到队列",
    )?;
    start_queue_worker(&app);
    Ok(task)
}

#[tauri::command]
pub fn queue_set_speed_limit(
    app: AppHandle,
    state: State<'_, AppState>,
    request: SpeedLimitTaskRequest,
) -> CommandResult<DownloadTask> {
    let speed_limit = parse_speed_limit(request.speed_limit_bytes_per_second, "单任务下载限速")?;
    let task = state.set_task_speed_limit(&request.task_id, speed_limit)?;
    events::emit(&app, events::QUEUE_TASK_UPDATED, &task)?;
    let message = speed_limit.map_or_else(
        || "单任务下载限速已取消".to_owned(),
        |limit| format!("单任务下载限速已设为 {limit} B/s"),
    );
    emit_queue_log(&app, state.inner(), &task.id, QueueLogLevel::Info, &message)?;
    start_queue_worker(&app);
    Ok(task)
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
pub fn queue_bulk_cancel(
    app: AppHandle,
    state: State<'_, AppState>,
    request: BulkQueueRequest,
) -> CommandResult<BulkQueueResult> {
    Ok(bulk_update_task_status(
        &app,
        state.inner(),
        request.task_ids,
        TaskStatus::Cancelled,
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
        let _ = state.cancel_running_task(&task_id);
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
                if matches!(status, TaskStatus::Paused | TaskStatus::Cancelled) {
                    let _ = state.cancel_running_task(&task_id);
                }
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
pub fn open_external_url(app: AppHandle, url: String) -> CommandResult<()> {
    const ALLOWED_URLS: [&str; 2] = [
        "https://space.bilibili.com/4279370",
        "https://www.yuelili.com",
    ];
    if !ALLOWED_URLS.contains(&url.as_str()) {
        return Err(CommandError {
            code: "external_url_not_allowed".to_owned(),
            message: "该链接不在允许打开的列表中。".to_owned(),
        });
    }

    app.opener()
        .open_url(url, None::<String>)
        .map_err(|error| CommandError {
            code: "open_url_failed".to_owned(),
            message: format!("打开链接失败：{error}"),
        })
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
pub async fn environment_health(
    request: EnvironmentHealthRequest,
) -> CommandResult<EnvironmentHealthSnapshot> {
    let download_path = resolve_download_directory(request.download_dir.as_deref())?;
    let download_directory = check_download_directory(download_path).await;
    let ffmpeg = check_ffmpeg(request.ffmpeg_path.as_deref()).await;
    let ready = download_directory.status == DownloadDirectoryStatus::Ready
        && ffmpeg.status == FfmpegStatus::Ready;

    Ok(EnvironmentHealthSnapshot {
        ready,
        download_directory,
        ffmpeg,
    })
}

#[tauri::command]
pub async fn environment_create_download_directory(
    request: CreateDownloadDirectoryRequest,
) -> CommandResult<DownloadDirectoryHealth> {
    let path = resolve_download_directory(Some(&request.path))?;
    fs::create_dir_all(&path).await.map_err(BdlError::from)?;
    Ok(check_download_directory(path).await)
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
    let redacted_tasks = tasks
        .iter()
        .cloned()
        .map(redact_task_for_diagnostics)
        .collect::<Vec<_>>();
    let task_logs = tasks
        .iter()
        .map(|task| {
            let logs = state
                .task_logs(&task.id, 50)?
                .into_iter()
                .map(redact_log_for_diagnostics)
                .collect::<Vec<_>>();
            Ok(serde_json::json!({
                "task_id": task.id,
                "logs": logs,
            }))
        })
        .collect::<BdlResult<Vec<_>>>()?;

    let report = serde_json::json!({
        "generated_at": Utc::now().to_rfc3339(),
        "version": crate::version(),
        "data_dir": state.data_dir(),
        "account": state.account()?,
        "settings": settings_json,
        "tasks": redacted_tasks,
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
pub async fn account_verify(
    app: AppHandle,
    state: State<'_, AppState>,
) -> CommandResult<AccountSnapshot> {
    let account = state.verify_account().await?;
    events::emit(&app, events::ACCOUNT_UPDATED, &account)?;
    Ok(account)
}

#[tauri::command]
pub async fn account_library_list(
    state: State<'_, AppState>,
    request: AccountLibraryRequest,
) -> CommandResult<AccountLibraryPage> {
    Ok(state
        .account_library(request.kind, request.page, request.page_size)
        .await?)
}

pub fn start_account_startup_verification(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        let state = app.state::<AppState>();
        let account = match state.verify_account().await {
            Ok(account) => account,
            Err(error) => {
                tracing::warn!("startup account verification failed: {error}");
                match state.account() {
                    Ok(account) => account,
                    Err(error) => {
                        tracing::warn!(
                            "failed to read account after startup verification: {error}"
                        );
                        return;
                    }
                }
            }
        };

        if let Err(error) = events::emit(&app, events::ACCOUNT_UPDATED, &account) {
            tracing::warn!("failed to emit startup account update: {}", error.message);
        }
    });
}

fn update_task_status(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: &str,
    status: TaskStatus,
) -> CommandResult<DownloadTask> {
    if matches!(status, TaskStatus::Paused | TaskStatus::Cancelled) {
        let _ = state.cancel_running_task(task_id)?;
    }
    let task = state.update_task_status(task_id, status)?;
    events::emit(&app, events::QUEUE_TASK_UPDATED, &task)?;
    if status.can_start() {
        start_queue_worker(&app);
    }
    Ok(task)
}

fn resolve_download_directory(configured: Option<&str>) -> BdlResult<PathBuf> {
    let candidate = configured
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("downloads"));
    if candidate.is_absolute() {
        Ok(candidate)
    } else {
        Ok(std::env::current_dir()?.join(candidate))
    }
}

fn parse_future_schedule(
    value: Option<&str>,
    now: DateTime<Utc>,
) -> BdlResult<Option<DateTime<Utc>>> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    let scheduled_at = DateTime::parse_from_rfc3339(value)
        .map_err(|_| BdlError::Planning {
            message: "开始时间格式无效，请重新选择。".to_owned(),
        })?
        .with_timezone(&Utc);
    if scheduled_at <= now {
        return Err(BdlError::Planning {
            message: "开始时间必须晚于当前时间。".to_owned(),
        });
    }
    Ok(Some(scheduled_at))
}

fn parse_speed_limit(value: Option<u64>, label: &str) -> CommandResult<Option<u64>> {
    let value = value.filter(|limit| *limit > 0);
    validate_speed_limit(value, label)?;
    Ok(value)
}

async fn check_download_directory(path: PathBuf) -> DownloadDirectoryHealth {
    let display_path = path.to_string_lossy().into_owned();
    let metadata = match fs::metadata(&path).await {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return DownloadDirectoryHealth {
                status: DownloadDirectoryStatus::Missing,
                path: display_path,
                message: "保存目录尚未创建。".to_owned(),
            };
        }
        Err(error) => {
            return DownloadDirectoryHealth {
                status: DownloadDirectoryStatus::Unwritable,
                path: display_path,
                message: format!("无法访问保存目录：{error}"),
            };
        }
    };

    if !metadata.is_dir() {
        return DownloadDirectoryHealth {
            status: DownloadDirectoryStatus::NotDirectory,
            path: display_path,
            message: "保存路径指向文件，不是目录。".to_owned(),
        };
    }

    let probe_path = path.join(format!(
        ".bdl-write-probe-{}-{}",
        std::process::id(),
        Utc::now().timestamp_nanos_opt().unwrap_or_default()
    ));
    let probe_result = async {
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&probe_path)
            .await?;
        file.write_all(b"bdl").await?;
        file.flush().await?;
        drop(file);
        fs::remove_file(&probe_path).await
    }
    .await;

    match probe_result {
        Ok(()) => DownloadDirectoryHealth {
            status: DownloadDirectoryStatus::Ready,
            path: display_path,
            message: "保存目录可写。".to_owned(),
        },
        Err(error) => {
            let _ = fs::remove_file(&probe_path).await;
            DownloadDirectoryHealth {
                status: DownloadDirectoryStatus::Unwritable,
                path: display_path,
                message: format!("保存目录不可写：{error}"),
            }
        }
    }
}

async fn check_ffmpeg(configured: Option<&str>) -> FfmpegHealth {
    let configured_path = configured
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .map(PathBuf::from);
    let source = if configured_path.is_some() {
        FfmpegSource::Configured
    } else {
        FfmpegSource::System
    };
    let muxer = match MediaMuxer::new(MediaMuxerConfig {
        ffmpeg_path: configured_path,
    }) {
        Ok(muxer) => muxer,
        Err(MuxError::FfmpegNotFound { path }) => {
            return FfmpegHealth {
                status: FfmpegStatus::Missing,
                source,
                path: Some(path),
                version: None,
                message: "未找到可用的 FFmpeg。".to_owned(),
            };
        }
        Err(error) => {
            return FfmpegHealth {
                status: FfmpegStatus::Invalid,
                source,
                path: None,
                version: None,
                message: format!("FFmpeg 配置无效：{error}"),
            };
        }
    };

    match tokio::time::timeout(std::time::Duration::from_secs(3), muxer.probe()).await {
        Ok(Ok(probe)) => FfmpegHealth {
            status: FfmpegStatus::Ready,
            source,
            path: Some(probe.path.to_string_lossy().into_owned()),
            version: Some(probe.version),
            message: "FFmpeg 可用。".to_owned(),
        },
        Ok(Err(error)) => FfmpegHealth {
            status: FfmpegStatus::Invalid,
            source,
            path: Some(muxer.ffmpeg_path().to_string_lossy().into_owned()),
            version: None,
            message: format!("FFmpeg 无法运行：{error}"),
        },
        Err(_) => FfmpegHealth {
            status: FfmpegStatus::Invalid,
            source,
            path: Some(muxer.ffmpeg_path().to_string_lossy().into_owned()),
            version: None,
            message: "FFmpeg 检查超时。".to_owned(),
        },
    }
}

fn start_queue_worker(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        state.notify_queue_changed();
        if !state.try_start_queue_worker() {
            return;
        }

        let result = run_queue_worker(&app, state.inner()).await;
        state.finish_queue_worker();

        if let Err(error) = result {
            tracing::error!("queue worker failed: {}", error.message);
        }

        let should_restart = matches!(state.has_startable_task(), Ok(true))
            || matches!(state.next_scheduled_at(), Ok(Some(_)));
        if should_restart {
            start_queue_worker(&app);
        }
    });
}

async fn run_queue_worker(app: &AppHandle, state: &AppState) -> CommandResult<()> {
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

async fn wait_for_schedule_or_queue_change(state: &AppState, scheduled_at: DateTime<Utc>) {
    let delay = (scheduled_at - Utc::now())
        .to_std()
        .unwrap_or(std::time::Duration::ZERO);
    tokio::select! {
        () = tokio::time::sleep(delay) => {},
        () = state.wait_for_queue_change() => {},
    }
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
            emit_queue_log(
                app,
                state,
                &task.id,
                QueueLogLevel::Warning,
                &format!("成品已生成，但收尾处理未完全完成：{}", error.message),
            )?;
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

async fn run_download_task(
    app: &AppHandle,
    state: &AppState,
    fetcher: &ReqwestFetcher,
    task: DownloadTask,
    runtime_options: DownloadRuntimeOptions,
) -> CommandResult<()> {
    if has_recoverable_completed_output(&task).await? {
        let completed = state.update_task_status(&task.id, TaskStatus::Completed)?;
        events::emit(app, events::QUEUE_TASK_UPDATED, &completed)?;
        emit_queue_log(
            app,
            state,
            &task.id,
            QueueLogLevel::Info,
            "检测到已生成的成品，任务已恢复为完成状态",
        )?;
        if let Err(error) = state.record_completed_task(&completed) {
            tracing::warn!("failed to save recovered completed task record: {error}");
        }
        return Ok(());
    }

    let task_id = task.id.clone();
    let cancel_token = state.register_task_cancel_token(&task_id)?;
    let outcome =
        run_download_task_inner(app, state, fetcher, task, runtime_options, cancel_token).await;
    state.clear_task_cancel_token(&task_id)?;
    outcome
}

async fn run_download_task_inner(
    app: &AppHandle,
    state: &AppState,
    fetcher: &ReqwestFetcher,
    task: DownloadTask,
    runtime_options: DownloadRuntimeOptions,
    cancel_token: FetchCancelToken,
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

        let progress = progress_sender(app, &task.id);
        if let Err(error) = fetcher
            .fetch_cancelable(resource, Some(progress), cancel_token.clone())
            .await
        {
            if cancel_token.is_cancelled() {
                let Ok(status) = state.task_status(&task.id) else {
                    return Ok(());
                };
                let resource_status = match status {
                    TaskStatus::Paused => ResourceStatus::Paused,
                    TaskStatus::Cancelled => ResourceStatus::Cancelled,
                    TaskStatus::Waiting => ResourceStatus::Pending,
                    _ => ResourceStatus::Failed,
                };
                let updated =
                    state.update_resource_status(&task.id, &resource.id, resource_status)?;
                events::emit(app, events::QUEUE_TASK_UPDATED, &updated)?;
                emit_queue_log(
                    app,
                    state,
                    &task.id,
                    QueueLogLevel::Warning,
                    "任务已暂停或取消",
                )?;
                return Ok(());
            }

            let resource_status = match state.task_status(&task.id) {
                Ok(TaskStatus::Paused) => ResourceStatus::Paused,
                Ok(TaskStatus::Cancelled) => ResourceStatus::Cancelled,
                _ => ResourceStatus::Failed,
            };
            let updated = state.update_resource_status(&task.id, &resource.id, resource_status)?;
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
    if muxing.status != TaskStatus::Muxing {
        return Ok(());
    }
    events::emit(app, events::QUEUE_TASK_UPDATED, &muxing)?;
    emit_queue_log(
        app,
        state,
        &task.id,
        QueueLogLevel::Info,
        task_mux_label(&task),
    )?;

    let latest = state.task_snapshot(&task.id)?;
    let attachments = select_mux_attachments(
        &latest,
        runtime_options.embed_cover,
        runtime_options.embed_subtitles,
    );
    for warning in &attachments.warnings {
        emit_queue_log(app, state, &latest.id, QueueLogLevel::Warning, warning)?;
    }
    if attachments.cover_path.is_some() || !attachments.subtitle_paths.is_empty() {
        emit_queue_log(app, state, &latest.id, QueueLogLevel::Info, "嵌入归档素材")?;
    }

    let video_path = completed_resource_by_intent(&latest, DownloadResourceIntent::Video)
        .map(|resource| resource.target_path.clone());
    let audio_path = completed_resource_by_intent(&latest, DownloadResourceIntent::Audio)
        .map(|resource| resource.target_path.clone());
    let muxer = MediaMuxer::new(MediaMuxerConfig {
        ffmpeg_path: runtime_options.ffmpeg_path.clone(),
    })
    .map_err(BdlError::from)?;
    muxer
        .mux(&MuxRequest {
            video_path,
            audio_path,
            output_path: latest.output_path.clone(),
            cover_path: attachments.cover_path,
            subtitle_paths: attachments.subtitle_paths,
        })
        .await
        .map_err(BdlError::from)?;

    let completed = state.update_task_status(&task.id, TaskStatus::Completed)?;
    events::emit(app, events::QUEUE_TASK_UPDATED, &completed)?;
    emit_queue_log(app, state, &task.id, QueueLogLevel::Info, "下载完成")?;

    if !runtime_options.retain_raw_streams {
        cleanup_raw_streams(app, state, &task).await?;
    }

    finalize_archive_assets(app, state, &task).await?;

    if let Err(error) = state.record_completed_task(&completed) {
        tracing::warn!("failed to save completed task record: {error}");
        let _ = emit_queue_log(
            app,
            state,
            &task.id,
            QueueLogLevel::Warning,
            &format!("完成记录保存失败：{error}"),
        );
    }

    Ok(())
}

async fn has_recoverable_completed_output(task: &DownloadTask) -> CommandResult<bool> {
    let media_resources = task.resources.iter().filter(|resource| {
        matches!(
            resource.intent,
            DownloadResourceIntent::Video | DownloadResourceIntent::Audio
        )
    });
    let media_resources = media_resources.collect::<Vec<_>>();
    if media_resources.is_empty()
        || media_resources
            .iter()
            .any(|resource| resource.status != ResourceStatus::Completed)
    {
        return Ok(false);
    }

    let Ok(output_metadata) = fs::metadata(&task.output_path).await else {
        return Ok(false);
    };
    if output_metadata.len() == 0 {
        return Ok(false);
    }

    Ok(media_resources
        .iter()
        .all(|resource| !resource.target_path.is_file()))
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

fn progress_sender(app: &AppHandle, task_id: &str) -> ProgressSender {
    let (sender, mut receiver) = unbounded_channel::<FetchProgress>();
    let app = app.clone();
    let task_id = task_id.to_owned();

    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(200));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut latest = None;

        loop {
            tokio::select! {
                progress = receiver.recv() => match progress {
                    Some(progress) => latest = Some(progress),
                    None => {
                        if let Some(progress) = latest.take() {
                            emit_progress(&app, &task_id, progress);
                        }
                        break;
                    }
                },
                _ = interval.tick() => {
                    if let Some(progress) = latest.take() {
                        emit_progress(&app, &task_id, progress);
                    }
                }
            }
        }
    });

    sender
}

fn emit_progress(app: &AppHandle, task_id: &str, progress: FetchProgress) {
    let entry = QueueProgressEntry {
        task_id: task_id.to_owned(),
        resource_id: progress.resource_id,
        downloaded_bytes: progress.downloaded_bytes,
        total_bytes: progress.total_bytes,
        created_at: Utc::now().to_rfc3339(),
    };
    if let Err(error) = events::emit(app, events::QUEUE_PROGRESS_UPDATED, &entry) {
        tracing::warn!("failed to emit queue progress: {}", error.message);
    }
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

fn task_mux_label(task: &DownloadTask) -> &'static str {
    match (
        task_has_resource_intent(task, DownloadResourceIntent::Video),
        task_has_resource_intent(task, DownloadResourceIntent::Audio),
    ) {
        (true, true) => "合并音视频",
        (true, false) => "封装视频",
        (false, true) => "封装音频",
        (false, false) => "封装媒体",
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

fn segment_count(settings: &SettingsSnapshot) -> usize {
    match settings.segment_count {
        1 | 2 | 4 | 8 => settings.segment_count,
        _ => 1,
    }
}

fn is_expired_url_error(message: &str) -> bool {
    message.contains("HTTP 404") || message.contains("资源长度失败")
}

fn is_login_expired_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("http 401")
        || lower.contains("http 403")
        || lower.contains("forbidden")
        || lower.contains("unauthorized")
        || message.contains("登录")
        || message.contains("Cookie")
        || message.contains("权限")
}

fn is_private_resource_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("private")
        || message.contains("私密")
        || message.contains("不可见")
        || message.contains("无权访问")
        || message.contains("访问受限")
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
    use super::{
        has_recoverable_completed_output, parse_future_schedule, queue_task_launch_context,
        redact_log_for_diagnostics, redact_task_for_diagnostics, should_fetch,
    };
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use bdl_core::BdlError;
    use bdl_core::model::HeaderPair;
    use bdl_core::muxer::MuxError;
    use bdl_core::queue::{
        DownloadResource, DownloadResourceIntent, DownloadResourceKind, DownloadTask,
        DownloadTaskMediaSelection, QueueLogEntry, QueueLogLevel, ResourceStatus, TaskStatus,
    };
    use chrono::{TimeZone, Utc};

    #[test]
    fn future_schedule_parses_and_normalizes_to_utc() {
        let now = Utc.with_ymd_and_hms(2026, 7, 10, 10, 0, 0).unwrap();

        let scheduled = parse_future_schedule(Some("2026-07-10T20:00:00+08:00"), now)
            .expect("future schedule should parse")
            .expect("schedule should be present");

        assert_eq!(
            scheduled,
            Utc.with_ymd_and_hms(2026, 7, 10, 12, 0, 0).unwrap()
        );
    }

    #[test]
    fn past_schedule_is_rejected() {
        let now = Utc.with_ymd_and_hms(2026, 7, 10, 12, 0, 0).unwrap();

        let result = parse_future_schedule(Some("2026-07-10T19:00:00+08:00"), now);

        assert!(result.is_err());
    }

    #[test]
    fn zero_speed_limit_normalizes_to_unlimited() {
        assert_eq!(
            super::parse_speed_limit(Some(0), "单任务下载限速")
                .expect("zero should mean unlimited"),
            None
        );
    }

    #[test]
    fn excessive_task_speed_limit_is_rejected() {
        let error = super::parse_speed_limit(Some(10 * 1024 * 1024 * 1024 + 1), "单任务下载限速")
            .expect_err("excessive task limit should fail");

        assert!(error.message.contains("单任务下载限速"));
    }

    #[test]
    fn task_launch_context_uses_the_supplied_latest_settings() {
        let settings = super::SettingsSnapshot {
            proxy_url: Some("http://127.0.0.1:7890".to_owned()),
            retry_count: 5,
            segment_count: 8,
            ffmpeg_path: Some("C:/tools/ffmpeg.exe".to_owned()),
            retain_raw_streams: true,
            embed_cover: true,
            embed_subtitles: true,
            ..Default::default()
        };

        let launch = queue_task_launch_context(settings.clone(), Some(3 * 1024 * 1024));

        assert_eq!(launch.settings, settings);
        assert_eq!(launch.fetch_config.max_retries, 5);
        assert_eq!(
            launch.fetch_config.proxy_url.as_deref(),
            Some("http://127.0.0.1:7890")
        );
        assert_eq!(launch.fetch_config.segment_count, 8);
        assert_eq!(
            launch.fetch_config.speed_limit_bytes_per_second,
            Some(3 * 1024 * 1024)
        );
        assert_eq!(
            launch.runtime_options.ffmpeg_path.as_deref(),
            Some(std::path::Path::new("C:/tools/ffmpeg.exe"))
        );
        assert!(launch.runtime_options.retain_raw_streams);
        assert!(launch.runtime_options.embed_cover);
        assert!(launch.runtime_options.embed_subtitles);
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

    #[tokio::test]
    async fn completed_output_with_cleaned_media_inputs_is_recoverable() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should follow Unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("bdl-recover-output-{nonce}"));
        std::fs::create_dir_all(&dir).expect("test directory should be created");
        let output_path = dir.join("finished.mp4");
        std::fs::write(&output_path, b"muxed-output").expect("output fixture should be written");
        let mut video = resource(
            DownloadResourceIntent::Video,
            vec!["https://example.invalid/video".to_owned()],
        );
        video.status = ResourceStatus::Completed;
        video.target_path = dir.join("cleaned-video.m4s");
        let task = DownloadTask {
            id: "task:recover".to_owned(),
            title: "recover".to_owned(),
            source_id: "video:recover".to_owned(),
            status: TaskStatus::Paused,
            resources: vec![video],
            output_path,
            refresh_intent: None,
            media_selection: DownloadTaskMediaSelection::default(),
            scheduled_at: None,
            speed_limit_bytes_per_second: None,
        };

        assert!(
            has_recoverable_completed_output(&task)
                .await
                .expect("recovery check should succeed")
        );

        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn download_directory_health_reports_missing_path_without_creating_it() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should follow Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("bdl-health-missing-{nonce}"));

        let health = super::check_download_directory(path.clone()).await;

        assert_eq!(health.status, super::DownloadDirectoryStatus::Missing);
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn download_directory_health_rejects_file_path() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should follow Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("bdl-health-file-{nonce}"));
        std::fs::write(&path, b"not a directory").expect("fixture file should be written");

        let health = super::check_download_directory(path.clone()).await;

        assert_eq!(health.status, super::DownloadDirectoryStatus::NotDirectory);
        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn download_directory_health_accepts_writable_directory_and_cleans_probe() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should follow Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("bdl-health-ready-{nonce}"));
        std::fs::create_dir_all(&path).expect("fixture directory should be created");

        let health = super::check_download_directory(path.clone()).await;

        assert_eq!(health.status, super::DownloadDirectoryStatus::Ready);
        assert_eq!(std::fs::read_dir(&path).unwrap().count(), 0);
        let _ = std::fs::remove_dir_all(path);
    }

    #[test]
    fn diagnostics_redaction_removes_resource_urls_headers_and_log_secrets() {
        let mut task = DownloadTask {
            id: "task:fixture".to_owned(),
            title: "fixture".to_owned(),
            source_id: "video:fixture".to_owned(),
            status: TaskStatus::Failed,
            resources: vec![DownloadResource {
                id: "resource:video".to_owned(),
                kind: DownloadResourceKind::Video,
                intent: DownloadResourceIntent::Video,
                current_urls: vec![
                    "https://cdn.test/video.m4s?token=secret&deadline=123".to_owned(),
                ],
                headers: vec![
                    HeaderPair {
                        name: "Cookie".to_owned(),
                        value: "SESSDATA=secret".to_owned(),
                    },
                    HeaderPair {
                        name: "Authorization".to_owned(),
                        value: "Bearer secret-token".to_owned(),
                    },
                ],
                target_path: PathBuf::from("video.m4s"),
                temp_path: PathBuf::from("video.m4s.bdlpart"),
                status: ResourceStatus::Failed,
            }],
            output_path: PathBuf::from("downloads/fixture.mp4"),
            refresh_intent: None,
            media_selection: DownloadTaskMediaSelection::default(),
            scheduled_at: None,
            speed_limit_bytes_per_second: None,
        };

        task = redact_task_for_diagnostics(task);
        let log = redact_log_for_diagnostics(QueueLogEntry {
            task_id: "task:fixture".to_owned(),
            level: QueueLogLevel::Error,
            message: "Authorization: Bearer secret-token".to_owned(),
            created_at: "2026-07-09T12:00:00Z".to_owned(),
        });

        assert_eq!(
            task.resources[0].current_urls,
            ["https://cdn.test/video.m4s?<redacted>"]
        );
        assert_eq!(task.resources[0].headers[0].value, "<redacted>");
        assert_eq!(task.resources[0].headers[1].value, "<redacted>");
        assert!(!log.message.contains("secret-token"));
    }

    #[test]
    fn command_error_maps_action_specific_codes() {
        let parse = super::CommandError::from(BdlError::InvalidInput {
            message: "无法识别这个输入。".to_owned(),
        });
        let ffmpeg = super::CommandError::from(BdlError::Mux(MuxError::FfmpegNotFound {
            path: "ffmpeg".to_owned(),
        }));
        let permission = super::CommandError::from(BdlError::Io(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "denied",
        )));
        let login = super::CommandError::from(BdlError::Fetch {
            message: "HTTP 403 Forbidden".to_owned(),
        });
        let private = super::CommandError::from(BdlError::Bpi("稿件不可见".to_owned()));

        assert_eq!(parse.code, "parse_unrecognized");
        assert_eq!(ffmpeg.code, "missing_ffmpeg");
        assert_eq!(permission.code, "unwritable_save_directory");
        assert_eq!(login.code, "login_required");
        assert_eq!(private.code, "private_resource");
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
