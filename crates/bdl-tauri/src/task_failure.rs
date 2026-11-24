use bdl_core::queue::QueueLogEntry;

pub(crate) fn should_refresh_media_url(message: &str) -> bool {
    bdl_core::diagnostics::classify_failure(message).is_some_and(|rule| {
        rule.code == "media_url_expired"
            || (rule.code == "remote_forbidden"
                && (message.contains("[stage=download_视频 code=remote_forbidden]")
                    || message.contains("[stage=download_音频 code=remote_forbidden]")))
    })
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
            task_id: "task:test".to_owned(),
            level: QueueLogLevel::Warning,
            message: message.to_owned(),
            created_at: "2026-09-25T00:00:00Z".to_owned(),
        }
    }

    #[test]
    fn classifies_refreshable_expired_media_errors() {
        assert!(should_refresh_media_url("HTTP 404 Not Found"));
        assert!(!should_refresh_media_url(
            "读取资源长度失败: HTTP 403 Forbidden"
        ));
        assert!(should_refresh_media_url(
            "[stage=download_视频 code=remote_forbidden] HTTP 403 Forbidden"
        ));
        assert!(!should_refresh_media_url(
            "[stage=bilibili_api code=source_verification_required] HTTP 403 Forbidden"
        ));
        assert!(!should_refresh_media_url(
            "读取资源长度失败: request timed out"
        ));
        assert!(!should_refresh_media_url("network timeout"));
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
