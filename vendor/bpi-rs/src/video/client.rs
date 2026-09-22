use crate::request::BilibiliRequest;
use crate::{BpiClient, BpiResult};

use super::collection::{
    GetSeasonsArchivesData, GetSeasonsSeriesData, GetSeriesArchivesData, GetSeriesData,
    HOME_SEASONS_SERIES_ENDPOINT, SEASONS_ARCHIVES_LIST_ENDPOINT, SEASONS_SERIES_LIST_ENDPOINT,
    SERIES_ARCHIVES_ENDPOINT, SERIES_INFO_ENDPOINT, VideoCollectionHomeSeasonsSeriesParams,
    VideoCollectionSeasonsArchivesParams, VideoCollectionSeasonsSeriesParams,
    VideoCollectionSeriesArchivesParams, VideoCollectionSeriesInfoParams,
};
use super::interact_video::{INTERACTIVE_INFO_ENDPOINT, InteractiveVideoInfoResponseData};
use super::model::{VideoDetail, VideoPage, VideoView};
use super::online::{ONLINE_TOTAL_ENDPOINT, OnlineTotalResponseData};
use super::params::{
    InteractiveVideoInfoParams, VideoAiSummaryParams, VideoDescParams, VideoDetailParams,
    VideoHomepageRecommendationsParams, VideoOnlineTotalParams, VideoPageListParams,
    VideoPlayUrlParams, VideoPlayerInfoParams, VideoRelatedParams, VideoTagsParams,
    VideoViewParams,
};
use super::player::{PLAYER_INFO_V2_ENDPOINT, PlayerInfoResponseData};
use super::recommend::{
    HOMEPAGE_RECOMMENDATIONS_ENDPOINT, RELATED_VIDEOS_ENDPOINT, RcmdFeedResponseData, RelatedVideo,
};
use super::summary::{AI_SUMMARY_ENDPOINT, AiSummaryResponseData};
use super::tags::{TAGS_ENDPOINT, VideoTag};
use super::videostream_url::{PLAY_URL_ENDPOINT, PlayUrlResponseData};

const DESC_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/archive/desc";
const DETAIL_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/view/detail";
const PAGELIST_ENDPOINT: &str = "https://api.bilibili.com/x/player/pagelist";
const VIEW_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/view";

