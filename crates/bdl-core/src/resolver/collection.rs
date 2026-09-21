use async_trait::async_trait;
use bpi_rs::BpiClient;
use bpi_rs::ids::{Mid, SeasonId};
use bpi_rs::video::collection::{
    Archive, GetSeasonsArchivesData, GetSeriesArchivesData, GetSeriesData,
};
use bpi_rs::video::{
    CollectionArchiveSort, VideoCollectionSeasonsArchivesParams,
    VideoCollectionSeriesArchivesParams, VideoCollectionSeriesInfoParams,
};
use std::sync::Arc;
use url::Url;

use super::paged::{PageRequest, PagedSourceKind, page_state};
use super::{ResolveOptions, Resolver, bilibili_publish_date};
use crate::error::{BdlError, BdlResult};
use crate::ids::{GroupId, ItemId, PartId, SourceId};
use crate::input::ClassifiedInput;
use crate::model::{
    AssetKind, FetchPolicy, NormalizedGroup, NormalizedItem, NormalizedPart, NormalizedSourceTree,
    SourceKind, SourceSummary,
};

const COLLECTION_API_MAX_PAGE_SIZE: u32 = 20;
const SERIES_API_MAX_PAGE_SIZE: u32 = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectionInputIds {
    pub mid: u64,
    pub season_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeriesInputIds {
    pub mid: Option<u64>,
    pub series_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedArchiveVideo {
    pub aid: u64,
    pub bvid: String,
    pub title: String,
    pub owner_mid: u64,
    pub owner_name: Option<String>,
    pub publish_date: Option<String>,
    pub cover_url: Option<String>,
    pub duration_seconds: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCollectionPage {
    pub mid: u64,
    pub season_id: u64,
    pub title: String,
    pub owner_name: Option<String>,
    pub request: PageRequest,
    pub total_count: Option<usize>,
    pub videos: Vec<ResolvedArchiveVideo>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSeriesPage {
    pub mid: u64,
    pub series_id: u64,
    pub title: String,
    pub owner_name: Option<String>,
    pub request: PageRequest,
    pub total_count: Option<usize>,
    pub videos: Vec<ResolvedArchiveVideo>,
}

#[async_trait]
pub trait CollectionApi: Send + Sync {
    async fn collection_page(
        &self,
        ids: CollectionInputIds,
        request: PageRequest,
    ) -> BdlResult<ResolvedCollectionPage>;
}

#[async_trait]
pub trait SeriesApi: Send + Sync {
    async fn series_page(
        &self,
        ids: SeriesInputIds,
        request: PageRequest,
    ) -> BdlResult<ResolvedSeriesPage>;
}

#[derive(Debug)]
pub struct CollectionResolver<A = BpiCollectionApi> {
    api: A,
}

#[derive(Debug)]
pub struct SeriesResolver<A = BpiSeriesApi> {
    api: A,
}

impl CollectionResolver<BpiCollectionApi> {
    pub fn new() -> BdlResult<Self> {
        Ok(Self::with_api(BpiCollectionApi::new()?))
    }

    pub fn from_cookie(cookie: &str) -> BdlResult<Self> {
        BpiClient::builder()
            .cookie(cookie)
            .build()
            .map(Self::from_bpi_client)
            .map_err(|error| BdlError::Bpi(error.to_string()))
    }

    pub fn from_bpi_client(client: impl Into<Arc<BpiClient>>) -> Self {
        Self::with_api(BpiCollectionApi::from_client(client))
    }
}

impl SeriesResolver<BpiSeriesApi> {
    pub fn new() -> BdlResult<Self> {
        Ok(Self::with_api(BpiSeriesApi::new()?))
    }

    pub fn from_cookie(cookie: &str) -> BdlResult<Self> {
        BpiClient::builder()
            .cookie(cookie)
            .build()
            .map(Self::from_bpi_client)
            .map_err(|error| BdlError::Bpi(error.to_string()))
    }

    pub fn from_bpi_client(client: impl Into<Arc<BpiClient>>) -> Self {
        Self::with_api(BpiSeriesApi::from_client(client))
    }
}

impl<A> CollectionResolver<A> {
    pub fn with_api(api: A) -> Self {
        Self { api }
    }
}

impl<A> SeriesResolver<A> {
    pub fn with_api(api: A) -> Self {
        Self { api }
    }
}

impl<A> CollectionResolver<A>
where
    A: CollectionApi,
{
    pub async fn resolve_page(
        &self,
        ids: CollectionInputIds,
        request: PageRequest,
    ) -> BdlResult<NormalizedSourceTree> {
        let page = self.api.collection_page(ids, request).await?;
        Ok(collection_tree_from_page(page, None))
    }
}

impl<A> SeriesResolver<A>
where
    A: SeriesApi,
{
    pub async fn resolve_page(
        &self,
        ids: SeriesInputIds,
        request: PageRequest,
    ) -> BdlResult<NormalizedSourceTree> {
        let page = self.api.series_page(ids, request).await?;
        Ok(series_tree_from_page(page, None))
    }
}

#[async_trait]
impl<A> Resolver for CollectionResolver<A>
where
    A: CollectionApi,
{
    async fn resolve(
        &self,
        input: ClassifiedInput,
        _options: ResolveOptions,
    ) -> BdlResult<NormalizedSourceTree> {
        let ClassifiedInput::Collection { raw_url } = input else {
            return Err(BdlError::UnsupportedSource {
                kind: source_kind_name(input.source_kind()).to_owned(),
            });
        };

        let ids = collection_ids_from_url(&raw_url)?;
        let request = PageRequest::first(
            PagedSourceKind::Collection,
            Some(COLLECTION_API_MAX_PAGE_SIZE),
        );
        let page = self.api.collection_page(ids, request).await?;

        Ok(collection_tree_from_page(page, Some(raw_url)))
    }
}

#[async_trait]
impl<A> Resolver for SeriesResolver<A>
where
    A: SeriesApi,
{
    async fn resolve(
        &self,
        input: ClassifiedInput,
        _options: ResolveOptions,
    ) -> BdlResult<NormalizedSourceTree> {
        let ClassifiedInput::Series { raw_url } = input else {
            return Err(BdlError::UnsupportedSource {
                kind: source_kind_name(input.source_kind()).to_owned(),
            });
        };

        let ids = series_ids_from_url(&raw_url)?;
        let request = PageRequest::first(PagedSourceKind::Series, Some(SERIES_API_MAX_PAGE_SIZE));
        let page = self.api.series_page(ids, request).await?;

        Ok(series_tree_from_page(page, Some(raw_url)))
    }
}

pub struct BpiCollectionApi {
    client: Arc<BpiClient>,
}

pub struct BpiSeriesApi {
    client: Arc<BpiClient>,
}

impl BpiCollectionApi {
    pub fn new() -> BdlResult<Self> {
        BpiClient::new()
            .map(Self::from_client)
            .map_err(|error| BdlError::Bpi(error.to_string()))
    }

    pub fn from_client(client: impl Into<Arc<BpiClient>>) -> Self {
        Self {
            client: client.into(),
        }
    }
}

impl BpiSeriesApi {
    pub fn new() -> BdlResult<Self> {
        BpiClient::new()
            .map(Self::from_client)
            .map_err(|error| BdlError::Bpi(error.to_string()))
    }

    pub fn from_client(client: impl Into<Arc<BpiClient>>) -> Self {
        Self {
            client: client.into(),
        }
    }
}

#[async_trait]
impl CollectionApi for BpiCollectionApi {
    async fn collection_page(
        &self,
        ids: CollectionInputIds,
        request: PageRequest,
    ) -> BdlResult<ResolvedCollectionPage> {
        let mid = Mid::new(ids.mid).map_err(|error| BdlError::Bpi(error.to_string()))?;
        let season_id =
            SeasonId::new(ids.season_id).map_err(|error| BdlError::Bpi(error.to_string()))?;
        let params = VideoCollectionSeasonsArchivesParams::new(mid, season_id)
            .with_sort_reverse(false)
            .with_page_num(u64::from(request.page_number))
            .map_err(|error| BdlError::Bpi(error.to_string()))?
            .with_page_size(u64::from(request.page_size))
            .map_err(|error| BdlError::Bpi(error.to_string()))?;
        let data = self
            .client
            .video()
            .seasons_archives_list(params)
            .await
            .map_err(|error| BdlError::Bpi(error.to_string()))?;

        Ok(ResolvedCollectionPage::from_bpi(data, request))
    }
}

#[async_trait]
impl SeriesApi for BpiSeriesApi {
    async fn series_page(
        &self,
        ids: SeriesInputIds,
        request: PageRequest,
    ) -> BdlResult<ResolvedSeriesPage> {
        let info_params = VideoCollectionSeriesInfoParams::new(ids.series_id)
            .map_err(|error| BdlError::Bpi(error.to_string()))?;
        let info = self
            .client
            .video()
            .series_info(info_params)
            .await
            .map_err(|error| BdlError::Bpi(error.to_string()))?;
        let mid = ids.mid.unwrap_or(info.meta.mid);
        let params = VideoCollectionSeriesArchivesParams::new(
            Mid::new(mid).map_err(|error| BdlError::Bpi(error.to_string()))?,
            ids.series_id,
        )
        .map_err(|error| BdlError::Bpi(error.to_string()))?
        .with_sort(CollectionArchiveSort::Asc)
        .with_page_num(u64::from(request.page_number))
        .map_err(|error| BdlError::Bpi(error.to_string()))?
        .with_page_size(u64::from(request.page_size))
        .map_err(|error| BdlError::Bpi(error.to_string()))?;
        let archives = self
            .client
            .video()
            .series_archives(params)
            .await
            .map_err(|error| BdlError::Bpi(error.to_string()))?;

        Ok(ResolvedSeriesPage::from_bpi(info, archives, request))
    }
}

impl ResolvedCollectionPage {
    fn from_bpi(data: GetSeasonsArchivesData, fallback_request: PageRequest) -> Self {
        let meta = data.meta;
        let request =
            page_request_from_bpi(data.page.page_num, data.page.page_size, fallback_request);
        let total_count = usize::try_from(meta.total)
            .ok()
            .or_else(|| usize::try_from(data.page.total).ok());
        let videos = data
            .archives
            .into_iter()
            .filter_map(|archive| ResolvedArchiveVideo::from_archive(meta.mid, None, archive))
            .collect();

        Self {
            mid: meta.mid,
            season_id: meta.season_id,
            title: meta.name,
            owner_name: None,
            request,
            total_count,
            videos,
        }
    }
}

impl ResolvedSeriesPage {
    fn from_bpi(
        info: GetSeriesData,
        archives: GetSeriesArchivesData,
        fallback_request: PageRequest,
    ) -> Self {
        let meta = info.meta;
        let request = page_request_from_bpi(
            archives.page.page_num,
            archives.page.page_size,
            fallback_request,
        );
        let total_count = usize::try_from(meta.total)
            .ok()
            .or_else(|| usize::try_from(archives.page.total).ok());
        let owner_name = non_empty(meta.creator);
        let videos = archives
            .archives
            .into_iter()
            .filter_map(|archive| {
                ResolvedArchiveVideo::from_archive(meta.mid, owner_name.clone(), archive)
            })
            .collect();

        Self {
            mid: meta.mid,
            series_id: meta.series_id,
            title: meta.name,
            owner_name,
            request,
            total_count,
            videos,
        }
    }
}

impl ResolvedArchiveVideo {
    fn from_archive(owner_mid: u64, owner_name: Option<String>, archive: Archive) -> Option<Self> {
        (!archive.bvid.trim().is_empty()).then_some(Self {
            aid: archive.aid,
            bvid: archive.bvid,
            title: archive.title,
            owner_mid,
            owner_name,
            publish_date: bilibili_publish_date(archive.pubdate),
            cover_url: non_empty(archive.pic),
            duration_seconds: Some(archive.duration),
        })
    }
}

pub fn collection_ids_from_url(raw_url: &str) -> BdlResult<CollectionInputIds> {
    let url = Url::parse(raw_url).map_err(|_| BdlError::InvalidInput {
        message: "无法识别合集链接。".to_owned(),
    })?;
    let segments = path_segments(&url);
    let mid = query_u64(&url, &["mid", "owner_mid"])
        .or_else(|| mid_from_path(&url, &segments))
        .ok_or_else(|| BdlError::InvalidInput {
            message: "合集链接缺少 UP 主 mid，无法调用合集分页接口。".to_owned(),
        })?;
    let season_id = query_u64(&url, &["season_id"])
        .or_else(|| list_id_from_space_path(&url, &segments, &["season", "collection"]))
        .or_else(|| collected_favorite_id(&url, &segments))
        .ok_or_else(|| BdlError::InvalidInput {
            message: "合集链接缺少 season_id 参数。".to_owned(),
        })?;

    Ok(CollectionInputIds { mid, season_id })
}

pub fn series_ids_from_url(raw_url: &str) -> BdlResult<SeriesInputIds> {
    let url = Url::parse(raw_url).map_err(|_| BdlError::InvalidInput {
        message: "无法识别系列链接。".to_owned(),
    })?;
    let segments = path_segments(&url);
    let mid = query_u64(&url, &["mid", "owner_mid"]).or_else(|| mid_from_path(&url, &segments));
    let series_id = query_u64(&url, &["series_id", "sid"])
        .or_else(|| list_id_from_space_path(&url, &segments, &["series"]))
        .ok_or_else(|| BdlError::InvalidInput {
            message: "系列链接缺少 series_id 参数。".to_owned(),
        })?;

    Ok(SeriesInputIds { mid, series_id })
}

fn collection_tree_from_page(
    page: ResolvedCollectionPage,
    input_override: Option<String>,
) -> NormalizedSourceTree {
    let page_state = page_state(page.request, page.videos.len(), page.total_count);
    let title = source_title(&page.title, || format!("合集 {}", page.season_id));
    let source_key = format!("collection:{}:{}", page.mid, page.season_id);
    let items = page
        .videos
        .into_iter()
        .map(|video| map_archive_item(&source_key, video))
        .collect::<Vec<_>>();

    NormalizedSourceTree {
        source: SourceSummary {
            id: SourceId(source_key.clone()),
            kind: SourceKind::Collection,
            input: input_override.unwrap_or_else(|| {
                format!(
                    "https://space.bilibili.com/{}/lists/{}?type=season",
                    page.mid, page.season_id
                )
            }),
            title: title.clone(),
            loaded_count: items.len(),
            total_count: page.total_count,
            has_more: page_state.has_more,
        },
        groups: vec![NormalizedGroup {
            id: GroupId(format!("group:{source_key}")),
            kind: "collection".to_owned(),
            title,
            items,
            page: Some(page_state),
        }],
    }
}

fn series_tree_from_page(
    page: ResolvedSeriesPage,
    input_override: Option<String>,
) -> NormalizedSourceTree {
    let page_state = page_state(page.request, page.videos.len(), page.total_count);
    let title = source_title(&page.title, || format!("系列 {}", page.series_id));
    let source_key = format!("series:{}:{}", page.mid, page.series_id);
    let items = page
        .videos
        .into_iter()
        .map(|video| map_archive_item(&source_key, video))
        .collect::<Vec<_>>();

    NormalizedSourceTree {
        source: SourceSummary {
            id: SourceId(source_key.clone()),
            kind: SourceKind::Series,
            input: input_override.unwrap_or_else(|| {
                format!(
                    "https://space.bilibili.com/{}/lists/{}?type=series",
                    page.mid, page.series_id
                )
            }),
            title: title.clone(),
            loaded_count: items.len(),
            total_count: page.total_count,
            has_more: page_state.has_more,
        },
        groups: vec![NormalizedGroup {
            id: GroupId(format!("group:{source_key}")),
            kind: "series".to_owned(),
            title,
            items,
            page: Some(page_state),
        }],
    }
}

fn map_archive_item(source_key: &str, video: ResolvedArchiveVideo) -> NormalizedItem {
    let video_key = format!("{source_key}:{}", video.bvid);

    NormalizedItem {
        id: ItemId(format!("item:{video_key}")),
        title: video.title.clone(),
        owner_name: video.owner_name.clone(),
        owner_mid: Some(video.owner_mid),
        publish_date: video.publish_date,
        cover_url: video.cover_url.clone(),
        duration_seconds: video.duration_seconds,
        parts: vec![NormalizedPart {
            id: PartId(format!("part:{video_key}")),
            title: video.title,
            aid: Some(video.aid),
            bvid: Some(video.bvid),
            cid: None,
            duration_seconds: video.duration_seconds,
            streams: Vec::new(),
            assets: vec![AssetKind::Cover.with_policy(FetchPolicy::OnDemand)],
        }],
    }
}

fn page_request_from_bpi(page_number: u64, page_size: u64, fallback: PageRequest) -> PageRequest {
    PageRequest {
        page_number: positive_u32(page_number).unwrap_or(fallback.page_number),
        page_size: positive_u32(page_size).unwrap_or(fallback.page_size),
    }
}

fn positive_u32(value: u64) -> Option<u32> {
    u32::try_from(value).ok().filter(|value| *value > 0)
}

fn path_segments(url: &Url) -> Vec<&str> {
    url.path_segments()
        .map(|segments| segments.filter(|segment| !segment.is_empty()).collect())
        .unwrap_or_default()
}

fn mid_from_path(url: &Url, segments: &[&str]) -> Option<u64> {
    if is_space_host(url.host_str()?) {
        return segments.first()?.parse::<u64>().ok();
    }

    if path_starts_with(segments, &["medialist", "play"]) {
        return segments.get(2)?.parse::<u64>().ok();
    }

    None
}

fn list_id_from_space_path(url: &Url, segments: &[&str], allowed_types: &[&str]) -> Option<u64> {
    if !is_space_host(url.host_str()?)
        || !segments
            .get(1)
            .is_some_and(|segment| segment.eq_ignore_ascii_case("lists"))
    {
        return None;
    }

    let list_type = query_value(url, "type")?;
    if !allowed_types
        .iter()
        .any(|allowed| list_type.eq_ignore_ascii_case(allowed))
    {
        return None;
    }

    segments.get(2)?.parse::<u64>().ok()
}

fn collected_favorite_id(url: &Url, segments: &[&str]) -> Option<u64> {
    if !is_space_host(url.host_str()?)
        || !segments
            .get(1)
            .is_some_and(|segment| segment.eq_ignore_ascii_case("favlist"))
        || !query_value(url, "ftype")?.eq_ignore_ascii_case("collect")
        || !query_value(url, "ctype")?.eq_ignore_ascii_case("21")
    {
        return None;
    }

    query_u64(url, &["fid"])
}

fn query_u64(url: &Url, keys: &[&str]) -> Option<u64> {
    url.query_pairs().find_map(|(key, value)| {
        keys.iter()
            .any(|expected| key == *expected)
            .then(|| value.parse::<u64>().ok().filter(|value| *value > 0))
            .flatten()
    })
}

fn query_value(url: &Url, key: &str) -> Option<String> {
    url.query_pairs()
        .find_map(|(name, value)| (name == key).then(|| value.into_owned()))
}

fn path_starts_with(segments: &[&str], prefix: &[&str]) -> bool {
    segments
        .iter()
        .zip(prefix.iter())
        .all(|(segment, expected)| segment.eq_ignore_ascii_case(expected))
        && segments.len() >= prefix.len()
}

fn is_space_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("space.bilibili.com")
}

fn source_title(title: &str, fallback: impl FnOnce() -> String) -> String {
    if title.trim().is_empty() {
        fallback()
    } else {
        title.to_owned()
    }
}

fn source_kind_name(kind: SourceKind) -> &'static str {
    match kind {
        SourceKind::Video => "video",
        SourceKind::Bangumi => "bangumi",
        SourceKind::Cheese => "cheese",
        SourceKind::Favorite => "favorite",
        SourceKind::Collection => "collection",
        SourceKind::Series => "series",
        SourceKind::Uploader => "uploader",
        SourceKind::Unknown => "unknown",
    }
}

fn non_empty(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}
