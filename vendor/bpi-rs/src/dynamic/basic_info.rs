use crate::models::Vip;
use serde::{Deserialize, Serialize};
use serde_json::Value;
// 以下结构体为API文档中未完全列出的部分，根据描述进行了推断和简化。
// 如果您有完整的文档，请替换这些结构体。

/// 动态转发列表中的转发项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RepostItem {
    pub desc: Desc,
    pub card: String,
    pub extend_json: String,
    pub display: Display,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Desc {
    pub uid: i64,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub rid: i64,
    pub acl: i64,
    pub view: i64,
    pub repost: i64,
    pub comment: i64,
    pub like: i64,
    pub is_liked: i64,
    pub dynamic_id: i64,
    pub timestamp: i64,
    pub pre_dy_id: i64,
    pub orig_dy_id: i64,
    pub orig_type: i64,
    pub user_profile: UserProfile,
    pub spec_type: i64,
    pub uid_type: i64,
    pub stype: i64,
    pub r_type: i64,
    pub inner_id: i64,
    pub status: i64,
    pub dynamic_id_str: String,
    pub pre_dy_id_str: String,
    pub orig_dy_id_str: String,
    pub rid_str: String,
    pub origin: Origin,
    pub bvid: String,
    pub previous: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub info: Info,
    pub card: Card,
    pub vip: Vip,
    pub pendant: Pendant,
    pub rank: String,
    pub sign: String,
    pub level_info: LevelInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub uid: i64,
    pub uname: String,
    pub face: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub official_verify: OfficialVerify,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficialVerify {
    #[serde(rename = "type")]
    pub type_field: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub path: String,
    pub text: String,
    pub label_theme: String,
    pub text_color: String,
    pub bg_style: i64,
    pub bg_color: String,
    pub border_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pendant {
    pub pid: i64,
    pub name: String,
    pub image: String,
    pub expire: i64,
    pub image_enhance: String,
    pub image_enhance_frame: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelInfo {
    pub current_level: i64,
    pub current_min: i64,
    pub current_exp: i64,
    pub next_exp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Origin {
    pub uid: i64,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub rid: i64,
    pub acl: i64,
    pub view: i64,
    pub repost: i64,
    pub comment: i64,
    pub like: i64,
    pub is_liked: i64,
    pub dynamic_id: i64,
    pub timestamp: i64,
    pub pre_dy_id: i64,
    pub orig_dy_id: i64,
    pub orig_type: i64,
    pub user_profile: Value,
    pub spec_type: i64,
    pub uid_type: i64,
    pub stype: i64,
    pub r_type: i64,
    pub inner_id: i64,
    pub status: i64,
    pub dynamic_id_str: String,
    pub pre_dy_id_str: String,
    pub orig_dy_id_str: String,
    pub rid_str: String,
    pub origin: Value,
    pub bvid: String,
    pub previous: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Display {
    pub origin: Origin2,
    pub usr_action_txt: String,
    pub relation: Relation2,
    pub live_info: Value,
    pub emoji_info: EmojiInfo2,
    pub highlight: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Origin2 {
    pub origin: Value,
    pub usr_action_txt: String,
    pub relation: Relation,
    pub live_info: Value,
    pub emoji_info: EmojiInfo,
    pub highlight: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub status: i64,
    pub is_follow: i64,
    pub is_followed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmojiInfo {
    pub emoji_details: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation2 {
    pub status: i64,
    pub is_follow: i64,
    pub is_followed: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmojiInfo2 {
    pub emoji_details: Value,
}

/// 动态点赞列表中的点赞项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LikeItem {
    // 由于API文档未详细列出字段，这里作为占位符。
    // 请根据实际API响应填充此结构体。
}

/// 纯文本动态内容
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlainTextRequest {
    // 假设纯文本动态内容有一个名为 `content` 的字段。
    pub content: String,
    // 其他字段...
}

/// 获取草稿列表中的单项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Draft {
    /// 草稿id
    pub draft_id: String,
    /// 定时发送的秒级时间戳
    pub publish_time: u64,
    /// 动态类型
    #[serde(rename = "type")]
    pub type_num: u8,
    /// 自己的mid
    pub uid: u64,
    /// 自己的用户信息
    pub user_profile: UserProfile,
    /// 动态内容
    pub request: String,
}

/// 动态转发列表响应数据结构体
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RepostDetailResponseData {
    /// 是否还有下一页
    pub has_more: Option<bool>,
    /// 总计
    pub total: u64,
    /// 转发列表
    pub items: Vec<RepostItem>,
}

/// 动态点赞列表响应数据结构体
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpecItemLikesResponseData {
    /// 点赞信息列表主体
    pub item_likes: Vec<LikeItem>,
    /// 是否还有下一页
    pub has_more: bool,
    /// 总计点赞数
    pub total_count: u64,
    /// 作用尚不明确
    pub gt: u64,
}

/// 获取草稿列表响应数据结构体
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetDraftsResponseData {
    /// 草稿列表
    pub drafts: Vec<Draft>,
}
