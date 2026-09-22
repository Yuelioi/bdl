use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchData<T> {
    pub seid: String,
    pub page: i64,
    #[serde(rename = "pagesize")]
    pub page_size: i64,
    #[serde(rename = "numResults")]
    pub num_results: i64,
    #[serde(rename = "numPages")]
    pub num_pages: i64,

    /// 搜索类型为直播间及主播时：obj
    //  搜索类型为其他时：array
    pub result: Option<T>,

    /// 只在搜索类型为直播间及主播有效
    pageinfo: Option<PageInfo>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageInfo {
    pub live_user: LivePageInfo,
    pub live_room: LivePageInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LivePageInfo {
    pub total: i64,
    #[serde(rename = "numResults")]
    pub num_results: i64,
    pub pages: i64,
    #[serde(rename = "numPages")]
    pub num_pages: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Article {
    pub category_id: i64,
    pub category_name: String,
    pub comment_url: String,
    pub desc: String,
    pub id: i64,
    pub image_urls: Vec<String>,
    pub is_comment: i64,
    pub is_fold: bool,
    pub is_rk1: bool,
    pub like: i64,
    pub mid: i64,
    pub pub_time: i64,
    pub rank_index: i64,
    pub rank_offset: i64,
    pub reply: i64,
    pub spread_id: i64,
    pub sub_type: i64,
    pub template_id: i64,
    pub title: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub version: String,
    pub view: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bangumi {
    #[serde(rename = "type")]
    pub type_field: String,
    pub media_id: i64,
    pub title: String,
    pub org_title: String,
    pub media_type: i64,
    pub cv: String,
    pub staff: String,
    pub season_id: i64,
    pub is_avid: bool,
    pub hit_epids: String,
    pub season_type: i64,
    pub season_type_name: String,
    pub selection_style: String,
    pub ep_size: i64,
    pub url: String,
    pub button_text: String,
    pub is_follow: i64,
    pub is_selection: i64,
    pub eps: Vec<Ep>,
    pub badges: Vec<Badge>,
    pub cover: String,
    pub areas: String,
    pub styles: String,
    pub goto_url: String,
    pub desc: String,
    pub pubtime: i64,
    pub media_mode: i64,
    pub fix_pubtime_str: String,
    pub media_score: MediaScore,
    pub display_info: Vec<Badge>,
    pub pgc_season_id: i64,
    pub corner: i64,
    pub index_show: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ep {
    pub id: i64,
    pub cover: String,
    pub title: String,
    pub url: String,
    pub release_date: String,
    pub badges: Vec<Badge>,
    pub index_title: String,
    pub long_title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Badge {
    pub text: String,
    pub text_color: String,
    pub text_color_night: String,
    pub bg_color: String,
    pub bg_color_night: String,
    pub border_color: String,
    pub border_color_night: String,
    pub bg_style: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaScore {
    pub score: f32,
    pub user_count: i64,
}

/// 视频信息结构体
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Video {
    /// 视频类型
    pub r#type: String,

    /// 视频ID 也就是aid
    pub id: u64,

    /// 作者昵称
    pub author: String,

    /// 作者mid
    pub mid: u64,

    /// 视频分区ID
    pub typeid: String,

    /// 视频分区名称
    pub typename: String,

    /// 视频链接
    pub arcurl: String,

    /// 视频aid
    pub aid: u64,

    /// 视频bvid
    pub bvid: String,

    /// 视频标题
    pub title: String,

    /// 封面图
    pub pic: String,

    /// 播放量
    pub play: u64,

    /// 弹幕数量
    pub danmaku: u64,

    /// 收藏数量
    pub favorites: u64,

    /// 点赞数量
    pub like: u64,

    /// 视频标签
    pub tag: String,

    /// 评论数
    pub review: u64,

    /// 发布时间 (时间戳)
    pub pubdate: u64,

    /// 视频时长 (格式: mm:ss)
    pub duration: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Movie {
    #[serde(rename = "type")]
    pub type_field: String,
    pub media_id: i64,
    pub title: String,
    pub org_title: String,
    pub media_type: i64,
    pub cv: String,
    pub staff: String,
    pub season_id: i64,
    pub is_avid: bool,
    pub hit_epids: String,
    pub season_type: i64,
    pub season_type_name: String,
    pub selection_style: String,
    pub ep_size: i64,
    pub url: String,
    pub button_text: String,
    pub is_follow: i64,
    pub is_selection: i64,
    pub badges: Vec<Badge>,
    pub cover: String,
    pub areas: String,
    pub styles: String,
    pub goto_url: String,
    pub desc: String,
    pub pubtime: i64,
    pub media_mode: i64,
    pub fix_pubtime_str: String,
    pub media_score: MediaScore,
    pub display_info: Vec<Badge>,
    pub pgc_season_id: i64,
    pub corner: i64,
    pub index_show: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveData {
    live_room: Vec<LiveRoom>,
    live_user: Vec<LiveUser>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveUser {
    /// 所在分区ID
    pub area: i64,
    /// 所在分区v2 ID
    pub area_v2_id: i64,
    /// 关注数
    pub attentions: i64,
    /// 分区名称
    pub cate_name: String,
    /// 命中列
    pub hit_columns: Vec<String>,
    /// 是否正在直播
    pub is_live: bool,
    /// 直播状态
    pub live_status: i64,
    /// 直播开始时间
    pub live_time: String,
    /// 排名索引
    pub rank_index: i64,
    /// 排名偏移
    pub rank_offset: i64,
    /// 直播间ID
    #[serde(rename = "roomid")]
    pub room_id: i64,
    /// 标签
    pub tags: String,
    /// 类型字段
    #[serde(rename = "type")]
    pub type_field: String,
    /// 头像
    pub uface: String,
    /// 用户ID
    pub uid: i64,
    /// 用户名
    pub uname: String,
    /// 可选字段：仅 LiveUser2 有的 id
    pub id: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LiveRoom {
    /// 所在分区ID
    pub area: i64,

    /// 关注数
    pub attentions: i64,

    /// 分区名称
    pub cate_name: String,

    /// 封面图
    pub cover: String,

    /// 是否直播间内联
    pub is_live_room_inline: i64,

    /// 直播状态
    pub live_status: i64,

    /// 直播时间
    pub live_time: String,

    /// 当前在线人数
    pub online: i64,

    /// 排名索引
    pub rank_index: i64,

    /// 排名偏移
    pub rank_offset: i64,

    /// 房间ID
    pub roomid: i64,

    /// 短ID
    pub short_id: i64,

    /// 标签
    pub tags: String,

    /// 标题
    pub title: String,

    /// 类型字段
    #[serde(rename = "type")]
    pub type_field: String,

    /// 头像
    pub uface: String,

    /// 用户ID
    pub uid: i64,

    /// 用户名
    pub uname: String,

    /// 用户封面
    pub user_cover: String,

    /// 仅 type = live 有的字段
    pub watched_show: Option<WatchedShow>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchedShow {
    pub switch: bool,
    pub num: i64,
    pub text_small: String,
    pub text_large: String,
    pub icon: String,
    pub icon_location: String,
    pub icon_web: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OfficialVerify {
    #[serde(rename = "type")]
    pub r#type: i64,
    pub desc: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BiliUserVideo {
    pub aid: i64,
    pub bvid: String,
    pub title: String,
    pub pubdate: i64,
    pub arcurl: String,
    pub pic: String,
    pub play: String,
    pub dm: i64,
    pub coin: i64,
    pub fav: i64,
    pub desc: String,
    pub duration: String,
    pub is_pay: i64,
    pub is_union_video: i64,
    pub is_charge_video: i64,
    pub vt: i64,
    pub enable_vt: i64,
    pub vt_display: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BiliUser {
    #[serde(rename = "type")]
    pub r#type: String,
    pub mid: i64,
    pub uname: String,
    pub usign: String,
    pub fans: i64,
    pub videos: i64,
    pub upic: String,
    pub face_nft: i64,
    pub face_nft_type: i64,
    pub verify_info: String,
    pub level: i64,
    pub gender: i64,
    pub is_upuser: i64,
    pub is_live: i64,
    pub room_id: i64,
    pub res: Vec<BiliUserVideo>,
    pub official_verify: OfficialVerify,
    pub is_senior_member: i64,
}
