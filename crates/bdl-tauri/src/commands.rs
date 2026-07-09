use std::path::PathBuf;

use bdl_core::BdlError;
use bdl_core::account::{QrLoginSession, QrLoginStatus, poll_qr_login, start_qr_login};
use bdl_core::ids::{PartId, SourceId};
use bdl_core::model::NormalizedSourceTree;
use bdl_core::planner::{ArchiveMode, DownloadOptions, plan_selected_parts};
use bdl_core::queue::{DownloadTask, TaskStatus};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

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
pub async fn parse_load_more() -> CommandResult<()> {
    unsupported("parse_load_more")
}

#[tauri::command]
pub async fn parse_load_all() -> CommandResult<()> {
    unsupported("parse_load_all")
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
pub fn selection_create_tasks(
    app: AppHandle,
    state: State<'_, AppState>,
    request: SelectionCreateTasksRequest,
) -> CommandResult<Vec<DownloadTask>> {
    let tree = state.source_snapshot(&SourceId(request.source_id))?;
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

    let tasks = plan_selected_parts(&tree, &selected_part_ids, &options)?;
    state.enqueue_tasks(tasks.clone())?;

    for task in &tasks {
        events::emit(&app, events::QUEUE_TASK_UPDATED, task)?;
    }

    Ok(tasks)
}

#[tauri::command]
pub fn queue_list(state: State<'_, AppState>) -> CommandResult<Vec<DownloadTask>> {
    Ok(state.queue_snapshot()?)
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
    update_task_status(app, state, &task_id, TaskStatus::Waiting)
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
pub async fn queue_open_file(_task_id: String) -> CommandResult<()> {
    unsupported("queue_open_file")
}

#[tauri::command]
pub async fn queue_open_dir(_task_id: String) -> CommandResult<()> {
    unsupported("queue_open_dir")
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
    Ok(task)
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
