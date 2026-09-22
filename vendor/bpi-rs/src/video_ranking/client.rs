use crate::video_ranking::dynamic::{NewListRankData, RegionArchivesData};
use crate::video_ranking::params::{
    PopularSeriesOneParams, VideoPopularListParams, VideoRankingListParams,
    VideoRegionDynamicParams, VideoRegionNewListParams, VideoRegionNewListRankParams,
    VideoRegionTagDynamicParams,
};
use crate::video_ranking::popular::{PopularListData, PopularSeriesListData, PopularSeriesOneData};
use crate::video_ranking::precious_videos::PreciousVideoData;
use crate::video_ranking::ranking::RankingListData;
use crate::video_ranking::{
    POPULAR_LIST_ENDPOINT, POPULAR_PRECIOUS_ENDPOINT, POPULAR_SERIES_LIST_ENDPOINT,
    POPULAR_SERIES_ONE_ENDPOINT, RANKING_LIST_ENDPOINT, REGION_DYNAMIC_ENDPOINT,
    REGION_NEWLIST_ENDPOINT, REGION_NEWLIST_RANK_ENDPOINT, REGION_TAG_DYNAMIC_ENDPOINT,
};
use crate::{BilibiliRequest, BpiClient, BpiResult};

/// 视频排行 API 客户端。
#[derive(Clone, Copy)]
pub struct VideoRankingClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> VideoRankingClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn popular_list_endpoint(&self) -> &'static str {
        POPULAR_LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn popular_series_list_endpoint(&self) -> &'static str {
        POPULAR_SERIES_LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn popular_series_one_endpoint(&self) -> &'static str {
        POPULAR_SERIES_ONE_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn popular_precious_endpoint(&self) -> &'static str {
        POPULAR_PRECIOUS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn ranking_list_endpoint(&self) -> &'static str {
        RANKING_LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn region_dynamic_endpoint(&self) -> &'static str {
        REGION_DYNAMIC_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn region_tag_dynamic_endpoint(&self) -> &'static str {
        REGION_TAG_DYNAMIC_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn region_newlist_endpoint(&self) -> &'static str {
        REGION_NEWLIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn region_newlist_rank_endpoint(&self) -> &'static str {
        REGION_NEWLIST_RANK_ENDPOINT
    }

    /// 获取当前热门视频列表。
    pub async fn popular_list(&self, params: VideoPopularListParams) -> BpiResult<PopularListData> {
        self.client
            .get(POPULAR_LIST_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("video_ranking.popular_list")
            .await
    }

    /// 获取全部每周热门系列条目。
    pub async fn popular_series_list(&self) -> BpiResult<PopularSeriesListData> {
        self.client
            .get(POPULAR_SERIES_LIST_ENDPOINT)
            .send_bpi_payload("video_ranking.popular_series_list")
            .await
    }

    /// 获取单个每周热门系列详情。
    pub async fn popular_series_one(
        &self,
        params: PopularSeriesOneParams,
    ) -> BpiResult<PopularSeriesOneData> {
        let signed_params = self.client.get_wbi_sign2(params.query_pairs()).await?;

        self.client
            .get(POPULAR_SERIES_ONE_ENDPOINT)
            .query(&signed_params)
            .send_bpi_payload("video_ranking.popular_series_one")
            .await
    }

    /// 获取精选必看热门视频。
    pub async fn popular_precious(&self) -> BpiResult<PreciousVideoData> {
        self.client
            .get(POPULAR_PRECIOUS_ENDPOINT)
            .send_bpi_payload("video_ranking.popular_precious")
            .await
    }

    /// 获取视频排行列表。
    pub async fn ranking_list(&self, params: VideoRankingListParams) -> BpiResult<RankingListData> {
        self.client
            .get(RANKING_LIST_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("video_ranking.ranking_list")
            .await
    }

    /// 获取分区最新视频列表。
    pub async fn region_dynamic(
        &self,
        params: VideoRegionDynamicParams,
    ) -> BpiResult<RegionArchivesData> {
        self.client
            .get(REGION_DYNAMIC_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("video_ranking.region_dynamic")
            .await
    }

    /// 获取分区 tag 下的近期互动视频。
    pub async fn region_tag_dynamic(
        &self,
        params: VideoRegionTagDynamicParams,
    ) -> BpiResult<RegionArchivesData> {
        self.client
            .get(REGION_TAG_DYNAMIC_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("video_ranking.region_tag_dynamic")
            .await
    }

    /// 获取分区近期投稿。
    pub async fn region_newlist(
        &self,
        params: VideoRegionNewListParams,
    ) -> BpiResult<RegionArchivesData> {
        self.client
            .get(REGION_NEWLIST_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("video_ranking.region_newlist")
            .await
    }

    /// 获取分区近期投稿排行。
    pub async fn region_newlist_rank(
        &self,
        params: VideoRegionNewListRankParams,
    ) -> BpiResult<NewListRankData> {
        self.client
            .get(REGION_NEWLIST_RANK_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("video_ranking.region_newlist_rank")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::video_ranking::dynamic::{NewListRankData, RegionArchivesData};
    use crate::video_ranking::params::{
        PopularSeriesOneParams, VideoNewListRankOrder, VideoPopularListParams,
        VideoRankingListParams, VideoRankingType, VideoRegionDynamicParams,
        VideoRegionNewListParams, VideoRegionNewListRankParams, VideoRegionTagDynamicParams,
    };
    use crate::video_ranking::popular::{
        PopularListData, PopularSeriesListData, PopularSeriesOneData,
    };
    use crate::video_ranking::precious_videos::PreciousVideoData;
    use crate::video_ranking::ranking::RankingListData;
    use crate::{BpiClient, BpiResult};

    fn assert_popular_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<PopularListData>>,
    {
    }

    fn assert_series_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<PopularSeriesListData>>,
    {
    }

    fn assert_series_one_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<PopularSeriesOneData>>,
    {
    }

    fn assert_precious_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<PreciousVideoData>>,
    {
    }

    fn assert_ranking_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<RankingListData>>,
    {
    }

    fn assert_region_archives_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<RegionArchivesData>>,
    {
    }

    fn assert_newlist_rank_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<NewListRankData>>,
    {
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "popular-list" => include_bytes!(
                "../../tests/contracts/video_ranking/read/popular-list/contract.json"
            )
            .as_slice(),
            "popular-series-list" => include_bytes!(
                "../../tests/contracts/video_ranking/read/popular-series-list/contract.json"
            )
            .as_slice(),
            "popular-series-one" => include_bytes!(
                "../../tests/contracts/video_ranking/read/popular-series-one/contract.json"
            )
            .as_slice(),
            "popular-precious" => include_bytes!(
                "../../tests/contracts/video_ranking/read/popular-precious/contract.json"
            )
            .as_slice(),
            "ranking-list" => include_bytes!(
                "../../tests/contracts/video_ranking/read/ranking-list/contract.json"
            )
            .as_slice(),
            "region-dynamic" => include_bytes!(
                "../../tests/contracts/video_ranking/read/region-dynamic/contract.json"
            )
            .as_slice(),
            "region-tag-dynamic" => include_bytes!(
                "../../tests/contracts/video_ranking/read/region-tag-dynamic/contract.json"
            )
            .as_slice(),
            "region-newlist" => include_bytes!(
                "../../tests/contracts/video_ranking/read/region-newlist/contract.json"
            )
            .as_slice(),
            "region-newlist-rank" => include_bytes!(
                "../../tests/contracts/video_ranking/read/region-newlist-rank/contract.json"
            )
            .as_slice(),
            _ => unreachable!("unknown video_ranking contract"),
        };
        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn video_ranking_client_exposes_promoted_endpoint_urls() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let video_ranking = client.video_ranking();

        assert_eq!(
            video_ranking.popular_list_endpoint(),
            "https://api.bilibili.com/x/web-interface/popular"
        );
        assert_eq!(
            video_ranking.popular_series_list_endpoint(),
            "https://api.bilibili.com/x/web-interface/popular/series/list"
        );
        assert_eq!(
            video_ranking.popular_series_one_endpoint(),
            "https://api.bilibili.com/x/web-interface/popular/series/one"
        );
        assert_eq!(
            video_ranking.popular_precious_endpoint(),
            "https://api.bilibili.com/x/web-interface/popular/precious"
        );
        assert_eq!(
            video_ranking.ranking_list_endpoint(),
            "https://api.bilibili.com/x/web-interface/ranking/v2"
        );
        assert_eq!(
            video_ranking.region_dynamic_endpoint(),
            "https://api.bilibili.com/x/web-interface/dynamic/region"
        );
        assert_eq!(
            video_ranking.region_tag_dynamic_endpoint(),
            "https://api.bilibili.com/x/web-interface/dynamic/tag"
        );
        assert_eq!(
            video_ranking.region_newlist_endpoint(),
            "https://api.bilibili.com/x/web-interface/newlist"
        );
        assert_eq!(
            video_ranking.region_newlist_rank_endpoint(),
            "https://api.bilibili.com/x/web-interface/newlist_rank"
        );
        Ok(())
    }

    #[test]
    fn video_ranking_methods_return_payload_futures() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let video_ranking = client.video_ranking();

        assert_popular_list_future(
            video_ranking.popular_list(
                VideoPopularListParams::new()
                    .with_page(1)?
                    .with_page_size(2)?,
            ),
        );
        assert_series_list_future(video_ranking.popular_series_list());
        assert_series_one_future(video_ranking.popular_series_one(PopularSeriesOneParams::new(1)?));
        assert_precious_future(video_ranking.popular_precious());
        assert_ranking_list_future(
            video_ranking.ranking_list(
                VideoRankingListParams::new()
                    .with_rid(1)?
                    .with_type(VideoRankingType::All),
            ),
        );
        assert_region_archives_future(
            video_ranking.region_dynamic(
                VideoRegionDynamicParams::new(21)?
                    .with_page(1)?
                    .with_page_size(2)?,
            ),
        );
        assert_region_archives_future(
            video_ranking.region_tag_dynamic(
                VideoRegionTagDynamicParams::new(136, 10026108)?
                    .with_page(1)?
                    .with_page_size(2)?,
            ),
        );
        assert_region_archives_future(
            video_ranking.region_newlist(
                VideoRegionNewListParams::new(231)?
                    .with_page(1)?
                    .with_page_size(2)?
                    .with_type(1)?,
            ),
        );
        assert_newlist_rank_future(
            video_ranking.region_newlist_rank(
                VideoRegionNewListRankParams::new(231, 2, "20260701", "20260703")?
                    .with_order(VideoNewListRankOrder::Click)
                    .with_page(1)?,
            ),
        );
        Ok(())
    }

    #[test]
    fn video_ranking_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let video_ranking = client.video_ranking();

        let expectations = [
            (
                "popular-list",
                "video_ranking.popular_list",
                video_ranking.popular_list_endpoint(),
                false,
            ),
            (
                "popular-series-list",
                "video_ranking.popular_series_list",
                video_ranking.popular_series_list_endpoint(),
                false,
            ),
            (
                "popular-series-one",
                "video_ranking.popular_series_one",
                video_ranking.popular_series_one_endpoint(),
                true,
            ),
            (
                "popular-precious",
                "video_ranking.popular_precious",
                video_ranking.popular_precious_endpoint(),
                false,
            ),
            (
                "ranking-list",
                "video_ranking.ranking_list",
                video_ranking.ranking_list_endpoint(),
                false,
            ),
            (
                "region-dynamic",
                "video_ranking.region_dynamic",
                video_ranking.region_dynamic_endpoint(),
                false,
            ),
            (
                "region-tag-dynamic",
                "video_ranking.region_tag_dynamic",
                video_ranking.region_tag_dynamic_endpoint(),
                false,
            ),
            (
                "region-newlist",
                "video_ranking.region_newlist",
                video_ranking.region_newlist_endpoint(),
                false,
            ),
            (
                "region-newlist-rank",
                "video_ranking.region_newlist_rank",
                video_ranking.region_newlist_rank_endpoint(),
                false,
            ),
        ];

        for (endpoint, name, url, requires_wbi) in expectations {
            let contract = contract(endpoint)?;

            assert_eq!(contract.name, name);
            assert_eq!(contract.request.method, HttpMethod::Get);
            assert_eq!(contract.request.url.as_str(), url);
            assert_eq!(contract.request.auth.requires_wbi(), requires_wbi);
        }

        Ok(())
    }
}
