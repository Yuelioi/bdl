use async_trait::async_trait;
use bpi_rs::ids::{Aid, Bvid, Cid};
use bpi_rs::video::videostream_url::{
    DashStream as BpiDashStream, PlayUrlResponseData as BpiPlayUrlResponseData,
};
use bpi_rs::video::{VideoPlayUrlParams, VideoView as BpiVideoView, VideoViewParams};
use bpi_rs::{BpiClient, BpiError};
use chrono::{DateTime, Utc};

use super::{ResolveOptions, Resolver};
use crate::error::{BdlError, BdlResult};
use crate::ids::{GroupId, ItemId, PartId, SourceId};
use crate::input::ClassifiedInput;
use crate::model::{
    AssetKind, FetchPolicy, HeaderPair, MediaKind, MediaStream, NormalizedGroup, NormalizedItem,
    NormalizedPart, NormalizedSourceTree, SourceKind, SourceSummary, StreamCodec, StreamQuality,
};

const DEFAULT_STREAM_REFERER: &str = "https://www.bilibili.com/";
const DEFAULT_STREAM_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
    AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoLookup {
    Bvid(String),
    Aid(u64),
}

impl VideoLookup {
    fn from_classified(input: ClassifiedInput) -> BdlResult<Self> {
        match input {
            ClassifiedInput::VideoBvid(bvid) => Ok(Self::Bvid(bvid)),
            ClassifiedInput::VideoAid(aid) => Ok(Self::Aid(aid)),
            other => Err(BdlError::UnsupportedSource {
                kind: source_kind_name(other.source_kind()).to_owned(),
            }),
        }
    }

