//! B站视频合集信息相关接口
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/video)
use serde::{Deserialize, Serialize};

pub(crate) const HOME_SEASONS_SERIES_ENDPOINT: &str =
    "https://api.bilibili.com/x/polymer/web-space/home/seasons_series";
pub(crate) const SEASONS_ARCHIVES_LIST_ENDPOINT: &str =
    "https://api.bilibili.com/x/polymer/web-space/seasons_archives_list";
pub(crate) const SEASONS_SERIES_LIST_ENDPOINT: &str =
    "https://api.bilibili.com/x/polymer/web-space/seasons_series_list";
pub(crate) const SERIES_ARCHIVES_ENDPOINT: &str = "https://api.bilibili.com/x/series/archives";
pub(crate) const SERIES_INFO_ENDPOINT: &str = "https://api.bilibili.com/x/series/series";

/// 稿件信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ArchiveStat {
    /// 稿件播放量
    pub view: u64,
    /// vt
    pub vt: Option<u64>,
}

/// 合集/系列中的视频信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Archive {
    /// 稿件 avid
    pub aid: u64,
    /// 稿件 bvid
    pub bvid: String,
    /// 创建时间Unix 时间戳
    pub ctime: u64,
    /// 视频时长，单位为秒
    pub duration: u64,
    /// 是否是互动视频
    pub interactive_video: bool,
    /// 封面 URL
    pub pic: String,
    /// 会随着播放时间增长，播放完成后为 -1。单位为 %
    pub playback_position: u64,
    /// 发布日期Unix 时间戳
    pub pubdate: u64,
    /// 稿件信息
    pub stat: ArchiveStat,
    /// state
    pub state: u64,
    /// 稿件标题
    pub title: String,
    /// UGC 付费? 0: 否
    pub ugc_pay: u64,
    /// vt_display
    pub vt_display: String,

    pub is_lesson_video: Option<u32>,
}

/// 分页信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PageInfo {
    /// 分页页码
    #[serde(alias = "num")]
    pub page_num: u64,
    /// 单页个数
    #[serde(alias = "size")]
    pub page_size: u64,
    /// 总页数/总数量
    pub total: u64,
}

/// 合集元数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SeasonsArchivesMeta {
    /// category
    pub category: u64,
    /// 合集封面 URL
    pub cover: String,
    /// 合集描述
    pub description: String,
    /// UP 主 ID
    pub mid: u64,
    /// 合集标题
    pub name: String,
    /// 发布时间Unix 时间戳
    pub ptime: u64,
    /// 合集 ID
    pub season_id: u64,
    /// 合集内视频数量
    pub total: u64,
}

/// 获取视频合集信息响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetSeasonsArchivesData {
    /// 稿件 avid 列表
    pub aids: Vec<u64>,
    /// 合集中的视频
    pub archives: Vec<Archive>,
    /// 合集元数据
    pub meta: SeasonsArchivesMeta,
    /// 分页信息
    pub page: PageInfo,
}

/// 合集元数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SeasonsMeta {
    /// category
    pub category: u64,
    /// 封面 URL
    pub cover: String,
    /// 描述
    pub description: String,
    /// UP 主 ID
    pub mid: u64,
    /// 标题
    pub name: String,
    /// 创建时间?
    pub ptime: u64,
    /// 合集 ID
    pub season_id: u64,
    /// 视频数量
    pub total: u64,
}

/// 系列元数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SeriesMeta {
    pub category: u64,
    pub creator: String,
    pub ctime: u64,
    pub description: String,
    pub keywords: Vec<String>,
    pub last_update_ts: u64,
    pub mid: u64,
    pub mtime: u64,
    pub name: String,
    pub raw_keywords: String,
    pub series_id: u64,
    pub state: u64,
    pub total: u64,
    pub cover: Option<String>,
}
/// 合集列表中的单个合集信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SeasonsItem {
    /// 系列视频列表
    pub archives: Vec<Archive>,
    /// 系列元数据
    pub meta: SeasonsMeta,
    /// 系列视频 aid 列表
    pub recent_aids: Vec<u64>,
}
/// 系列列表中的单个系列信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SeriesItem {
    /// 系列视频列表
    pub archives: Vec<Archive>,
    /// 系列元数据
    pub meta: SeriesMeta,
    /// 系列视频 aid 列表
    pub recent_aids: Vec<u64>,
}

/// 系列和合集列表信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemsList {
    /// 分页信息
    pub page: PageInfo,
    /// 合集列表
    pub seasons_list: Vec<SeasonsItem>,
    /// 系列列表
    pub series_list: Vec<SeriesItem>,
}

/// 获取系列视频列表响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetSeasonsSeriesData {
    /// 内容列表
    pub items_lists: ItemsList,
}

