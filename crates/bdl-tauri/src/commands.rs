use std::path::{Path, PathBuf};

use bdl_core::BdlError;
use bdl_core::account::{QrLoginSession, QrLoginStatus, poll_qr_login, start_qr_login};
use bdl_core::fetcher::{Fetcher, ReqwestFetcher};
use bdl_core::ids::{PartId, SourceId};
use bdl_core::model::NormalizedSourceTree;
use bdl_core::muxer::{MediaMuxer, MediaMuxerConfig, MuxRequest};
use bdl_core::planner::{ArchiveMode, DownloadOptions, plan_selected_parts};
use bdl_core::queue::{
    DownloadResource, DownloadResourceIntent, DownloadTask, QueueLogEntry, QueueLogLevel,
    ResourceStatus, TaskStatus,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

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
pub async fn parse_refresh_source() -> CommandResult<()> {
    unsupported("parse_refresh_source")
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
    if let Some(extension) = request.output_extension {
        options.output_extension = extension;
    }

    let prepared = state
        .prepare_selection(&source_id, &selected_part_ids)
        .await?;
    let tasks = plan_selected_parts(&prepared.tree, &prepared.part_ids, &options)?;
    state.enqueue_tasks(tasks.clone())?;

    if prepared.tree_updated {
        events::emit(&app, events::PARSE_SOURCE_UPDATED, &prepared.tree)?;
    }

    for task in &tasks {
        events::emit(&app, events::QUEUE_TASK_UPDATED, task)?;
    }
    start_queue_worker(&app);

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
pub async fn queue_retry(
    app: AppHandle,
    state: State<'_, AppState>,
    task_id: String,
) -> CommandResult<DownloadTask> {
    emit_queue_log(
        &app,
        state.inner(),
        &task_id,
        QueueLogLevel::Info,
        "刷新下载地址",
    )?;
    if let Err(error) = state.refresh_task_media_urls(&task_id).await {
        emit_queue_log(
            &app,
            state.inner(),
            &task_id,
            QueueLogLevel::Error,
            &format!("刷新下载地址失败：{error}"),
        )?;
        return Err(error.into());
    }

    let task = state.retry_task(&task_id)?;
    events::emit(&app, events::QUEUE_TASK_UPDATED, &task)?;
    emit_queue_log(
        &app,
        state.inner(),
        &task_id,
        QueueLogLevel::Info,
        "重试任务",
    )?;
    start_queue_worker(&app);
    Ok(task)
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

fn unsupported<T>(command: &'static str) -> CommandResult<T> {
    Err(CommandError {
        code: "unsupported".to_owned(),
        message: format!("command `{command}` is not implemented in this phase"),
    })
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
    let fetcher = ReqwestFetcher::new()?;

    loop {
        let Some(task) = state.take_next_startable_task()? else {
            break;
        };

        events::emit(app, events::QUEUE_TASK_UPDATED, &task)?;
        emit_queue_log(app, state, &task.id, QueueLogLevel::Info, "开始下载任务")?;

        if let Err(error) = run_download_task(app, state, &fetcher, task.clone()).await {
            match state.task_status(&task.id) {
                Ok(TaskStatus::Paused | TaskStatus::Cancelled) => {
                    emit_queue_log(app, state, &task.id, QueueLogLevel::Warning, "任务已停止")?;
                }
                _ => {
                    let failed = state.update_task_status(&task.id, TaskStatus::Failed)?;
                    events::emit(app, events::QUEUE_TASK_UPDATED, &failed)?;
                    emit_queue_log(app, state, &task.id, QueueLogLevel::Error, &error.message)?;
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

    let video = resource_by_intent(&task, DownloadResourceIntent::Video)?;
    let audio = resource_by_intent(&task, DownloadResourceIntent::Audio)?;
    let muxer = MediaMuxer::new(MediaMuxerConfig::default()).map_err(BdlError::from)?;
    muxer
        .mux(&MuxRequest {
            video_path: video.target_path.clone(),
            audio_path: audio.target_path.clone(),
            output_path: task.output_path.clone(),
        })
        .await
        .map_err(BdlError::from)?;

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
    ) && !resource.current_urls.is_empty()
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

fn emit_queue_log(
    app: &AppHandle,
    state: &AppState,
    task_id: &str,
    level: QueueLogLevel,
    message: &str,
) -> CommandResult<()> {
    let entry = state.append_task_log(QueueLogEntry {
        task_id: task_id.to_owned(),
        level,
        message: message.to_owned(),
        created_at: Utc::now().to_rfc3339(),
    })?;
    events::emit(app, events::QUEUE_LOG_APPENDED, &entry)
}

fn parse_archive_mode(value: &str) -> CommandResult<ArchiveMode> {
    match value {
        "fast" => Ok(ArchiveMode::Fast),
        "complete_archive" => Ok(ArchiveMode::CompleteArchive),
        other => Err(CommandError {
            code: "invalid_archive_mode".to_owned(),
            message: format!("unknown archive mode `{other}`"),
        }),
    }
}
