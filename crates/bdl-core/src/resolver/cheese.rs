use async_trait::async_trait;
use bpi_rs::cheese::{CheeseEpListParams, CheeseInfoParams, CheeseVideoStreamParams};
use bpi_rs::ids::{Aid, Cid, EpisodeId, SeasonId};
use bpi_rs::models::{DashTrack, Fnval, VideoQuality};
use bpi_rs::{BpiClient, BpiError};
use chrono::{DateTime, Utc};
use url::Url;

use super::paged::{PageRequest, PagedSourceKind};
use super::{ResolveOptions, Resolver};
use crate::error::{BdlError, BdlResult};
use crate::ids::{GroupId, ItemId, PartId, SourceId};
use crate::input::ClassifiedInput;
use crate::model::{
    AssetKind, FetchPolicy, HeaderPair, MediaKind, MediaStream, NormalizedGroup, NormalizedItem,
    NormalizedPart, NormalizedSourceTree, PageState, SourceKind, SourceSummary, StreamCodec,
    StreamQuality,
};

const CHEESE_API_MAX_PAGE_SIZE: u32 = 100;
const DEFAULT_REFERER: &str = "https://www.bilibili.com/";
const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
    AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheeseInputId {
    Season(u64),
    Episode(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCheesePage {
    pub season_id: u64,
    pub title: String,
    pub owner_name: Option<String>,
    pub cover_url: Option<String>,
    pub request: PageRequest,
    pub total_count: Option<usize>,
    pub has_more: bool,
    pub focused_ep_id: Option<u64>,
    pub episodes: Vec<ResolvedCheeseEpisode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCheeseEpisode {
    pub aid: u64,
    pub cid: u64,
    pub ep_id: u64,
    pub index: u32,
    pub title: String,
    pub duration_seconds: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedCheesePlayUrl {
    pub video: Vec<ResolvedCheeseDashStream>,
    pub audio: Vec<ResolvedCheeseDashStream>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCheeseDashStream {
    pub id: u64,
    pub base_url: String,
    pub backup_urls: Vec<String>,
    pub bandwidth: Option<u64>,
    pub codecs: String,
}

#[async_trait]
pub trait CheeseApi: Send + Sync {
    async fn page(&self, id: CheeseInputId, request: PageRequest) -> BdlResult<ResolvedCheesePage>;

    async fn play_url(&self, episode: &ResolvedCheeseEpisode) -> BdlResult<ResolvedCheesePlayUrl>;
}

#[derive(Debug)]
pub struct CheeseResolver<A = BpiCheeseApi> {
    api: A,
}

impl CheeseResolver<BpiCheeseApi> {
    pub fn new() -> BdlResult<Self> {
        Ok(Self::with_api(BpiCheeseApi::new()?))
    }

    pub fn from_cookie(cookie: &str) -> BdlResult<Self> {
        BpiClient::builder()
            .cookie(cookie)
            .build()
            .map(Self::from_bpi_client)
            .map_err(|error| BdlError::Bpi(error.to_string()))
    }

    pub fn from_bpi_client(client: BpiClient) -> Self {
        Self::with_api(BpiCheeseApi::from_client(client))
    }
}

impl<A> CheeseResolver<A> {
    pub fn with_api(api: A) -> Self {
        Self { api }
    }
}

impl<A> CheeseResolver<A>
where
    A: CheeseApi,
{
    pub async fn resolve_page(
        &self,
        season_id: u64,
        request: PageRequest,
    ) -> BdlResult<NormalizedSourceTree> {
        let page = self
            .api
            .page(CheeseInputId::Season(season_id), request)
            .await?;
        self.tree_from_page(page, None, None, ResolveOptions::default())
            .await
    }

    async fn tree_from_page(
        &self,
        page: ResolvedCheesePage,
        input_override: Option<String>,
        focused_ep_id: Option<u64>,
        options: ResolveOptions,
    ) -> BdlResult<NormalizedSourceTree> {
        let title = source_title(&page.title, || format!("课程 {}", page.season_id));
        let source_key = format!("cheese:{}", page.season_id);
        let mut items = Vec::with_capacity(page.episodes.len());

        for episode in page.episodes {
            let should_fetch_streams =
                options.fetch_streams && focused_ep_id == Some(episode.ep_id);
            let part_key = format!(
                "cheese:{}:{}:{}",
                page.season_id, episode.ep_id, episode.cid
            );
            let streams = if should_fetch_streams {
                let play_url = self.api.play_url(&episode).await?;
                map_play_url(&part_key, play_url, Utc::now())
            } else {
                Vec::new()
            };

            items.push(map_episode_item(
                &source_key,
                &part_key,
                episode,
                page.owner_name.clone(),
                page.cover_url.clone(),
                streams,
            ));
        }

        let page_state = PageState {
            page_number: page.request.page_number,
            page_size: page.request.page_size,
            loaded_count: items.len(),
            total_count: page.total_count,
            has_more: page.has_more
                && page
                    .total_count
                    .map(|total| items.len() < total)
                    .unwrap_or(true),
        };

        Ok(NormalizedSourceTree {
            source: SourceSummary {
                id: SourceId(source_key.clone()),
                kind: SourceKind::Cheese,
                input: input_override.unwrap_or_else(|| {
                    format!("https://www.bilibili.com/cheese/play/ss{}", page.season_id)
                }),
                title: title.clone(),
                loaded_count: items.len(),
                total_count: page.total_count,
                has_more: page_state.has_more,
            },
            groups: vec![NormalizedGroup {
                id: GroupId(format!("group:{source_key}")),
                kind: "cheese".to_owned(),
                title,
                items,
                page: Some(page_state),
            }],
        })
    }
}

#[async_trait]
impl<A> Resolver for CheeseResolver<A>
where
    A: CheeseApi,
{
    async fn resolve(
        &self,
        input: ClassifiedInput,
        options: ResolveOptions,
    ) -> BdlResult<NormalizedSourceTree> {
        let ClassifiedInput::Cheese { raw_url } = input else {
            return Err(BdlError::UnsupportedSource {
                kind: source_kind_name(input.source_kind()).to_owned(),
            });
        };

        let input_id = cheese_id_from_url(&raw_url)?;
        let request = PageRequest::first(
            PagedSourceKind::CheeseEpisodes,
            Some(CHEESE_API_MAX_PAGE_SIZE),
        );
        let page = self.api.page(input_id, request).await?;
        let focused_ep_id = match input_id {
            CheeseInputId::Episode(ep_id) => Some(ep_id),
            CheeseInputId::Season(_) => page.focused_ep_id,
        };

        self.tree_from_page(page, Some(raw_url), focused_ep_id, options)
            .await
    }
}

pub struct BpiCheeseApi {
    client: BpiClient,
}

impl BpiCheeseApi {
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
impl CheeseApi for BpiCheeseApi {
    async fn page(&self, id: CheeseInputId, request: PageRequest) -> BdlResult<ResolvedCheesePage> {
        let info = match id {
            CheeseInputId::Season(season_id) => {
                self.client
                    .cheese()
                    .info(CheeseInfoParams::from_season_id(
                        SeasonId::new(season_id).map_err(bpi_error)?,
                    ))
                    .await
            }
            CheeseInputId::Episode(ep_id) => {
                self.client
                    .cheese()
                    .info(CheeseInfoParams::from_episode_id(
                        EpisodeId::new(ep_id).map_err(bpi_error)?,
                    ))
                    .await
            }
        }
        .map_err(bpi_error)?;

        let focused_ep_id = match id {
            CheeseInputId::Episode(ep_id) => Some(ep_id),
            CheeseInputId::Season(_) => None,
        };

        if focused_ep_id.is_some() {
            return Ok(ResolvedCheesePage::from_info_episodes(
                info,
                request,
                focused_ep_id,
            ));
        }

        let season_id =
            SeasonId::new(info.season_id).map_err(|error| BdlError::Bpi(error.to_string()))?;
        let ep_list = self
            .client
            .cheese()
            .ep_list(
                CheeseEpListParams::new(season_id)
                    .with_page_size(request.page_size)
                    .map_err(bpi_error)?
                    .with_page(request.page_number)
                    .map_err(bpi_error)?,
            )
            .await
            .map_err(bpi_error)?;

        Ok(ResolvedCheesePage::from_info_and_list(
            info, ep_list, request,
        ))
    }

    async fn play_url(&self, episode: &ResolvedCheeseEpisode) -> BdlResult<ResolvedCheesePlayUrl> {
        let data = self
            .client
            .cheese()
            .video_stream(
                CheeseVideoStreamParams::new(
                    Aid::new(episode.aid).map_err(bpi_error)?,
                    EpisodeId::new(episode.ep_id).map_err(bpi_error)?,
                    Cid::new(episode.cid).map_err(bpi_error)?,
                )
                .with_quality(VideoQuality::P8K)
                .with_fnval(
                    Fnval::DASH
                        | Fnval::FOURK
                        | Fnval::EIGHTK
                        | Fnval::HDR
                        | Fnval::DOLBY_AUDIO
                        | Fnval::DOLBY_VISION
                        | Fnval::AV1,
                ),
            )
            .await
            .map_err(bpi_error)?;

        Ok(ResolvedCheesePlayUrl::from_dash(data.base.dash))
    }
}

impl ResolvedCheesePage {
    fn from_info_episodes(
        info: bpi_rs::cheese::info::CourseInfo,
        fallback_request: PageRequest,
        focused_ep_id: Option<u64>,
    ) -> Self {
        let request = page_request_from_info(
            info.episode_page.num,
            info.episode_page.size,
            fallback_request,
        );
        let total_count = usize::try_from(info.episode_page.total).ok();
        let has_more = info.episode_page.next;
        let episodes = info
            .episodes
            .into_iter()
            .filter_map(ResolvedCheeseEpisode::from_bpi)
            .collect();

        Self {
            season_id: info.season_id,
            title: info.title,
            owner_name: non_empty(info.up_info.uname),
            cover_url: non_empty(info.cover),
            request,
            total_count,
            has_more,
            focused_ep_id,
            episodes,
        }
    }

    fn from_info_and_list(
        info: bpi_rs::cheese::info::CourseInfo,
        list: bpi_rs::cheese::info::CourseEpList,
        fallback_request: PageRequest,
    ) -> Self {
        let request = page_request_from_info(list.page.num, list.page.size, fallback_request);
        let total_count = usize::try_from(list.page.total).ok();
        let episodes = list
            .items
            .into_iter()
            .filter_map(ResolvedCheeseEpisode::from_bpi)
            .collect();

        Self {
            season_id: info.season_id,
            title: info.title,
            owner_name: non_empty(info.up_info.uname),
            cover_url: non_empty(info.cover),
            request,
            total_count,
            has_more: list.page.next,
            focused_ep_id: None,
            episodes,
        }
    }
}

impl ResolvedCheeseEpisode {
    fn from_bpi(episode: bpi_rs::cheese::info::CourseEpisode) -> Option<Self> {
        (episode.id > 0 && episode.cid > 0).then_some(Self {
            aid: episode.aid,
            cid: episode.cid,
            ep_id: episode.id,
            index: episode.index,
            title: cheese_episode_title(episode.index, &episode.title),
            duration_seconds: (episode.duration > 0).then_some(episode.duration),
        })
    }
}

impl ResolvedCheesePlayUrl {
    fn from_dash(dash: Option<bpi_rs::models::DashStreams>) -> Self {
        let Some(dash) = dash else {
            return Self::default();
        };

        Self {
            video: dash
                .video
                .into_iter()
                .map(ResolvedCheeseDashStream::from)
                .collect(),
            audio: dash
                .audio
                .into_iter()
                .map(ResolvedCheeseDashStream::from)
                .collect(),
        }
    }
}

impl From<DashTrack> for ResolvedCheeseDashStream {
    fn from(stream: DashTrack) -> Self {
        Self {
            id: u64::from(stream.id),
            base_url: stream.base_url,
            backup_urls: stream.backup_url,
            bandwidth: Some(u64::from(stream.bandwidth)),
            codecs: stream.codecs,
        }
    }
}

pub fn cheese_id_from_url(raw_url: &str) -> BdlResult<CheeseInputId> {
    let url = Url::parse(raw_url).map_err(|_| BdlError::InvalidInput {
        message: "无法识别课程链接。".to_owned(),
    })?;
    let segments = path_segments(&url);

    query_u64(&url, "ep_id")
        .map(CheeseInputId::Episode)
        .or_else(|| query_u64(&url, "season_id").map(CheeseInputId::Season))
        .or_else(|| {
            segments
                .iter()
                .find_map(|segment| parse_prefixed_id(segment, "ep").map(CheeseInputId::Episode))
        })
        .or_else(|| {
            segments
                .iter()
                .find_map(|segment| parse_prefixed_id(segment, "ss").map(CheeseInputId::Season))
        })
        .ok_or_else(|| BdlError::InvalidInput {
            message: "课程链接缺少 ss 或 ep ID。".to_owned(),
        })
}

fn map_episode_item(
    source_key: &str,
    part_key: &str,
    episode: ResolvedCheeseEpisode,
    owner_name: Option<String>,
    cover_url: Option<String>,
    streams: Vec<MediaStream>,
) -> NormalizedItem {
    let item_key = format!("{source_key}:{}", episode.ep_id);

    NormalizedItem {
        id: ItemId(format!("item:{item_key}")),
        title: episode.title.clone(),
        owner_name,
        owner_mid: None,
        cover_url: cover_url.clone(),
        duration_seconds: episode.duration_seconds,
        parts: vec![NormalizedPart {
            id: PartId(format!("part:{part_key}")),
            title: episode.title,
            aid: Some(episode.aid),
            bvid: None,
            cid: Some(episode.cid),
            duration_seconds: episode.duration_seconds,
            streams,
            assets: vec![AssetKind::Cover.with_policy(FetchPolicy::OnDemand)],
        }],
    }
}

fn map_play_url(
    part_key: &str,
    play_url: ResolvedCheesePlayUrl,
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
    stream: ResolvedCheeseDashStream,
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

fn page_request_from_info(page_number: u32, page_size: u32, fallback: PageRequest) -> PageRequest {
    PageRequest {
        page_number: positive_u32(page_number).unwrap_or(fallback.page_number),
        page_size: positive_u32(page_size).unwrap_or(fallback.page_size),
    }
}

fn positive_u32(value: u32) -> Option<u32> {
    (value > 0).then_some(value)
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

fn path_segments(url: &Url) -> Vec<&str> {
    url.path_segments()
        .map(|segments| segments.filter(|segment| !segment.is_empty()).collect())
        .unwrap_or_default()
}

fn query_u64(url: &Url, key: &str) -> Option<u64> {
    url.query_pairs()
        .find_map(|(name, value)| (name == key).then(|| parse_positive_u64(&value)).flatten())
}

fn parse_prefixed_id(token: &str, prefix: &str) -> Option<u64> {
    token.strip_prefix(prefix).and_then(parse_positive_u64)
}

fn parse_positive_u64(value: &str) -> Option<u64> {
    value.parse::<u64>().ok().filter(|value| *value > 0)
}

fn cheese_episode_title(index: u32, title: &str) -> String {
    let title = title.trim();
    if index == 0 {
        title.to_owned()
    } else if title.is_empty() {
        format!("P{index}")
    } else {
        format!("P{index} - {title}")
    }
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

fn bpi_error(error: BpiError) -> BdlError {
    BdlError::Bpi(error.to_string())
}
