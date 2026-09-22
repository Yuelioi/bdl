use serde::{Deserialize, Serialize};

use crate::ids::{Aid, Bvid, Cid, Mid};

/// `/x/web-interface/view` 返回的载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoView {
    /// AV 数字视频 ID。
    pub aid: Aid,
    /// BV 字符串视频 ID。
    pub bvid: Bvid,
    /// 视频分 P 数。
    pub videos: u32,
    /// 视频标题。
    pub title: String,
    /// 视频封面 URL。
    pub pic: String,
    /// UP 主信息。
    pub owner: VideoOwner,
    /// 视频统计。
    pub stat: VideoStat,
    /// 默认内容或分 P ID。
    pub cid: Cid,
    /// 分 P 列表。
    #[serde(default)]
    pub pages: Vec<VideoPage>,
}

/// `/x/web-interface/view/detail` 返回的载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoDetail {
    /// 主视频 view 载荷。
    #[serde(rename = "View")]
    pub view: VideoView,
    /// 视频关联标签。
    #[serde(default, rename = "Tags")]
    pub tags: Vec<VideoTag>,
    /// Bilibili 推荐场景返回的相关视频。
    #[serde(default, rename = "Related")]
    pub related: Vec<VideoRelated>,
    /// UP 主卡片和空间数据。该载荷偏展示用途，因此保留原始值。
    #[serde(default, rename = "Card")]
    pub card: Option<serde_json::Value>,
    /// 评论预览数据。它会随评论场景实验变化，因此保留原始值。
    #[serde(default, rename = "Reply")]
    pub reply: Option<serde_json::Value>,
}

/// 视频 view 载荷中嵌入的 UP 主信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoOwner {
    /// UP 主 member ID。
    pub mid: Mid,
    /// UP 主显示名称。
    pub name: String,
    /// UP 主头像 URL。
    pub face: String,
}

/// 视频 view endpoint 返回的稳定统计字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoStat {
    /// AV 数字视频 ID。
    pub aid: Aid,
    /// 播放数。
    pub view: u64,
    /// 弹幕数。
    pub danmaku: u64,
    /// 回复数。
    pub reply: u64,
    /// 较新载荷中的收藏数。
    #[serde(default)]
    pub favorite: Option<u64>,
    /// 部分载荷中观察到的收藏数字段别名。
    #[serde(default)]
    pub fav: Option<u64>,
    /// 投币数。
    pub coin: u64,
    /// 分享数。
    pub share: u64,
    /// 点赞数。
    pub like: u64,
}

/// 多 P 视频中的一个分 P。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoPage {
    /// 内容或分 P ID。
    pub cid: Cid,
    /// 从 1 开始的分 P 索引。
    pub page: u32,
    /// 分 P 标题。
    pub part: String,
    /// 时长，单位秒。
    pub duration: u64,
}

/// detail endpoint 返回的一个视频标签。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoTag {
    /// 标签 ID。
    pub tag_id: u64,
    /// 标签显示名称。
    #[serde(default)]
    pub tag_name: String,
    /// 可选标签跳转 URL。
    #[serde(default)]
    pub jump_url: String,
}

/// detail endpoint 返回的相关视频稳定字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoRelated {
    /// AV 数字视频 ID。
    pub aid: Aid,
    /// BV 字符串视频 ID。
    pub bvid: Bvid,
    /// 相关视频标题。
    #[serde(default)]
    pub title: String,
    /// Bilibili 返回时的默认内容或分 P ID。
    #[serde(default)]
    pub cid: Option<Cid>,
    /// 存在时的 UP 主信息。
    #[serde(default)]
    pub owner: Option<VideoOwner>,
    /// 存在时的视频统计。
    #[serde(default)]
    pub stat: Option<VideoStat>,
}
