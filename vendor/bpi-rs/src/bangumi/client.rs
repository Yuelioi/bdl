use crate::bangumi::info::{BangumiDetailResult, BangumiInfoResult, BangumiSectionResult};
use crate::bangumi::timeline::BangumiTimelineDay;
use crate::bangumi::videostream_url::BangumiVideoStreamData;
use crate::bangumi::{
    BangumiDetailParams, BangumiInfoParams, BangumiSectionsParams, BangumiTimelineParams,
    BangumiVideoStreamParams,
};
use crate::ids::{EpisodeId, SeasonId};
use crate::{BilibiliRequest, BpiClient, BpiResult};

const INFO_ENDPOINT: &str = "https://api.bilibili.com/pgc/review/user";
const DETAIL_ENDPOINT: &str = "https://api.bilibili.com/pgc/view/web/season";
const SECTIONS_ENDPOINT: &str = "https://api.bilibili.com/pgc/web/season/section";
const TIMELINE_ENDPOINT: &str = "https://api.bilibili.com/pgc/web/timeline";
const VIDEO_STREAM_ENDPOINT: &str = "https://api.bilibili.com/pgc/player/web/playurl";

/// Bangumi API 客户端。
#[derive(Clone, Copy)]
pub struct BangumiClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> BangumiClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn info_endpoint(&self) -> &'static str {
        INFO_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn detail_endpoint(&self) -> &'static str {
        DETAIL_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn sections_endpoint(&self) -> &'static str {
        SECTIONS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn timeline_endpoint(&self) -> &'static str {
        TIMELINE_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn video_stream_endpoint(&self) -> &'static str {
        VIDEO_STREAM_ENDPOINT
    }

    /// 按 media ID 获取 bangumi 媒体信息。
    pub async fn info(&self, params: BangumiInfoParams) -> BpiResult<BangumiInfoResult> {
        self.client
            .get(INFO_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("bangumi.info.review_user")
            .await
    }

    /// 按 season 或 episode ID 获取 bangumi 详情。
    pub async fn detail(&self, params: BangumiDetailParams) -> BpiResult<BangumiDetailResult> {
        self.client
            .get(DETAIL_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("bangumi.info.detail")
            .await
    }

    /// 按 season ID 获取 bangumi 详情。
    pub async fn detail_by_season_id(&self, season_id: SeasonId) -> BpiResult<BangumiDetailResult> {
        self.client
            .get(DETAIL_ENDPOINT)
            .query(&BangumiDetailParams::from_season_id(season_id).query_pairs())
            .send_bpi_payload("bangumi.info.season_detail_by_season_id")
            .await
    }

    /// 按 episode ID 获取 bangumi 详情。
    pub async fn detail_by_ep_id(&self, episode_id: EpisodeId) -> BpiResult<BangumiDetailResult> {
        self.client
            .get(DETAIL_ENDPOINT)
            .query(&BangumiDetailParams::from_episode_id(episode_id).query_pairs())
            .send_bpi_payload("bangumi.info.season_detail_by_ep_id")
            .await
    }

    /// 按 season ID 获取 bangumi season 分区。
    pub async fn sections(&self, params: BangumiSectionsParams) -> BpiResult<BangumiSectionResult> {
        self.client
            .get(SECTIONS_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("bangumi.info.season_section")
            .await
    }

    /// 获取 bangumi 或影视时间线。
    pub async fn timeline(
        &self,
        params: BangumiTimelineParams,
    ) -> BpiResult<Vec<BangumiTimelineDay>> {
        self.client
            .get(TIMELINE_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("bangumi.timeline")
            .await
    }

    /// 获取 bangumi 视频流数据。
    pub async fn video_stream(
        &self,
        params: BangumiVideoStreamParams,
    ) -> BpiResult<BangumiVideoStreamData> {
        self.client
            .get(VIDEO_STREAM_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("bangumi.playurl")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::bangumi::info::{BangumiDetailResult, BangumiInfoResult, BangumiSectionResult};
    use crate::bangumi::timeline::{BangumiTimelineDay, BangumiTimelineType};
    use crate::bangumi::videostream_url::BangumiVideoStreamData;
    use crate::bangumi::{
        BangumiDetailParams, BangumiInfoParams, BangumiSectionsParams, BangumiTimelineParams,
        BangumiVideoStreamParams,
    };
    use crate::ids::{EpisodeId, MediaId, SeasonId};
    use crate::models::{Fnval, VideoQuality};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiResult};

    const TEST_MEDIA_ID: u64 = 28_220_978;
    const TEST_SEASON_ID: u64 = 1_172;
    const TEST_EP_ID: u64 = 21_265;

    fn media_id() -> BpiResult<MediaId> {
        MediaId::new(TEST_MEDIA_ID)
    }

    fn season_id() -> BpiResult<SeasonId> {
        SeasonId::new(TEST_SEASON_ID)
    }

    fn episode_id() -> BpiResult<EpisodeId> {
        EpisodeId::new(TEST_EP_ID)
    }

    fn playurl_params() -> BpiResult<BangumiVideoStreamParams> {
        Ok(BangumiVideoStreamParams::from_episode_id(episode_id()?)
            .with_quality(VideoQuality::P480)
            .with_fnval(Fnval::DASH))
    }

    fn assert_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<BangumiInfoResult>>,
    {
    }

    fn assert_detail_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<BangumiDetailResult>>,
    {
    }

    fn assert_sections_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<BangumiSectionResult>>,
    {
    }

    fn assert_timeline_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<Vec<BangumiTimelineDay>>>,
    {
    }

    fn assert_video_stream_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<BangumiVideoStreamData>>,
    {
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
            "timeline" => {
                include_bytes!("../../tests/contracts/bangumi/timeline/contract.json").as_slice()
            }
            "playurl" => {
                include_bytes!("../../tests/contracts/bangumi/playurl/contract.json").as_slice()
            }
            _ => unreachable!("unknown bangumi contract"),
        };
        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn bangumi_client_exposes_promoted_endpoint_urls() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let bangumi = client.bangumi();

        assert_eq!(
            bangumi.info_endpoint(),
            "https://api.bilibili.com/pgc/review/user"
        );
        assert_eq!(
            bangumi.detail_endpoint(),
            "https://api.bilibili.com/pgc/view/web/season"
        );
        assert_eq!(
            bangumi.sections_endpoint(),
            "https://api.bilibili.com/pgc/web/season/section"
        );
        assert_eq!(
            bangumi.timeline_endpoint(),
            "https://api.bilibili.com/pgc/web/timeline"
        );
        assert_eq!(
            bangumi.video_stream_endpoint(),
            "https://api.bilibili.com/pgc/player/web/playurl"
        );
        Ok(())
    }

    #[test]
    fn bangumi_methods_return_payload_futures() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let bangumi = client.bangumi();

        assert_info_future(bangumi.info(BangumiInfoParams::new(media_id()?)));
        assert_detail_future(bangumi.detail(BangumiDetailParams::from_season_id(season_id()?)));
        assert_detail_future(bangumi.detail_by_season_id(season_id()?));
        assert_detail_future(bangumi.detail_by_ep_id(episode_id()?));
        assert_sections_future(bangumi.sections(BangumiSectionsParams::new(season_id()?)));
        assert_timeline_future(bangumi.timeline(BangumiTimelineParams::new(
            BangumiTimelineType::Anime,
            3,
            7,
        )?));
        assert_video_stream_future(bangumi.video_stream(playurl_params()?));
        Ok(())
    }

    #[test]
    fn bangumi_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let bangumi = client.bangumi();
        let review_user = contract("review-user")?;
        let season_detail = contract("season-detail-season")?;
        let episode_detail = contract("season-detail-episode")?;
        let season_section = contract("season-section")?;
        let timeline = contract("timeline")?;
        let playurl = contract("playurl")?;

        assert_eq!(review_user.name, "bangumi.info.review_user");
        assert_eq!(review_user.request.method, HttpMethod::Get);
        assert_eq!(review_user.request.url.as_str(), bangumi.info_endpoint());

        assert_eq!(
            season_detail.name,
            "bangumi.info.season_detail_by_season_id"
        );
        assert_eq!(season_detail.request.method, HttpMethod::Get);
        assert_eq!(
            season_detail.request.url.as_str(),
            bangumi.detail_endpoint()
        );
        assert_eq!(
            season_detail
                .request
                .query
                .get("season_id")
                .map(String::as_str),
            Some("1172")
        );

        assert_eq!(episode_detail.name, "bangumi.info.season_detail_by_ep_id");
        assert_eq!(episode_detail.request.method, HttpMethod::Get);
        assert_eq!(
            episode_detail.request.url.as_str(),
            bangumi.detail_endpoint()
        );

        assert_eq!(season_section.name, "bangumi.info.season_section");
        assert_eq!(season_section.request.method, HttpMethod::Get);
        assert_eq!(
            season_section.request.url.as_str(),
            bangumi.sections_endpoint()
        );

        assert_eq!(timeline.name, "bangumi.timeline");
        assert_eq!(timeline.request.method, HttpMethod::Get);
        assert_eq!(timeline.request.url.as_str(), bangumi.timeline_endpoint());

        assert_eq!(playurl.name, "bangumi.playurl");
        assert_eq!(playurl.request.method, HttpMethod::Get);
        assert_eq!(
            playurl.request.url.as_str(),
            bangumi.video_stream_endpoint()
        );
        assert_eq!(
            playurl.request.query.get("fnval").map(String::as_str),
            Some("16")
        );
        Ok(())
    }
}
