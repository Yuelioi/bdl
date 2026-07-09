use bdl_core::queue::{DownloadResourceIntent, ResourceStatus, TaskStatus};

#[test]
fn task_status_serializes_frontend_safe_values() {
    assert_eq!(
        serde_json::to_string(&TaskStatus::Waiting).unwrap(),
        "\"waiting\""
    );
    assert_eq!(
        serde_json::to_string(&TaskStatus::Downloading).unwrap(),
        "\"downloading\""
    );
    assert_eq!(
        serde_json::to_string(&TaskStatus::Completed).unwrap(),
        "\"completed\""
    );
}

#[test]
fn terminal_task_statuses_are_explicit() {
    assert!(TaskStatus::Completed.is_terminal());
    assert!(TaskStatus::Failed.is_terminal());
    assert!(TaskStatus::Cancelled.is_terminal());
    assert!(!TaskStatus::Waiting.is_terminal());
    assert!(!TaskStatus::Paused.is_terminal());
}

#[test]
fn only_waiting_tasks_can_enter_download_flow() {
    assert!(TaskStatus::Waiting.can_start());
    assert!(!TaskStatus::Failed.can_start());
    assert!(!TaskStatus::Downloading.can_start());
    assert!(!TaskStatus::Paused.can_start());
    assert!(!TaskStatus::Completed.can_start());
}

#[test]
fn pending_or_failed_resources_can_enter_download_flow() {
    assert!(ResourceStatus::Pending.can_start());
    assert!(ResourceStatus::Failed.can_start());
    assert!(!ResourceStatus::Downloading.can_start());
    assert!(!ResourceStatus::Completed.can_start());
}

#[test]
fn resource_intents_serialize_as_stable_strings() {
    assert_eq!(
        serde_json::to_string(&DownloadResourceIntent::Danmaku).unwrap(),
        "\"danmaku\""
    );
    assert_eq!(
        serde_json::to_string(&DownloadResourceIntent::Nfo).unwrap(),
        "\"nfo\""
    );
}
