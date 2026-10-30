use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

use crate::queue::{
    DownloadResourceIntent, DownloadTask, QueueLogEntry, QueueLogLevel, ResourceStatus, TaskStatus,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendedAction {
    Retry,
    RefreshUrlsAndRetry,
    LoginThenRetry,
    ConfigureFfmpeg,
    InspectRawLog,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaskDiagnostic {
    pub summary: String,
    pub detail: String,
    pub recommended_action: RecommendedAction,
    pub failed_intent: Option<DownloadResourceIntent>,
}

/// Shared with the frontend so persisted legacy logs and new errors use the same guidance.
#[derive(Debug, Deserialize)]
pub struct FailureRule {
    pub code: String,
    pub summary: String,
    pub detail: String,
    pub recommended_action: RecommendedAction,
    pub contains: Vec<String>,
    pub http_statuses: Vec<u16>,
}

fn rules() -> &'static [FailureRule] {
    static RULES: OnceLock<Vec<FailureRule>> = OnceLock::new();
    RULES.get_or_init(|| {
        serde_json::from_str(include_str!("diagnostic-rules.json"))
            .expect("bundled diagnostic rules must be valid")
    })
}

pub fn explicit_failure(message: &str) -> Option<&'static FailureRule> {
    let rules = rules();
    let lower = message.to_lowercase();
    // A producer's contextual code takes priority over incidental words in paths or causes.
    for marker in lower.split("code=").skip(1) {
        if let Some(rule) = marker
            .split_once(']')
            .and_then(|(code, _)| rules.iter().find(|rule| rule.code == code))
        {
            return Some(rule);
        }
    }
    None
}

pub fn classify_failure(message: &str) -> Option<&'static FailureRule> {
    if let Some(rule) = explicit_failure(message) {
        return Some(rule);
    }
    let lower = message.to_lowercase();
    rules().iter().find(|rule| {
        rule.contains.iter().any(|text| lower.contains(text))
            || rule
                .http_statuses
                .iter()
                .any(|status| has_http_status(&lower, *status))
    })
}

fn has_http_status(message: &str, status: u16) -> bool {
    ["http ", "status ", "status code ", "状态码: ", "状态码："]
        .iter()
        .any(|prefix| {
            let needle = format!("{prefix}{status}");
            message.match_indices(&needle).any(|(offset, _)| {
                !message
                    .as_bytes()
                    .get(offset + needle.len())
                    .is_some_and(u8::is_ascii_digit)
            })
        })
}

pub fn latest_failure(logs: &[QueueLogEntry]) -> Option<&str> {
    // Storage and live events are newest first. Preserve that order for equal
    // timestamps so an earlier error cannot cross a new attempt boundary.
    let mut ordered: Vec<_> = logs.iter().collect();
    ordered.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    for log in ordered {
        if matches!(
            log.message.as_str(),
            "重试任务" | "开始下载任务" | "刷新下载地址"
        ) {
            return None;
        }
        if log.level == QueueLogLevel::Error {
            return Some(log.message.as_str());
        }
    }
    None
}

pub fn diagnose_task(task: &DownloadTask, logs: &[QueueLogEntry]) -> TaskDiagnostic {
    let failed_intent = task.resources.iter().find_map(|resource| {
        matches!(
            resource.status,
            ResourceStatus::Failed | ResourceStatus::Cancelled
        )
        .then_some(resource.intent)
    });
    if let Some(rule) = latest_failure(logs).and_then(classify_failure) {
        return TaskDiagnostic {
            summary: rule.summary.clone(),
            detail: rule.detail.clone(),
            recommended_action: rule.recommended_action,
            failed_intent,
        };
    }

    if task.status == TaskStatus::Cancelled {
        return TaskDiagnostic {
            summary: "任务已取消".to_owned(),
            detail: "任务被取消，可以重新入队后继续尝试。".to_owned(),
            recommended_action: RecommendedAction::Retry,
            failed_intent,
        };
    }

    TaskDiagnostic {
        summary: if task.status == TaskStatus::Failed {
            "任务失败".to_owned()
        } else {
            "暂无异常".to_owned()
        },
        detail: "未识别到可分类的失败原因。".to_owned(),
        recommended_action: RecommendedAction::InspectRawLog,
        failed_intent,
    }
}