    fn input_identifier(&self) -> String {
        match self {
            Self::Bvid(bvid) => bvid.clone(),
            Self::Aid(aid) => format!("av{aid}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedVideoView {
    pub aid: u64,
    pub bvid: String,
    pub title: String,
    pub owner_name: Option<String>,
    pub cover_url: Option<String>,
    pub duration_seconds: Option<u64>,
    pub pages: Vec<ResolvedVideoPage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedVideoPage {
    pub cid: u64,
    pub index: u32,
    pub title: String,
    pub duration_seconds: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedVideoPlayUrl {
    pub acquired_at: DateTime<Utc>,
    pub headers: Vec<HeaderPair>,
    pub video: Vec<ResolvedDashStream>,
    pub audio: Vec<ResolvedDashStream>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedDashStream {
    pub id: u64,
    pub base_url: String,
    pub backup_urls: Vec<String>,
    pub bandwidth: Option<u64>,
    pub codecs: String,
}

#[async_trait]
pub trait VideoResolverAdapter: Send + Sync {
    async fn view(&self, lookup: &VideoLookup) -> BdlResult<ResolvedVideoView>;

    async fn play_url(&self, lookup: &VideoLookup, cid: u64) -> BdlResult<ResolvedVideoPlayUrl>;
}

pub struct VideoResolver<A = BpiVideoAdapter> {
    adapter: A,
}

impl VideoResolver<BpiVideoAdapter> {
    pub fn new() -> BdlResult<Self> {
        Ok(Self {
            adapter: BpiVideoAdapter::new()?,
        })
    }

    pub fn from_bpi_client(client: BpiClient) -> Self {
        Self {
            adapter: BpiVideoAdapter::from_client(client),
        }
    }
}

impl<A> VideoResolver<A> {
    pub fn with_adapter(adapter: A) -> Self {
        Self { adapter }
    }
}

#[async_trait]
impl<A> Resolver for VideoResolver<A>
where
    A: VideoResolverAdapter,
{
    async fn resolve(
        &self,
        input: ClassifiedInput,
        options: ResolveOptions,
    ) -> BdlResult<NormalizedSourceTree> {
        let lookup = VideoLookup::from_classified(input)?;
        let input_identifier = lookup.input_identifier();
        let tree_key = input_identifier.clone();
        let source_id = SourceId(format!("video:{input_identifier}"));
        let view = self.adapter.view(&lookup).await?;

        self.normalize_video(source_id, input_identifier, tree_key, view, &lookup, options)
            .await
    }
}

impl<A> VideoResolver<A>
where
    A: VideoResolverAdapter,
{
    async fn normalize_video(
        &self,
        source_id: SourceId,
        input_identifier: String,
        tree_key: String,
        view: ResolvedVideoView,
        lookup: &VideoLookup,
        options: ResolveOptions,
    ) -> BdlResult<NormalizedSourceTree> {
        let parts = self.normalize_parts(&tree_key, &view, lookup, options).await?;
        let item = NormalizedItem {
            id: ItemId(format!("item:video:{tree_key}")),
            title: view.title.clone(),
            owner_name: view.owner_name.clone(),
            cover_url: view.cover_url.clone(),
            duration_seconds: view.duration_seconds,
            parts,
        };

        Ok(NormalizedSourceTree {
            source: SourceSummary {
                id: source_id,
                kind: SourceKind::Video,
                input: input_identifier,
                title: view.title.clone(),
                loaded_count: 1,
                total_count: Some(1),
                has_more: false,
            },
            groups: vec![NormalizedGroup {
                id: GroupId(format!("group:video:{tree_key}")),
                kind: "video".into(),
                title: view.title,
                items: vec![item],
                page: None,
            }],
        })
    }

    async fn normalize_parts(
        &self,
        tree_key: &str,
        view: &ResolvedVideoView,
        lookup: &VideoLookup,
        options: ResolveOptions,
    ) -> BdlResult<Vec<NormalizedPart>> {
        let pages = normalized_pages(view);
        let mut parts = Vec::with_capacity(pages.len());

        for page in &pages {
            let streams = if options.fetch_streams {
                let play_url = self.adapter.play_url(lookup, page.cid).await?;
                media_streams_from_play_url(page.cid, &play_url)
            } else {
                Vec::new()
            };

            parts.push(NormalizedPart {
                id: PartId(format!("part:video:{tree_key}:{}", page.cid)),
                title: part_title(&page.title, &view.title),
                aid: Some(view.aid),
                bvid: Some(view.bvid.clone()),
                cid: Some(page.cid),
                streams,
                assets: vec![AssetKind::Cover.with_policy(FetchPolicy::OnDemand)],
            });
        }

        Ok(parts)
    }
}

pub struct BpiVideoAdapter {
    client: BpiClient,
}

impl BpiVideoAdapter {
    pub fn new() -> BdlResult<Self> {
        Ok(Self {
            client: BpiClient::new().map_err(map_bpi_error)?,
        })
    }

    pub fn from_client(client: BpiClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl VideoResolverAdapter for BpiVideoAdapter {
    async fn view(&self, lookup: &VideoLookup) -> BdlResult<ResolvedVideoView> {
        let params = match lookup {
            VideoLookup::Bvid(bvid) => VideoViewParams::from_bvid(parse_bvid(bvid)?),
            VideoLookup::Aid(aid) => VideoViewParams::from_aid(parse_aid(*aid)?),
        };

        let view = self
            .client
            .video()
            .view(params)
            .await
            .map_err(map_bpi_error)?;

        Ok(resolved_view_from_bpi(view))
    }

    async fn play_url(&self, lookup: &VideoLookup, cid: u64) -> BdlResult<ResolvedVideoPlayUrl> {
        let cid = parse_cid(cid)?;
        let params = match lookup {
            VideoLookup::Bvid(bvid) => VideoPlayUrlParams::from_bvid(parse_bvid(bvid)?, cid),
            VideoLookup::Aid(aid) => VideoPlayUrlParams::from_aid(parse_aid(*aid)?, cid),
        }
        .format_flags(16)
        .format_version(0)
        .fourk(true)
        .high_quality(true);

        let play_url = self
            .client
            .video()
            .play_url(params)
            .await
            .map_err(map_bpi_error)?;

        Ok(resolved_play_url_from_bpi(play_url))
    }
}

fn resolved_view_from_bpi(view: BpiVideoView) -> ResolvedVideoView {
    let pages: Vec<_> = view
        .pages
        .into_iter()
        .map(|page| ResolvedVideoPage {
            cid: page.cid.get(),
            index: page.page,
            title: page.part,
            duration_seconds: Some(page.duration),
        })
        .collect();

    let duration_seconds = sum_page_durations(&pages);

    ResolvedVideoView {
        aid: view.aid.get(),
        bvid: view.bvid.as_str().to_owned(),
        title: view.title,
        owner_name: non_empty_string(view.owner.name),
        cover_url: None,
        duration_seconds,
        pages: if pages.is_empty() {
            vec![ResolvedVideoPage {
                cid: view.cid.get(),
                index: 1,
                title: String::new(),
                duration_seconds: None,
            }]
        } else {
            pages
        },
    }
}

fn resolved_play_url_from_bpi(play_url: BpiPlayUrlResponseData) -> ResolvedVideoPlayUrl {
    let (video, audio) = match play_url.dash {
        Some(dash) => (
            dash.video.into_iter().map(resolved_dash_stream).collect(),
            dash.audio.into_iter().map(resolved_dash_stream).collect(),
        ),
        None => (Vec::new(), Vec::new()),
    };

    ResolvedVideoPlayUrl {
        acquired_at: Utc::now(),
        headers: default_stream_headers(),
        video,
        audio,
    }
}

fn resolved_dash_stream(stream: BpiDashStream) -> ResolvedDashStream {
    ResolvedDashStream {
        id: stream.id,
        base_url: stream.base_url,
        backup_urls: stream.backup_url,
        bandwidth: Some(stream.bandwidth),
        codecs: stream.codecs,
    }
}

fn normalized_pages(view: &ResolvedVideoView) -> Vec<ResolvedVideoPage> {
    view.pages.clone()
}

fn media_streams_from_play_url(cid: u64, play_url: &ResolvedVideoPlayUrl) -> Vec<MediaStream> {
    let mut streams = Vec::with_capacity(play_url.video.len() + play_url.audio.len());

    streams.extend(play_url.video.iter().enumerate().map(|(index, stream)| {
        media_stream_from_dash(cid, MediaKind::Video, "video", index, stream, play_url)
    }));
    streams.extend(play_url.audio.iter().enumerate().map(|(index, stream)| {
        media_stream_from_dash(cid, MediaKind::Audio, "audio", index, stream, play_url)
    }));

    streams
}

fn media_stream_from_dash(
    cid: u64,
    kind: MediaKind,
    kind_label: &str,
    index: usize,
    stream: &ResolvedDashStream,
    play_url: &ResolvedVideoPlayUrl,
) -> MediaStream {
    MediaStream {
        id: format!("stream:{cid}:{kind_label}:{}:{index}", stream.id),
        kind,
        quality: stream_quality(stream.id),
        codec: stream_codec(kind, &stream.codecs),
        bandwidth: stream.bandwidth,
        urls: stream_urls(stream),
        headers: play_url.headers.clone(),
        acquired_at: play_url.acquired_at,
    }
}

fn stream_quality(id: u64) -> StreamQuality {
    u32::try_from(id)
        .map(StreamQuality::Quality)
        .unwrap_or(StreamQuality::Best)
}

fn stream_codec(kind: MediaKind, codecs: &str) -> StreamCodec {
    if kind == MediaKind::Audio {
        return StreamCodec::Auto;
    }

    let normalized = codecs.to_ascii_lowercase();
    if normalized.starts_with("avc1") || normalized.contains("h264") {
        StreamCodec::Avc
    } else if normalized.starts_with("hev1")
        || normalized.starts_with("hvc1")
        || normalized.contains("h265")
    {
        StreamCodec::Hevc
    } else if normalized.starts_with("av01") {
        StreamCodec::Av1
    } else {
        StreamCodec::Unknown
    }
}

fn stream_urls(stream: &ResolvedDashStream) -> Vec<String> {
    std::iter::once(&stream.base_url)
        .chain(stream.backup_urls.iter())
        .filter(|url| !url.is_empty())
        .cloned()
        .collect()
}

fn part_title(page_title: &str, video_title: &str) -> String {
    if page_title.trim().is_empty() {
        video_title.to_owned()
    } else {
        page_title.to_owned()
    }
}

fn sum_page_durations(pages: &[ResolvedVideoPage]) -> Option<u64> {
    pages
        .iter()
        .map(|page| page.duration_seconds)
        .try_fold(0_u64, |total, duration| {
            duration.map(|duration| total + duration)
        })
}

fn non_empty_string(value: String) -> Option<String> {
    (!value.trim().is_empty()).then_some(value)
}

fn default_stream_headers() -> Vec<HeaderPair> {
    vec![
        HeaderPair {
            name: "Referer".into(),
            value: DEFAULT_STREAM_REFERER.into(),
        },
        HeaderPair {
            name: "User-Agent".into(),
            value: DEFAULT_STREAM_USER_AGENT.into(),
        },
    ]
}

fn parse_bvid(value: &str) -> BdlResult<Bvid> {
    value.parse().map_err(map_bpi_error)
}

fn parse_aid(value: u64) -> BdlResult<Aid> {
    Aid::new(value).map_err(map_bpi_error)
}

fn parse_cid(value: u64) -> BdlResult<Cid> {
    Cid::new(value).map_err(map_bpi_error)
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

fn map_bpi_error(error: BpiError) -> BdlError {
    BdlError::Bpi(error.to_string())
}
