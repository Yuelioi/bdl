use serde::{Deserialize, Serialize};

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

pub fn diagnose_task(task: &DownloadTask, logs: &[QueueLogEntry]) -> TaskDiagnostic {
    let failed_intent = task.resources.iter().find_map(|resource| {
        matches!(
            resource.status,
            ResourceStatus::Failed | ResourceStatus::Cancelled
        )
        .then_some(resource.intent)
    });
    let error_text = logs
        .iter()
        .filter(|log| log.level == QueueLogLevel::Error)
        .map(|log| log.message.as_str())
        .collect::<Vec<_>>()
        .join("\n")
        .to_lowercase();

    if contains_any(&error_text, &["404", "资源长度失败"]) {
        return TaskDiagnostic {
            summary: "链接可能已过期".to_owned(),
            detail: "B 站媒体直链有时效性，重新解析或刷新下载地址后再重试。".to_owned(),
            recommended_action: RecommendedAction::RefreshUrlsAndRetry,
            failed_intent,
        };
    }

    if contains_any(&error_text, &["403", "permission", "forbidden", "权限"]) {
        return TaskDiagnostic {
            summary: "权限或登录状态异常".to_owned(),
            detail: "当前账号可能未登录、Cookie 已失效，或该清晰度需要登录/VIP 权限。".to_owned(),
            recommended_action: RecommendedAction::LoginThenRetry,
            failed_intent,
        };
    }

    if contains_any(
        &error_text,
        &["ffmpeg not found", "ffmpeg-not-found", "未找到 ffmpeg"],
    ) {
        return TaskDiagnostic {
            summary: "未找到 FFmpeg".to_owned(),
            detail: "未检测到可用的 FFmpeg，需要配置后才能合并音视频。".to_owned(),
            recommended_action: RecommendedAction::ConfigureFfmpeg,
            failed_intent,
        };
    }

    if contains_any(&error_text, &["ffmpeg", "mux", "合并"]) {
        return TaskDiagnostic {
            summary: "合并失败".to_owned(),
            detail: "音视频后处理失败，需要查看原始日志确认 FFmpeg 输出。".to_owned(),
            recommended_action: RecommendedAction::InspectRawLog,
            failed_intent,
        };
    }

    if contains_any(&error_text, &["timeout", "timed out", "超时"]) {
        return TaskDiagnostic {
            summary: "网络超时".to_owned(),
            detail: "请求在限定时间内没有完成，通常可以直接重试。".to_owned(),
            recommended_action: RecommendedAction::Retry,
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

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}