/// 视频领域 API 客户端。
#[derive(Clone, Copy)]
pub struct VideoClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> VideoClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn endpoint(&self) -> &'static str {
        VIEW_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn detail_endpoint(&self) -> &'static str {
        DETAIL_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn page_list_endpoint(&self) -> &'static str {
        PAGELIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn desc_endpoint(&self) -> &'static str {
        DESC_ENDPOINT
    }

    /// 按 AV ID 或 BV ID 获取 Web 视频详情。
    pub async fn view(&self, params: VideoViewParams) -> BpiResult<VideoView> {
        self.client
            .get(VIEW_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("video.view")
            .await
    }

    /// 获取 Web 视频详情，包括标签和相关视频。
    pub async fn detail(&self, params: VideoDetailParams) -> BpiResult<VideoDetail> {
        self.client
            .get(DETAIL_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("video.detail")
            .await
    }

    /// 获取视频的分 P/内容 ID。
    pub async fn page_list(&self, params: VideoPageListParams) -> BpiResult<Vec<VideoPage>> {
        self.client
            .get(PAGELIST_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("video.pagelist")
            .await
    }

    /// 获取纯文本视频简介。
    pub async fn desc(&self, params: VideoDescParams) -> BpiResult<String> {
        self.client
            .get(DESC_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("video.desc")
            .await
    }

    /// 按 AV ID 或 BV ID 以及分 P/内容 ID 获取已签名的 Web 播放 URL。
    pub async fn play_url(&self, params: VideoPlayUrlParams) -> BpiResult<PlayUrlResponseData> {
        let params = self.client.get_wbi_sign2(params.query_pairs()).await?;

        self.client
            .get(PLAY_URL_ENDPOINT)
            .with_bilibili_headers()
            .query(&params)
            .send_bpi_payload("video.play_url")
            .await
    }

    /// 获取指定视频合集中的视频。
    pub async fn seasons_archives_list(
        &self,
        params: VideoCollectionSeasonsArchivesParams,
    ) -> BpiResult<GetSeasonsArchivesData> {
        let params = self.client.get_wbi_sign2(params.query_pairs()).await?;

        self.client
            .get(SEASONS_ARCHIVES_LIST_ENDPOINT)
            .with_bilibili_headers()
            .query(&params)
            .send_bpi_payload("video.collection.seasons_archives_list")
            .await
    }

    /// 获取用户主页的合集和系列列表。
    pub async fn home_seasons_series(
        &self,
        params: VideoCollectionHomeSeasonsSeriesParams,
    ) -> BpiResult<GetSeasonsSeriesData> {
        let params = self.client.get_wbi_sign2(params.query_pairs()).await?;

        self.client
            .get(HOME_SEASONS_SERIES_ENDPOINT)
            .query(&params)
            .send_bpi_payload("video.collection.home_seasons_series")
            .await
    }

    /// 分页获取用户的合集和系列列表。
    pub async fn seasons_series_list(
        &self,
        params: VideoCollectionSeasonsSeriesParams,
    ) -> BpiResult<GetSeasonsSeriesData> {
        let params = self.client.get_wbi_sign2(params.query_pairs()).await?;

        self.client
            .get(SEASONS_SERIES_LIST_ENDPOINT)
            .query(&params)
            .send_bpi_payload("video.collection.seasons_series_list")
            .await
    }

    /// 获取指定视频系列的元数据。
    pub async fn series_info(
        &self,
        params: VideoCollectionSeriesInfoParams,
    ) -> BpiResult<GetSeriesData> {
        self.client
            .get(SERIES_INFO_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("video.collection.series_info")
            .await
    }

    /// 获取指定视频系列中的视频。
    pub async fn series_archives(
        &self,
        params: VideoCollectionSeriesArchivesParams,
    ) -> BpiResult<GetSeriesArchivesData> {
        self.client
            .get(SERIES_ARCHIVES_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("video.collection.series_archives")
            .await
    }

    /// 获取视频分 P 的在线人数计数。
    pub async fn online_total(
        &self,
        params: VideoOnlineTotalParams,
    ) -> BpiResult<OnlineTotalResponseData> {
        self.client
            .get(ONLINE_TOTAL_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("video.online_total")
            .await
    }

    /// 获取视频分 P 的 Web 播放器元数据。
    pub async fn player_info_v2(
        &self,
        params: VideoPlayerInfoParams,
    ) -> BpiResult<PlayerInfoResponseData> {
        let params = self.client.get_wbi_sign2(params.query_pairs()).await?;

        self.client
            .get(PLAYER_INFO_V2_ENDPOINT)
            .query(&params)
            .send_bpi_payload("video.player_info_v2")
            .await
    }

    /// 获取与某个视频相关的视频。
    pub async fn related_videos(&self, params: VideoRelatedParams) -> BpiResult<Vec<RelatedVideo>> {
        self.client
            .get(RELATED_VIDEOS_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("video.related_videos")
            .await
    }

    /// 获取首页视频推荐。
    pub async fn homepage_recommendations(
        &self,
        params: VideoHomepageRecommendationsParams,
    ) -> BpiResult<RcmdFeedResponseData> {
        let params = self.client.get_wbi_sign2(params.query_pairs()).await?;

        self.client
            .get(HOMEPAGE_RECOMMENDATIONS_ENDPOINT)
            .query(&params)
            .send_bpi_payload("video.homepage_recommendations")
            .await
    }

    /// 获取视频的 AI 总结。
    pub async fn ai_summary(
        &self,
        params: VideoAiSummaryParams,
    ) -> BpiResult<AiSummaryResponseData> {
        let params = self.client.get_wbi_sign2(params.query_pairs()).await?;

        self.client
            .get(AI_SUMMARY_ENDPOINT)
            .query(&params)
            .send_bpi_payload("video.ai_summary")
            .await
    }

    /// 获取视频关联的标签。
    pub async fn tags(&self, params: VideoTagsParams) -> BpiResult<Vec<VideoTag>> {
        self.client
            .get(TAGS_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("video.tags")
            .await
    }

    /// 获取互动视频节点的元数据。
    pub async fn interactive_video_info(
        &self,
        params: InteractiveVideoInfoParams,
    ) -> BpiResult<InteractiveVideoInfoResponseData> {
        self.client
            .get(INTERACTIVE_INFO_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("video.interactive_video_info")
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ApiEnvelope, BpiClient, BpiError, BpiResult,
        ids::{Aid, Cid, Mid, SeasonId},
        probe::{contract::HttpMethod, endpoint_contract::EndpointContract},
        video::params::VideoHomepageRecommendationsParams,
        video::{
            InteractiveVideoInfoParams, VideoAiSummaryParams,
            VideoCollectionHomeSeasonsSeriesParams, VideoCollectionSeasonsArchivesParams,
            VideoCollectionSeasonsSeriesParams, VideoCollectionSeriesArchivesParams,
            VideoCollectionSeriesInfoParams, VideoOnlineTotalParams, VideoPlayerInfoParams,
            VideoRelatedParams, VideoTagsParams,
        },
    };
    use serde::de::DeserializeOwned;

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes: &[u8] = match endpoint {
            "view" => include_bytes!("../../tests/contracts/video/info-read/view/contract.json"),
            "detail" => {
                include_bytes!("../../tests/contracts/video/info-read/detail/contract.json")
            }
            "pagelist" => {
                include_bytes!("../../tests/contracts/video/info-read/pagelist/contract.json")
            }
            "desc" => include_bytes!("../../tests/contracts/video/info-read/desc/contract.json"),
            _ => {
                return Err(BpiError::invalid_parameter(
                    "endpoint",
                    "unknown video contract",
                ));
            }
        };

        EndpointContract::from_slice(bytes)
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/video/info-read/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    fn parse_local_probe_outputs<T>(endpoint: &str, profiles: &[&str]) -> BpiResult<()>
    where
        T: DeserializeOwned,
    {
        for profile in profiles {
            let Some(body) = local_probe_body(endpoint, profile) else {
                continue;
            };

            let _payload = serde_json::from_value::<ApiEnvelope<T>>(body)?.into_payload()?;
        }

        Ok(())
    }

    #[test]
    fn video_client_borrows_root_client() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let video = client.video();

        assert_eq!(
            video.endpoint(),
            "https://api.bilibili.com/x/web-interface/view"
        );
        Ok(())
    }

    #[test]
    fn video_client_exposes_info_read_endpoints() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let video = client.video();

        assert_eq!(
            video.detail_endpoint(),
            "https://api.bilibili.com/x/web-interface/view/detail"
        );
        assert_eq!(
            video.page_list_endpoint(),
            "https://api.bilibili.com/x/player/pagelist"
        );
        assert_eq!(
            video.desc_endpoint(),
            "https://api.bilibili.com/x/web-interface/archive/desc"
        );
        Ok(())
    }

    #[test]
    fn video_client_methods_use_payload_request_helpers() {
        let source = include_str!("client.rs");
        let payload_helper = concat!(".send_", "bpi_payload");
        let legacy_envelope_helper = concat!(".send_", "bpi::<");
        let legacy_flat_playurl = concat!(".video_", "playurl(");

        assert!(
            source.matches(payload_helper).count() >= 5,
            "VideoClient read methods should return decoded payloads directly"
        );
        assert!(
            !source.contains(legacy_envelope_helper),
            "VideoClient should not use legacy envelope-returning request helpers"
        );
        assert!(
            !source.contains(legacy_flat_playurl),
            "VideoClient::play_url should be implemented as a payload-helper-backed domain method"
        );
    }

    #[test]
    fn video_client_exposes_collection_and_player_read_methods() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let video = client.video();

        std::mem::drop(
            video.seasons_archives_list(VideoCollectionSeasonsArchivesParams::new(
                Mid::new(1000001)?,
                SeasonId::new(4294056)?,
            )),
        );
        std::mem::drop(
            video.home_seasons_series(VideoCollectionHomeSeasonsSeriesParams::new(Mid::new(
                1000001,
            )?)),
        );
        std::mem::drop(
            video.seasons_series_list(VideoCollectionSeasonsSeriesParams::new(Mid::new(1000001)?)),
        );
        std::mem::drop(video.series_info(VideoCollectionSeriesInfoParams::new(250285)?));
        std::mem::drop(
            video.series_archives(VideoCollectionSeriesArchivesParams::new(
                Mid::new(1000001)?,
                250285,
            )?),
        );
        std::mem::drop(video.online_total(VideoOnlineTotalParams::from_bvid(
            "BV1xx411c7mD".parse()?,
            Cid::new(62131)?,
        )));
        std::mem::drop(video.player_info_v2(VideoPlayerInfoParams::from_bvid(
            "BV1xx411c7mD".parse()?,
            Cid::new(62131)?,
        )));
        std::mem::drop(
            video.related_videos(VideoRelatedParams::from_bvid("BV1xx411c7mD".parse()?)),
        );
        std::mem::drop(video.homepage_recommendations(VideoHomepageRecommendationsParams::new()));
        std::mem::drop(video.ai_summary(VideoAiSummaryParams::from_bvid(
            "BV1xx411c7mD".parse()?,
            Cid::new(62131)?,
            928123,
        )?));
        std::mem::drop(
            video.tags(VideoTagsParams::from_bvid("BV1xx411c7mD".parse()?).cid(Cid::new(62131)?)),
        );
        std::mem::drop(
            video.interactive_video_info(InteractiveVideoInfoParams::from_aid(
                Aid::new(114347430905959)?,
                1273647,
            )?),
        );

        let source = include_str!("client.rs");
        let payload_helper = concat!(".send_", "bpi_payload");

        assert!(
            source.matches(payload_helper).count() >= 17,
            "VideoClient should use payload helpers for info, playurl, collection, and player read methods"
        );
        Ok(())
    }

    #[test]
    fn video_info_read_contracts_match_endpoint_requests() -> BpiResult<()> {
        let expectations = [
            (
                "view",
                "video.view",
                VIEW_ENDPOINT,
                &[("bvid", "BV1xx411c7mD")][..],
                "VideoView",
            ),
            (
                "detail",
                "video.detail",
                DETAIL_ENDPOINT,
                &[("bvid", "BV1xx411c7mD"), ("need_elec", "0")][..],
                "VideoDetail",
            ),
            (
                "pagelist",
                "video.pagelist",
                PAGELIST_ENDPOINT,
                &[("bvid", "BV1xx411c7mD")][..],
                "Vec<VideoPage>",
            ),
            (
                "desc",
                "video.desc",
                DESC_ENDPOINT,
                &[("bvid", "BV1xx411c7mD")][..],
                "String",
            ),
        ];

        for (endpoint, name, url, query_pairs, rust_model) in expectations {
            let contract = contract(endpoint)?;

            assert_eq!(contract.name, name);
            assert_eq!(contract.request.method, HttpMethod::Get);
            assert_eq!(contract.request.url.as_str(), url);
            assert_eq!(contract.cases.len(), 3);
            assert!(
                contract
                    .cases
                    .iter()
                    .all(|case| case.response.api_code == Some(0)),
                "{endpoint} should have successful anonymous, normal, and vip cases"
            );
            assert!(
                contract
                    .cases
                    .iter()
                    .any(|case| case.response.rust_model.as_deref() == Some(rust_model)),
                "{endpoint} should declare {rust_model}"
            );

            for &(key, value) in query_pairs {
                assert_eq!(
                    contract.request.query.get(key).map(String::as_str),
                    Some(value)
                );
            }
        }

        Ok(())
    }

    #[test]
    fn video_info_read_response_fixtures_parse_declared_models() -> BpiResult<()> {
        let view = ApiEnvelope::<VideoView>::from_slice(include_bytes!(
            "../../tests/contracts/video/info-read/view/responses/success.json"
        ))?
        .into_payload()?;
        let detail = ApiEnvelope::<VideoDetail>::from_slice(include_bytes!(
            "../../tests/contracts/video/info-read/detail/responses/success.json"
        ))?
        .into_payload()?;
        let pagelist = ApiEnvelope::<Vec<VideoPage>>::from_slice(include_bytes!(
            "../../tests/contracts/video/info-read/pagelist/responses/success.json"
        ))?
        .into_payload()?;
        let desc = ApiEnvelope::<String>::from_slice(include_bytes!(
            "../../tests/contracts/video/info-read/desc/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(view.bvid.as_str(), "BV1xx411c7mD");
        assert_eq!(view.pic, "https://i0.hdslb.com/bfs/archive/sanitized.jpg");
        assert_eq!(detail.view.bvid.as_str(), "BV1xx411c7mD");
        assert_eq!(pagelist.len(), 1);
        assert_eq!(desc, "www");
        Ok(())
    }

    #[test]
    fn video_info_read_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        parse_local_probe_outputs::<VideoView>("view", &["anonymous", "normal", "vip"])?;
        parse_local_probe_outputs::<VideoDetail>("detail", &["anonymous", "normal", "vip"])?;
        parse_local_probe_outputs::<Vec<VideoPage>>("pagelist", &["anonymous", "normal", "vip"])?;
        parse_local_probe_outputs::<String>("desc", &["anonymous", "normal", "vip"])?;

        Ok(())
    }
}
