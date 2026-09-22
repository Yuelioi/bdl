use serde::{Deserialize, Deserializer, Serialize, de};

use crate::ids::{Aid, Bvid, Mid};

/// `/x/web-interface/card` 返回的载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCardProfile {
    /// 用户卡片摘要。
    pub card: UserCardSummary,
    /// 当前会话是否关注该用户。
    pub following: bool,
    /// 已上传稿件数。
    #[serde(default)]
    pub archive_count: u64,
    /// 专栏文章数。
    #[serde(default)]
    pub article_count: u64,
    /// 粉丝数。
    #[serde(default)]
    pub follower: u64,
    /// 收到的总点赞数。
    #[serde(default)]
    pub like_num: u64,
}

/// 嵌套 `card` 对象中的稳定字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCardSummary {
    /// 用户 member ID。
    #[serde(deserialize_with = "deserialize_mid_from_string_or_number")]
    pub mid: Mid,
    /// 显示名称。
    pub name: String,
    /// 个人资料性别文本。
    #[serde(default)]
    pub sex: Option<String>,
    /// 头像 URL。
    pub face: String,
    /// 个人签名。
    #[serde(default)]
    pub sign: String,
    /// 卡片摘要中嵌入的粉丝数。
    #[serde(default)]
    pub fans: u64,
    /// 卡片摘要中嵌入的关注数。
    #[serde(default)]
    pub attention: u64,
}

/// `/account/v1/user/cards` 返回的简略用户卡片。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserBatchCard {
    /// 用户 member ID。
    pub mid: Mid,
    /// 显示名称。
    pub name: String,
    /// 头像 URL。
    pub face: String,
    /// 个人签名。
    #[serde(default)]
    pub sign: String,
    /// Bilibili 返回的显示 rank。
    #[serde(default)]
    pub rank: i32,
    /// 当前账号等级。
    #[serde(default)]
    pub level: i32,
    /// Bilibili 返回的用户禁言或封禁状态。
    #[serde(default)]
    pub silence: i32,
}

/// `/x/im/user_infos` 返回的批量用户详细信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBatchInfo {
    /// 用户 member ID。
    pub mid: Mid,
    /// 显示名称。
    pub name: String,
    /// 个人签名。
    #[serde(default)]
    pub sign: String,
    /// Bilibili 返回的显示 rank。
    #[serde(default)]
    pub rank: i32,
    /// 当前账号等级。
    #[serde(default)]
    pub level: i32,
    /// Bilibili 返回的用户禁言或封禁状态。
    #[serde(default)]
    pub silence: i32,
    /// 个人资料性别文本。
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub sex: Option<String>,
    /// 头像 URL。
    pub face: String,
    /// VIP 状态摘要。
    #[serde(default)]
    pub vip: Option<UserBatchVip>,
    /// 官方认证摘要。
    #[serde(default)]
    pub official: Option<UserOfficialSummary>,
    /// Bilibili 是否将该账号标记为假账号。
    #[serde(default)]
    pub is_fake_account: Option<u32>,
    /// Bilibili 返回的 expert 载荷；其结构偏展示用途，因此保留原始值。
    #[serde(default)]
    pub expert_info: Option<serde_json::Value>,
}

/// `/x/im/user_infos` 批量响应中嵌入的 VIP 状态字段。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserBatchVip {
    /// VIP 类型代码。
    #[serde(default, rename = "type")]
    pub kind: i32,
    /// VIP 状态代码。
    #[serde(default)]
    pub status: i32,
    /// VIP 到期时间戳，单位毫秒。
    #[serde(default)]
    pub due_date: i64,
    /// VIP 支付类型代码。
    #[serde(default)]
    pub vip_pay_type: i32,
    /// VIP 主题类型代码。
    #[serde(default)]
    pub theme_type: i32,
}

