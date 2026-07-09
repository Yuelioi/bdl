use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::model::HeaderPair;

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
