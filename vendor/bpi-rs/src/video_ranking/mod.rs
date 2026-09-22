//! 视频排行

pub(crate) const POPULAR_LIST_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/popular";
pub(crate) const POPULAR_SERIES_LIST_ENDPOINT: &str =
    "https://api.bilibili.com/x/web-interface/popular/series/list";
pub(crate) const POPULAR_SERIES_ONE_ENDPOINT: &str =
    "https://api.bilibili.com/x/web-interface/popular/series/one";
pub(crate) const POPULAR_PRECIOUS_ENDPOINT: &str =
    "https://api.bilibili.com/x/web-interface/popular/precious";
pub(crate) const RANKING_LIST_ENDPOINT: &str =
    "https://api.bilibili.com/x/web-interface/ranking/v2";
pub(crate) const REGION_DYNAMIC_ENDPOINT: &str =
    "https://api.bilibili.com/x/web-interface/dynamic/region";
pub(crate) const REGION_NEWLIST_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/newlist";
pub(crate) const REGION_NEWLIST_RANK_ENDPOINT: &str =
    "https://api.bilibili.com/x/web-interface/newlist_rank";
pub(crate) const REGION_TAG_DYNAMIC_ENDPOINT: &str =
    "https://api.bilibili.com/x/web-interface/dynamic/tag";

pub mod client;
pub mod dynamic;
pub mod params;
pub mod popular;
pub mod precious_videos;
pub mod ranking;

pub use client::VideoRankingClient;