/// `/x/space/wbi/acc/info` 返回的公开用户空间信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSpaceProfile {
    /// 用户 member ID。
    pub mid: Mid,
    /// 显示名称。
    pub name: String,
    /// 个人资料性别文本。
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub sex: Option<String>,
    /// 头像 URL。
    pub face: String,
    /// 个人签名。
    #[serde(default)]
    pub sign: String,
    /// 当前账号等级。
    pub level: u8,
    /// Bilibili 返回的用户禁言或封禁状态。
    pub silence: u8,
    /// 可见硬币数。查询其他用户时 Bilibili 返回 `0`。
    #[serde(default)]
    pub coins: f64,
    /// 该用户是否有粉丝勋章信息。
    #[serde(default)]
    pub fans_badge: bool,
    /// 当前会话是否关注该用户。
    #[serde(default)]
    pub is_followed: bool,
    /// Bilibili 返回时的空间顶部图片 URL。
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub top_photo: Option<String>,
    /// 官方认证摘要。
    #[serde(default)]
    pub official: Option<UserOfficialSummary>,
    /// VIP 状态摘要。
    #[serde(default)]
    pub vip: Option<UserVipSummary>,
    /// 直播间摘要。
    #[serde(default)]
    pub live_room: Option<UserSpaceLiveRoom>,
}

/// 用户空间载荷中常见的官方认证字段。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserOfficialSummary {
    /// 官方角色代码。
    #[serde(default)]
    pub role: i32,
    /// 认证标题。
    #[serde(default)]
    pub title: String,
    /// 认证描述。
    #[serde(default)]
    pub desc: String,
    /// 认证类型代码。
    #[serde(default, rename = "type")]
    pub kind: i32,
}

/// 用户空间载荷中常见的 VIP 状态字段。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserVipSummary {
    /// VIP 类型代码。
    #[serde(default, rename = "type")]
    pub kind: i32,
    /// VIP 状态代码。
    #[serde(default)]
    pub status: i32,
}

/// 用户空间载荷中嵌入的直播间字段。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSpaceLiveRoom {
    /// 该用户是否存在直播间。
    #[serde(default, rename = "roomStatus")]
    pub room_status: u8,
    /// 直播间当前是否开播。
    #[serde(default, rename = "liveStatus")]
    pub live_status: u8,
    /// 直播间 URL。
    #[serde(default)]
    pub url: String,
    /// 直播间标题。
    #[serde(default)]
    pub title: String,
    /// 直播间 ID。
    #[serde(default, rename = "roomid")]
    pub room_id: u64,
}

/// `/x/space/notice` 返回的公开空间公告。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UserSpaceNotice {
    /// 公告文本。用户没有公开公告时为空。
    pub content: String,
}

/// `/x/space/bangumi/follow/list` 返回的已追番剧或影视 season。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBangumiFollowList {
    /// 当前页的已追 season。
    #[serde(default, rename = "list")]
    pub items: Vec<UserBangumiFollow>,
    /// 当前页码。
    #[serde(default, rename = "pn")]
    pub page: u32,
    /// 每页数量。
    #[serde(default, rename = "ps")]
    pub page_size: u32,
    /// 该分类下已追 season 总数。
    #[serde(default)]
    pub total: u64,
}

/// 一个已追番剧或影视 season。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserBangumiFollow {
    /// Season ID。
    pub season_id: i64,
    /// Media ID。
    pub media_id: i64,
    /// Bilibili 返回的 season 类型代码。
    #[serde(default)]
    pub season_type: i64,
    /// Season 类型显示名称。
    #[serde(default)]
    pub season_type_name: String,
    /// Season 标题。
    #[serde(default)]
    pub title: String,
    /// 封面图片 URL。
    #[serde(default)]
    pub cover: String,
    /// 总集数。
    #[serde(default)]
    pub total_count: i64,
    /// 该 season 是否已完结。
    #[serde(default)]
    pub is_finish: i64,
    /// 该 season 是否已开播。
    #[serde(default)]
    pub is_started: i64,
    /// 该 season 是否可播放。
    #[serde(default)]
    pub is_play: i64,
    /// Bilibili 返回的 badge 文本。
    #[serde(default)]
    pub badge: String,
    /// Badge 类型代码。
    #[serde(default)]
    pub badge_type: i64,
    /// 最新一集摘要。
    #[serde(default, rename = "new_ep")]
    pub latest_episode: UserBangumiLatestEpisode,
    /// Bilibili 返回时的 season 评分。
    #[serde(default)]
    pub rating: Option<UserBangumiRating>,
    /// Season 页面 URL。
    #[serde(default)]
    pub url: String,
    /// Bilibili 返回的短 URL。
    #[serde(default)]
    pub short_url: String,
    /// Season 描述。
    #[serde(default)]
    pub summary: String,
    /// 风格标签。
    #[serde(default)]
    pub styles: Vec<String>,
    /// 追番状态代码。
    #[serde(default)]
    pub follow_status: i64,
    /// 用户进度文本。
    #[serde(default)]
    pub progress: String,
    /// 双方是否都追了该 season。
    #[serde(default)]
    pub both_follow: bool,
}

