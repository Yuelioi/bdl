use std::path::PathBuf;

use bdl_core::diagnostics::{RecommendedAction, diagnose_task};
use bdl_core::queue::{
    DownloadResource, DownloadResourceIntent, DownloadResourceKind, DownloadTask,
    DownloadTaskMediaSelection, QueueLogEntry, QueueLogLevel, ResourceStatus, TaskStatus,
};

#[test]
fn http_404_resource_length_failure_refreshes_urls() {
    let task = failed_task(ResourceStatus::Failed);
    let diagnostic = diagnose_task(
        &task,
        &[log(
            QueueLogLevel::Error,
            "请求资源长度失败: HTTP 404 Not Found",
        )],
    );

    assert_eq!(diagnostic.summary, "链接可能已过期");
    assert_eq!(
        diagnostic.recommended_action,
        RecommendedAction::RefreshUrlsAndRetry
    );
    assert_eq!(
        diagnostic.failed_intent,
        Some(DownloadResourceIntent::Audio)
    );
}

#[test]
fn http_403_requires_login_then_retry() {
    let diagnostic = diagnose_task(
        &failed_task(ResourceStatus::Failed),
        &[log(QueueLogLevel::Error, "fetch error: HTTP 403 Forbidden")],
    );

    assert_eq!(diagnostic.summary, "权限或登录状态异常");
    assert_eq!(
        diagnostic.recommended_action,
        RecommendedAction::LoginThenRetry
    );
}

#[test]
fn ffmpeg_not_found_requires_configuration() {
    let diagnostic = diagnose_task(
        &failed_task(ResourceStatus::Completed),
        &[log(QueueLogLevel::Error, "ffmpeg not found in PATH")],
    );

    assert_eq!(diagnostic.summary, "未找到 FFmpeg");
    assert_eq!(
        diagnostic.recommended_action,
        RecommendedAction::ConfigureFfmpeg
    );
}

#[test]
fn merge_command_failure_keeps_raw_log_action() {
    let diagnostic = diagnose_task(
        &failed_task(ResourceStatus::Completed),
        &[log(
            QueueLogLevel::Error,
            "合并音视频失败: ffmpeg exited with code 1",
        )],
    );

    assert_eq!(diagnostic.summary, "合并失败");
    assert_eq!(
        diagnostic.recommended_action,
        RecommendedAction::InspectRawLog
    );
}

#[test]
fn network_timeout_is_plain_retry() {
    let diagnostic = diagnose_task(
        &failed_task(ResourceStatus::Failed),
        &[log(QueueLogLevel::Error, "request timed out after 30s")],
    );

    assert_eq!(diagnostic.summary, "网络超时");
    assert_eq!(diagnostic.recommended_action, RecommendedAction::Retry);
}

fn failed_task(audio_status: ResourceStatus) -> DownloadTask {
    DownloadTask {
        id: "task:video:1".to_owned(),
        title: "示例视频".to_owned(),
        source_id: "source:1".to_owned(),
        status: TaskStatus::Failed,
        output_path: PathBuf::from("downloads/example.mp4"),
        media_selection: DownloadTaskMediaSelection::default(),
        resources: vec![
            resource(
                "resource:video",
                DownloadResourceKind::Video,
                DownloadResourceIntent::Video,
                ResourceStatus::Completed,
            ),
            resource(
                "resource:audio",
                DownloadResourceKind::Audio,
                DownloadResourceIntent::Audio,
                audio_status,
            ),
        ],
    }
}

fn resource(
    id: &str,
    kind: DownloadResourceKind,
    intent: DownloadResourceIntent,
    status: ResourceStatus,
) -> DownloadResource {
    DownloadResource {
        id: id.to_owned(),
        kind,
        intent,
        current_urls: Vec::new(),
        headers: Vec::new(),
        target_path: PathBuf::from(format!("{id}.m4s")),
        temp_path: PathBuf::from(format!("{id}.bdlpart")),
        status,
    }
}

fn log(level: QueueLogLevel, message: &str) -> QueueLogEntry {
    QueueLogEntry {
        task_id: "task:video:1".to_owned(),
        level,
        message: message.to_owned(),
        created_at: "2026-07-09T12:00:00Z".to_owned(),
    }
}
