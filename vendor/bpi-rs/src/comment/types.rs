//! 评论相关数据结构
//!
//! 本模块定义了评论系统相关的所有数据结构，包括评论内容、用户信息、表情等。

use std::collections::HashMap;

use crate::models::{LevelInfo, Nameplate, OfficialVerify, Pendant};
use serde::{Deserialize, Serialize};

/// 评论条目对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub rpid: i64, // 评论 rpid
    pub oid: i64,  // 评论区对象 id
    #[serde(rename = "type")]
    pub oid_type: i64, // 评论区类型代码（type）
    pub mid: i64,  // 评论发送者 mid
    pub root: i64, // 根评论 rpid，一级评论为0
    pub parent: i64, // 回复父评论 rpid
    pub dialog: i64, // 回复对方 rpid
    pub count: i64, // 二级评论条数
    pub rcount: i64, // 回复评论条数
    pub state: i64, // 评论状态，0正常，17隐藏
    pub fansgrade: i64, // 是否具有粉丝标签，0无，1有
    pub attr: i64, // 某属性位
    pub ctime: i64, // 评论发送时间戳
    pub like: i64, // 评论获赞数
    pub action: i64, // 当前用户操作状态，0无，1已点赞，2已点踩
    pub member: Member, // 评论发送者信息
    pub content: Content, // 评论内容信息
    pub up_action: UpAction, // 评论 UP 主操作信息
    pub invisible: bool, // 评论是否被隐藏
    pub reply_control: ReplyControl, // 评论提示文案信息
    pub folder: Folder, // 折叠信息

    pub floor: Option<u64>,
    pub show_follow: Option<bool>,
    pub card_label: Option<Vec<CardLabel>>,
    pub rpid_str: Option<String>,
    pub root_str: Option<String>,
    pub parent_str: Option<String>,
    pub dialog_str: Option<String>,
    pub mid_str: Option<String>,
    pub oid_str: Option<String>,
    pub replies: Option<Vec<Comment>>,
    pub assist: Option<u64>,

    pub dynamic_id_str: Option<String>,
    pub note_cvid_str: Option<String>,
    pub track_info: Option<String>,
}

/// 页信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageInfo {
    pub num: u64,            // 当前页码
    pub size: u64,           // 每页条数
    pub count: u64,          // 根评论条数
    pub acount: Option<u64>, // 总评论条数
}

/// 评论发送者信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub mid: String,                     // 发送者 mid
    pub uname: String,                   // 昵称
    pub sex: String,                     // 性别
    pub sign: String,                    // 用户签名
    pub avatar: String,                  // 头像 url
    pub level_info: LevelInfo,           // 等级信息
    pub pendant: Pendant,                // 头像框信息
    pub nameplate: Nameplate,            // 勋章信息
    pub official_verify: OfficialVerify, // 认证信息
    pub vip: Vip,                        // 大会员信息
    #[serde(default)]
    pub user_sailing: serde_json::Value, // 评论条目装饰信息
    #[serde(default)]
    pub is_contractor: bool, // 是否合作用户
    #[serde(default)]
    pub contract_desc: String, // 合作用户说明

    pub rank: Option<String>,
    pub display_rank: Option<String>,
    pub fans_detail_a: Option<FansDetail>, // 粉丝标签信息，仅 A 有
    pub following: Option<u64>,            // 是否关注该用户
    pub is_followed: Option<u64>,          // 是否被关注

    pub rank_b: Option<String>,    //
    pub face_nft_new: Option<i64>, // 是否有头像 NFT
    pub senior: Option<JumpUrl>,   // 高级会员信息
    pub fans_detail_b: Option<serde_json::Value>,
    pub user_sailing_v2: Option<JumpUrl>,           // 装饰信息
    pub nft_interaction: Option<serde_json::Value>, // NFT 交互信息
    pub avatar_item: Option<serde_json::Value>,     // 头像物品信息
}

/// 大会员信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vip {
    pub vip_type: i64,
    pub vip_due_date: i64,
    pub due_remark: String,
    pub access_status: i64,
    pub vip_status: i64,
    pub vip_status_warn: String,
    pub theme_type: i64,
    #[serde(rename = "avatar_subscript")]
    pub avatar_subscript: i64,
    #[serde(rename = "nickname_color")]
    pub nickname_color: String,
}

/// 会员铭牌样式
///
/// 粉丝标签信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FansDetail {
    pub uid: u64,
    pub medal_id: u64,
    pub medal_name: String,
    pub score: Option<u64>,
    pub level: u64,
    pub intimacy: Option<u64>,
    pub master_status: Option<u64>,
    pub is_receive: Option<u64>,
}

/// 评论条目装扮信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSailing {
    pub pendant: Option<Pendant>,
    pub cardbg: Option<CardBg>,
    pub cardbg_with_focus: Option<()>, // 待确认
}

/// 评论条目装扮信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardBg {
    pub id: u64,
    pub name: String,
    pub image: String,
    pub jump_url: Option<String>,
    pub fan: Option<FanInfo>,
    pub r#type: String, // suit/vip_suit
}