/// 番剧追番列表项中嵌入的最新一集摘要。
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserBangumiLatestEpisode {
    /// Episode ID。
    #[serde(default)]
    pub id: i64,
    /// 显示用集数索引。
    #[serde(default)]
    pub index_show: String,
    /// 剧集封面图片 URL。
    #[serde(default)]
    pub cover: String,
    /// 剧集标题。
    #[serde(default)]
    pub title: String,
    /// 剧集长标题。
    #[serde(default)]
    pub long_title: Option<String>,
    /// Bilibili 返回的发布时间文本。
    #[serde(default)]
    pub pub_time: String,
    /// 剧集时长，单位秒。
    #[serde(default)]
    pub duration: i64,
}

/// 番剧追番列表项中嵌入的评分摘要。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UserBangumiRating {
    /// 评分分数。
    #[serde(default)]
    pub score: f64,
    /// 评分人数。
    #[serde(default)]
    pub count: i64,
}

/// `/x/relation/stat` 返回的关系计数。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserRelationStat {
    /// 用户 member ID。
    pub mid: Mid,
    /// 公开关注数。
    pub following: u64,
    /// 悄悄关注数。
    #[serde(default)]
    pub whisper: u64,
    /// 黑名单数量。
    #[serde(default)]
    pub black: u64,
    /// 粉丝数。
    pub follower: u64,
}

/// `/x/relation/followings` 返回的关注列表载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFollowings {
    /// 指定用户关注的用户列表。
    #[serde(default)]
    pub list: Vec<UserFollowing>,
    /// Bilibili 关系列表 schema 版本。
    #[serde(default)]
    pub re_version: u32,
    /// 已关注用户总数。
    #[serde(default)]
    pub total: u64,
}

/// [`UserFollowings`] 中的一个已关注用户。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFollowing {
    /// 被关注用户 member ID。
    pub mid: Mid,
    /// Bilibili 返回的关系属性。
    #[serde(default)]
    pub attribute: u8,
    /// 关注时间戳，单位秒。
    #[serde(default)]
    pub mtime: u64,
    /// 关系分组 ID 列表。
    #[serde(default)]
    pub tag: Option<Vec<u64>>,
    /// 该用户是否被特别关注。
    #[serde(default)]
    pub special: u8,
    /// 显示名称。
    #[serde(default, rename = "uname")]
    pub name: String,
    /// 头像 URL。
    #[serde(default)]
    pub face: String,
    /// 个人签名。
    #[serde(default)]
    pub sign: String,
    /// Bilibili 是否将头像标记为 NFT。
    #[serde(default)]
    pub face_nft: u8,
    /// 关系列表返回的官方认证摘要。
    #[serde(default)]
    pub official_verify: Option<UserRelationOfficialVerify>,
    /// VIP 摘要。Bilibili 经常调整该展示载荷，因此保留原始值。
    #[serde(default)]
    pub vip: Option<serde_json::Value>,
}

/// `/x/relation/fans` 返回的粉丝列表载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFollowers {
    /// 关注指定用户的用户列表。
    #[serde(default)]
    pub list: Vec<UserFollower>,
    /// 传给下一次请求的分页 offset。
    #[serde(default)]
    pub offset: String,
    /// Bilibili 关系列表 schema 版本。
    #[serde(default)]
    pub re_version: u32,
    /// 粉丝总数。
    #[serde(default)]
    pub total: u64,
}

/// [`UserFollowers`] 中的一个粉丝。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFollower {
    /// 粉丝 member ID。
    pub mid: Mid,
    /// Bilibili 返回的关系属性。
    #[serde(default)]
    pub attribute: u8,
    /// Bilibili 返回时的关注时间戳，单位秒。
    #[serde(default)]
    pub mtime: Option<u64>,
    /// 关系分组 ID 列表。
    #[serde(default)]
    pub tag: Option<Vec<u64>>,
    /// 该用户是否被特别关注。
    #[serde(default)]
    pub special: u8,
    /// Contract 展示载荷。该嵌套 schema 不稳定，因此保留原始值。
    #[serde(default)]
    pub contract_info: Option<serde_json::Value>,
    /// 显示名称。
    #[serde(default, rename = "uname")]
    pub name: String,
    /// 头像 URL。
    #[serde(default)]
    pub face: String,
    /// 个人签名。
    #[serde(default)]
    pub sign: String,
    /// Bilibili 是否将头像标记为 NFT。
    #[serde(default)]
    pub face_nft: u8,
    /// 关系列表返回的官方认证摘要。
    #[serde(default)]
    pub official_verify: Option<UserRelationOfficialVerify>,
    /// VIP 摘要。Bilibili 经常调整该展示载荷，因此保留原始值。
    #[serde(default)]
    pub vip: Option<serde_json::Value>,
}

