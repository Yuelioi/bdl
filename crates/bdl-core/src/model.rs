use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::ids::{GroupId, ItemId, PartId, SourceId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Video,
    Bangumi,
    Cheese,
    Favorite,
    Collection,
    Series,
    Uploader,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceSummary {
    pub id: SourceId,
    pub kind: SourceKind,
    pub input: String,
    pub title: String,
    pub loaded_count: usize,
    pub total_count: Option<usize>,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedSourceTree {
    pub source: SourceSummary,
    pub groups: Vec<NormalizedGroup>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedGroup {
    pub id: GroupId,
    pub kind: String,
    pub title: String,
    pub items: Vec<NormalizedItem>,
    pub page: Option<PageState>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageState {
    pub page_number: u32,
    pub page_size: u32,
    pub loaded_count: usize,
    pub total_count: Option<usize>,
    pub has_more: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedItem {
    pub id: ItemId,
    pub title: String,
    pub owner_name: Option<String>,
    pub cover_url: Option<String>,
    pub duration_seconds: Option<u64>,
    pub parts: Vec<NormalizedPart>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedPart {
    pub id: PartId,
    pub title: String,
    pub aid: Option<u64>,
    pub bvid: Option<String>,
    pub cid: Option<u64>,
    pub streams: Vec<MediaStream>,
    pub assets: Vec<DerivedAsset>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaStream {
    pub id: String,
    pub kind: MediaKind,
    pub quality: StreamQuality,
    pub codec: StreamCodec,
    pub bandwidth: Option<u64>,
    pub urls: Vec<String>,
    pub headers: Vec<HeaderPair>,
    pub acquired_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderPair {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Video,
    Audio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum StreamQuality {
    Best,
    Quality(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StreamCodec {
    Auto,
    Avc,
    Hevc,
    Av1,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Cover,
    Subtitle,
    Danmaku,
    Nfo,
}

impl AssetKind {
    pub fn with_policy(self, fetch_policy: FetchPolicy) -> DerivedAsset {
        DerivedAsset {
            kind: self,
            format: None,
            fetch_policy,
            urls: Vec::new(),
            headers: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedAsset {
    pub kind: AssetKind,
    pub format: Option<String>,
    pub fetch_policy: FetchPolicy,
    #[serde(default)]
    pub urls: Vec<String>,
    #[serde(default)]
    pub headers: Vec<HeaderPair>,
}

impl DerivedAsset {
    pub fn with_urls(
        kind: AssetKind,
        fetch_policy: FetchPolicy,
        format: impl Into<String>,
        urls: Vec<String>,
        headers: Vec<HeaderPair>,
    ) -> Self {
        Self {
            kind,
            format: Some(format.into()),
            fetch_policy,
            urls,
            headers,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FetchPolicy {
    Never,
    OnDemand,
    Always,
}
