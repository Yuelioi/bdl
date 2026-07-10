use std::path::PathBuf;

use bdl_core::queue::{
    DownloadResourceIntent, DownloadTask, DownloadTaskMediaSelection, ResourceStatus, TaskStatus,
};
use chrono::{Duration, TimeZone, Utc};

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
fn waiting_task_cannot_start_before_its_schedule() {
    let now = Utc.with_ymd_and_hms(2026, 7, 10, 12, 0, 0).unwrap();
    let mut task = waiting_task();
    task.scheduled_at = Some(now + Duration::minutes(5));

    assert!(!task.can_start_at(now));
}

#[test]
fn waiting_task_can_start_when_its_schedule_is_due() {
    let now = Utc.with_ymd_and_hms(2026, 7, 10, 12, 0, 0).unwrap();
    let mut task = waiting_task();
    task.scheduled_at = Some(now);

    assert!(task.can_start_at(now));
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

fn waiting_task() -> DownloadTask {
    DownloadTask {
        id: "task:scheduled".to_owned(),
        title: "scheduled task".to_owned(),
        source_id: "video:scheduled".to_owned(),
        status: TaskStatus::Waiting,
        resources: Vec::new(),
        output_path: PathBuf::from("scheduled.mp4"),
        refresh_intent: None,
        media_selection: DownloadTaskMediaSelection::default(),
        scheduled_at: None,
        speed_limit_bytes_per_second: None,
    }
}
