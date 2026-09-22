//! 番剧基本信息
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/bangumi/info.md)
use crate::models::VipLabel;
use serde::{Deserialize, Serialize};

/// 剧集地区
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BangumiArea {
    /// 中国大陆
    MainlandChina = 1,
    /// 日本
    Japan = 2,
    /// 美国
    UnitedStates = 3,
    /// 英国
    UnitedKingdom = 4,
    /// 加拿大
    Canada = 5,
    /// 中国香港
    HongKong = 6,
    /// 中国台湾
    Taiwan = 7,
    /// 韩国
    SouthKorea = 8,
    /// 法国
    France = 9,
    /// 泰国
    Thailand = 10,
    /// 马来西亚
    Malaysia = 11,
    /// 新加坡
    Singapore = 12,
    /// 西班牙
    Spain = 13,
    /// 俄罗斯
    Russia = 14,
    /// 德国
    Germany = 15,
    /// 其他
    Other = 16,
    /// 丹麦
    Denmark = 17,
    /// 乌克兰
    Ukraine = 18,
    /// 以色列
    Israel = 19,
    /// 伊朗
    Iran = 20,
    /// 保加利亚
    Bulgaria = 21,
    /// 克罗地亚
    Croatia = 22,
    /// 冰岛
    Iceland = 23,
    /// 匈牙利
    Hungary = 24,
    /// 南非
    SouthAfrica = 25,
    /// 印尼
    Indonesia = 26,
    /// 印度
    India = 27,
    /// 哥伦比亚
    Colombia = 28,
    /// 土耳其
    Turkey = 30,
    /// 墨西哥
    Mexico = 31,
    /// 委内瑞拉
    Venezuela = 32,
    /// 巴西
    Brazil = 33,
    /// 希腊
    Greece = 34,
    /// 意大利
    Italy = 35,
    /// 挪威
    Norway = 36,
    /// 捷克
    CzechRepublic = 37,
    /// 摩洛哥
    Morocco = 38,
    /// 新西兰
    NewZealand = 39,
    /// 智利
    Chile = 40,
    /// 比利时
    Belgium = 41,
    /// 波兰
    Poland = 42,
    /// 澳大利亚
    Australia = 43,
    /// 爱尔兰
    Ireland = 44,
    /// 瑞典
    Sweden = 45,
    /// 瑞士
    Switzerland = 46,
    /// 芬兰
    Finland = 47,
    /// 苏联
    SovietUnion = 48,
    /// 荷兰
    Netherlands = 49,
    /// 越南
    Vietnam = 50,
    /// 阿根廷
    Argentina = 51,
    /// 马耳他
    Malta = 52,
    /// 古巴
    Cuba = 53,
    /// 菲律宾
    Philippines = 54,
    /// 哈萨克斯坦
    Kazakhstan = 55,
    /// 黎巴嫩
    Lebanon = 56,
    /// 塞浦路斯
    Cyprus = 57,
    /// 卡塔尔
    Qatar = 58,
    /// 阿联酋
    UnitedArabEmirates = 59,
    /// 奥地利
    Austria = 60,
    /// 西德
    WestGermany = 61,
    /// 卢森堡
    Luxembourg = 62,
    /// 罗马尼亚
    Romania = 63,
    /// 印度尼西亚
    Indonesia2 = 64,
    /// 南斯拉夫
    Yugoslavia = 65,
    /// 蒙古
    Mongolia = 66,
    /// 葡萄牙
    Portugal = 70,
}

impl BangumiArea {
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// 剧集类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BangumiType {
    /// 番剧
    Anime = 1,
    /// 电影
    Movie = 2,
    /// 纪录片
    Documentary = 3,
    /// 国创
    ChineseAnimation = 4,
    /// 电视剧
    TVSeries = 5,
    /// 漫画
    Manga = 6,
    /// 综艺
    Variety = 7,
}