/// `/x/relation/tags` 返回的关注分组条目。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserFollowTag {
    /// 关注分组 ID。Bilibili 对内置分组使用负数 ID。
    #[serde(rename = "tagid")]
    pub id: i64,
    /// 关注分组显示名称。
    pub name: String,
    /// 分组中的用户数量。
    #[serde(default)]
    pub count: i64,
    /// Bilibili 返回的可选 UI 提示。
    #[serde(default)]
    pub tip: Option<String>,
}

/// `/xlive/web-ucenter/user/MedalWall` 返回的粉丝勋章墙。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMedalWall {
    /// 用户勋章墙上可见的勋章条目。
    #[serde(default)]
    pub list: Vec<UserMedalWallItem>,
    /// 可见勋章条目数量。
    #[serde(default)]
    pub count: u32,
    /// 该用户是否关闭空间勋章墙。
    #[serde(default)]
    pub close_space_medal: u32,
    /// 该用户是否只展示正在佩戴的勋章。
    #[serde(default)]
    pub only_show_wearing: u32,
    /// 勋章墙所有者显示名称。
    #[serde(default)]
    pub name: String,
    /// 勋章墙所有者头像 URL。
    #[serde(default)]
    pub icon: String,
    /// 勋章墙所有者 member ID。
    pub uid: Mid,
    /// 勋章墙所有者等级。
    #[serde(default)]
    pub level: u32,
}

/// [`UserMedalWall`] 中的一个粉丝勋章条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMedalWallItem {
    /// 目标创作者的勋章元数据。
    pub medal_info: UserMedalInfo,
    /// 目标创作者显示名称。
    #[serde(default)]
    pub target_name: String,
    /// 目标创作者头像 URL。
    #[serde(default)]
    pub target_icon: String,
    /// 目标直播间链接。
    #[serde(default)]
    pub link: String,
    /// 目标当前直播状态。
    #[serde(default)]
    pub live_status: u32,
    /// 官方状态字段。Bilibili 将该 key 拼写为 `offical`。
    #[serde(default, rename = "offical")]
    pub official: Option<u32>,
    /// 从勋章墙所有者视角看到的勋章展示字段。
    #[serde(default)]
    pub uinfo_medal: Option<UserMedalOwnerInfo>,
}

impl UserMedalWallItem {
    /// 返回该勋章对应目标创作者的 member ID。
    pub fn target_id(&self) -> Mid {
        self.medal_info.target_id
    }
}

/// 一个目标创作者的稳定粉丝勋章元数据。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMedalInfo {
    /// 目标创作者 member ID。
    pub target_id: Mid,
    /// 勋章等级。
    #[serde(default)]
    pub level: u32,
    /// 勋章显示名称。
    #[serde(default, rename = "medal_name")]
    pub name: String,
    /// 勋章渐变起始颜色。
    #[serde(default)]
    pub medal_color_start: u32,
    /// 勋章渐变结束颜色。
    #[serde(default)]
    pub medal_color_end: u32,
    /// 勋章边框颜色。
    #[serde(default)]
    pub medal_color_border: u32,
    /// 与该勋章关联的大航海等级。
    #[serde(default)]
    pub guard_level: u32,
    /// 该勋章当前是否正在佩戴。
    #[serde(default)]
    pub wearing_status: u32,
    /// 勋章 ID。
    #[serde(default)]
    pub medal_id: u64,
    /// 当前亲密度分数。
    #[serde(default)]
    pub intimacy: u64,
    /// 升到下一级所需亲密度分数。
    #[serde(default)]
    pub next_intimacy: u64,
    /// 今日已投喂亲密度。
    #[serde(default)]
    pub today_feed: u64,
    /// 每日亲密度投喂上限。
    #[serde(default)]
    pub day_limit: u64,
    /// Bilibili 返回时的大航海图标 URL。
    #[serde(default)]
    pub guard_icon: Option<String>,
    /// Bilibili 返回时的荣誉图标 URL。
    #[serde(default)]
    pub honor_icon: Option<String>,
}

