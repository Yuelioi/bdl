use bdl_core::queue::QueueLogEntry;

pub(crate) fn is_expired_url_error(message: &str) -> bool {
    message.contains("HTTP 404") || message.contains("资源长度失败")
}

pub(crate) fn is_login_expired_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("http 401")
        || lower.contains("http 403")
        || lower.contains("forbidden")
        || lower.contains("unauthorized")
        || message.contains("登录")
        || message.contains("Cookie")
        || message.contains("权限")
}

pub(crate) fn is_private_resource_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("private")
        || message.contains("私密")
        || message.contains("不可见")
        || message.contains("无权访问")
        || message.contains("访问受限")
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
    use super::{
        is_expired_url_error, is_login_expired_error, is_private_resource_error,
        recent_auto_refresh_attempt_count,
    };
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
        assert!(is_expired_url_error("HTTP 404 Not Found"));
        assert!(is_expired_url_error("读取资源长度失败"));
        assert!(!is_expired_url_error("network timeout"));
    }

    #[test]
    fn distinguishes_login_and_private_resource_failures() {
        assert!(is_login_expired_error("HTTP 403 Forbidden"));
        assert!(is_login_expired_error("Cookie 已失效"));
        assert!(is_private_resource_error("稿件不可见"));
        assert!(!is_private_resource_error("HTTP 403 Forbidden"));
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