pub use params::{
    PopularSeriesOneParams, VideoNewListRankOrder, VideoPopularListParams, VideoRankingListParams,
    VideoRankingType, VideoRegionDynamicParams, VideoRegionNewListParams,
    VideoRegionNewListRankParams, VideoRegionTagDynamicParams,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiError, BpiResult};

    use super::dynamic::{NewListRankData, RegionArchivesData};
    use super::popular::{PopularListData, PopularSeriesListData, PopularSeriesOneData};
    use super::precious_videos::PreciousVideoData;
    use super::ranking::RankingListData;

    const PROFILES: [&str; 3] = ["anonymous", "normal", "vip"];

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let path = format!("tests/contracts/video_ranking/read/{endpoint}/contract.json");
        let bytes = std::fs::read(&path)
            .map_err(|err| BpiError::parse(format!("failed to read {path}: {err}")))?;
        EndpointContract::from_slice(&bytes)
    }

    fn fixture(endpoint: &str, profile: &str, suffix: &str) -> BpiResult<Vec<u8>> {
        let path = format!(
            "tests/contracts/video_ranking/read/{endpoint}/responses/{profile}.{suffix}.json"
        );
        std::fs::read(&path).map_err(|err| BpiError::parse(format!("failed to read {path}: {err}")))
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/video_ranking/read/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn video_ranking_read_contracts_match_endpoint_requests() -> BpiResult<()> {
        let expectations = [
            (
                "popular-list",
                "video_ranking.popular_list",
                POPULAR_LIST_ENDPOINT,
                VideoPopularListParams::new()
                    .with_page(1)?
                    .with_page_size(2)?
                    .query_pairs(),
                Some("PopularListData"),
                false,
                Some(0),
            ),
            (
                "popular-series-list",
                "video_ranking.popular_series_list",
                POPULAR_SERIES_LIST_ENDPOINT,
                Vec::new(),
                Some("PopularSeriesListData"),
                false,
                Some(0),
            ),
            (
                "popular-series-one",
                "video_ranking.popular_series_one",
                POPULAR_SERIES_ONE_ENDPOINT,
                PopularSeriesOneParams::new(1)?.query_pairs(),
                Some("PopularSeriesOneData"),
                true,
                Some(0),
            ),
            (
                "popular-precious",
                "video_ranking.popular_precious",
                POPULAR_PRECIOUS_ENDPOINT,
                Vec::new(),
                Some("PreciousVideoData"),
                false,
                Some(0),
            ),
            (
                "ranking-list",
                "video_ranking.ranking_list",
                RANKING_LIST_ENDPOINT,
                VideoRankingListParams::new()
                    .with_rid(1)?
                    .with_type(VideoRankingType::All)
                    .query_pairs(),
                Some("RankingListData"),
                false,
                Some(0),
            ),
            (
                "region-dynamic",
                "video_ranking.region_dynamic",
                REGION_DYNAMIC_ENDPOINT,
                VideoRegionDynamicParams::new(21)?
                    .with_page(1)?
                    .with_page_size(2)?
                    .query_pairs(),
                None,
                false,
                Some(-404),
            ),
            (
                "region-tag-dynamic",
                "video_ranking.region_tag_dynamic",
                REGION_TAG_DYNAMIC_ENDPOINT,
                VideoRegionTagDynamicParams::new(136, 10026108)?
                    .with_page(1)?
                    .with_page_size(2)?
                    .query_pairs(),
                Some("RegionArchivesData"),
                false,
                Some(0),
            ),
            (
                "region-newlist",
                "video_ranking.region_newlist",
                REGION_NEWLIST_ENDPOINT,
                VideoRegionNewListParams::new(231)?
                    .with_page(1)?
                    .with_page_size(2)?
                    .with_type(1)?
                    .query_pairs(),
                Some("RegionArchivesData"),
                false,
                Some(0),
            ),
            (
                "region-newlist-rank",
                "video_ranking.region_newlist_rank",
                REGION_NEWLIST_RANK_ENDPOINT,
                VideoRegionNewListRankParams::new(231, 2, "20260701", "20260703")?
                    .with_order(VideoNewListRankOrder::Click)
                    .with_page(1)?
                    .query_pairs(),
                Some("NewListRankData"),
                false,
                Some(0),
            ),
        ];

        for (endpoint, name, url, query_pairs, rust_model, requires_wbi, api_code) in expectations {
            let contract = contract(endpoint)?;

            assert_eq!(contract.name, name);
            assert_eq!(contract.request.method, HttpMethod::Get);
            assert_eq!(contract.request.url.as_str(), url);
            assert_eq!(contract.request.auth.requires_wbi(), requires_wbi);
            assert_eq!(contract.cases.len(), 3);

            for (key, value) in query_pairs {
                assert_eq!(
                    contract.request.query.get(key).map(String::as_str),
                    Some(value.as_str())
                );
            }

            for case in &contract.cases {
                assert_eq!(case.response.http_status, Some(200));
                assert_eq!(case.response.api_code, api_code);
                assert_eq!(case.response.rust_model.as_deref(), rust_model);
                assert_eq!(case.auth.requires_wbi(), requires_wbi);
                if case.name == "anonymous" {
                    assert!(!case.auth.requires_cookie());
                } else {
                    assert!(case.auth.requires_cookie());
                }
            }
        }

        Ok(())
    }

    #[test]
    fn video_ranking_read_response_fixtures_parse_declared_models() -> BpiResult<()> {
        for profile in PROFILES {
            let popular = ApiEnvelope::<PopularListData>::from_slice(&fixture(
                "popular-list",
                profile,
                "success",
            )?)?
            .into_payload()?;
            assert!(popular.list.len() <= 2);

            let series_list = ApiEnvelope::<PopularSeriesListData>::from_slice(&fixture(
                "popular-series-list",
                profile,
                "success",
            )?)?
            .into_payload()?;
            assert!(!series_list.list.is_empty());

            let series_one = ApiEnvelope::<PopularSeriesOneData>::from_slice(&fixture(
                "popular-series-one",
                profile,
                "success",
            )?)?
            .into_payload()?;
            assert_eq!(series_one.config.number, 1);

            let precious = ApiEnvelope::<PreciousVideoData>::from_slice(&fixture(
                "popular-precious",
                profile,
                "success",
            )?)?
            .into_payload()?;
            assert!(!precious.list.is_empty());

            let ranking = ApiEnvelope::<RankingListData>::from_slice(&fixture(
                "ranking-list",
                profile,
                "success",
            )?)?
            .into_payload()?;
            assert!(!ranking.note.is_empty());
            assert!(ranking.list.len() <= 2);

            let region_dynamic = ApiEnvelope::<RegionArchivesData>::from_slice(&fixture(
                "region-dynamic",
                profile,
                "error",
            )?)?;
            assert_eq!(
                region_dynamic.ensure_success().unwrap_err().code(),
                Some(-404)
            );

            let tag_dynamic = ApiEnvelope::<RegionArchivesData>::from_slice(&fixture(
                "region-tag-dynamic",
                profile,
                "success",
            )?)?
            .into_payload()?;
            assert_eq!(tag_dynamic.page.size, 2);

            let newlist = ApiEnvelope::<RegionArchivesData>::from_slice(&fixture(
                "region-newlist",
                profile,
                "success",
            )?)?
            .into_payload()?;
            assert_eq!(newlist.page.size, 2);

            let newlist_rank = ApiEnvelope::<NewListRankData>::from_slice(&fixture(
                "region-newlist-rank",
                profile,
                "success",
            )?)?
            .into_payload()?;
            assert_eq!(newlist_rank.pagesize, 2);
        }

        Ok(())
    }

    #[test]
    fn video_ranking_read_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in PROFILES {
            if let Some(body) = local_probe_body("popular-list", profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<PopularListData>>(body)?.into_payload()?;
                assert!(!payload.list.is_empty());
            }

            if let Some(body) = local_probe_body("popular-series-list", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<PopularSeriesListData>>(body)?
                    .into_payload()?;
                assert!(!payload.list.is_empty());
            }

            if let Some(body) = local_probe_body("popular-series-one", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<PopularSeriesOneData>>(body)?
                    .into_payload()?;
                assert_eq!(payload.config.number, 1);
            }

            if let Some(body) = local_probe_body("popular-precious", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<PreciousVideoData>>(body)?
                    .into_payload()?;
                assert!(!payload.list.is_empty());
            }

            if let Some(body) = local_probe_body("ranking-list", profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<RankingListData>>(body)?.into_payload()?;
                assert!(!payload.list.is_empty());
            }

            if let Some(body) = local_probe_body("region-dynamic", profile) {
                let envelope = serde_json::from_value::<ApiEnvelope<RegionArchivesData>>(body)?;
                assert_eq!(envelope.ensure_success().unwrap_err().code(), Some(-404));
            }

            for endpoint in ["region-tag-dynamic", "region-newlist"] {
                if let Some(body) = local_probe_body(endpoint, profile) {
                    let payload = serde_json::from_value::<ApiEnvelope<RegionArchivesData>>(body)?
                        .into_payload()?;
                    assert_eq!(payload.page.size, 2);
                }
            }

            if let Some(body) = local_probe_body("region-newlist-rank", profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<NewListRankData>>(body)?.into_payload()?;
                assert_eq!(payload.pagesize, 2);
            }
        }

        Ok(())
    }
}
