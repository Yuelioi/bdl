use crate::creativecenter::params::{
    UpArchiveCompareParams, UpArchiveVideosParams, UpArchivesListParams, UpArticleTrendParams,
    UpVideoTrendParams,
};
use crate::creativecenter::railgun::ElectromagneticInfo;
use crate::creativecenter::season::list::SeasonListData;
use crate::creativecenter::season::section::SeasonSectionEpisodesData;
use crate::creativecenter::season::{
    SeasonByAidData, SeasonByAidParams, SeasonInfoData, SeasonInfoParams, SeasonListParams,
    SeasonSectionEpisodesParams,
};
use crate::creativecenter::statistics_data::{
    ArchiveCompareData, ArticleTrendItem, PlaySourceData, UpArticleStatData, UpStatData,
    VideoTrendItem, ViewerData,
};
use crate::creativecenter::videos::{ArchiveVideosData, SpArchivesData};
use crate::{BilibiliRequest, BpiClient, BpiResult};

const SEASON_LIST_ENDPOINT: &str = "https://member.bilibili.com/x2/creative/web/seasons";
const SEASON_INFO_ENDPOINT: &str = "https://member.bilibili.com/x2/creative/web/season";
const SEASON_BY_AID_ENDPOINT: &str = "https://member.bilibili.com/x2/creative/web/season/aid";
const SEASON_SECTION_EPISODES_ENDPOINT: &str =
    "https://member.bilibili.com/x2/creative/web/season/section";
const ARCHIVES_LIST_ENDPOINT: &str = "https://member.bilibili.com/x2/creative/web/archives/sp";
const ARCHIVE_VIDEOS_ENDPOINT: &str = "https://member.bilibili.com/x/web/archive/videos";
const UP_STAT_ENDPOINT: &str = "https://member.bilibili.com/x/web/index/stat";
const ARCHIVE_COMPARE_ENDPOINT: &str =
    "https://member.bilibili.com/x/web/data/archive_diagnose/compare";
const ARTICLE_STAT_ENDPOINT: &str = "https://member.bilibili.com/x/web/data/article";
const VIDEO_TREND_ENDPOINT: &str = "https://member.bilibili.com/x/web/data/pandect";
const ARTICLE_TREND_ENDPOINT: &str = "https://member.bilibili.com/x/web/data/article/thirty";
const PLAY_SOURCE_ENDPOINT: &str = "https://member.bilibili.com/x/web/data/playsource";
const VIEWER_DATA_ENDPOINT: &str = "https://member.bilibili.com/x/web/data/base";
const ELECTROMAGNETIC_INFO_ENDPOINT: &str =
    "https://api.bilibili.com/studio/up-rating/v3/rating/info";