impl BangumiType {
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiInfoResult {
    pub media: BangumiMedia,
    pub review: Option<BangumiReview>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiMedia {
    pub areas: Vec<BangumiAreaInfo>,
    pub cover: String,
    pub horizontal_picture: String,
    pub media_id: u64,
    pub new_ep: BangumiMediaNewEp,
    #[serde(default)]
    pub rating: BangumiRating,
    pub season_id: u64,
    pub share_url: String,
    pub title: String,
    pub r#type: u32,
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiAreaInfo {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiMediaNewEp {
    pub id: u64,
    pub index: String,
    pub index_show: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiDetailNewEp {
    pub id: u64,
    pub desc: String,
    pub is_new: u32,
    pub title: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BangumiRating {
    pub count: u64,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiReview {
    pub is_coin: u32,
    pub is_open: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiDetailResult {
    pub activity: Option<BangumiActivity>,
    pub actors: String,
    pub alias: String,
    pub areas: Vec<BangumiAreaInfo>,
    pub bkg_cover: String,
    pub cover: String,
    pub delivery_fragment_video: bool,
    pub enable_vt: bool,
    pub episodes: Vec<BangumiEpisode>,
    pub evaluate: String,
    pub freya: Option<BangumiFreya>,
    pub hide_ep_vv_vt_dm: u32,
    pub icon_font: Option<BangumiIconFont>,
    pub jp_title: String,
    pub link: String,
    pub media_id: u64,
    pub mode: u32,
    pub multi_view_info: Option<BangumiMultiViewInfo>,
    pub new_ep: BangumiDetailNewEp,
    pub payment: Option<BangumiPayment>,
    #[serde(rename = "payPack")]
    pub pay_pack: Option<BangumiPayPack>,
    pub play_strategy: Option<BangumiPlayStrategy>,
    pub positive: Option<BangumiPositive>,
    pub publish: BangumiPublish,
    pub rating: Option<BangumiRating>,
    pub record: String,
    pub rights: BangumiRights,
    pub season_id: u64,
    pub season_title: String,
    pub seasons: Vec<BangumiSeason>,
    pub section: Option<Vec<BangumiSection>>,
    pub section_bottom_desc: Option<String>,
    pub series: Option<BangumiSeries>,
    pub share_copy: String,
    pub share_sub_title: String,
    pub share_url: String,
    pub show: Option<BangumiShow>,
    pub show_season_type: u32,
    pub square_cover: String,
    pub staff: String,
    pub stat: BangumiStat,
    pub status: u32,
    pub styles: Vec<String>,
    pub subtitle: String,
    pub title: String,
    /// 分集总数；上游在未知时返回 `-1`。
    pub total: i32,
    pub r#type: u32,
    pub up_info: Option<BangumiUpInfo>,
    pub user_status: Option<BangumiUserStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiActivity {
    pub head_bg_url: String,
    pub id: u64,
    pub title: String,
    pub link: Option<String>,
    pub pendants: Option<Vec<BangumiPendant>>,
    pub cover: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiPendant {
    pub image: String,
    pub name: String,
    /// 上游在无有效挂件时可能返回负数哨兵。
    pub pid: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiEpisode {
    pub aid: u64,
    pub badge: String,
    pub badge_info: Option<BangumiBadgeInfo>,
    pub badge_type: u32,
    pub bvid: String,
    pub cid: u64,
    pub cover: String,
    pub dimension: Option<BangumiDimension>,
    pub duration: u64,
    pub enable_vt: bool,
    pub ep_id: u64,
    pub from: String,
    pub id: u64,
    pub interaction: Option<BangumiInteraction>,
    pub is_view_hide: bool,
    pub link: String,
    pub long_title: String,
    pub multi_view_eps: Option<Vec<BangumiMultiViewEp>>,
    pub pub_time: u64,
    pub pv: u64,
    pub release_date: String,
    pub rights: Option<BangumiEpisodeRights>,
    pub section_type: u32,
    pub share_copy: String,
    pub share_url: String,
    pub short_link: String,
    pub show_title: String,
    #[serde(rename = "showDrmLoginDialog")]
    pub show_drm_login_dialog: bool,
    pub skip: Option<BangumiSkip>,
    pub status: u32,
    pub subtitle: String,
    pub title: String,
    pub vid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiBadgeInfo {
    pub bg_color: String,
    pub bg_color_night: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiDimension {
    pub height: u32,
    pub rotate: u32,
    pub width: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiInteraction {
    pub graph_version: u32,
    pub interaction: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiMultiViewEp {
    pub ep_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiEpisodeRights {
    pub allow_dm: u32,
    pub allow_download: u32,
    pub area_limit: u32,

    pub allow_demand: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiSkip {
    pub ed: Option<BangumiSkipTime>,
    pub op: Option<BangumiSkipTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiSkipTime {
    pub end: u32,
    pub start: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiFreya {
    #[serde(default)]
    pub bubble_desc: String,
    pub bubble_show_cnt: u32,
    pub icon_show: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiIconFont {
    pub name: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiMultiViewInfo {
    pub changing_dance: String,
    pub is_multi_view_season: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiPayment {
    pub discount: u32,
    pub pay_type: BangumiPayType,
    pub price: String,
    pub promotion: String,
    #[serde(default)]
    pub tip: String,
    pub view_start_time: u64,
    pub vip_discount: u32,
    pub vip_first_promotion: String,
    pub vip_price: String,
    pub vip_promotion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiPayType {
    pub allow_discount: u32,
    pub allow_pack: u32,
    pub allow_ticket: u32,
    pub allow_time_limit: u32,
    pub allow_vip_discount: u32,
    pub forbid_bb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiPayPack {
    pub id: u64,
    pub not_paid_text_for_app: String,
    pub paid_text_for_app: String,
    pub pay_pack_url: String,
    pub status: u32,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiPlayStrategy {
    pub strategies: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiPositive {
    pub id: u64,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiPublish {
    pub is_finish: u32,
    pub is_started: u32,
    pub pub_time: String,
    pub pub_time_show: String,
    pub unknow_pub_date: u32,
    pub weekday: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiRights {
    pub allow_bp: u32,
    pub allow_bp_rank: u32,
    pub allow_download: u32,
    pub allow_review: u32,
    pub area_limit: u32,
    pub ban_area_show: u32,
    pub can_watch: u32,
    pub copyright: String,
    pub forbid_pre: u32,
    pub freya_white: u32,
    pub is_cover_show: u32,
    pub is_preview: u32,
    pub only_vip_download: u32,
    pub resource: String,
    pub watch_platform: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiSeason {
    pub badge: String,
    pub badge_info: Option<BangumiBadgeInfo>,
    pub badge_type: u32,
    pub cover: String,
    pub enable_vt: bool,
    pub horizontal_cover_1610: String,
    pub horizontal_cover_169: String,
    pub icon_font: Option<BangumiIconFont>,
    pub media_id: u64,
    pub new_ep: Option<BangumiSeasonNewEp>,
    pub season_id: u64,
    pub season_title: String,
    pub season_type: u32,
    pub stat: Option<BangumiSeasonStat>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiSeasonNewEp {
    pub cover: String,
    pub id: u64,
    pub index_show: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiSeasonStat {
    pub favorites: u64,
    pub series_follow: u64,
    pub views: u64,
    pub vt: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiSection {
    pub attr: u32,
    pub episode_id: u64,
    #[serde(default)]
    pub episode_ids: Vec<u64>,
    pub episodes: Vec<BangumiSectionEpisode>,
    pub id: u64,
    pub report: Option<BangumiReport>,
    pub title: String,
    pub r#type: u32,
    pub type2: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BangumiSectionEpisode {
    pub aid: u64,
    pub archive_attr: Option<u32>,
    pub badge: String,
    pub badge_info: Option<BangumiBadgeInfo>,
    pub badge_type: u32,
    pub bvid: String,
    pub cid: u64,
    pub cover: String,
    pub dimension: Option<BangumiDimension>,
    pub duration: u64,
    pub enable_vt: bool,
    pub ep_id: u64,
    pub from: String,
    pub icon_font: Option<BangumiIconFont>,
    pub id: u64,
    pub interaction: Option<BangumiInteraction>,
    pub is_view_hide: bool,
    pub link: String,
    pub link_type: String,
    pub long_title: String,
    pub pub_time: u64,
    pub pv: u64,
    pub release_date: String,
    pub report: Option<BangumiReport>,
    pub rights: Option<BangumiEpisodeRights>,
    pub section_type: u32,
    pub share_copy: String,
    pub share_url: String,
    pub short_link: String,
    pub show_title: String,
    #[serde(rename = "showDrmLoginDialog")]
    pub show_drm_login_dialog: bool,
    pub skip: Option<BangumiSkip>,
    pub stat: Option<BangumiStat>,
    pub stat_for_unity: Option<BangumiStatForUnity>,
    pub status: u32,
    pub subtitle: String,
    pub title: String,
    pub toast_title: String,
    pub up_info: Option<BangumiUpInfo>,
    pub vid: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BangumiReport {
    pub aid: String,
    pub ep_title: String,
    pub position: String,
    pub season_id: String,
    pub season_type: String,
    pub section_id: String,
    pub section_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiStatForUnity {
    pub coin: u64,
    pub danmaku: Option<BangumiDanmaku>,
    pub likes: u64,
    pub reply: u64,
    pub vt: Option<BangumiVt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiDanmaku {
    pub icon: String,
    pub pure_text: String,
    pub text: String,
    pub value: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiVt {
    pub icon: String,
    pub pure_text: String,
    pub text: String,
    pub value: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BangumiStat {
    #[serde(alias = "coin")]
    pub coins: u64,
    pub danmakus: u64,
    pub favorite: u64,
    pub favorites: u64,
    pub follow_text: String,
    pub hot: Option<u64>,
    pub likes: u64,
    pub reply: u64,
    pub share: u64,
    #[serde(alias = "play")]
    pub views: u64,
    pub vt: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiSeries {
    pub display_type: u32,
    pub series_id: u64,
    pub series_title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiShow {
    pub wide_screen: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct BangumiUpInfo {
    pub avatar: String,
    pub avatar_subscript_url: String,
    pub follower: u64,
    pub is_follow: u32,
    pub mid: u64,
    pub nickname_color: String,
    pub pendant: Option<BangumiPendant>,
    pub theme_type: u32,
    pub uname: String,
    pub verify_type: u32,
    pub vip_label: Option<VipLabel>,
    pub vip_status: u32,
    pub vip_type: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiUserStatus {
    pub area_limit: u32,
    pub ban_area_show: u32,
    pub follow: u32,
    pub follow_status: u32,
    pub login: u32,
    pub pay: u32,
    pub pay_pack_paid: u32,
    pub sponsor: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiSectionResult {
    #[serde(default)]
    pub main_section: BangumiMainSection,
    pub section: Vec<BangumiMainSection>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BangumiMainSection {
    pub episodes: Vec<BangumiSectionEpisodeInfo>,
    pub id: u64,
    pub r#type: u32,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiSectionEpisodeInfo {
    pub aid: u64,
    pub badge: String,
    pub badge_info: BangumiBadgeInfo,
    pub badge_type: u32,
    pub cid: u64,
    pub cover: String,
    pub from: String,
    pub id: u64,
    pub is_premiere: u32,
    pub long_title: String,
    pub share_url: String,
    pub status: u32,
    pub title: String,
    pub vid: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bangumi::params::{BangumiDetailParams, BangumiInfoParams, BangumiSectionsParams};
    use crate::ids::{EpisodeId, MediaId, SeasonId};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    const TEST_SEASON_ID: u64 = 1172; // ssid
    const TEST_EP_ID: u64 = 21265; // epid
    const TEST_MEDIA_ID: u64 = 28220978; //  mdid

    #[test]
    fn bangumi_section_episode_stat_accepts_compact_stat_shape() {
        let stat = serde_json::from_value::<BangumiStat>(serde_json::json!({
            "coin": 1,
            "danmakus": 2,
            "likes": 3,
            "play": 4,
            "reply": 5,
            "vt": 6
        }))
        .expect("compact section episode stat should parse");

        assert_eq!(stat.coins, 1);
        assert_eq!(stat.views, 4);
        assert_eq!(stat.favorite, 0);
        assert_eq!(stat.follow_text, "");
    }

    #[test]
    fn bangumi_optional_display_fields_accept_absence() {
        let freya = serde_json::from_value::<BangumiFreya>(serde_json::json!({
            "bubble_show_cnt": 0,
            "icon_show": 0
        }))
        .expect("freya bubble description should be optional");
        assert_eq!(freya.bubble_desc, "");

        let payment = serde_json::from_value::<BangumiPayment>(serde_json::json!({
            "discount": 0,
            "pay_type": {
                "allow_discount": 0,
                "allow_pack": 0,
                "allow_ticket": 0,
                "allow_time_limit": 0,
                "allow_vip_discount": 0,
                "forbid_bb": 0
            },
            "price": "0",
            "promotion": "",
            "view_start_time": 0,
            "vip_discount": 0,
            "vip_first_promotion": "",
            "vip_price": "0",
            "vip_promotion": ""
        }))
        .expect("payment tip should be optional");
        assert_eq!(payment.tip, "");
    }

    #[test]
    fn bangumi_unreleased_media_and_sections_accept_empty_state() {
        let media = serde_json::from_value::<BangumiMedia>(serde_json::json!({
            "areas": [],
            "cover": "",
            "horizontal_picture": "",
            "media_id": 1,
            "new_ep": { "id": 1, "index": "", "index_show": "" },
            "season_id": 1,
            "share_url": "",
            "title": "",
            "type": 1,
            "type_name": "番剧"
        }))
        .expect("unreleased media should not require a rating");
        assert_eq!(media.rating.count, 0);
        assert_eq!(media.rating.score, 0.0);

        let sections = serde_json::from_value::<BangumiSectionResult>(serde_json::json!({
            "section": []
        }))
        .expect("unreleased season should not require a main section");
        assert!(sections.main_section.episodes.is_empty());
        assert!(sections.section.is_empty());
    }

    #[test]
    fn bangumi_section_episode_accepts_compact_historical_shape() {
        let episode = serde_json::from_value::<BangumiSectionEpisode>(serde_json::json!({
            "aid": 1,
            "cid": 2,
            "ep_id": 3,
            "id": 3,
            "title": "1"
        }))
        .expect("historical section episode should allow absent display fields");

        assert_eq!((episode.aid, episode.cid, episode.ep_id), (1, 2, 3));
        assert_eq!(episode.badge_type, 0);
        assert_eq!(episode.link_type, "");
        assert!(episode.stat.is_none());
    }

    #[test]
    fn bangumi_section_metadata_accepts_compact_display_shape() {
        let report = serde_json::from_value::<BangumiReport>(serde_json::json!({}))
            .expect("section report metadata fields should be optional");
        assert_eq!(report.aid, "");

        let up_info = serde_json::from_value::<BangumiUpInfo>(serde_json::json!({
            "mid": 1,
            "uname": "up"
        }))
        .expect("section up info display fields should be optional");
        assert_eq!(up_info.mid, 1);
        assert_eq!(up_info.avatar_subscript_url, "");
    }

    #[test]
    fn bangumi_section_accepts_missing_redundant_episode_ids() {
        let section = serde_json::from_value::<BangumiSection>(serde_json::json!({
            "attr": 0,
            "episode_id": 1,
            "episodes": [],
            "id": 1,
            "report": null,
            "title": "",
            "type": 0,
            "type2": 0
        }))
        .expect("section should not require redundant episode_ids");

        assert!(section.episode_ids.is_empty());
    }

    #[test]
    fn bangumi_pendant_accepts_negative_sentinel_id() {
        let pendant = serde_json::from_value::<BangumiPendant>(serde_json::json!({
            "image": "",
            "name": "",
            "pid": -1014771399
        }))
        .expect("pendant sentinel id should remain representable");

        assert_eq!(i128::from(pendant.pid), -1014771399_i128);
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "review-user" => {
                include_bytes!("../../tests/contracts/bangumi/info/review-user/contract.json")
                    .as_slice()
            }
            "season-detail-season" => include_bytes!(
                "../../tests/contracts/bangumi/info/season-detail-season/contract.json"
            )
            .as_slice(),
            "season-detail-episode" => include_bytes!(
                "../../tests/contracts/bangumi/info/season-detail-episode/contract.json"
            )
            .as_slice(),
            "season-section" => {
                include_bytes!("../../tests/contracts/bangumi/info/season-section/contract.json")
                    .as_slice()
            }
            _ => unreachable!("unknown bangumi info contract"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_bangumi_info() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .bangumi()
            .info(BangumiInfoParams::new(MediaId::new(TEST_MEDIA_ID)?))
            .await?;
        tracing::info!("{:#?}", data);

        assert_eq!(data.media.media_id, TEST_MEDIA_ID);
        assert!(!data.media.title.is_empty());
        assert!(!data.media.areas.is_empty());

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_bangumi_detail_by_season_id() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .bangumi()
            .detail_by_season_id(SeasonId::new(TEST_SEASON_ID)?)
            .await?;
        tracing::info!("{:#?}", data);

        assert_eq!(data.season_id, TEST_SEASON_ID);
        assert!(!data.title.is_empty());
        assert!(!data.episodes.is_empty());

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_bangumi_detail_by_epid() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .bangumi()
            .detail_by_ep_id(EpisodeId::new(TEST_EP_ID)?)
            .await?;
        tracing::info!("{:#?}", data);

        assert!(!data.title.is_empty());
        assert!(!data.episodes.is_empty());

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_bangumi_section() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .bangumi()
            .sections(BangumiSectionsParams::new(SeasonId::new(TEST_SEASON_ID)?))
            .await?;
        tracing::info!("{:#?}", data);

        assert!(!data.main_section.episodes.is_empty());

        Ok(())
    }

    #[test]
    fn bangumi_review_user_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("review-user")?;
        let params = BangumiInfoParams::new(MediaId::new(TEST_MEDIA_ID)?);

        assert_eq!(contract.name, "bangumi.info.review_user");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/pgc/review/user"
        );
        assert_eq!(
            contract.request.query.get("media_id").map(String::as_str),
            Some("28220978")
        );
        assert_eq!(
            params.query_pairs(),
            vec![("media_id", "28220978".to_string())]
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("BangumiInfoResult")
        );
        Ok(())
    }

    #[test]
    fn bangumi_review_user_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/bangumi/info/review-user/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/bangumi/info/review-user/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/bangumi/info/review-user/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<BangumiInfoResult>::from_slice(bytes)?.into_payload()?;

            assert_eq!(payload.media.media_id, TEST_MEDIA_ID);
            assert_eq!(payload.media.season_id, TEST_SEASON_ID);
            assert!(!payload.media.title.is_empty());
        }
        Ok(())
    }

    #[test]
    fn bangumi_detail_by_season_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("season-detail-season")?;
        let params = BangumiDetailParams::from_season_id(SeasonId::new(TEST_SEASON_ID)?);

        assert_eq!(contract.name, "bangumi.info.season_detail_by_season_id");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/pgc/view/web/season"
        );
        assert_eq!(
            contract.request.query.get("season_id").map(String::as_str),
            Some("1172")
        );
        assert_eq!(
            params.query_pairs(),
            vec![("season_id", "1172".to_string())]
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("BangumiDetailResult")
        );
        Ok(())
    }

    #[test]
    fn bangumi_detail_by_episode_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("season-detail-episode")?;
        let params = BangumiDetailParams::from_episode_id(EpisodeId::new(TEST_EP_ID)?);

        assert_eq!(contract.name, "bangumi.info.season_detail_by_ep_id");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/pgc/view/web/season"
        );
        assert_eq!(
            contract.request.query.get("ep_id").map(String::as_str),
            Some("21265")
        );
        assert_eq!(params.query_pairs(), vec![("ep_id", "21265".to_string())]);
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("BangumiDetailResult")
        );
        Ok(())
    }

    #[test]
    fn bangumi_detail_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/bangumi/info/season-detail-season/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/bangumi/info/season-detail-season/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/bangumi/info/season-detail-season/responses/vip.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/bangumi/info/season-detail-episode/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/bangumi/info/season-detail-episode/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/bangumi/info/season-detail-episode/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<BangumiDetailResult>::from_slice(bytes)?.into_payload()?;

            assert_eq!(payload.season_id, TEST_SEASON_ID);
            assert!(!payload.title.is_empty());
            assert!(!payload.episodes.is_empty());
        }
        Ok(())
    }

    #[test]
    fn bangumi_section_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("season-section")?;
        let params = BangumiSectionsParams::new(SeasonId::new(TEST_SEASON_ID)?);

        assert_eq!(contract.name, "bangumi.info.season_section");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/pgc/web/season/section"
        );
        assert_eq!(
            contract.request.query.get("season_id").map(String::as_str),
            Some("1172")
        );
        assert_eq!(
            params.query_pairs(),
            vec![("season_id", "1172".to_string())]
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("BangumiSectionResult")
        );
        Ok(())
    }

    #[test]
    fn bangumi_section_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/bangumi/info/season-section/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/bangumi/info/season-section/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/bangumi/info/season-section/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<BangumiSectionResult>::from_slice(bytes)?.into_payload()?;

            assert!(!payload.main_section.episodes.is_empty());
        }
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/bangumi/info/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn bangumi_info_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body("review-user", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<BangumiInfoResult>>(body)?
                    .into_payload()?;

                assert_eq!(payload.media.media_id, TEST_MEDIA_ID);
            }

            if let Some(body) = local_probe_body("season-detail-season", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<BangumiDetailResult>>(body)?
                    .into_payload()?;

                assert_eq!(payload.season_id, TEST_SEASON_ID);
            }

            if let Some(body) = local_probe_body("season-detail-episode", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<BangumiDetailResult>>(body)?
                    .into_payload()?;

                assert_eq!(payload.season_id, TEST_SEASON_ID);
            }

            if let Some(body) = local_probe_body("season-section", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<BangumiSectionResult>>(body)?
                    .into_payload()?;

                assert!(!payload.main_section.episodes.is_empty());
            }
        }
        Ok(())
    }
}