/// 粉丝专属装扮信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanInfo {
    pub is_fan: u64, // 0否 1是
    pub number: u64,
    pub color: String,
    pub name: String,
    pub num_desc: String,
}

/// 评论内容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub message: String,
    pub members: Option<Vec<Member>>,               // at 用户
    pub jump_url: Option<HashMap<String, JumpUrl>>, // 高亮超链，以超链转义符为键
    pub max_line: Option<u64>,

    pub plat: Option<u64>, // 发送端
    pub device: Option<String>,
    pub emote: Option<HashMap<String, Emote>>, // 表情转义，以表情转义符为键
    pub pictures: Option<Vec<Picture>>,
}

/// 单个表情对象
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emote {
    pub id: u64,                    // 表情 id
    pub package_id: u64,            // 表情包 id
    pub state: u64,                 // 0
    pub r#type: u64,                // 表情类型：1免费/2会员专属/3购买/4颜文字
    pub attr: Option<u64>,          // 待确认
    pub text: String,               // 表情转义符
    pub url: String,                // 表情图片 url
    pub meta: Option<EmoteMeta>,    // 属性信息
    pub mtime: Option<u64>,         // 表情创建时间戳
    pub jump_title: Option<String>, // 表情名称
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JumpUrl {
    pub title: Option<String>,            // 标题
    pub state: Option<u64>,               // 图标 url 或状态
    pub prefix_icon: Option<String>,      // 待确认
    pub app_url_schema: Option<String>,   // APP 跳转 schema
    pub app_name: Option<String>,         // APP 名称
    pub app_package_name: Option<String>, // APP 包名
    pub click_report: Option<String>,     // 上报 id
}

/// 表情属性信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmoteMeta {
    pub size: Option<u64>,     // 表情尺寸信息，1小/2大
    pub alias: Option<String>, // 简写名
}

/// 评论图片信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Picture {
    pub img_src: String,
    pub img_width: Option<u64>,
    pub img_height: Option<u64>,
    pub img_size: Option<f64>, // KB
}

/// 折叠信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub has_folded: bool,
    pub is_folded: bool,
    pub rule: String, // 相关规则页面 url
}

/// UP主操作信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpAction {
    pub like: bool,
    pub reply: bool,
}

/// 卡片标签信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardLabel {
    pub rpid: u64,
    pub text_content: String,
    pub text_color_day: String,
    pub text_color_night: String,
    pub label_color_day: String,
    pub label_color_night: String,
    pub image: Option<String>,
    pub r#type: Option<u64>,
    pub background: Option<String>,
    pub background_width: Option<u64>,
    pub background_height: Option<u64>,
    pub jump_url: Option<String>,
    pub effect: Option<u64>,
    pub effect_start_time: Option<u64>,
}

/// 回复提示文案信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyControl {
    pub sub_reply_entry_text: Option<String>,
    pub sub_reply_title_text: Option<String>,
    pub time_desc: Option<String>,
    pub location: Option<String>,
}

/// 评论区顶部信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Top {
    pub admin: serde_json::Value,
    pub upper: serde_json::Value,
    pub vote: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub showtopic: u32,
    pub show_up_flag: bool,
    pub read_only: bool,
}

/// 评论区分页信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cursor {
    pub is_begin: bool,
    pub prev: i64,
    pub next: i64,
    pub is_end: bool,
    pub pagination_reply: serde_json::Value,
    pub session_id: String,
    pub mode: i64,
    pub mode_text: String,
    pub all_count: Option<i64>,
    pub support_mode: Option<Vec<i64>>,
}

/// 评论区顶部信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upper {
    pub mid: u64, // UP 主 mid
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Control {
    pub input_disable: bool,
    pub root_input_text: String,
    pub child_input_text: String,
    pub giveup_input_text: String,
    pub screenshot_icon_state: i64,
    pub upload_picture_icon_state: i64,
    pub answer_guide_text: String,
    pub answer_guide_icon_url: String,
    pub answer_guide_ios_url: String,
    pub answer_guide_android_url: String,
    pub bg_text: String,
    pub empty_page: Option<serde_json::Value>,
    pub show_type: i64,
    pub show_text: String,
    pub web_selection: bool,
    pub disable_jump_emote: bool,
    pub enable_charged: bool,
    pub enable_cm_biz_helper: bool,
    pub preload_resources: Option<serde_json::Value>,
}

/// 广告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CM {
    pub id: i64,
    pub contract_id: String,
    pub pos_num: i64,
    pub name: String,
    pub pic: String,
    pub litpic: String,
    pub url: String,
    pub style: i64,
    pub agency: String,
    pub label: String,
    pub intro: String,
    pub creative_type: i64,
    pub request_id: String,
    pub src_id: i64,
    pub area: i64,
    pub is_ad_loc: bool,
    pub ad_cb: String,
    pub title: String,
    pub server_type: i64,
    pub cm_mark: i64,
    pub stime: i64,
    pub mid: String,
    pub activity_type: i64,
    pub epid: i64,
    pub sub_title: String,
    pub ad_desc: String,
    pub adver_name: String,
    pub null_frame: bool,
    pub pic_main_color: String,
}
