use async_trait::async_trait;
use bpi_rs::BpiClient;
use bpi_rs::ids::{Aid, Bvid, Cid};
use bpi_rs::video::videostream_url::DashStream;
use bpi_rs::video::{VideoPlayUrlParams, VideoViewParams};
use chrono::{DateTime, Utc};

use super::{ResolveOptions, Resolver};
use crate::error::{BdlError, BdlResult};
use crate::ids::{GroupId, ItemId, PartId, SourceId};
use crate::input::ClassifiedInput;
use crate::model::{
    AssetKind, FetchPolicy, HeaderPair, MediaKind, MediaStream, NormalizedGroup, NormalizedItem,
    NormalizedPart, NormalizedSourceTree, SourceKind, SourceSummary, StreamCodec, StreamQuality,
};

const DEFAULT_REFERER: &str = "https://www.bilibili.com/";
const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
    AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoInputId {
    Aid(u64),
    Bvid(String),
}

impl VideoInputId {
    fn from_classified(input: ClassifiedInput) -> BdlResult<Self> {
        match input {
            ClassifiedInput::VideoAid(aid) => Ok(Self::Aid(aid)),
            ClassifiedInput::VideoBvid(bvid) => Ok(Self::Bvid(bvid)),
            other => Err(BdlError::UnsupportedSource {
                kind: source_kind_name(other.source_kind()).to_owned(),
            }),
        }
    }

