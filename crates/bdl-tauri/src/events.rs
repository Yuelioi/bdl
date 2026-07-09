use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};

use crate::commands::CommandError;

pub const PARSE_SOURCE_UPDATED: &str = "parse://source-updated";
pub const PARSE_ITEMS_APPENDED: &str = "parse://items-appended";
pub const QUEUE_TASK_UPDATED: &str = "queue://task-updated";
pub const QUEUE_RESOURCE_UPDATED: &str = "queue://resource-updated";
pub const QUEUE_PROGRESS_UPDATED: &str = "queue://progress-updated";
pub const QUEUE_LOG_APPENDED: &str = "queue://log-appended";
pub const SETTINGS_UPDATED: &str = "settings://updated";
pub const ACCOUNT_UPDATED: &str = "account://updated";

pub fn emit<R, T>(app: &AppHandle<R>, event: &str, payload: &T) -> Result<(), CommandError>
where
    R: Runtime,
    T: Serialize + Clone,
{
    app.emit(event, payload).map_err(|error| CommandError {
        code: "event_emit_failed".to_owned(),
        message: error.to_string(),
    })
}