/// 查询指定系列响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetSeriesData {
    /// 系列信息
    pub meta: SeriesMeta,
    /// 系列 aid 列表
    pub recent_aids: Vec<u64>,
}

/// 获取指定系列视频响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetSeriesArchivesData {
    /// 视频 aid 列表
    pub aids: Vec<u64>,
    /// 页码信息
    pub page: PageInfo,
    /// 视频信息列表
    pub archives: Vec<Archive>,
}

#[cfg(test)]
mod tests {
    use super::super::params::{
        VideoCollectionHomeSeasonsSeriesParams, VideoCollectionSeasonsArchivesParams,
        VideoCollectionSeasonsSeriesParams, VideoCollectionSeriesArchivesParams,
        VideoCollectionSeriesInfoParams,
    };
    use super::*;
    use crate::ids::{Mid, SeasonId};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use tracing::info;

    use super::super::params::CollectionArchiveSort;

    // 测试用的 mid
    const TEST_MID: u64 = 1000001;
    // 测试用的合集 ID
    const TEST_SEASON_ID: u64 = 4294056;

    const TEST_SERIES_ID: u64 = 250285;

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "seasons-archives-list" => include_bytes!(
                "../../../tests/contracts/video/collection-read/seasons-archives-list/contract.json"
            )
            .as_slice(),
            "home-seasons-series" => include_bytes!(
                "../../../tests/contracts/video/collection-read/home-seasons-series/contract.json"
            )
            .as_slice(),
            "seasons-series-list" => include_bytes!(
                "../../../tests/contracts/video/collection-read/seasons-series-list/contract.json"
            )
            .as_slice(),
            "series-info" => include_bytes!(
                "../../../tests/contracts/video/collection-read/series-info/contract.json"
            )
            .as_slice(),
            "series-archives" => include_bytes!(
                "../../../tests/contracts/video/collection-read/series-archives/contract.json"
            )
            .as_slice(),
            _ => unreachable!("unknown video collection-read contract endpoint"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_seasons_archives_list() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let params = VideoCollectionSeasonsArchivesParams::new(
            Mid::new(TEST_MID)?,
            SeasonId::new(TEST_SEASON_ID)?,
        )
        .with_sort_reverse(false);
        let data = bpi.video().seasons_archives_list(params).await?;

        info!("测试结果: {:?}", data);
        assert!(!data.archives.is_empty(), "返回的合集视频列表不应为空");
        assert_eq!(data.meta.season_id, TEST_SEASON_ID, "合集ID应与请求ID一致");
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_seasons_series_only() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let params = VideoCollectionHomeSeasonsSeriesParams::new(Mid::new(TEST_MID)?);
        let data = bpi.video().home_seasons_series(params).await?;

        info!("测试结果: {:?}", data);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_seasons_series_list() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let params = VideoCollectionSeasonsSeriesParams::new(Mid::new(TEST_MID)?)
            .with_page_num(1)?
            .with_page_size(5)?;
        let data = bpi.video().seasons_series_list(params).await?;

        info!("测试结果: {:?}", data);
        assert!(
            !data.items_lists.series_list.is_empty(),
            "返回的系列列表不应为空"
        );
        // 注意：合集列表可能为空，无法直接断言不为空
        assert_eq!(data.items_lists.page.page_size, 5, "返回的每页数量应为5");
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_series_info() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let params = VideoCollectionSeriesInfoParams::new(TEST_SERIES_ID)?;
        let data = bpi.video().series_info(params).await?;

        info!("测试结果: {:?}", data);
        assert_eq!(
            data.meta.series_id, TEST_SERIES_ID,
            "返回的系列ID应与请求ID一致"
        );
        assert!(!data.recent_aids.is_empty(), "最近的aid列表不应为空");
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_series_archives() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let params = VideoCollectionSeriesArchivesParams::new(Mid::new(TEST_MID)?, TEST_SERIES_ID)?
            .with_sort(CollectionArchiveSort::Asc)
            .with_page_num(1)?
            .with_page_size(10)?;
        let data = bpi.video().series_archives(params).await?;

        info!("测试结果: {:?}", data);
        assert!(!data.archives.is_empty(), "返回的系列视频列表不应为空");
        Ok(())
    }