    fn input_label(&self) -> String {
        match self {
            Self::Aid(aid) => format!("av{aid}"),
            Self::Bvid(bvid) => bvid.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedVideo {
    pub aid: u64,
    pub bvid: String,
    pub default_cid: u64,
    pub title: String,
    pub owner_name: Option<String>,
    pub cover_url: Option<String>,
    pub pages: Vec<ResolvedVideoPage>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedVideoPage {
    pub cid: u64,
    pub index: u32,
    pub title: String,
    pub duration_seconds: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedPlayUrl {
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
pub trait VideoApi: Send + Sync {
    async fn view(&self, id: &VideoInputId) -> BdlResult<ResolvedVideo>;

    async fn play_url(&self, id: &VideoInputId, cid: u64) -> BdlResult<ResolvedPlayUrl>;
}

#[derive(Debug)]
pub struct VideoResolver<A = BpiVideoApi> {
    api: A,
}

impl VideoResolver<BpiVideoApi> {
    pub fn new() -> BdlResult<Self> {
        Ok(Self::with_api(BpiVideoApi::new()?))
    }

    pub fn from_cookie(cookie: &str) -> BdlResult<Self> {
        BpiClient::builder()
            .cookie(cookie)
            .build()
            .map(Self::from_bpi_client)
            .map_err(|error| BdlError::Bpi(error.to_string()))
    }

    pub fn from_bpi_client(client: BpiClient) -> Self {
        Self::with_api(BpiVideoApi::from_client(client))
    }
}

impl<A> VideoResolver<A> {
    pub fn with_api(api: A) -> Self {
        Self { api }
    }
}

#[async_trait]
impl<A> Resolver for VideoResolver<A>
where
    A: VideoApi,
{
    async fn resolve(
        &self,
        input: ClassifiedInput,
        options: ResolveOptions,
    ) -> BdlResult<NormalizedSourceTree> {
        let input_id = VideoInputId::from_classified(input)?;
        let input_label = input_id.input_label();
        let video = self.api.view(&input_id).await?;
        let canonical_key = canonical_video_key(&video);
        let pages = normalized_pages(&video);

        let mut parts = Vec::with_capacity(pages.len());
        for page in pages {
            let part_key = format!("{canonical_key}:{}", page.cid);
            let streams = if options.fetch_streams {
                let play_url = self.api.play_url(&input_id, page.cid).await?;
                map_play_url(&part_key, play_url, Utc::now())
            } else {
                Vec::new()
            };

            parts.push(NormalizedPart {
                id: PartId(format!("part:{part_key}")),
                title: page_title(&video.title, &page),
                aid: Some(video.aid),
                bvid: Some(video.bvid.clone()),
                cid: Some(page.cid),
                streams,
                assets: vec![AssetKind::Cover.with_policy(FetchPolicy::OnDemand)],
            });
        }

        let item = NormalizedItem {
            id: ItemId(format!("item:{canonical_key}")),
            title: video.title.clone(),
            owner_name: video.owner_name.clone(),
            cover_url: video.cover_url.clone(),
            duration_seconds: total_duration_seconds(&video.pages),
            parts,
        };

        Ok(NormalizedSourceTree {
            source: SourceSummary {
                id: SourceId(format!("video:{input_label}")),
                kind: SourceKind::Video,
                input: input_label,
                title: video.title.clone(),
                loaded_count: 1,
                total_count: Some(1),
                has_more: false,
            },
            groups: vec![NormalizedGroup {
                id: GroupId(format!("group:{canonical_key}")),
                kind: "video".to_owned(),
                title: video.title,
                items: vec![item],
                page: None,
            }],
        })
    }
}

pub struct BpiVideoApi {
    client: BpiClient,
}

impl BpiVideoApi {
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
impl VideoApi for BpiVideoApi {
    async fn view(&self, id: &VideoInputId) -> BdlResult<ResolvedVideo> {
        let view = self
            .client
            .video()
            .view(view_params(id)?)
            .await
            .map_err(|error| BdlError::Bpi(error.to_string()))?;

        Ok(ResolvedVideo {
            aid: view.aid.get(),
            bvid: view.bvid.as_str().to_owned(),
            default_cid: view.cid.get(),
            title: view.title,
            owner_name: non_empty(view.owner.name),
            cover_url: None,
            pages: view
                .pages
                .into_iter()
                .map(|page| ResolvedVideoPage {
                    cid: page.cid.get(),
                    index: page.page,
                    title: page.part,
                    duration_seconds: Some(page.duration),
                })
                .collect(),
        })
    }

    async fn play_url(&self, id: &VideoInputId, cid: u64) -> BdlResult<ResolvedPlayUrl> {
        let data = self
            .client
            .video()
            .play_url(play_url_params(id, cid)?)
            .await
            .map_err(|error| BdlError::Bpi(error.to_string()))?;

        let Some(dash) = data.dash else {
            return Ok(ResolvedPlayUrl::default());
        };

        Ok(ResolvedPlayUrl {
            video: dash
                .video
                .into_iter()
                .map(ResolvedDashStream::from)
                .collect(),
            audio: dash
                .audio
                .into_iter()
                .map(ResolvedDashStream::from)
                .collect(),
        })
    }
}

impl From<DashStream> for ResolvedDashStream {
    fn from(stream: DashStream) -> Self {
        Self {
            id: stream.id,
            base_url: stream.base_url,
            backup_urls: stream.backup_url,
            bandwidth: Some(stream.bandwidth),
            codecs: stream.codecs,
        }
    }
}

fn view_params(id: &VideoInputId) -> BdlResult<VideoViewParams> {
    match id {
        VideoInputId::Aid(aid) => Aid::new(*aid)
            .map(VideoViewParams::from_aid)
            .map_err(|error| BdlError::Bpi(error.to_string())),
        VideoInputId::Bvid(bvid) => bvid
            .parse::<Bvid>()
            .map(VideoViewParams::from_bvid)
            .map_err(|error| BdlError::Bpi(error.to_string())),
    }
}

fn play_url_params(id: &VideoInputId, cid: u64) -> BdlResult<VideoPlayUrlParams> {
    let cid = Cid::new(cid).map_err(|error| BdlError::Bpi(error.to_string()))?;
    let params = match id {
        VideoInputId::Aid(aid) => Aid::new(*aid)
            .map(|aid| VideoPlayUrlParams::from_aid(aid, cid))
            .map_err(|error| BdlError::Bpi(error.to_string()))?,
        VideoInputId::Bvid(bvid) => bvid
            .parse::<Bvid>()
            .map(|bvid| VideoPlayUrlParams::from_bvid(bvid, cid))
            .map_err(|error| BdlError::Bpi(error.to_string()))?,
    };

    Ok(params
        .format_flags(16)
        .format_version(0)
        .fourk(true)
        .high_quality(true))
}

fn normalized_pages(video: &ResolvedVideo) -> Vec<ResolvedVideoPage> {
    if video.pages.is_empty() {
        return vec![ResolvedVideoPage {
            cid: video.default_cid,
            index: 1,
            title: video.title.clone(),
            duration_seconds: None,
        }];
    }

    video.pages.clone()
}

fn canonical_video_key(video: &ResolvedVideo) -> String {
    if video.bvid.is_empty() {
        format!("av{}", video.aid)
    } else {
        video.bvid.clone()
    }
}

fn page_title(video_title: &str, page: &ResolvedVideoPage) -> String {
    if page.title.trim().is_empty() {
        return video_title.to_owned();
    }

    page.title.clone()
}

fn total_duration_seconds(pages: &[ResolvedVideoPage]) -> Option<u64> {
    pages
        .iter()
        .map(|page| page.duration_seconds)
        .try_fold(0_u64, |sum, duration| {
            duration.map(|duration| sum + duration)
        })
}

fn map_play_url(
    part_key: &str,
    play_url: ResolvedPlayUrl,
    acquired_at: DateTime<Utc>,
) -> Vec<MediaStream> {
    play_url
        .video
        .into_iter()
        .enumerate()
        .map(|(index, stream)| map_stream(part_key, MediaKind::Video, index, stream, acquired_at))
        .chain(
            play_url
                .audio
                .into_iter()
                .enumerate()
                .map(|(index, stream)| {
                    map_stream(part_key, MediaKind::Audio, index, stream, acquired_at)
                }),
        )
        .collect()
}

fn map_stream(
    part_key: &str,
    kind: MediaKind,
    index: usize,
    stream: ResolvedDashStream,
    acquired_at: DateTime<Utc>,
) -> MediaStream {
    let mut urls = Vec::with_capacity(1 + stream.backup_urls.len());
    push_unique_url(&mut urls, stream.base_url);
    for url in stream.backup_urls {
        push_unique_url(&mut urls, url);
    }

    MediaStream {
        id: format!(
            "stream:{part_key}:{}:{index}:{}",
            media_kind_label(kind),
            stream.id
        ),
        kind,
        quality: u32::try_from(stream.id)
            .map(StreamQuality::Quality)
            .unwrap_or(StreamQuality::Best),
        codec: stream_codec(&stream.codecs),
        bandwidth: stream.bandwidth,
        urls,
        headers: default_stream_headers(),
        acquired_at,
    }
}

fn push_unique_url(urls: &mut Vec<String>, url: String) {
    if !url.trim().is_empty() && !urls.iter().any(|existing| existing == &url) {
        urls.push(url);
    }
}

fn default_stream_headers() -> Vec<HeaderPair> {
    vec![
        HeaderPair {
            name: "Referer".to_owned(),
            value: DEFAULT_REFERER.to_owned(),
        },
        HeaderPair {
            name: "User-Agent".to_owned(),
            value: DEFAULT_USER_AGENT.to_owned(),
        },
    ]
}

fn stream_codec(codecs: &str) -> StreamCodec {
    let lower = codecs.to_ascii_lowercase();
    if lower.trim().is_empty() {
        StreamCodec::Auto
    } else if lower.contains("av01") || lower.contains("av1") {
        StreamCodec::Av1
    } else if lower.contains("hev") || lower.contains("hvc") || lower.contains("h265") {
        StreamCodec::Hevc
    } else if lower.contains("avc") || lower.contains("h264") {
        StreamCodec::Avc
    } else {
        StreamCodec::Unknown
    }
}

fn media_kind_label(kind: MediaKind) -> &'static str {
    match kind {
        MediaKind::Video => "video",
        MediaKind::Audio => "audio",
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
