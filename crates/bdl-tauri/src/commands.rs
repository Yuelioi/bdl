use bdl_core::BdlError;
use bdl_core::ids::SourceId;
use bdl_core::model::NormalizedSourceTree;
use bdl_core::queue::DownloadTask;
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
pub async fn selection_create_tasks() -> CommandResult<()> {
    unsupported("selection_create_tasks")
}

#[tauri::command]
pub fn queue_list(state: State<'_, AppState>) -> CommandResult<Vec<DownloadTask>> {
    Ok(state.queue_snapshot()?)
}

#[tauri::command]
pub async fn queue_pause() -> CommandResult<()> {
    unsupported("queue_pause")
}

#[tauri::command]
pub async fn queue_resume() -> CommandResult<()> {
    unsupported("queue_resume")
}

#[tauri::command]
pub async fn queue_cancel() -> CommandResult<()> {
    unsupported("queue_cancel")
}

#[tauri::command]
pub async fn queue_retry() -> CommandResult<()> {
    unsupported("queue_retry")
}

#[tauri::command]
pub async fn queue_remove() -> CommandResult<()> {
    unsupported("queue_remove")
}

#[tauri::command]
pub async fn queue_open_file() -> CommandResult<()> {
    unsupported("queue_open_file")
}

#[tauri::command]
pub async fn queue_open_dir() -> CommandResult<()> {
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
pub async fn account_login_qr_start() -> CommandResult<()> {
    unsupported("account_login_qr_start")
}

#[tauri::command]
pub async fn account_login_qr_poll() -> CommandResult<()> {
    unsupported("account_login_qr_poll")
}

#[tauri::command]
pub async fn account_import_cookie() -> CommandResult<()> {
    unsupported("account_import_cookie")
}

#[tauri::command]
pub async fn account_logout() -> CommandResult<()> {
    unsupported("account_logout")
}

#[tauri::command]
pub async fn account_verify() -> CommandResult<()> {
    unsupported("account_verify")
}

fn unsupported<T>(command: &'static str) -> CommandResult<T> {
    Err(CommandError {
        code: "unsupported".to_owned(),
        message: format!("command `{command}` is not implemented in this phase"),
    })
}
