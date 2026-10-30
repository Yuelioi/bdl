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
fn http_403_does_not_assume_login_failure() {
    let diagnostic = diagnose_task(
        &failed_task(ResourceStatus::Failed),
        &[log(QueueLogLevel::Error, "fetch error: HTTP 403 Forbidden")],
    );

    assert_eq!(diagnostic.summary, "源站拒绝访问");
    assert_eq!(
        diagnostic.recommended_action,
        RecommendedAction::InspectRawLog
    );
}

#[test]
fn permission_denied_reports_unwritable_save_directory() {
    let diagnostic = diagnose_task(
        &failed_task(ResourceStatus::Failed),
        &[log(QueueLogLevel::Error, "io error: Access is denied")],
    );

    assert_eq!(diagnostic.summary, "本地文件访问被拒绝");
    assert_eq!(
        diagnostic.recommended_action,
        RecommendedAction::InspectRawLog
    );
}

#[test]
fn private_resource_requires_account_or_access_check() {
    let diagnostic = diagnose_task(
        &failed_task(ResourceStatus::Failed),
        &[log(QueueLogLevel::Error, "稿件不可见或无权访问")],
    );

    assert_eq!(diagnostic.summary, "资源不可访问");
    assert_eq!(
        diagnostic.recommended_action,
        RecommendedAction::InspectRawLog
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

#[test]
fn shared_failure_cases_report_specific_causes() {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/failure-diagnostics.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let message = case["message"].as_str().unwrap();
        let diagnostic = diagnose_task(
            &failed_task(ResourceStatus::Failed),
            &[log(QueueLogLevel::Error, message)],
        );
        assert_eq!(
            diagnostic.summary,
            case["summary"].as_str().unwrap(),
            "{message}"
        );
        assert_eq!(
            serde_json::to_value(diagnostic.recommended_action).unwrap(),
            case["action"],
            "{message}"
        );
    }
}

#[test]
fn latest_attempt_does_not_inherit_previous_login_failure() {
    let mut newest = log(QueueLogLevel::Error, "下载请求失败: HTTP 403 Forbidden");
    newest.created_at = "2026-07-09T12:02:00Z".into();
    let mut retry = log(QueueLogLevel::Info, "重试任务");
    retry.created_at = "2026-07-09T12:01:00Z".into();
    let logs = vec![newest, retry, log(QueueLogLevel::Error, "Cookie 已失效")];
    assert_eq!(
        diagnose_task(&failed_task(ResourceStatus::Failed), &logs).summary,
        "源站拒绝访问"
    );
    let mut old_error = log(QueueLogLevel::Error, "Cookie 已失效");
    let mut retry = log(QueueLogLevel::Info, "重试任务");
    old_error.created_at = "2026-07-09T12:03:00Z".into();
    retry.created_at = old_error.created_at.clone();
    assert_eq!(
        diagnose_task(&failed_task(ResourceStatus::Failed), &[retry, old_error]).summary,
        "任务失败"
    );
}

fn failed_task(audio_status: ResourceStatus) -> DownloadTask {
    DownloadTask {
        id: "task:video:1".to_owned(),
        title: "示例视频".to_owned(),
        source_id: "source:1".to_owned(),
        status: TaskStatus::Failed,
        output_path: PathBuf::from("downloads/example.mp4"),
        export_target: None,
        refresh_intent: None,
        media_selection: DownloadTaskMediaSelection::default(),
        scheduled_at: None,
        speed_limit_bytes_per_second: None,
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
