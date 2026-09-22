use crate::models::{Pendant, VipLabel};
use serde::{Deserialize, Serialize};

/// 作者模块 √
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ModuleAuthor {
    /// 头像信息，主要用于网页渲染
    pub avatar: Option<serde_json::Value>,
    /// 装扮，仅当动态接口且无 decorationCard 时存在
    pub decorate: Option<serde_json::Value>,
    /// 装扮，仅当图文接口时存在
    /// 头像 URL
    pub face: String,
    /// 是否为 NFT 头像
    pub face_nft: bool,
    /// 是否关注此 UP 主，自己的动态为 null
    pub following: Option<bool>,
    /// 跳转链接
    pub jump_url: String,
    /// 名称前标签，如 "合集", "电视剧", "番剧"
    pub label: String,
    /// UP 主 UID 或 剧集 SeasonId
    pub mid: i64,
    /// UP 主名称、剧集名称或合集名称
    pub name: String,
    /// UP 主认证信息，仅图文接口
    pub official: Option<AuthorOfficial>,
    /// UP 主认证信息，仅动态接口
    #[serde(rename = "official_verify")]
    pub official_verify: Option<AuthorOfficial>,
    /// UP 主头像框
    pub pendant: Option<Pendant>,
    /// 更新动作描述，仅动态接口
    pub pub_action: Option<String>,
    /// 更新时间，如 "x分钟前", "x小时前", "昨天"
    pub pub_time: String,
    /// 更新时间戳，UNIX 秒级时间戳
    pub pub_ts: i64,
    pub views_text: String,
    /// UP 主大会员信息
    pub vip: Option<AuthorVip>,
    /// 作者类型，仅动态接口
    #[serde(rename = "type")]
    pub type_field: Option<String>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AuthorOfficial {
    /// 认证说明
    pub desc: String,
    pub role: i64,
    pub title: String,
    /// 认证类型
    #[serde(rename = "type")]
    pub type_field: i64,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AuthorVip {
    /// 大会员过期时间戳，UNIX 毫秒时间戳
    pub due_date: i64,
    /// 大会员标签
    pub label: VipLabel,
    /// 名字显示颜色，大会员：#FB7299
    pub nickname_color: String,
    /// 大会员状态，0：无，1：有，2：封禁?
    pub status: i64,
    /// 大会员类型?
    pub r#type: i64,

    /// 仅图文接口
    pub role: Option<i64>,
    /// 主题类型? 仅图文接口
    pub theme_type: Option<i64>,
    /// TV 端过期时间? 仅图文接口
    pub tv_due_date: Option<i64>,
    /// TV 端付费状态? 仅图文接口
    pub tv_vip_pay_type: Option<i64>,
    /// TV 端会员状态?
    pub tv_vip_status: Option<i64>,
    /// 大会员付费类型?
    pub vip_pay_type: Option<i64>,
}

/// 更多模块 (三点菜单) √
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleMore {
    /// 右上角三点菜单
    #[serde(rename = "three_point_items")]
    pub three_point_items: Vec<ThreePointItem>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThreePointItem {
    /// 显示文本
    pub label: String,
    /// 弹出框文本
    pub modal: Option<Modal>,
    /// # 参数信息
    pub params: Option<Params>,
    /// 三点操作类型，参见 右上角三点菜单
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Modal {
    /// 取消文本
    pub cancel: String,
    /// 确认文本
    pub confirm: String,
    /// 内容文本
    pub content: String,
    /// 标题文本
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Params {
    /// 动态 id 字符串
    #[serde(rename = "dyn_id_str")]
    pub dyn_id_str: String,
    /// 动态类型
    #[serde(rename = "dyn_type")]
    pub dyn_type: i64,
    /// 动态 id 字符串
    #[serde(rename = "rid_str")]
    pub rid_str: String,
}

/// 统计模块 √
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleStat {
    /// 硬币数据，仅图文接口
    pub coin: Option<StatData>,
    /// 评论数据
    pub comment: StatData,
    /// 收藏数据，仅图文接口
    pub favorite: Option<StatData>,
    /// 转发数据
    pub forward: StatData,
    /// 点赞数据
    pub like: StatData,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatData {
    /// 数量
    pub count: i64,
    /// 是否屏蔽
    pub forbidden: bool,
    /// 是否隐藏
    pub hidden: Option<bool>,
    /// 当前状态，是否已进行该操作
    pub status: Option<bool>,
}

/// 内容模块
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleContent {
    /// 段落
    pub paragraphs: Vec<Paragraph>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Paragraph {
    /// 对齐方式，0: 左对齐, 1: 居中, 2: 右对齐
    pub align: i64,
    /// 段落类型，1: 文本, 2: 图片, 3: 分割线, 4: 块引用, 5: 列表, 6: 链接卡片, 7: 代码
    #[serde(rename = "para_type")]
    pub para_type: i64,
    /// 文本内容，仅 `para_type=1` 或 `para_type=4`
    pub text: Option<Text>,
    /// 图片内容，仅 `para_type=2`
    pub pics: Option<ParagraphPics>,
    /// 分割线，仅 `para_type=3`
    pub line: Option<ParagraphLine>,
    /// 列表，仅 `para_type=5`
    pub list: Option<ParagraphList>,
    /// 链接卡片，仅 `para_type=6`
    #[serde(rename = "link_card")]
    pub link_card: Option<LinkCard>,
    /// 代码，仅 `para_type=7`
    pub code: Option<ParagraphCode>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Text {
    /// 文本节点
    pub nodes: Vec<TextNode>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextNode {
    /// 文本节点类型，"TEXT_NODE_TYPE_WORD" 或 "TEXT_NODE_TYPE_RICH"
    #[serde(rename = "type")]
    pub type_field: String,
    /// 纯文本，仅 `type='TEXT_NODE_TYPE_WORD'`
    pub word: Option<TextNodeWord>,
    /// 富文本，仅 `type='TEXT_NODE_TYPE_RICH'`
    pub rich: Option<serde_json::Value>, // TODO: 详细的富文本节点结构未提供，用 Value 代替
    /// 公式，仅 `type='TEXT_NODE_TYPE_FORMULA'`
    pub formula: Option<TextNodeFormula>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextNodeWord {
    /// 字体大小，用于控制文本所用标签名及行高
    #[serde(rename = "font_size")]
    pub font_size: i64,
    /// 补充样式
    pub style: serde_json::Value,
    /// 文本内容
    pub words: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextNodeFormula {
    /// 公式内容，LaTeX 格式
    #[serde(rename = "latex_content")]
    pub latex_content: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParagraphLine {
    /// 图片信息
    pub pic: LinePic,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinePic {
    /// 高度
    pub height: i64,
    /// 图片 URL
    pub url: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParagraphList {
    /// 样式，1: 有序列表, 2: 无序列表
    pub style: i64,
    /// 列表项
    pub items: Vec<ListItem>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ListItem {
    /// 级别
    pub level: i64,
    /// 文本节点，同 `ModuleContent.paragraphs[].text.nodes[]`
    pub nodes: Vec<TextNode>,
    /// 序号
    pub order: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParagraphPics {
    /// 图片数组
    pub pics: Vec<PicItem>,
    /// 样式，1: isAlbum
    pub style: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PicItem {
    /// 高度
    pub height: i64,
    /// 动图 URL?
    #[serde(rename = "live_url")]
    pub live_url: serde_json::Value,
    /// 大小，单位: ki
    pub size: Option<i64>,
    /// 图片 URL
    pub url: String,
    /// 宽度
    pub width: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinkCard {
    /// 卡片内容
    pub card: Card,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Card {
    /// 关联 id，可能为 "undefined"
    pub oid: String,
    /// 卡片类型
    #[serde(rename = "type")]
    pub type_field: String,
    // 以下字段根据 `type_field` 的值而存在，用 Option 包裹
    /// 一般信息，仅 `type='LINK_CARD_TYPE_COMMON'`
    pub common: Option<serde_json::Value>,
    /// 商品信息，仅 `type='LINK_CARD_TYPE_GOODS'`
    pub goods: Option<serde_json::Value>,
    /// 比赛信息? 仅 `type='LINK_CARD_TYPE_MATCH'`
    #[serde(rename = "match")]
    pub match_field: Option<MatchCard>,
    /// 投票信息，仅 `type='LINK_CARD_TYPE_VOTE'`
    pub vote: Option<serde_json::Value>,
    /// 视频信息，仅 `type='LINK_CARD_TYPE_UGC'`
    pub ugc: Option<serde_json::Value>,
    /// 预约信息，仅 `type='LINK_CARD_TYPE_RESERVE'`
    pub reserve: Option<serde_json::Value>,
    /// 充电专属抽奖信息，仅 `type='LINK_CARD_TYPE_UPOWER_LOTTERY'`
    #[serde(rename = "upower_lottery")]
    pub upower_lottery: Option<UpowerLottery>,
    /// 图文信息，仅 `type='LINK_CARD_TYPE_OPUS'`
    pub opus: Option<OpusCard>,
    /// 音乐信息，仅 `type='LINK_CARD_TYPE_MUSIC'`
    pub music: Option<serde_json::Value>,
    /// 直播信息，仅 `type='LINK_CARD_TYPE_LIVE'`
    pub live: Option<serde_json::Value>,
    /// 提示信息，仅 `type='LINK_CARD_TYPE_ITEM_NULL'`
    #[serde(rename = "item_null")]
    pub item_null: Option<ItemNullCard>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchCard {
    /// 动态 ID
    #[serde(rename = "id_str")]
    pub id_str: String,
    /// 跳转 URL
    #[serde(rename = "jump_url")]
    pub jump_url: String,
    /// 比赛信息
    #[serde(rename = "match_info")]
    pub match_info: MatchInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchInfo {
    /// 中间区域底部的信息
    #[serde(rename = "center_bottom")]
    pub center_bottom: String,
    /// 中间区域顶部的信息，会循环显示，可能用来显示比分或时间
    #[serde(rename = "center_top")]
    pub center_top: Vec<String>,
    /// 右边队伍的信息
    #[serde(rename = "left_team")]
    pub left_team: TeamInfo,
    /// 左边队伍的信息
    #[serde(rename = "right_team")]
    pub right_team: TeamInfo,
    /// 比赛状态，2: 进行中 (文字高亮)
    pub status: i64,
    /// 副标题
    #[serde(rename = "sub_title")]
    pub sub_title: String,
    /// 标题
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeamInfo {
    /// 队伍名字
    pub name: String,
    /// 图片 URL
    pub pic: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpowerLottery {
    /// 按钮
    pub button: UpowerLotteryButton,
    /// 描述
    pub desc: UpowerLotteryDesc,
    /// 提示
    pub hint: UpowerLotteryHint,
    /// 跳转 URL
    #[serde(rename = "jump_url")]
    pub jump_url: String,
    /// 关联 id
    pub rid: i64,
    /// 状态
    pub state: i64,
    /// 标题
    pub title: String,
    /// UP 主 mid (UID)
    #[serde(rename = "up_mid")]
    pub up_mid: i64,
    /// 充电操作状态
    #[serde(rename = "upower_action_state")]
    pub upower_action_state: i64,
    /// 充电级别
    #[serde(rename = "upower_level")]
    pub upower_level: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpowerLotteryButton {
    /// 选中状态
    pub check: UpowerLotteryButtonCheck,
    /// 状态
    pub status: i64,
    /// 类型: 0, 1, 2
    #[serde(rename = "type")]
    pub type_field: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpowerLotteryButtonCheck {
    /// 是否禁用，1: 禁用
    pub disable: i64,
    /// 图标 URL
    #[serde(rename = "icon_url")]
    pub icon_url: String,
    /// 文字
    pub text: String,
    /// 提示
    pub toast: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpowerLotteryDesc {
    /// 跳转 URL
    #[serde(rename = "jump_url")]
    pub jump_url: String,
    /// 样式
    pub style: i64,
    /// 文字
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpowerLotteryHint {
    /// 样式
    pub style: i64,
    /// 文字
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpusCard {
    /// 作者信息
    pub author: OpusAuthor,
    /// 封面 URL
    pub cover: String,
    /// 跳转 URL
    #[serde(rename = "jump_url")]
    pub jump_url: String,
    /// 状态信息
    pub stat: OpusStat,
    /// 标题
    pub title: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpusAuthor {
    /// 作者名
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpusStat {
    /// 阅读数
    pub view: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ItemNullCard {
    /// 文字
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParagraphCode {
    /// 内容
    pub content: String,
    /// 语言，如 "language-html"
    pub lang: String,
}

/// 话题模块
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleTopic {
    /// 话题 id
    pub id: i64,
    /// 跳转 URL
    #[serde(rename = "jump_url")]
    pub jump_url: String,
    /// 话题名称
    pub name: String,
}

/// 文集模块
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleCollection {
    /// 文章数
    pub count: String,
    /// 文集 id
    pub id: i64,
    /// 文集名
    pub name: String,
    /// 标题，如 "收录于文集"
    pub title: String,
}

/// 扩展模块
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleExtend {
    /// 项
    pub items: Vec<ExtendItem>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtendItem {
    /// 图标
    pub icon: Option<String>,
    /// SVG 图版
    #[serde(rename = "icon_svg")]
    pub icon_svg: serde_json::Value,
    /// 跳转 URL
    #[serde(rename = "jump_url")]
    pub jump_url: String,
    /// 文本
    pub text: String,
}

/// 底部模块
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModuleBottom {
    /// 分享信息
    #[serde(rename = "share_info")]
    pub share_info: ShareInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShareInfo {
    /// 图片 URL
    pub pic: String,
    /// 总结
    pub summary: String,
    /// 标题
    pub title: String,
}

/// 动态模块
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Root {
    pub additional: Option<serde_json::Value>,
    pub desc: Option<serde_json::Value>,
    pub major: Major,
    pub topic: Option<serde_json::Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Major {
    pub archive: Archive,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Archive {
    pub aid: String,
    pub badge: Badge,
    pub bvid: String,
    pub cover: String,
    pub desc: String,
    pub disable_preview: i64,
    pub duration_text: String,
    pub jump_url: String,
    pub stat: Stat,
    pub title: String,
    #[serde(rename = "type")]
    pub type_field: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Badge {
    pub bg_color: String,
    pub color: String,
    pub icon_url: serde_json::Value,
    pub text: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stat {
    pub danmaku: String,
    pub play: String,
}
