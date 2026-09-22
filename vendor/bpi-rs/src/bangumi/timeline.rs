//! 番剧或影视时间线
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/bangumi/timeline.md)
use serde::{Deserialize, Serialize};

/// 番剧类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BangumiTimelineType {
    /// 番剧
    Anime = 1,
    /// 电影
    Movie = 3,
    /// 国创
    ChineseAnimation = 4,
}

impl BangumiTimelineType {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiTimelineDay {
    pub date: String,
    pub date_ts: i64,
    pub day_of_week: i32,
    pub episodes: Vec<BangumiTimelineEpisode>,
    pub is_today: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiTimelineEpisode {
    pub cover: String,
    pub delay: i32,
    pub delay_id: i64,
    pub delay_index: String,
    pub delay_reason: String,
    pub ep_cover: String,
    pub episode_id: i64,
    pub pub_index: String,
    pub pub_time: String,
    pub pub_ts: i64,
    pub published: i32,
    pub follows: String,
    pub plays: String,
    pub season_id: i64,
    pub square_cover: String,
    pub title: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bangumi::BangumiTimelineParams;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/bangumi/timeline/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_bangumi_timeline() {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .bangumi()
            .timeline(
                BangumiTimelineParams::new(BangumiTimelineType::Anime, 3, 7)
                    .expect("valid timeline params"),
            )
            .await;
        assert!(data.is_ok());
        let data = data.unwrap();

        assert!(!data.is_empty());
        for day in &data {
            assert!(!day.date.is_empty());
            assert!(day.day_of_week >= 1 && day.day_of_week <= 7);
            assert!(!day.episodes.is_empty());
            for episode in &day.episodes {
                assert!(!episode.title.is_empty());
                assert!(episode.season_id > 0);
            }
        }
    }

    #[test]
    fn test_bangumi_timeline_invalid_before() {
        let error = BangumiTimelineParams::new(BangumiTimelineType::Anime, 8, 7).unwrap_err();
        match error {
            BpiError::InvalidParameter { field, message } => {
                assert_eq!(field, "before");
                assert_eq!(message, "value must be between 0 and 7");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[test]
    fn test_bangumi_timeline_invalid_after() {
        let error = BangumiTimelineParams::new(BangumiTimelineType::Anime, 3, 8).unwrap_err();
        match error {
            BpiError::InvalidParameter { field, message } => {
                assert_eq!(field, "after");
                assert_eq!(message, "value must be between 0 and 7");
            }
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[test]
    fn test_bangumi_timeline_type() {
        assert_eq!(BangumiTimelineType::Anime.as_i32(), 1);
        assert_eq!(BangumiTimelineType::Movie.as_i32(), 3);
        assert_eq!(BangumiTimelineType::ChineseAnimation.as_i32(), 4);
    }

    #[test]
    fn bangumi_timeline_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params = BangumiTimelineParams::new(BangumiTimelineType::Anime, 3, 7)?;

        assert_eq!(contract.name, "bangumi.timeline");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/pgc/web/timeline"
        );
        assert_eq!(
            contract.request.query.get("types").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            contract.request.query.get("before").map(String::as_str),
            Some("3")
        );
        assert_eq!(
            contract.request.query.get("after").map(String::as_str),
            Some("7")
        );
        assert_eq!(
            params.query_pairs(),
            vec![
                ("types", "1".to_string()),
                ("before", "3".to_string()),
                ("after", "7".to_string()),
            ]
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("Vec<BangumiTimelineDay>")
        );
        Ok(())
    }

    #[test]
    fn bangumi_timeline_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/bangumi/timeline/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!("../../tests/contracts/bangumi/timeline/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/bangumi/timeline/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload =
                ApiEnvelope::<Vec<BangumiTimelineDay>>::from_slice(bytes)?.into_payload()?;

            assert!(!payload.is_empty());
            assert!(payload.iter().any(|day| !day.episodes.is_empty()));
        }
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/bangumi/timeline/timeline/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn bangumi_timeline_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload = serde_json::from_value::<ApiEnvelope<Vec<BangumiTimelineDay>>>(body)?
                .into_payload()?;

            assert!(!payload.is_empty());
            assert!(payload.iter().all(|day| !day.date.is_empty()));
        }
        Ok(())
    }
}