/// 从勋章墙所有者视角看到的勋章展示字段。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMedalOwnerInfo {
    /// 勋章显示名称。
    #[serde(default)]
    pub name: String,
    /// 勋章等级。
    #[serde(default)]
    pub level: u32,
    /// 勋章渐变起始颜色。
    #[serde(default)]
    pub color_start: u32,
    /// 勋章渐变结束颜色。
    #[serde(default)]
    pub color_end: u32,
    /// 勋章边框颜色。
    #[serde(default)]
    pub color_border: u32,
    /// 勋章文本颜色。
    #[serde(default)]
    pub color: u32,
    /// 勋章 ID。
    #[serde(default)]
    pub id: u64,
    /// Bilibili 返回的勋章类型。
    #[serde(default)]
    pub typ: u32,
    /// 勋章是否点亮。
    #[serde(default)]
    pub is_light: u32,
    /// 目标创作者 member ID。
    pub ruid: Mid,
    /// 与该勋章关联的大航海等级。
    #[serde(default)]
    pub guard_level: u32,
    /// 当前亲密度分数。
    #[serde(default)]
    pub score: u64,
    /// Bilibili 返回时的大航海图标 URL。
    #[serde(default)]
    pub guard_icon: Option<String>,
    /// Bilibili 返回时的荣誉图标 URL。
    #[serde(default)]
    pub honor_icon: Option<String>,
    /// V2 勋章起始颜色 token。
    #[serde(default)]
    pub v2_medal_color_start: Option<String>,
    /// V2 勋章结束颜色 token。
    #[serde(default)]
    pub v2_medal_color_end: Option<String>,
    /// V2 勋章边框颜色 token。
    #[serde(default)]
    pub v2_medal_color_border: Option<String>,
    /// V2 勋章文本颜色 token。
    #[serde(default)]
    pub v2_medal_color_text: Option<String>,
    /// V2 勋章等级颜色 token。
    #[serde(default)]
    pub v2_medal_color_level: Option<String>,
    /// Bilibili 返回时已获得该勋章的用户数量。
    #[serde(default)]
    pub user_receive_count: Option<u32>,
}

/// 关系列表载荷中嵌入的官方认证字段。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserRelationOfficialVerify {
    /// 认证类型代码。
    #[serde(default, rename = "type")]
    pub kind: i8,
    /// 认证描述。
    #[serde(default)]
    pub desc: String,
}

/// `/x/space/upstat` 返回的创作者内容统计。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserUpStat {
    /// 投稿视频播放摘要。
    pub archive: UserUpStatArchive,
    /// 专栏阅读摘要。
    pub article: UserUpStatArticle,
    /// 该用户收到的总点赞数。
    pub likes: u64,
}

/// [`UserUpStat`] 中的投稿视频播放摘要。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserUpStatArchive {
    /// 视频总播放数。
    #[serde(default)]
    pub view: u64,
}

/// [`UserUpStat`] 中的专栏阅读摘要。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserUpStatArticle {
    /// 专栏总阅读数。
    #[serde(default)]
    pub view: u64,
}

/// `/x/space/navnum` 返回的用户空间导航计数。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserNavStat {
    /// 投稿视频数量。
    #[serde(default)]
    pub video: u64,
    /// 已追番剧数量。
    #[serde(default)]
    pub bangumi: u64,
    /// 已追影视数量。
    #[serde(default)]
    pub cinema: u64,
    /// 视频列表计数。
    #[serde(default)]
    pub channel: UserNavStatPair,
    /// 收藏夹计数。
    #[serde(default)]
    pub favourite: UserNavStatPair,
    /// 关注分组数量。
    #[serde(default)]
    pub tag: u64,
    /// 专栏数量。
    #[serde(default)]
    pub article: u64,
    /// 播单数量。
    #[serde(default)]
    pub playlist: u64,
    /// 相簿数量。
    #[serde(default)]
    pub album: u64,
    /// 音频数量。
    #[serde(default)]
    pub audio: u64,
    /// 课程数量。
    #[serde(default)]
    pub pugv: u64,
    /// 图文数量。
    #[serde(default)]
    pub opus: u64,
    /// 视频合集或列表数量。
    #[serde(default, rename = "season_num")]
    pub season_count: u64,
}

