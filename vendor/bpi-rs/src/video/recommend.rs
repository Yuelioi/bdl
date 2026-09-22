//! 视频推荐相关接口
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/video)
use serde::{Deserialize, Serialize};

pub(crate) const HOMEPAGE_RECOMMENDATIONS_ENDPOINT: &str =
    "https://api.bilibili.com/x/web-interface/wbi/index/top/feed/rcmd";
pub(crate) const RELATED_VIDEOS_ENDPOINT: &str =
    "https://api.bilibili.com/x/web-interface/archive/related";

// --- 视频推荐相关数据结构体 ---

/// 视频作者信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Owner {
    /// UP主mid
    pub mid: u64,
    /// UP昵称
    pub name: String,
    /// 头像URL
    pub face: String,
}

/// 视频统计数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Stat {
    /// 播放量
    pub view: u64,
    /// 视频aid
    pub aid: u64,
    /// 弹幕数
    pub danmaku: u64,
    /// 评论数
    pub reply: u64,
    /// 收藏数
    pub favorite: u64,
    /// 硬币数
    pub coin: u64,
    /// 分享数
    pub share: u64,
    /// 当前排名
    pub now_rank: u64,
    /// 历史最高排名
    pub his_rank: u64,
    /// 点赞数
    pub like: u64,
    /// 点踩数
    pub dislike: u64,
}

/// 主页推荐视频/直播统计数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HomeRmdStat {
    /// 播放量
    pub view: u64,
    /// 弹幕数
    pub danmaku: u64,
    /// 点赞数
    pub like: u64,
}

/// 视频版权信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rights {
    pub bp: u8,
    pub elec: u8,
    pub download: u8,
    pub movie: u8,
    pub pay: u8,
    pub hd5: u8,
    pub no_reprint: u8,
    pub autoplay: u8,
    pub ugc_pay: u8,
    pub is_cooperation: u8,
    pub ugc_pay_preview: u8,
    pub no_background: u8,
}

/// 视频分辨率信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Dimension {
    pub width: u32,
    pub height: u32,
    pub rotate: u8,
}

/// 单视频推荐列表项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelatedVideo {
    pub aid: u64,
    pub videos: u32,
    pub tid: u32,
    pub tname: String,
    pub copyright: u8,
    pub pic: String,
    pub title: String,
    pub pubdate: u64,
    pub ctime: u64,
    pub desc: String,
    pub state: i8,
    pub duration: u64,
    pub rights: Rights,
    pub owner: Owner,
    pub stat: Stat,
    pub dynamic: String,
    pub cid: u64,
    pub dimension: Dimension,
    pub bvid: String,
    #[serde(default)]
    pub short_link_v2: String,
}

/// 首页推荐视频列表项中的推荐理由
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RcmdReason {
    /// 原因类型, 0: 无, 1: 已关注, 3: 高点赞量
    #[serde(rename = "reason_type")]
    pub reason_type: u8,
    /// 原因描述
    pub content: Option<String>,
}

/// 首页推荐视频列表项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RcmdItem {
    pub av_feature: Option<serde_json::Value>,
    /// 商业推广信息，若无则为 null
    pub business_info: Option<serde_json::Value>,
    /// 视频bvid
    pub bvid: String,
    /// 稿件cid
    pub cid: u64,
    /// 视频时长
    pub duration: u64,
    /// 目标类型, "av": 视频, "ogv": 边栏, "live": 直播
    pub goto: String,
    /// 视频aid / 直播间id
    pub id: u64,
    /// 是否已关注, 0: 未关注, 1: 已关注
    pub is_followed: u8,
    pub is_stock: u8,
    /// UP主信息
    pub owner: Owner,
    /// 封面
    pub pic: String,
    pub pos: u8,
    /// 发布时间
    pub pubdate: u64,
    /// 推荐理由
    pub rcmd_reason: Option<RcmdReason>,
    /// 直播间信息
    pub room_info: Option<serde_json::Value>,
    pub show_info: u8,
    /// 视频状态信息
    pub stat: Option<HomeRmdStat>,
    /// 标题
    pub title: String,
    pub track_id: String,
    /// 目标页 URI
    pub uri: String,
}

/// 首页推荐列表响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RcmdFeedResponseData {
    /// 推荐列表
    pub item: Vec<RcmdItem>,
    /// 用户mid，未登录为0
    pub mid: u64,
    pub preload_expose_pct: f32,
    pub preload_floor_expose_pct: f32,
}

// --- 测试模块 ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::Aid;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::video::params::{VideoHomepageRecommendationsParams, VideoRelatedParams};
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use tracing::info;

    const TEST_AID: u64 = 10001;

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_related_videos_by_aid() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .video()
            .related_videos(VideoRelatedParams::from_aid(Aid::new(TEST_AID)?))
            .await?;

        info!("单视频推荐列表: {:?}", data);

        assert!(!data.is_empty());
        assert!(data.len() <= 40);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_homepage_recommendations() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let params = VideoHomepageRecommendationsParams::new()
            .page_size(12)?
            .fresh_idx(1)?
            .fetch_row(1)?;
        let data = bpi.video().homepage_recommendations(params).await?;

        info!("首页推荐列表: {:?}", data);

        assert!(!data.item.is_empty());
        assert!(data.item.len() <= 30);

        Ok(())
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "related-videos" => include_bytes!(
                "../../tests/contracts/video/player-read/related-videos/contract.json"
            )
            .as_slice(),
            "homepage-recommendations" => include_bytes!(
                "../../tests/contracts/video/player-read/homepage-recommendations/contract.json"
            )
            .as_slice(),
            _ => unreachable!("unknown video recommend contract"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn video_related_videos_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("related-videos")?;
        let params = VideoRelatedParams::from_bvid("BV1xx411c7mD".parse()?);

        assert_eq!(contract.name, "video.related_videos");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), RELATED_VIDEOS_ENDPOINT);
        assert_eq!(
            contract.request.query.get("bvid").map(String::as_str),
            Some("BV1xx411c7mD")
        );
        assert_eq!(
            params.query_pairs(),
            vec![("bvid", "BV1xx411c7mD".to_string())]
        );
        assert_eq!(contract.cases.len(), 3);
        Ok(())
    }

    #[test]
    fn video_related_videos_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<Vec<RelatedVideo>>::from_slice(include_bytes!(
            "../../tests/contracts/video/player-read/related-videos/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.len(), 1);
        Ok(())
    }

    #[test]
    fn video_homepage_recommendations_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("homepage-recommendations")?;
        let params = VideoHomepageRecommendationsParams::new();

        assert_eq!(contract.name, "video.homepage_recommendations");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            HOMEPAGE_RECOMMENDATIONS_ENDPOINT
        );
        assert!(contract.request.auth.requires_wbi());
        assert_eq!(
            contract.request.query.get("ps").map(String::as_str),
            Some("12")
        );
        assert_eq!(params.query_pairs().len(), 6);
        assert_eq!(contract.cases.len(), 3);
        Ok(())
    }

    #[test]
    fn video_homepage_recommendations_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<RcmdFeedResponseData>::from_slice(include_bytes!(
            "../../tests/contracts/video/player-read/homepage-recommendations/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.item.len(), 1);
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/video/player-read/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn video_recommend_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body("related-videos", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<Vec<RelatedVideo>>>(body)?
                    .into_payload()?;

                assert!(!payload.is_empty());
            }

            if let Some(body) = local_probe_body("homepage-recommendations", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<RcmdFeedResponseData>>(body)?
                    .into_payload()?;

                assert!(!payload.item.is_empty());
            }
        }
        Ok(())
    }
}
