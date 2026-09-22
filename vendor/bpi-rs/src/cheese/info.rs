//! 课程（PUGV）相关 API
//!
//! [参考文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/cheese/info.md)

use serde::{Deserialize, Serialize};

// ==========================
// 数据结构（/pugv/view/web/season）
// ==========================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseInfo {
    pub brief: CourseBrief,
    pub coupon: CourseCoupon,
    pub cover: String,
    pub episode_page: CourseEpisodePage,
    pub episode_sort: i32,
    pub episodes: Vec<CourseEpisode>,
    pub faq: CourseFaq,
    pub faq1: CourseFaq1,
    pub payment: CoursePayment,
    pub purchase_note: CoursePurchaseNote,
    pub purchase_protocol: CoursePurchaseProtocol,
    pub release_bottom_info: String,
    pub release_info: String,
    pub release_info2: String,
    pub release_status: String,
    pub season_id: u64,
    pub share_url: String,
    pub short_link: String,
    pub stat: CourseStat,
    pub status: i32,
    pub subtitle: String,
    pub title: String,
    pub up_info: CourseUpInfo,
    pub user_status: CourseUserStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseBrief {
    pub content: String,
    pub img: Vec<CourseBriefImg>,
    pub title: String,
    pub r#type: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseBriefImg {
    pub aspect_ratio: f64,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseCoupon {
    pub amount: f64,
    pub expire_time: String, // YYYY-MM-DD HH:MM:SS
    pub start_time: String,  // YYYY-MM-DD HH:MM:SS
    pub status: i32,
    pub title: String,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseEpisodePage {
    pub next: bool,
    pub num: u32,
    pub size: u32,
    pub total: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseEpisode {
    pub aid: u64,          // 课程分集 avid（与普通稿件部分不互通）
    pub cid: u64,          // 课程分集 cid（与普通视频部分不互通）
    pub duration: u64,     // 单位：秒
    pub from: String,      // "pugv"
    pub id: u64,           // 课程分集 epid（与番剧不互通）
    pub index: u32,        // 课程分集数
    pub page: u32,         // 一般为 1
    pub play: u64,         // 分集播放量
    pub release_date: u64, // 发布时间（时间戳）
    pub status: i32,       // 1 可看、2 不可看
    pub title: String,     // 分集标题
    pub watched: bool,     // 是否观看（需登录 + 正确 Referer）
    #[serde(rename = "watchedHistory")] // 文档里为驼峰
    pub watched_history: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseFaq {
    pub content: String,
    pub link: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseFaq1 {
    pub items: Vec<CourseFaqItem>,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseFaqItem {
    pub answer: String,
    pub question: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoursePayment {
    pub desc: String,
    pub discount_desc: String,
    #[serde(default)]
    pub discount_prefix: String,
    pub pay_shade: String,
    pub price: f64,
    pub price_format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoursePurchaseNote {
    pub content: String,
    pub link: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoursePurchaseProtocol {
    pub link: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseStat {
    pub play: u64,
    pub play_desc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseUpInfo {
    pub avatar: String,
    pub brief: String,
    pub follower: u64,
    pub is_follow: i32, // 0 未关注，1 已关注
    pub link: String,
    pub mid: u64,
    pub pendant: CoursePendant,
    pub uname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoursePendant {
    pub image: String,
    pub name: String,
    pub pid: u64,
    // pub follower: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseUserStatus {
    pub favored: i32, // 0 未收藏，1 已收藏
    pub favored_count: u64,
    pub payed: i32, // 0 未购买，1 已购买
    #[serde(default)]
    pub progress: Option<CourseProgress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseProgress {
    pub last_ep_id: u64,
    pub last_ep_index: String,
    pub last_time: u64, // 秒
}

// ==========================
// 数据结构（/pugv/view/web/ep/list）
// ==========================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseEpList {
    pub items: Vec<CourseEpisode>, // 结构与 CourseEpisode 一致
    pub page: CourseEpPage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseEpPage {
    pub next: bool, // 是否存在下一页
    pub num: u32,   // 当前页码
    pub size: u32,  // 每页项数
    pub total: u32, // 总计项数
}

// ==========================
// 测试
// ==========================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cheese::{CheeseEpListParams, CheeseInfoParams};
    use crate::ids::{EpisodeId, SeasonId};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    const TEST_SEASON_ID: u64 = 556;
    const TEST_EP_ID: u64 = 20767;

    fn contract(name: &str) -> BpiResult<EndpointContract> {
        let bytes = match name {
            "season-detail-season" => include_bytes!(
                "../../tests/contracts/cheese/info/season-detail-season/contract.json"
            )
            .as_slice(),
            "season-detail-episode" => include_bytes!(
                "../../tests/contracts/cheese/info/season-detail-episode/contract.json"
            )
            .as_slice(),
            "ep-list" => {
                include_bytes!("../../tests/contracts/cheese/info/ep-list/contract.json").as_slice()
            }
            _ => unreachable!("unknown cheese info contract"),
        };
        EndpointContract::from_slice(bytes)
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_cheese_info_by_season_id() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .cheese()
            .info_by_season_id(SeasonId::new(TEST_SEASON_ID)?)
            .await?;

        assert_eq!(data.season_id, TEST_SEASON_ID);
        tracing::info!("{:#?}", data);
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_cheese_info_by_ep_id() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .cheese()
            .info_by_ep_id(EpisodeId::new(TEST_EP_ID)?)
            .await?;
        assert_eq!(data.season_id, TEST_SEASON_ID);

        tracing::info!("课程标题: {:?}", data.title);
        tracing::info!("课程 ssid: {:?}", data.season_id);
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_cheese_ep_list() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .cheese()
            .ep_list(
                CheeseEpListParams::new(SeasonId::new(TEST_SEASON_ID)?)
                    .with_page_size(50)?
                    .with_page(1)?,
            )
            .await?;
        assert_eq!(data.items.first().unwrap().id, TEST_SEASON_ID);

        tracing::info!("课程标题: {:?}", data.items.first().unwrap().title);
        tracing::info!("课程 ssid: {:?}", data.items.first().unwrap());
        Ok(())
    }

    #[test]
    fn cheese_info_params_serializes_season_id() -> Result<(), BpiError> {
        let params = CheeseInfoParams::from_season_id(SeasonId::new(TEST_SEASON_ID)?);

        assert_eq!(
            params.query_pairs(),
            vec![("season_id", TEST_SEASON_ID.to_string())]
        );
        Ok(())
    }

    #[test]
    fn cheese_ep_list_params_rejects_zero_page() -> Result<(), BpiError> {
        let err = CheeseEpListParams::new(SeasonId::new(TEST_SEASON_ID)?)
            .with_page(0)
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "pn", .. }
        ));
        Ok(())
    }

    #[test]
    fn cheese_info_by_season_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("season-detail-season")?;
        let params = CheeseInfoParams::from_season_id(SeasonId::new(TEST_SEASON_ID)?);

        assert_eq!(contract.name, "cheese.info.season_detail_by_season_id");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/pugv/view/web/season"
        );
        assert_eq!(
            contract.request.query.get("season_id").map(String::as_str),
            Some("556")
        );
        assert_eq!(
            params.query_pairs(),
            vec![("season_id", TEST_SEASON_ID.to_string())]
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("CourseInfo")
        );
        Ok(())
    }

    #[test]
    fn cheese_info_by_episode_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("season-detail-episode")?;
        let params = CheeseInfoParams::from_episode_id(EpisodeId::new(TEST_EP_ID)?);

        assert_eq!(contract.name, "cheese.info.season_detail_by_ep_id");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/pugv/view/web/season"
        );
        assert_eq!(
            contract.request.query.get("ep_id").map(String::as_str),
            Some("20767")
        );
        assert_eq!(
            params.query_pairs(),
            vec![("ep_id", TEST_EP_ID.to_string())]
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.fixture_kind.as_deref(),
            Some("trimmed_probe_body")
        );
        Ok(())
    }

    #[test]
    fn cheese_ep_list_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("ep-list")?;
        let params = CheeseEpListParams::new(SeasonId::new(TEST_SEASON_ID)?)
            .with_page_size(50)?
            .with_page(1)?;

        assert_eq!(contract.name, "cheese.info.ep_list");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/pugv/view/web/ep/list"
        );
        assert_eq!(
            contract.request.query.get("season_id").map(String::as_str),
            Some("556")
        );
        assert_eq!(
            contract.request.query.get("ps").map(String::as_str),
            Some("50")
        );
        assert_eq!(
            contract.request.query.get("pn").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            params.query_pairs(),
            vec![
                ("season_id", TEST_SEASON_ID.to_string()),
                ("ps", "50".to_string()),
                ("pn", "1".to_string()),
            ]
        );
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("CourseEpList")
        );
        Ok(())
    }

    #[test]
    fn cheese_info_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/cheese/info/season-detail-season/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/cheese/info/season-detail-season/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/cheese/info/season-detail-season/responses/vip.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/cheese/info/season-detail-episode/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/cheese/info/season-detail-episode/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/cheese/info/season-detail-episode/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<CourseInfo>::from_slice(bytes)?.into_payload()?;

            assert_eq!(payload.season_id, TEST_SEASON_ID);
            assert_eq!(payload.episodes.len(), 2);
            assert_eq!(payload.user_status.payed, 0);
            assert_eq!(payload.title, "【暑期5折】法语0-B2高级班");
        }
        Ok(())
    }

    #[test]
    fn cheese_ep_list_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/cheese/info/ep-list/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/cheese/info/ep-list/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!("../../tests/contracts/cheese/info/ep-list/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<CourseEpList>::from_slice(bytes)?.into_payload()?;

            assert_eq!(payload.page.total, 603);
            assert_eq!(payload.items.len(), 2);
            assert_eq!(payload.items[0].id, 20766);
            assert_eq!(payload.items[0].aid, 640_041_584);
            assert_eq!(payload.items[0].cid, 1_641_007_864);
        }
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/cheese/read/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn cheese_info_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            for endpoint in ["info-season", "info-episode"] {
                let Some(body) = local_probe_body(endpoint, profile) else {
                    continue;
                };
                let payload =
                    serde_json::from_value::<ApiEnvelope<CourseInfo>>(body)?.into_payload()?;

                assert_eq!(payload.season_id, TEST_SEASON_ID);
                assert_eq!(payload.episodes.len(), 603);
                assert_eq!(payload.user_status.payed, 0);
            }

            let Some(body) = local_probe_body("ep-list", profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<CourseEpList>>(body)?.into_payload()?;

            assert_eq!(payload.page.total, 603);
            assert_eq!(payload.items.len(), 50);
        }
        Ok(())
    }
}
