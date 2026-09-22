//! 卡片信息
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/article/card.md)

use super::models::{ArticleAuthor, ArticleCategory, ArticleMedia, ArticleStats};
use serde::{Deserialize, Serialize};

/// 卡片信息响应类型
pub type CardData = std::collections::HashMap<String, CardItem>;

/// 卡片项目（可以是视频、专栏或直播间）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CardItem {
    /// 视频卡片
    Video(Box<VideoCard>),
    /// 专栏卡片
    Article(Box<ArticleCard>),
    /// 直播间卡片
    Live(Box<LiveCard>),

    /// 未知卡片类型
    Unknown(serde_json::Value),
}

/// 视频卡片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoCard {
    /// 视频aid
    pub aid: i64,
    /// 视频bvid
    pub bvid: String,
    /// 视频cid
    pub cid: i64,
    /// 版权信息
    pub copyright: i32,
    /// 封面图片
    pub pic: String,
    /// 创建时间
    pub ctime: i64,
    /// 视频描述
    pub desc: String,
    /// 视频尺寸信息
    pub dimension: VideoDimension,
    /// 视频时长
    pub duration: i64,
    /// 动态内容
    pub dynamic: String,
    /// UP主信息
    pub owner: VideoOwner,
    /// 发布时间
    pub pubdate: i64,
    /// 视频权限
    pub rights: VideoRights,
    /// 短链接
    pub short_link_v2: String,
    /// 视频统计信息
    pub stat: VideoStat,
    /// 视频状态
    pub state: i32,
    /// 分区ID
    pub tid: i32,
    /// 视频标题
    pub title: String,
    /// 分区名称
    pub tname: String,
    /// 分P数量
    pub videos: i32,
    /// VT开关
    pub vt_switch: bool,
}

/// 视频尺寸信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoDimension {
    /// 高度
    pub height: i32,
    /// 旋转角度
    pub rotate: i32,
    /// 宽度
    pub width: i32,
}

/// 视频UP主信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoOwner {
    /// UP主头像
    pub face: String,
    /// UP主mid
    pub mid: i64,
    /// UP主昵称
    pub name: String,
}

/// 视频权限信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoRights {
    /// 是否付费
    pub arc_pay: i32,
    /// 是否自动播放
    pub autoplay: i32,
    /// 是否可充电
    pub bp: i32,
    /// 是否可下载
    pub download: i32,
    /// 是否可充电
    pub elec: i32,
    /// 是否高清
    pub hd5: i32,
    /// 是否合作视频
    pub is_cooperation: i32,
    /// 是否电影
    pub movie: i32,
    /// 是否无背景
    pub no_background: i32,
    /// 是否禁止转载
    pub no_reprint: i32,
    /// 是否付费
    pub pay: i32,
    /// 是否付费观看
    pub pay_free_watch: i32,
    /// 是否UGC付费
    pub ugc_pay: i32,
    /// 是否UGC付费预览
    pub ugc_pay_preview: i32,
}

/// 视频统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoStat {
    /// 视频aid
    pub aid: i64,
    /// 投币数
    pub coin: i64,
    /// 弹幕数
    pub danmaku: i64,
    /// 点踩数
    pub dislike: i64,
    /// 收藏数
    pub favorite: i64,
    /// 历史排名
    pub his_rank: i32,
    /// 点赞数
    pub like: i64,
    /// 当前排名
    pub now_rank: i32,
    /// 评论数
    pub reply: i64,
    /// 分享数
    pub share: i64,
    /// 播放数
    pub view: i64,
    /// VT值
    pub vt: i32,
    /// VV值
    pub vv: i32,
}

/// 专栏卡片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleCard {
    /// 活动ID
    pub act_id: i64,
    /// 申请时间
    pub apply_time: String,
    /// 属性
    pub attributes: i32,
    /// 认证标记
    #[serde(rename = "authenMark")]
    pub authen_mark: Option<serde_json::Value>,
    /// 作者信息
    pub author: ArticleAuthor,
    /// 横幅URL
    pub banner_url: String,
    /// 分类列表
    pub categories: Vec<ArticleCategory>,
    /// 主分类
    pub category: ArticleCategory,
    /// 审核状态
    pub check_state: i32,
    /// 审核时间
    pub check_time: String,
    /// 内容图片列表
    pub content_pic_list: Option<serde_json::Value>,
    /// 封面视频ID
    pub cover_avid: i64,
    /// 创建时间
    pub ctime: i64,
    /// 争议信息
    pub dispute: Option<serde_json::Value>,
    /// 动态内容
    pub dynamic: String,
    /// 专栏ID
    pub id: i64,
    /// 图片URL列表
    pub image_urls: Vec<String>,
    /// 是否点赞
    pub is_like: bool,
    /// 文集信息
    pub list: Option<ArticleList>,
    /// 媒体信息
    pub media: ArticleMedia,
    /// 修改时间
    pub mtime: i64,
    /// 原始图片URL列表
    pub origin_image_urls: Vec<String>,
    /// 原始模板ID
    pub origin_template_id: i32,
    /// 是否原创
    pub original: i32,
    /// 是否私密发布
    pub private_pub: i32,
    /// 发布时间
    pub publish_time: i64,
    /// 是否转载
    pub reprint: i32,
    /// 状态
    pub state: i32,
    /// 统计信息
    pub stats: ArticleStats,
    /// 摘要
    pub summary: String,
    /// 模板ID
    pub template_id: i32,
    /// 标题
    pub title: String,
    /// 顶部视频信息
    pub top_video_info: Option<serde_json::Value>,
    /// 类型
    pub r#type: i32,
    /// 字数
    pub words: i64,
}

