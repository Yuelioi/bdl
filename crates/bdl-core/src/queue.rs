use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{BdlError, BdlResult};
use crate::model::HeaderPair;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DuplicateTaskPolicy {
    Skip,
    Create,
    #[default]
    Ask,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Waiting,
    Parsing,
    Downloading,
    Muxing,
    Completed,
    Failed,
    Paused,
    Cancelled,
}

impl TaskStatus {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }

    pub fn can_start(self) -> bool {
        matches!(self, Self::Waiting)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResourceStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
    Paused,
    Cancelled,
}

impl ResourceStatus {
    pub fn can_start(self) -> bool {
        matches!(self, Self::Pending | Self::Failed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueueLogLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueLogEntry {
    pub task_id: String,
    pub level: QueueLogLevel,
    pub message: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadResourceKind {
    Video,
    Audio,
    Asset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadResourceIntent {
    Video,
    Audio,
    Cover,
    Subtitle,
    Danmaku,
    Nfo,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: String,
    pub title: String,
    pub source_id: String,
    pub status: TaskStatus,
    pub resources: Vec<DownloadResource>,
    pub output_path: PathBuf,
    #[serde(default)]
    pub refresh_intent: Option<DownloadTaskRefreshIntent>,
    #[serde(default)]
    pub media_selection: DownloadTaskMediaSelection,
    #[serde(default)]
    pub scheduled_at: Option<DateTime<Utc>>,
}

impl DownloadTask {
    pub fn can_start_at(&self, now: DateTime<Utc>) -> bool {
        self.status.can_start()
            && self
                .scheduled_at
                .as_ref()
                .is_none_or(|scheduled_at| scheduled_at <= &now)
    }

    pub fn logical_id(&self) -> &str {
        logical_task_id(&self.id)
    }

    pub fn into_duplicate_copy(
        mut self,
        copy_index: usize,
        output_path: PathBuf,
    ) -> BdlResult<Self> {
        let original_id = self.id.clone();
        let copy_id = format!("{}:copy:{copy_index}", self.logical_id());
        let original_output = self.output_path.clone();

        for resource in &mut self.resources {
            let suffix =
                resource
                    .id
                    .strip_prefix(&original_id)
                    .ok_or_else(|| BdlError::Planning {
                        message: format!(
                            "资源 `{}` 不属于任务 `{original_id}`，无法创建重复任务。",
                            resource.id
                        ),
                    })?;
            resource.id = format!("{copy_id}{suffix}");
            resource.target_path =
                retarget_task_path(&resource.target_path, &original_output, &output_path)?;
            resource.temp_path =
                retarget_task_path(&resource.temp_path, &original_output, &output_path)?;
        }

        self.id = copy_id;
        self.output_path = output_path;
        Ok(self)
    }
}

pub fn logical_task_id(task_id: &str) -> &str {
    let Some((logical_id, copy_index)) = task_id.rsplit_once(":copy:") else {
        return task_id;
    };
    if copy_index.parse::<usize>().is_ok() {
        logical_id
    } else {
        task_id
    }
}

fn retarget_task_path(path: &Path, old_output: &Path, new_output: &Path) -> BdlResult<PathBuf> {
    let old_stem = old_output
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| BdlError::Planning {
            message: format!("输出路径 `{}` 没有有效文件名。", old_output.display()),
        })?;
    let new_stem = new_output
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| BdlError::Planning {
            message: format!("输出路径 `{}` 没有有效文件名。", new_output.display()),
        })?;
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(|value| value.strip_prefix(old_stem))
        .ok_or_else(|| BdlError::Planning {
            message: format!("资源路径 `{}` 与输出文件名不匹配。", path.display()),
        })?;

    Ok(new_output.with_file_name(format!("{new_stem}{file_name}")))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadTaskRefreshIntent {
    pub input: DownloadTaskRefreshInput,
    pub cid: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DownloadTaskRefreshInput {
    VideoBvid { bvid: String },
    VideoAid { aid: u64 },
    BangumiEpisode { ep_id: u64 },
    CheeseEpisode { ep_id: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadTaskMediaSelection {
    pub video_quality: String,
    pub audio_quality: String,
    pub video_codec: String,
    pub container: String,
}

impl Default for DownloadTaskMediaSelection {
    fn default() -> Self {
        Self {
            video_quality: "unknown".to_owned(),
            audio_quality: "unknown".to_owned(),
            video_codec: "unknown".to_owned(),
            container: "unknown".to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadResource {
    pub id: String,
    pub kind: DownloadResourceKind,
    pub intent: DownloadResourceIntent,
    pub current_urls: Vec<String>,
    pub headers: Vec<HeaderPair>,
    pub target_path: PathBuf,
    pub temp_path: PathBuf,
    pub status: ResourceStatus,
}