/// 创作中心 API 客户端。
#[derive(Clone, Copy)]
pub struct CreativeCenterClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> CreativeCenterClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn season_list_endpoint(&self) -> &'static str {
        SEASON_LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn season_info_endpoint(&self) -> &'static str {
        SEASON_INFO_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn season_by_aid_endpoint(&self) -> &'static str {
        SEASON_BY_AID_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn season_section_episodes_endpoint(&self) -> &'static str {
        SEASON_SECTION_EPISODES_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn archives_list_endpoint(&self) -> &'static str {
        ARCHIVES_LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn archive_videos_endpoint(&self) -> &'static str {
        ARCHIVE_VIDEOS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn up_stat_endpoint(&self) -> &'static str {
        UP_STAT_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn archive_compare_endpoint(&self) -> &'static str {
        ARCHIVE_COMPARE_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn article_stat_endpoint(&self) -> &'static str {
        ARTICLE_STAT_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn video_trend_endpoint(&self) -> &'static str {
        VIDEO_TREND_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn article_trend_endpoint(&self) -> &'static str {
        ARTICLE_TREND_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn play_source_endpoint(&self) -> &'static str {
        PLAY_SOURCE_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn viewer_data_endpoint(&self) -> &'static str {
        VIEWER_DATA_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn electromagnetic_info_endpoint(&self) -> &'static str {
        ELECTROMAGNETIC_INFO_ENDPOINT
    }

    /// 列出当前已认证用户创建的合集。
    pub async fn season_list(&self, params: SeasonListParams) -> BpiResult<SeasonListData> {
        self.client
            .get(SEASON_LIST_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("creativecenter.season.list")
            .await
    }

    /// 按 season id 获取一个创作中心合集。
    pub async fn season_info(&self, params: SeasonInfoParams) -> BpiResult<SeasonInfoData> {
        self.client
            .get(SEASON_INFO_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("creativecenter.season.info")
            .await
    }

    /// 获取包含指定 archive id 的合集。
    pub async fn season_by_aid(&self, params: SeasonByAidParams) -> BpiResult<SeasonByAidData> {
        self.client
            .get(SEASON_BY_AID_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("creativecenter.season.aid")
            .await
    }

    /// 获取创作中心合集分区中的视频。
    pub async fn season_section_episodes(
        &self,
        params: SeasonSectionEpisodesParams,
    ) -> BpiResult<SeasonSectionEpisodesData> {
        self.client
            .get(SEASON_SECTION_EPISODES_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("creativecenter.season.section")
            .await
    }

    /// 列出当前已认证创作者的稿件。
    pub async fn archives_list(&self, params: UpArchivesListParams) -> BpiResult<SpArchivesData> {
        self.client
            .get(ARCHIVES_LIST_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("creativecenter.videos.archives_list")
            .await
    }

    /// 获取创作者稿件的基础视频信息。
    pub async fn archive_videos(
        &self,
        params: UpArchiveVideosParams,
    ) -> BpiResult<ArchiveVideosData> {
        self.client
            .get(ARCHIVE_VIDEOS_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("creativecenter.videos.archive_videos")
            .await
    }

    /// 获取创作者视频统计。
    pub async fn up_stat(&self) -> BpiResult<UpStatData> {
        self.client
            .get(UP_STAT_ENDPOINT)
            .send_bpi_payload("creativecenter.statistics.up_stat")
            .await
    }

    /// 获取创作者稿件对比统计。
    pub async fn archive_compare(
        &self,
        params: UpArchiveCompareParams,
    ) -> BpiResult<ArchiveCompareData> {
        self.client
            .get(ARCHIVE_COMPARE_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("creativecenter.statistics.archive_compare")
            .await
    }

    /// 获取创作者专栏统计。
    pub async fn article_stat(&self) -> BpiResult<UpArticleStatData> {
        self.client
            .get(ARTICLE_STAT_ENDPOINT)
            .send_bpi_payload("creativecenter.statistics.article_stat")
            .await
    }

    /// 获取创作者视频趋势数据。
    pub async fn video_trend(&self, params: UpVideoTrendParams) -> BpiResult<Vec<VideoTrendItem>> {
        self.client
            .get(VIDEO_TREND_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("creativecenter.statistics.video_trend")
            .await
    }

    /// 获取创作者专栏趋势数据。
    pub async fn article_trend(
        &self,
        params: UpArticleTrendParams,
    ) -> BpiResult<Vec<ArticleTrendItem>> {
        self.client
            .get(ARTICLE_TREND_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("creativecenter.statistics.article_trend")
            .await
    }

    /// 获取创作者播放来源分布。
    pub async fn play_source(&self) -> BpiResult<PlaySourceData> {
        self.client
            .get(PLAY_SOURCE_ENDPOINT)
            .with_bilibili_headers()
            .send_bpi_payload("creativecenter.statistics.play_source")
            .await
    }

    /// 获取创作者观众分布数据。
    pub async fn viewer_data(&self) -> BpiResult<ViewerData> {
        self.client
            .get(VIEWER_DATA_ENDPOINT)
            .send_bpi_payload("creativecenter.statistics.viewer_data")
            .await
    }

    /// 获取当前账号的电磁力评级信息。
    pub async fn electromagnetic_info(&self) -> BpiResult<ElectromagneticInfo> {
        self.client
            .get(ELECTROMAGNETIC_INFO_ENDPOINT)
            .send_bpi_payload("creativecenter.railgun.electromagnetic_info")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::creativecenter::params::{
        UpArchiveCompareParams, UpArchiveVideosParams, UpArchivesListParams, UpArticleTrendMetric,
        UpArticleTrendParams, UpVideoTrendMetric, UpVideoTrendParams,
    };
    use crate::creativecenter::railgun::ElectromagneticInfo;
    use crate::creativecenter::season::list::SeasonListData;
    use crate::creativecenter::season::section::SeasonSectionEpisodesData;
    use crate::creativecenter::season::{
        SeasonByAidData, SeasonByAidParams, SeasonInfoData, SeasonInfoParams, SeasonListOrder,
        SeasonListParams, SeasonListSort, SeasonSectionEpisodesParams,
    };
    use crate::creativecenter::statistics_data::{
        ArchiveCompareData, ArticleTrendItem, PlaySourceData, UpArticleStatData, UpStatData,
        VideoTrendItem, ViewerData,
    };
    use crate::creativecenter::videos::{ArchiveVideosData, SpArchivesData};
    use crate::ids::{Aid, SeasonId};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiResult};

    fn assert_season_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<SeasonListData>>,
    {
    }

    fn assert_season_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<SeasonInfoData>>,
    {
    }

    fn assert_season_by_aid_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<SeasonByAidData>>,
    {
    }

    fn assert_season_section_episodes_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<SeasonSectionEpisodesData>>,
    {
    }

    fn assert_archives_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<SpArchivesData>>,
    {
    }

    fn assert_archive_videos_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ArchiveVideosData>>,
    {
    }

    fn assert_up_stat_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<UpStatData>>,
    {
    }

    fn assert_archive_compare_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ArchiveCompareData>>,
    {
    }

    fn assert_article_stat_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<UpArticleStatData>>,
    {
    }

    fn assert_video_trend_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<Vec<VideoTrendItem>>>,
    {
    }

    fn assert_article_trend_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<Vec<ArticleTrendItem>>>,
    {
    }

    fn assert_play_source_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<PlaySourceData>>,
    {
    }

    fn assert_viewer_data_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ViewerData>>,
    {
    }

    fn assert_electromagnetic_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ElectromagneticInfo>>,
    {
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "season-list" => include_bytes!(
                "../../tests/contracts/creativecenter/season/list/contract.json"
            )
            .as_slice(),
            "season-info" => include_bytes!(
                "../../tests/contracts/creativecenter/season/info/contract.json"
            )
            .as_slice(),
            "season-aid" => include_bytes!(
                "../../tests/contracts/creativecenter/season/aid/contract.json"
            )
            .as_slice(),
            "season-section" => include_bytes!(
                "../../tests/contracts/creativecenter/season/section/contract.json"
            )
            .as_slice(),
            "archives-list" => include_bytes!(
                "../../tests/contracts/creativecenter/videos/archives-list/contract.json"
            )
            .as_slice(),
            "archive-videos" => include_bytes!(
                "../../tests/contracts/creativecenter/videos/archive-videos/contract.json"
            )
            .as_slice(),
            "up-stat" => include_bytes!(
                "../../tests/contracts/creativecenter/statistics/up-stat/contract.json"
            )
            .as_slice(),
            "archive-compare" => include_bytes!(
                "../../tests/contracts/creativecenter/statistics/archive-compare/contract.json"
            )
            .as_slice(),
            "article-stat" => include_bytes!(
                "../../tests/contracts/creativecenter/statistics/article-stat/contract.json"
            )
            .as_slice(),
            "video-trend" => include_bytes!(
                "../../tests/contracts/creativecenter/statistics/video-trend/contract.json"
            )
            .as_slice(),
            "article-trend" => include_bytes!(
                "../../tests/contracts/creativecenter/statistics/article-trend/contract.json"
            )
            .as_slice(),
            "play-source" => include_bytes!(
                "../../tests/contracts/creativecenter/statistics/play-source/contract.json"
            )
            .as_slice(),
            "viewer-data" => include_bytes!(
                "../../tests/contracts/creativecenter/statistics/viewer-data/contract.json"
            )
            .as_slice(),
            "electromagnetic-info" => include_bytes!(
                "../../tests/contracts/creativecenter/railgun-read/electromagnetic-info/contract.json"
            )
            .as_slice(),
            _ => unreachable!("unknown creativecenter contract"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn creativecenter_methods_return_payload_futures() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let creativecenter = client.creativecenter();

        assert_season_list_future(
            creativecenter.season_list(
                SeasonListParams::new(1, 10)?
                    .with_order(SeasonListOrder::CreatedAt)
                    .with_sort(SeasonListSort::Desc),
            ),
        );
        assert_season_info_future(
            creativecenter.season_info(SeasonInfoParams::new(SeasonId::new(4294056)?)),
        );
        assert_season_by_aid_future(
            creativecenter.season_by_aid(SeasonByAidParams::new(Aid::new(113602455409683)?)),
        );
        assert_season_section_episodes_future(
            creativecenter
                .season_section_episodes(SeasonSectionEpisodesParams::new(SeasonId::new(176088)?)),
        );
        assert_archives_list_future(
            creativecenter.archives_list(UpArchivesListParams::new(1)?.with_page_size(10)?),
        );
        assert_archive_videos_future(
            creativecenter.archive_videos(UpArchiveVideosParams::new(Aid::new(113602455409683)?)),
        );
        assert_up_stat_future(creativecenter.up_stat());
        assert_archive_compare_future(
            creativecenter.archive_compare(
                UpArchiveCompareParams::new()
                    .with_timestamp(1_720_000_000)?
                    .with_size(1)?,
            ),
        );
        assert_article_stat_future(creativecenter.article_stat());
        assert_video_trend_future(
            creativecenter.video_trend(UpVideoTrendParams::new(UpVideoTrendMetric::Play)),
        );
        assert_article_trend_future(
            creativecenter.article_trend(UpArticleTrendParams::new(UpArticleTrendMetric::Read)),
        );
        assert_play_source_future(creativecenter.play_source());
        assert_viewer_data_future(creativecenter.viewer_data());
        assert_electromagnetic_info_future(creativecenter.electromagnetic_info());
        Ok(())
    }

    #[test]
    fn creativecenter_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let creativecenter = client.creativecenter();

        let expectations = [
            (
                "season-list",
                "creativecenter.season.list",
                creativecenter.season_list_endpoint(),
            ),
            (
                "season-info",
                "creativecenter.season.info",
                creativecenter.season_info_endpoint(),
            ),
            (
                "season-aid",
                "creativecenter.season.aid",
                creativecenter.season_by_aid_endpoint(),
            ),
            (
                "season-section",
                "creativecenter.season.section",
                creativecenter.season_section_episodes_endpoint(),
            ),
            (
                "archives-list",
                "creativecenter.videos.archives_list",
                creativecenter.archives_list_endpoint(),
            ),
            (
                "archive-videos",
                "creativecenter.videos.archive_videos",
                creativecenter.archive_videos_endpoint(),
            ),
            (
                "up-stat",
                "creativecenter.statistics.up_stat",
                creativecenter.up_stat_endpoint(),
            ),
            (
                "archive-compare",
                "creativecenter.statistics.archive_compare",
                creativecenter.archive_compare_endpoint(),
            ),
            (
                "article-stat",
                "creativecenter.statistics.article_stat",
                creativecenter.article_stat_endpoint(),
            ),
            (
                "video-trend",
                "creativecenter.statistics.video_trend",
                creativecenter.video_trend_endpoint(),
            ),
            (
                "article-trend",
                "creativecenter.statistics.article_trend",
                creativecenter.article_trend_endpoint(),
            ),
            (
                "play-source",
                "creativecenter.statistics.play_source",
                creativecenter.play_source_endpoint(),
            ),
            (
                "viewer-data",
                "creativecenter.statistics.viewer_data",
                creativecenter.viewer_data_endpoint(),
            ),
            (
                "electromagnetic-info",
                "creativecenter.railgun.electromagnetic_info",
                creativecenter.electromagnetic_info_endpoint(),
            ),
        ];

        for (endpoint, name, url) in expectations {
            let contract = contract(endpoint)?;

            assert_eq!(contract.name, name);
            assert_eq!(contract.request.method, HttpMethod::Get);
            assert_eq!(contract.request.url.as_str(), url);
        }

        Ok(())
    }
}
