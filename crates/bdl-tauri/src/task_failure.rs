use bdl_core::queue::QueueLogEntry;

pub(crate) fn should_refresh_media_url(code: bdl_core::public_error::ErrorCode) -> bool {
    use bdl_core::public_error::ErrorCode;
    matches!(
        code,
        ErrorCode::DownloadUrlExpired | ErrorCode::DownloadForbidden
    )
}

pub(crate) fn recent_auto_refresh_attempt_count(logs: &[QueueLogEntry]) -> usize {
    logs.iter()
        .take_while(|log| log.message != "重试任务")
        .filter(|log| {
            log.message == "自动刷新过期链接" || log.message.starts_with("自动刷新过期链接（")
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::{recent_auto_refresh_attempt_count, should_refresh_media_url};
    use bdl_core::queue::{QueueLogEntry, QueueLogLevel};

    fn log(message: &str) -> QueueLogEntry {
        QueueLogEntry {
            problem: None,
            task_id: "task:test".to_owned(),
            level: QueueLogLevel::Warning,
            message: message.to_owned(),
            created_at: "2026-09-25T00:00:00Z".to_owned(),
        }
    }

    #[test]
    fn classifies_refreshable_expired_media_errors() {
        use bdl_core::public_error::ErrorCode;
        assert!(should_refresh_media_url(ErrorCode::DownloadUrlExpired));
        assert!(should_refresh_media_url(ErrorCode::DownloadForbidden));
        for code in [
            ErrorCode::SourceForbidden,
            ErrorCode::SourceVerificationRequired,
            ErrorCode::NetworkTimeout,
            ErrorCode::SourceRateLimited,
            ErrorCode::FileDiskFull,
        ] {
            assert!(!should_refresh_media_url(code));
        }
    }

    #[test]
    fn counts_only_auto_refreshes_since_the_latest_manual_retry() {
        let logs = vec![
            log("开始下载任务"),
            log("自动刷新过期链接（2/3）"),
            log("自动刷新过期链接（1/3）"),
            log("重试任务"),
            log("自动刷新过期链接"),
        ];

        assert_eq!(recent_auto_refresh_attempt_count(&logs), 2);
    }
}
