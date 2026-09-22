use crate::cheese::info::{CourseEpList, CourseInfo};
use crate::cheese::videostream_url::CourseVideoStreamData;
use crate::cheese::{CheeseEpListParams, CheeseInfoParams, CheeseVideoStreamParams};
use crate::ids::{EpisodeId, SeasonId};
use crate::{BilibiliRequest, BpiClient, BpiResult};

const INFO_ENDPOINT: &str = "https://api.bilibili.com/pugv/view/web/season";
const EP_LIST_ENDPOINT: &str = "https://api.bilibili.com/pugv/view/web/ep/list";
const VIDEO_STREAM_ENDPOINT: &str = "https://api.bilibili.com/pugv/player/web/playurl";

/// 课堂课程 API 客户端。
#[derive(Clone, Copy)]
pub struct CheeseClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> CheeseClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn info_endpoint(&self) -> &'static str {
        INFO_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn ep_list_endpoint(&self) -> &'static str {
        EP_LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn video_stream_endpoint(&self) -> &'static str {
        VIDEO_STREAM_ENDPOINT
    }

    /// 按 season 或 episode ID 获取课堂课程信息。
    pub async fn info(&self, params: CheeseInfoParams) -> BpiResult<CourseInfo> {
        self.client
            .get(INFO_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("cheese.info")
            .await
    }

    /// 按 season ID 获取课堂课程信息。
    pub async fn info_by_season_id(&self, season_id: SeasonId) -> BpiResult<CourseInfo> {
        self.client
            .get(INFO_ENDPOINT)
            .query(&CheeseInfoParams::from_season_id(season_id).query_pairs())
            .send_bpi_payload("cheese.info.season_detail_by_season_id")
            .await
    }

    /// 按 episode ID 获取课堂课程信息。
    pub async fn info_by_ep_id(&self, episode_id: EpisodeId) -> BpiResult<CourseInfo> {
        self.client
            .get(INFO_ENDPOINT)
            .query(&CheeseInfoParams::from_episode_id(episode_id).query_pairs())
            .send_bpi_payload("cheese.info.season_detail_by_ep_id")
            .await
    }

    /// 获取课堂课程 episode 列表。
    pub async fn ep_list(&self, params: CheeseEpListParams) -> BpiResult<CourseEpList> {
        self.client
            .get(EP_LIST_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("cheese.info.ep_list")
            .await
    }

    /// 获取课堂课程视频流数据。
    pub async fn video_stream(
        &self,
        params: CheeseVideoStreamParams,
    ) -> BpiResult<CourseVideoStreamData> {
        self.client
            .get(VIDEO_STREAM_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("cheese.playurl")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::cheese::info::{CourseEpList, CourseInfo};
    use crate::cheese::videostream_url::CourseVideoStreamData;
    use crate::cheese::{CheeseEpListParams, CheeseInfoParams, CheeseVideoStreamParams};
    use crate::ids::{Aid, Cid, EpisodeId, SeasonId};
    use crate::models::{Fnval, VideoQuality};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiResult};

    const TEST_SEASON_ID: u64 = 556;
    const TEST_EP_ID: u64 = 20767;
    const TEST_AVID: u64 = 997984154;
    const TEST_PLAYURL_EP_ID: u64 = 163956;
    const TEST_CID: u64 = 1183682680;

    fn season_id() -> BpiResult<SeasonId> {
        SeasonId::new(TEST_SEASON_ID)
    }

    fn episode_id() -> BpiResult<EpisodeId> {
        EpisodeId::new(TEST_EP_ID)
    }

    fn playurl_params() -> BpiResult<CheeseVideoStreamParams> {
        Ok(CheeseVideoStreamParams::new(
            Aid::new(TEST_AVID)?,
            EpisodeId::new(TEST_PLAYURL_EP_ID)?,
            Cid::new(TEST_CID)?,
        )
        .with_quality(VideoQuality::P480)
        .with_fnval(Fnval::DASH))
    }

    fn assert_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<CourseInfo>>,
    {
    }

    fn assert_ep_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<CourseEpList>>,
    {
    }

    fn assert_video_stream_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<CourseVideoStreamData>>,
    {
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
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
            "playurl" => {
                include_bytes!("../../tests/contracts/cheese/playurl/contract.json").as_slice()
            }
            _ => unreachable!("unknown cheese contract"),
        };
        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn cheese_client_exposes_promoted_endpoint_urls() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let cheese = client.cheese();

        assert_eq!(
            cheese.info_endpoint(),
            "https://api.bilibili.com/pugv/view/web/season"
        );
        assert_eq!(
            cheese.ep_list_endpoint(),
            "https://api.bilibili.com/pugv/view/web/ep/list"
        );
        assert_eq!(
            cheese.video_stream_endpoint(),
            "https://api.bilibili.com/pugv/player/web/playurl"
        );
        Ok(())
    }

    #[test]
    fn cheese_methods_return_payload_futures() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let cheese = client.cheese();

        assert_info_future(cheese.info(CheeseInfoParams::from_season_id(season_id()?)));
        assert_info_future(cheese.info_by_season_id(season_id()?));
        assert_info_future(cheese.info_by_ep_id(episode_id()?));
        assert_ep_list_future(
            cheese.ep_list(
                CheeseEpListParams::new(season_id()?)
                    .with_page_size(50)?
                    .with_page(1)?,
            ),
        );
        assert_video_stream_future(cheese.video_stream(playurl_params()?));
        Ok(())
    }

    #[test]
    fn cheese_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let cheese = client.cheese();
        let season = contract("season-detail-season")?;
        let episode = contract("season-detail-episode")?;
        let ep_list = contract("ep-list")?;
        let playurl = contract("playurl")?;

        assert_eq!(season.name, "cheese.info.season_detail_by_season_id");
        assert_eq!(season.request.method, HttpMethod::Get);
        assert_eq!(season.request.url.as_str(), cheese.info_endpoint());
        assert_eq!(
            season.request.query.get("season_id").map(String::as_str),
            Some("556")
        );

        assert_eq!(episode.name, "cheese.info.season_detail_by_ep_id");
        assert_eq!(episode.request.method, HttpMethod::Get);
        assert_eq!(episode.request.url.as_str(), cheese.info_endpoint());
        assert_eq!(
            episode.request.query.get("ep_id").map(String::as_str),
            Some("20767")
        );

        assert_eq!(ep_list.name, "cheese.info.ep_list");
        assert_eq!(ep_list.request.method, HttpMethod::Get);
        assert_eq!(ep_list.request.url.as_str(), cheese.ep_list_endpoint());

        assert_eq!(playurl.name, "cheese.playurl");
        assert_eq!(playurl.request.method, HttpMethod::Get);
        assert_eq!(playurl.request.url.as_str(), cheese.video_stream_endpoint());
        assert_eq!(
            playurl.request.query.get("fnval").map(String::as_str),
            Some("16")
        );
        Ok(())
    }
}