    #[test]
    fn video_collection_read_contracts_match_endpoint_requests() -> BpiResult<()> {
        let expectations = [
            (
                "seasons-archives-list",
                "video.collection.seasons_archives_list",
                SEASONS_ARCHIVES_LIST_ENDPOINT,
                VideoCollectionSeasonsArchivesParams::new(
                    Mid::new(TEST_MID)?,
                    SeasonId::new(TEST_SEASON_ID)?,
                )
                .with_sort_reverse(false)
                .query_pairs(),
                "GetSeasonsArchivesData",
                true,
            ),
            (
                "home-seasons-series",
                "video.collection.home_seasons_series",
                HOME_SEASONS_SERIES_ENDPOINT,
                VideoCollectionHomeSeasonsSeriesParams::new(Mid::new(TEST_MID)?).query_pairs(),
                "GetSeasonsSeriesData",
                true,
            ),
            (
                "seasons-series-list",
                "video.collection.seasons_series_list",
                SEASONS_SERIES_LIST_ENDPOINT,
                VideoCollectionSeasonsSeriesParams::new(Mid::new(TEST_MID)?)
                    .with_page_num(1)?
                    .with_page_size(5)?
                    .query_pairs(),
                "GetSeasonsSeriesData",
                true,
            ),
            (
                "series-info",
                "video.collection.series_info",
                SERIES_INFO_ENDPOINT,
                VideoCollectionSeriesInfoParams::new(TEST_SERIES_ID)?.query_pairs(),
                "GetSeriesData",
                false,
            ),
            (
                "series-archives",
                "video.collection.series_archives",
                SERIES_ARCHIVES_ENDPOINT,
                VideoCollectionSeriesArchivesParams::new(Mid::new(TEST_MID)?, TEST_SERIES_ID)?
                    .with_sort(CollectionArchiveSort::Asc)
                    .with_page_num(1)?
                    .with_page_size(10)?
                    .query_pairs(),
                "GetSeriesArchivesData",
                false,
            ),
        ];

        for (endpoint, name, url, query_pairs, rust_model, requires_wbi) in expectations {
            let contract = contract(endpoint)?;

            assert_eq!(contract.name, name);
            assert_eq!(contract.request.method, HttpMethod::Get);
            assert_eq!(contract.request.url.as_str(), url);
            assert_eq!(contract.request.auth.requires_wbi(), requires_wbi);
            assert_eq!(contract.cases.len(), 3);
            assert!(
                contract
                    .cases
                    .iter()
                    .all(|case| case.response.api_code == Some(0))
            );
            assert!(
                contract
                    .cases
                    .iter()
                    .all(|case| case.response.rust_model.as_deref() == Some(rust_model))
            );

            for (key, value) in query_pairs {
                assert_eq!(
                    contract.request.query.get(key).map(String::as_str),
                    Some(value.as_str())
                );
            }
        }

        Ok(())
    }

    #[test]
    fn video_collection_read_response_fixtures_parse_declared_models() -> BpiResult<()> {
        let seasons = ApiEnvelope::<GetSeasonsArchivesData>::from_slice(include_bytes!(
            "../../../tests/contracts/video/collection-read/seasons-archives-list/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(seasons.meta.season_id, TEST_SEASON_ID);

        let home_series = ApiEnvelope::<GetSeasonsSeriesData>::from_slice(include_bytes!(
            "../../../tests/contracts/video/collection-read/home-seasons-series/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(home_series.items_lists.page.page_num, 1);

        let seasons_series = ApiEnvelope::<GetSeasonsSeriesData>::from_slice(include_bytes!(
            "../../../tests/contracts/video/collection-read/seasons-series-list/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(seasons_series.items_lists.page.page_size, 5);

        let series = ApiEnvelope::<GetSeriesData>::from_slice(include_bytes!(
            "../../../tests/contracts/video/collection-read/series-info/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(series.meta.series_id, TEST_SERIES_ID);

        let series_archives = ApiEnvelope::<GetSeriesArchivesData>::from_slice(include_bytes!(
            "../../../tests/contracts/video/collection-read/series-archives/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(series_archives.page.page_size, 10);
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path = format!(
            "target/bpi-probe-runs/video/collection-read/{endpoint}/{profile}.response.json"
        );
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn video_collection_read_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body("seasons-archives-list", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<GetSeasonsArchivesData>>(body)?
                    .into_payload()?;
                assert_eq!(payload.meta.season_id, TEST_SEASON_ID);
            }

            for endpoint in ["home-seasons-series", "seasons-series-list"] {
                if let Some(body) = local_probe_body(endpoint, profile) {
                    let payload =
                        serde_json::from_value::<ApiEnvelope<GetSeasonsSeriesData>>(body)?
                            .into_payload()?;
                    assert!(payload.items_lists.page.page_num >= 1);
                    assert!(payload.items_lists.page.page_size >= 1);
                }
            }

            if let Some(body) = local_probe_body("series-info", profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<GetSeriesData>>(body)?.into_payload()?;
                assert_eq!(payload.meta.series_id, TEST_SERIES_ID);
            }

            if let Some(body) = local_probe_body("series-archives", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<GetSeriesArchivesData>>(body)?
                    .into_payload()?;
                assert_eq!(payload.page.page_size, 10);
            }
        }

        Ok(())
    }
}