/// 作者VIP信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorVip {
    /// 头像订阅
    pub avatar_subscript: i32,
    /// 到期时间
    pub due_date: i64,
    /// 标签信息
    pub label: VipLabel,
    /// 昵称颜色
    pub nickname_color: String,
    /// VIP状态
    pub status: i32,
    /// 主题类型
    pub theme_type: i32,
    /// VIP类型
    pub r#type: i32,
    /// 支付类型
    pub vip_pay_type: i32,
}

/// VIP标签
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VipLabel {
    /// 标签主题
    pub label_theme: String,
    /// 标签路径
    pub path: String,
    /// 标签文本
    pub text: String,
}

/// 专栏文集信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleList {
    /// 申请时间
    pub apply_time: String,
    /// 文章数量
    pub articles_count: i32,
    /// 审核时间
    pub check_time: String,
    /// 创建时间
    pub ctime: i64,
    /// 文集ID
    pub id: i64,
    /// 文集图片
    pub image_url: String,
    /// 作者ID
    pub mid: i64,
    /// 文集名称
    pub name: String,
    /// 发布时间
    pub publish_time: i64,
    /// 阅读量
    pub read: i64,
    /// 原因
    pub reason: String,
    /// 状态
    pub state: i32,
    /// 摘要
    pub summary: String,
    /// 更新时间
    pub update_time: i64,
    /// 字数
    pub words: i64,
}

/// 直播间卡片
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveCard {
    /// 分区完整名称
    pub area_v2_name: String,
    /// 直播封面
    pub cover: String,
    /// 主播头像
    pub face: String,
    /// 直播状态
    pub live_status: i32,
    /// 在线人数
    pub online: i64,
    /// 挂件RU
    pub pendent_ru: String,
    /// 挂件RU颜色
    pub pendent_ru_color: String,
    /// 挂件RU图片
    pub pendent_ru_pic: String,
    /// 角色
    pub role: i32,
    /// 直播间长ID
    pub room_id: i64,
    /// 直播间标题
    pub title: String,
    /// 主播UID
    pub uid: i64,
    /// 主播用户名
    pub uname: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::article::params::ArticleCardsParams;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use std::mem;

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/article/cards/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_article_cards() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");

        let params = ArticleCardsParams::new("av2,cv1,cv2")?;

        let data = bpi.article().cards(params).await?;
        tracing::info!("{:#?}", data);

        Ok(())
    }

    #[test]
    fn card_item_keeps_large_payloads_boxed() {
        assert!(mem::size_of::<CardItem>() <= 64);
    }

    #[test]
    fn article_cards_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params = ArticleCardsParams::new("av2,cv1,cv2")?;

        assert_eq!(contract.name, "article.cards");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/article/cards"
        );
        assert_eq!(
            contract.request.query.get("ids").map(String::as_str),
            Some("av2,cv1,cv2")
        );
        assert_eq!(
            contract
                .request
                .query
                .get("web_location")
                .map(String::as_str),
            Some("333.1305")
        );
        assert_eq!(
            params.query_pairs(),
            vec![
                ("ids", "av2,cv1,cv2".to_string()),
                ("web_location", "333.1305".to_string()),
            ]
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.error.as_deref(),
            Some("wbi_risk_control")
        );
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("CardData")
        );
        Ok(())
    }

    #[test]
    fn article_cards_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!("../../tests/contracts/article/cards/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/article/cards/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<CardData>::from_slice(bytes)?.into_payload()?;

            assert!(payload.contains_key("av2"));
            assert!(payload.contains_key("cv1"));
        }
        Ok(())
    }

    #[test]
    fn article_cards_anonymous_fixture_records_wbi_error() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/article/cards/responses/anonymous.error.json"
        ))?
        .ensure_success()
        .unwrap_err();

        assert_eq!(err.code(), Some(-352));
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/article/read/cards/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn article_cards_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload = serde_json::from_value::<ApiEnvelope<CardData>>(body)?.into_payload()?;

            assert!(payload.contains_key("cv1"));
        }
        Ok(())
    }
}