/// [`UserNavStat`] 使用的所有者/访客拆分计数。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserNavStatPair {
    /// 所有者可见的总数。
    #[serde(default)]
    pub master: u64,
    /// 访客可见的公开数量。
    #[serde(default)]
    pub guest: u64,
}

/// `/link_draw/v1/doc/upload_count` 返回的相簿投稿计数。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserAlbumCount {
    /// 相簿投稿总数。
    #[serde(default)]
    pub all_count: u64,
    /// 绘画投稿数。
    #[serde(default)]
    pub draw_count: u64,
    /// 摄影投稿数。
    #[serde(default)]
    pub photo_count: u64,
    /// 日常图片动态投稿数。
    #[serde(default)]
    pub daily_count: u64,
}

/// `/x/polymer/web-dynamic/v1/name-to-uid` 返回的用户名查询载荷。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserNameToUid {
    /// 匹配到的用户名和 member ID 条目。
    #[serde(default)]
    pub uid_list: Vec<UserNameToUidItem>,
}

/// 一个用户名查询结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserNameToUidItem {
    /// 匹配查询的显示名称。
    pub name: String,
    /// 匹配到的 member ID。
    #[serde(
        rename = "uid",
        deserialize_with = "deserialize_mid_from_string_or_number"
    )]
    pub mid: Mid,
}

/// `/x/space/wbi/arc/search` 返回的投稿视频。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUploadedVideos {
    /// 投稿视频列表和分区摘要。
    pub list: UserUploadedVideoList,
    /// 分页摘要。
    pub page: UserUploadedVideosPage,
    /// Bilibili 返回的可选全部播放按钮。
    #[serde(default)]
    pub episodic_button: Option<UserUploadedVideosButton>,
    /// 响应是否被 Bilibili 风控。
    #[serde(default)]
    pub is_risk: bool,
}

/// 投稿视频列表载荷。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUploadedVideoList {
    /// 以分区 ID 为 key 的分区摘要。
    #[serde(default)]
    pub tlist: serde_json::Value,
    /// 当前页的投稿视频。
    #[serde(default, rename = "vlist")]
    pub videos: Vec<UserUploadedVideo>,
}

/// 用户空间中的一个投稿视频。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserUploadedVideo {
    /// AV 数字视频 ID。
    pub aid: Aid,
    /// BV 字符串视频 ID。
    pub bvid: Bvid,
    /// UP 主 member ID。
    pub mid: Mid,
    /// 视频标题。
    pub title: String,
    /// UP 主显示名称。
    #[serde(default)]
    pub author: String,
    /// 封面图片 URL。
    #[serde(default)]
    pub pic: String,
    /// Bilibili 返回的视频时长文本。
    #[serde(default)]
    pub length: String,
    /// 视频描述。
    #[serde(default)]
    pub description: String,
    /// 发布时间戳，单位秒。
    #[serde(default)]
    pub created: u64,
    /// 播放数。
    #[serde(default)]
    pub play: u64,
    /// 回复数。
    #[serde(default)]
    pub comment: u64,
    /// 分区 ID。
    #[serde(default)]
    pub typeid: u64,
    /// 弹幕数。
    #[serde(default)]
    pub video_review: u64,
    /// Bilibili 是否隐藏该视频的点击详情。
    #[serde(default)]
    pub hide_click: bool,
}

/// 投稿视频分页摘要。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserUploadedVideosPage {
    /// 匹配到的视频总数。
    #[serde(default)]
    pub count: u64,
    /// 当前页码。
    #[serde(default)]
    pub pn: u32,
    /// 每页数量。
    #[serde(default)]
    pub ps: u32,
}

/// 投稿视频搜索返回的可选全部播放按钮。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserUploadedVideosButton {
    /// 按钮文本。
    #[serde(default)]
    pub text: String,
    /// 按钮目标 URI。
    #[serde(default)]
    pub uri: String,
}

fn deserialize_mid_from_string_or_number<'de, D>(deserializer: D) -> Result<Mid, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;

    match value {
        serde_json::Value::Number(number) => {
            let mid = number
                .as_u64()
                .ok_or_else(|| de::Error::custom("mid must be a non-negative integer"))?;
            Mid::new(mid).map_err(de::Error::custom)
        }
        serde_json::Value::String(text) => text.parse::<Mid>().map_err(de::Error::custom),
        _ => Err(de::Error::custom("mid must be a string or number")),
    }
}

fn deserialize_optional_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?
        .and_then(|value| (!value.trim().is_empty()).then_some(value)))
}
