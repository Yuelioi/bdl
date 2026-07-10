use async_trait::async_trait;
use bpi_rs::BpiClient;
use bpi_rs::fav::FavListDetailParams;
use bpi_rs::fav::list::FavListMedia;
use bpi_rs::ids::MediaId;
use url::Url;

use super::paged::{PageRequest, PagedSourceKind};
use super::{ResolveOptions, Resolver};
use crate::error::{BdlError, BdlResult};
use crate::ids::{GroupId, ItemId, PartId, SourceId};
use crate::input::ClassifiedInput;
use crate::model::{
    AssetKind, FetchPolicy, NormalizedGroup, NormalizedItem, NormalizedPart, NormalizedSourceTree,
    PageState, SourceKind, SourceSummary,
};

const FAVORITE_API_MAX_PAGE_SIZE: u32 = 20;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFavoriteVideo {
    pub aid: u64,
    pub bvid: String,
    pub title: String,
    pub owner_mid: u64,
    pub owner_name: Option<String>,
    pub cover_url: Option<String>,
    pub duration_seconds: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedFavoritePage {
    pub media_id: u64,
    pub title: String,
    pub owner_name: Option<String>,
    pub request: PageRequest,
    pub total_count: Option<usize>,
    pub has_more: bool,
    pub videos: Vec<ResolvedFavoriteVideo>,
}

#[async_trait]
pub trait FavoriteApi: Send + Sync {
    async fn list_detail(
        &self,
        media_id: u64,
        request: PageRequest,
    ) -> BdlResult<ResolvedFavoritePage>;
}

#[derive(Debug)]
pub struct FavoriteResolver<A = BpiFavoriteApi> {
    api: A,
}

impl FavoriteResolver<BpiFavoriteApi> {
    pub fn new() -> BdlResult<Self> {
        Ok(Self::with_api(BpiFavoriteApi::new()?))
    }

    pub fn from_cookie(cookie: &str) -> BdlResult<Self> {
        BpiClient::builder()
            .cookie(cookie)
            .build()
            .map(Self::from_bpi_client)
            .map_err(|error| BdlError::Bpi(error.to_string()))
    }

    pub fn from_bpi_client(client: BpiClient) -> Self {
        Self::with_api(BpiFavoriteApi::from_client(client))
    }
}

impl<A> FavoriteResolver<A> {
    pub fn with_api(api: A) -> Self {
        Self { api }
    }
}

impl<A> FavoriteResolver<A>
where
    A: FavoriteApi,
{
    pub async fn resolve_page(
        &self,
        media_id: u64,
        request: PageRequest,
    ) -> BdlResult<NormalizedSourceTree> {
        let page = self.api.list_detail(media_id, request).await?;
        Ok(normalized_tree_from_page(page, None))
    }
}

#[async_trait]
impl<A> Resolver for FavoriteResolver<A>
where
    A: FavoriteApi,
{
    async fn resolve(
        &self,
        input: ClassifiedInput,
        _options: ResolveOptions,
    ) -> BdlResult<NormalizedSourceTree> {
        let ClassifiedInput::Favorite { raw_url } = input else {
            return Err(BdlError::UnsupportedSource {
                kind: source_kind_name(input.source_kind()).to_owned(),
            });
        };
        let media_id = favorite_media_id_from_url(&raw_url)?;
        let request =
            PageRequest::first(PagedSourceKind::Favorite, Some(FAVORITE_API_MAX_PAGE_SIZE));
        let page = self.api.list_detail(media_id, request).await?;

        Ok(normalized_tree_from_page(page, Some(raw_url)))
    }
}

pub struct BpiFavoriteApi {
    client: BpiClient,
}

impl BpiFavoriteApi {
    pub fn new() -> BdlResult<Self> {
        BpiClient::new()
            .map(Self::from_client)
            .map_err(|error| BdlError::Bpi(error.to_string()))
    }

    pub fn from_client(client: BpiClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl FavoriteApi for BpiFavoriteApi {
    async fn list_detail(
        &self,
        media_id: u64,
        request: PageRequest,
    ) -> BdlResult<ResolvedFavoritePage> {
        let media_id = MediaId::new(media_id).map_err(|error| BdlError::Bpi(error.to_string()))?;
        let params = FavListDetailParams::new(media_id)
            .order("mtime")
            .map_err(|error| BdlError::Bpi(error.to_string()))?
            .content_type(0)
            .page_size(request.page_size)
            .map_err(|error| BdlError::Bpi(error.to_string()))?
            .page(request.page_number)
            .map_err(|error| BdlError::Bpi(error.to_string()))?;
        let data = self
            .client
            .fav()
            .list_detail(params)
            .await
            .map_err(|error| BdlError::Bpi(error.to_string()))?;

        Ok(ResolvedFavoritePage {
            media_id: media_id.get(),
            title: data.info.title,
            owner_name: non_empty(data.info.upper.name),
            request,
            total_count: Some(data.info.media_count as usize),
            has_more: data.has_more,
            videos: data
                .medias
                .into_iter()
                .filter_map(ResolvedFavoriteVideo::from_media)
                .collect(),
        })
    }
}

impl ResolvedFavoriteVideo {
    fn from_media(media: FavListMedia) -> Option<Self> {
        let bvid = media.bvid.or(media.bv_id)?;
        Some(Self {
            aid: media.id,
            bvid,
            title: media.title,
            owner_mid: media.upper.mid,
            owner_name: non_empty(media.upper.name),
            cover_url: non_empty(media.cover),
            duration_seconds: Some(media.duration as u64),
        })
    }
}

pub fn favorite_media_id_from_url(raw_url: &str) -> BdlResult<u64> {
    let url = Url::parse(raw_url).map_err(|_| BdlError::InvalidInput {
        message: "无法识别收藏夹链接。".to_owned(),
    })?;

    url.query_pairs()
        .find_map(|(key, value)| {
            matches!(key.as_ref(), "fid" | "media_id")
                .then(|| value.parse::<u64>().ok())
                .flatten()
        })
        .ok_or_else(|| BdlError::InvalidInput {
            message: "收藏夹链接缺少 fid 或 media_id 参数。".to_owned(),
        })
}

fn normalized_tree_from_page(
    page: ResolvedFavoritePage,
    input_override: Option<String>,
) -> NormalizedSourceTree {
    let page_state = PageState {
        page_number: page.request.page_number,
        page_size: page.request.page_size,
        loaded_count: page.videos.len(),
        total_count: page.total_count,
        has_more: page.has_more,
    };
    let title = if page.title.trim().is_empty() {
        format!("收藏夹 {}", page.media_id)
    } else {
        page.title
    };
    let items = page
        .videos
        .into_iter()
        .map(|video| map_video_item(page.media_id, video))
        .collect::<Vec<_>>();

    NormalizedSourceTree {
        source: SourceSummary {
            id: SourceId(format!("favorite:{}", page.media_id)),
            kind: SourceKind::Favorite,
            input: input_override.unwrap_or_else(|| format!("favorite:{}", page.media_id)),
            title: title.clone(),
            loaded_count: items.len(),
            total_count: page.total_count,
            has_more: page.has_more,
        },
        groups: vec![NormalizedGroup {
            id: GroupId(format!("group:favorite:{}", page.media_id)),
            kind: "favorite".to_owned(),
            title,
            items,
            page: Some(page_state),
        }],
    }
}

fn map_video_item(media_id: u64, video: ResolvedFavoriteVideo) -> NormalizedItem {
    let video_key = format!("favorite:{media_id}:{}", video.bvid);

    NormalizedItem {
        id: ItemId(format!("item:{video_key}")),
        title: video.title.clone(),
        owner_name: video.owner_name.clone(),
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
