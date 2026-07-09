use async_trait::async_trait;
use bpi_rs::BpiClient;
use bpi_rs::ids::Mid;
use bpi_rs::user::{UserUploadedVideo, UserUploadedVideosParams};

use super::paged::{PageRequest, PagedSourceKind, page_state};
use super::{ResolveOptions, Resolver};
use crate::error::{BdlError, BdlResult};
use crate::ids::{GroupId, ItemId, PartId, SourceId};
use crate::input::ClassifiedInput;
use crate::model::{
    AssetKind, FetchPolicy, NormalizedGroup, NormalizedItem, NormalizedPart, NormalizedSourceTree,
    SourceKind, SourceSummary,
};

const UPLOADER_API_MAX_PAGE_SIZE: u32 = 30;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedUploaderVideo {
    pub aid: u64,
    pub bvid: String,
    pub title: String,
    pub owner_mid: u64,
    pub owner_name: Option<String>,
    pub cover_url: Option<String>,
    pub duration_seconds: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedUploaderPage {
    pub mid: u64,
    pub owner_name: Option<String>,
    pub request: PageRequest,
    pub total_count: Option<usize>,
    pub videos: Vec<ResolvedUploaderVideo>,
}

#[async_trait]
pub trait UploaderApi: Send + Sync {
    async fn uploaded_videos(
        &self,
        mid: u64,
        request: PageRequest,
    ) -> BdlResult<ResolvedUploaderPage>;
}

#[derive(Debug)]
pub struct UploaderResolver<A = BpiUploaderApi> {
    api: A,
}

impl UploaderResolver<BpiUploaderApi> {
    pub fn new() -> BdlResult<Self> {
        Ok(Self::with_api(BpiUploaderApi::new()?))
    }

    pub fn from_cookie(cookie: &str) -> BdlResult<Self> {
        BpiClient::builder()
            .cookie(cookie)
            .build()
            .map(Self::from_bpi_client)
            .map_err(|error| BdlError::Bpi(error.to_string()))
    }

    pub fn from_bpi_client(client: BpiClient) -> Self {
        Self::with_api(BpiUploaderApi::from_client(client))
    }
}

impl<A> UploaderResolver<A> {
    pub fn with_api(api: A) -> Self {
        Self { api }
    }
}

#[async_trait]
impl<A> Resolver for UploaderResolver<A>
where
    A: UploaderApi,
{
    async fn resolve(
        &self,
        input: ClassifiedInput,
        _options: ResolveOptions,
    ) -> BdlResult<NormalizedSourceTree> {
        let ClassifiedInput::Uploader { mid } = input else {
            return Err(BdlError::UnsupportedSource {
                kind: source_kind_name(input.source_kind()).to_owned(),
            });
        };

        let request = PageRequest::first(
            PagedSourceKind::UploaderVideos,
            Some(UPLOADER_API_MAX_PAGE_SIZE),
        );
        let page = self.api.uploaded_videos(mid, request).await?;
        let page_state = page_state(page.request, page.videos.len(), page.total_count);
        let title = uploader_title(page.mid, page.owner_name.as_deref());

        let items = page
            .videos
            .into_iter()
            .map(|video| map_video_item(page.mid, video))
            .collect::<Vec<_>>();

        Ok(NormalizedSourceTree {
            source: SourceSummary {
                id: SourceId(format!("uploader:{}:videos", page.mid)),
                kind: SourceKind::Uploader,
                input: format!("https://space.bilibili.com/{}/video", page.mid),
                title: title.clone(),
                loaded_count: items.len(),
                total_count: page.total_count,
                has_more: page_state.has_more,
            },
            groups: vec![NormalizedGroup {
                id: GroupId(format!("group:uploader:{}:videos", page.mid)),
                kind: "uploader_videos".to_owned(),
                title,
                items,
                page: Some(page_state),
            }],
        })
    }
}

pub struct BpiUploaderApi {
    client: BpiClient,
}

impl BpiUploaderApi {
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
impl UploaderApi for BpiUploaderApi {
    async fn uploaded_videos(
        &self,
        mid: u64,
        request: PageRequest,
    ) -> BdlResult<ResolvedUploaderPage> {
        let mid = Mid::new(mid).map_err(|error| BdlError::Bpi(error.to_string()))?;
        let params = UserUploadedVideosParams::new(mid)
            .with_page(request.page_number)
            .map_err(|error| BdlError::Bpi(error.to_string()))?
            .with_page_size(request.page_size)
            .map_err(|error| BdlError::Bpi(error.to_string()))?;
        let data = self
            .client
            .user()
            .uploaded_videos(params)
            .await
            .map_err(|error| BdlError::Bpi(error.to_string()))?;

        let videos = data
            .list
            .videos
            .into_iter()
            .map(ResolvedUploaderVideo::from)
            .collect::<Vec<_>>();
        let owner_name = videos.iter().find_map(|video| video.owner_name.clone());

        Ok(ResolvedUploaderPage {
            mid: mid.get(),
            owner_name,
            request: PageRequest {
                page_number: data.page.pn.max(1),
                page_size: data.page.ps.max(1),
            },
            total_count: usize::try_from(data.page.count).ok(),
            videos,
        })
    }
}

impl From<UserUploadedVideo> for ResolvedUploaderVideo {
    fn from(video: UserUploadedVideo) -> Self {
        Self {
            aid: video.aid.get(),
            bvid: video.bvid.as_str().to_owned(),
            title: video.title,
            owner_mid: video.mid.get(),
            owner_name: non_empty(video.author),
            cover_url: non_empty(video.pic),
            duration_seconds: duration_seconds_from_label(&video.length),
        }
    }
}

fn map_video_item(source_mid: u64, video: ResolvedUploaderVideo) -> NormalizedItem {
    let video_key = format!("uploader:{source_mid}:{}", video.bvid);

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
            streams: Vec::new(),
            assets: vec![AssetKind::Cover.with_policy(FetchPolicy::OnDemand)],
        }],
    }
}

fn uploader_title(mid: u64, owner_name: Option<&str>) -> String {
    match owner_name {
        Some(owner_name) if !owner_name.trim().is_empty() => format!("{owner_name} 的投稿"),
        _ => format!("UP {mid} 的投稿"),
    }
}

fn duration_seconds_from_label(label: &str) -> Option<u64> {
    let parts = label
        .split(':')
        .map(str::trim)
        .map(str::parse::<u64>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;

    match parts.as_slice() {
        [minutes, seconds] => Some(minutes.saturating_mul(60).saturating_add(*seconds)),
        [hours, minutes, seconds] => Some(
            hours
                .saturating_mul(3600)
                .saturating_add(minutes.saturating_mul(60))
                .saturating_add(*seconds),
        ),
        _ => None,
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

#[cfg(test)]
mod tests {
    use super::duration_seconds_from_label;

    #[test]
    fn duration_seconds_from_label_parses_minutes_and_seconds() {
        assert_eq!(duration_seconds_from_label("12:34"), Some(754));
    }

    #[test]
    fn duration_seconds_from_label_parses_hours_minutes_and_seconds() {
        assert_eq!(duration_seconds_from_label("1:02:03"), Some(3723));
    }

    #[test]
    fn duration_seconds_from_label_ignores_unrecognized_values() {
        assert_eq!(duration_seconds_from_label(""), None);
        assert_eq!(duration_seconds_from_label("live"), None);
    }
}
