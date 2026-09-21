use async_trait::async_trait;
use bpi_rs::bangumi::{BangumiDetailParams, BangumiVideoStreamParams};
use bpi_rs::danmaku::DanmakuXmlListParams;
use bpi_rs::ids::{Aid, Bvid, Cid, EpisodeId, SeasonId};
use bpi_rs::models::{DashTrack, Fnval, VideoQuality};
use bpi_rs::video::VideoPlayerInfoParams;
use bpi_rs::{BpiClient, BpiError};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use url::Url;

use super::{ResolveOptions, Resolver, bilibili_publish_date};
use crate::error::{BdlError, BdlResult};
use crate::ids::{GroupId, ItemId, PartId, SourceId};
use crate::input::ClassifiedInput;
use crate::model::{
    AssetKind, DerivedAsset, FetchPolicy, HeaderPair, MediaKind, MediaStream, NormalizedGroup,
    NormalizedItem, NormalizedPart, NormalizedSourceTree, SourceKind, SourceSummary, StreamCodec,
    StreamQuality,
};

const DEFAULT_REFERER: &str = "https://www.bilibili.com/";
const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
    AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BangumiInputId {
    Season(u64),
    Episode(u64),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBangumiSeason {
    pub season_id: u64,
    pub title: String,
    pub owner_name: Option<String>,
    pub cover_url: Option<String>,
    pub total_count: Option<usize>,
    pub focused_ep_id: Option<u64>,
    pub episodes: Vec<ResolvedBangumiEpisode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBangumiEpisode {
    pub aid: u64,
    pub bvid: String,
    pub cid: u64,
    pub ep_id: u64,
    pub title: String,
    pub publish_date: Option<String>,
    pub cover_url: Option<String>,
    pub duration_seconds: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedBangumiPlayUrl {
    pub video: Vec<ResolvedBangumiDashStream>,
    pub audio: Vec<ResolvedBangumiDashStream>,
    pub subtitles: Vec<ResolvedBangumiSubtitle>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBangumiSubtitle {
    pub lan: String,
    pub lan_doc: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBangumiDashStream {
    pub id: u64,
    pub base_url: String,
    pub backup_urls: Vec<String>,
    pub bandwidth: Option<u64>,
    pub codecs: String,
}

#[async_trait]
pub trait BangumiApi: Send + Sync {
    async fn season(&self, id: BangumiInputId) -> BdlResult<ResolvedBangumiSeason>;

    async fn play_url(&self, episode: &ResolvedBangumiEpisode)
    -> BdlResult<ResolvedBangumiPlayUrl>;
}

#[derive(Debug)]
pub struct BangumiResolver<A = BpiBangumiApi> {
    api: A,
}

impl BangumiResolver<BpiBangumiApi> {
    pub fn new() -> BdlResult<Self> {
        Ok(Self::with_api(BpiBangumiApi::new()?))
    }

    pub fn from_cookie(cookie: &str) -> BdlResult<Self> {
        BpiClient::builder()
            .cookie(cookie)
            .build()
            .map(Self::from_bpi_client)
            .map_err(|error| BdlError::Bpi(error.to_string()))
    }

    pub fn from_bpi_client(client: impl Into<Arc<BpiClient>>) -> Self {
        Self::with_api(BpiBangumiApi::from_client(client))
    }
}

impl<A> BangumiResolver<A> {
    pub fn with_api(api: A) -> Self {
        Self { api }
    }
}

#[async_trait]
impl<A> Resolver for BangumiResolver<A>
where
    A: BangumiApi,
{
    async fn resolve(
        &self,
        input: ClassifiedInput,
        options: ResolveOptions,
    ) -> BdlResult<NormalizedSourceTree> {
        let ClassifiedInput::Bangumi { raw_url } = input else {
            return Err(BdlError::UnsupportedSource {
                kind: source_kind_name(input.source_kind()).to_owned(),
            });
        };

        let input_id = bangumi_id_from_url(&raw_url)?;
        let season = self.api.season(input_id).await?;
        let focused_ep_id = match input_id {
            BangumiInputId::Episode(ep_id) => Some(ep_id),
            BangumiInputId::Season(_) => season.focused_ep_id,
        };
        let tree = self
            .tree_from_season(season, raw_url, focused_ep_id, options)
            .await?;

        Ok(tree)
    }
}

impl<A> BangumiResolver<A>
where
    A: BangumiApi,
{
    async fn tree_from_season(
        &self,
        season: ResolvedBangumiSeason,
        input: String,
        focused_ep_id: Option<u64>,
        options: ResolveOptions,
    ) -> BdlResult<NormalizedSourceTree> {
        let title = source_title(&season.title, || format!("番剧 {}", season.season_id));
        let source_key = format!("bangumi:{}", season.season_id);
        let mut items = Vec::with_capacity(season.episodes.len());

        for episode in season.episodes {
            let should_fetch_streams =
                options.fetch_streams && focused_ep_id == Some(episode.ep_id);
            let part_key = format!(
                "bangumi:{}:{}:{}",
                season.season_id, episode.ep_id, episode.cid
            );
            let (streams, assets) = if should_fetch_streams {
                let play_url = self.api.play_url(&episode).await?;
                let assets = archive_assets(episode.cid, &play_url.subtitles);
                (map_play_url(&part_key, play_url, Utc::now()), assets)
            } else {
                (
                    Vec::new(),
                    vec![AssetKind::Cover.with_policy(FetchPolicy::OnDemand)],
                )
            };

            items.push(map_episode_item(
                &source_key,
                &part_key,
                episode,
                season.owner_name.clone(),
                streams,
                assets,
            ));
        }

        Ok(NormalizedSourceTree {
            source: SourceSummary {
                id: SourceId(source_key.clone()),
                kind: SourceKind::Bangumi,
                input,
                title: title.clone(),
                loaded_count: items.len(),
                total_count: season.total_count.or(Some(items.len())),
                has_more: false,
            },
            groups: vec![NormalizedGroup {
                id: GroupId(format!("group:{source_key}")),
                kind: "bangumi".to_owned(),
                title,
                items,
                page: None,
            }],
        })
    }
}

pub struct BpiBangumiApi {
    client: Arc<BpiClient>,
}

impl BpiBangumiApi {
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
impl BangumiApi for BpiBangumiApi {
    async fn season(&self, id: BangumiInputId) -> BdlResult<ResolvedBangumiSeason> {
        let detail = match id {
            BangumiInputId::Season(season_id) => {
                self.client
                    .bangumi()
                    .detail(BangumiDetailParams::from_season_id(
                        SeasonId::new(season_id).map_err(bpi_error)?,
                    ))
                    .await
            }
            BangumiInputId::Episode(ep_id) => {
                self.client
                    .bangumi()
                    .detail(BangumiDetailParams::from_episode_id(
                        EpisodeId::new(ep_id).map_err(bpi_error)?,
                    ))
                    .await
            }
        }
        .map_err(bpi_error)?;

        let cover_url = non_empty(detail.cover.clone());
        let owner_name = detail
            .up_info
            .as_ref()
            .and_then(|up_info| non_empty(up_info.uname.clone()));
        let focused_ep_id = match id {
            BangumiInputId::Episode(ep_id) => Some(ep_id),
            BangumiInputId::Season(_) => None,
        };
        let episodes = detail
            .episodes
            .into_iter()
            .filter_map(|episode| ResolvedBangumiEpisode::from_bpi(episode, cover_url.clone()))
            .collect();

        Ok(ResolvedBangumiSeason {
            season_id: detail.season_id,
            title: first_non_empty(&[&detail.season_title, &detail.title]),
            owner_name,
            cover_url,
            total_count: usize::try_from(detail.total).ok(),
            focused_ep_id,
            episodes,
        })
    }

    async fn play_url(
        &self,
        episode: &ResolvedBangumiEpisode,
    ) -> BdlResult<ResolvedBangumiPlayUrl> {
        let data = self
            .client
            .bangumi()
            .video_stream(
                BangumiVideoStreamParams::from_episode_id(
                    EpisodeId::new(episode.ep_id).map_err(bpi_error)?,
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

        let subtitles = match self
            .client
            .video()
            .player_info_v2(bangumi_player_info_params(episode)?)
            .await
        {
            Ok(player_info) => player_info
                .subtitle
                .map(|info| {
                    info.subtitles
                        .into_iter()
                        .filter_map(|subtitle| {
                            normalize_asset_url(&subtitle.subtitle_url).map(|url| {
                                ResolvedBangumiSubtitle {
                                    lan: subtitle.lan,
                                    lan_doc: subtitle.lan_doc,
                                    url,
                                }
                            })
                        })
                        .collect()
                })
                .unwrap_or_default(),
            Err(error) => {
                tracing::warn!(
                    ep_id = episode.ep_id,
                    "failed to load optional bangumi subtitles: {error}"
                );
                Vec::new()
            }
        };

        Ok(ResolvedBangumiPlayUrl::from_dash(data.base.dash, subtitles))
    }
}

impl ResolvedBangumiEpisode {
    fn from_bpi(
        episode: bpi_rs::bangumi::info::BangumiEpisode,
        fallback_cover_url: Option<String>,
    ) -> Option<Self> {
        let title = bangumi_episode_title(
            &episode.show_title,
            &episode.title,
            &episode.long_title,
            episode.ep_id,
        );

        (episode.ep_id > 0 && episode.cid > 0).then_some(Self {
            aid: episode.aid,
            bvid: episode.bvid,
            cid: episode.cid,
            ep_id: episode.ep_id,
            title,
            publish_date: bilibili_publish_date(episode.pub_time),
            cover_url: non_empty(episode.cover).or(fallback_cover_url),
            duration_seconds: duration_seconds(episode.duration),
        })
    }
}

impl ResolvedBangumiPlayUrl {
    fn from_dash(
        dash: Option<bpi_rs::models::DashStreams>,
        subtitles: Vec<ResolvedBangumiSubtitle>,
    ) -> Self {
        let Some(dash) = dash else {
            return Self {
                subtitles,
                ..Self::default()
            };
        };

        Self {
            video: dash
                .video
                .into_iter()
                .map(ResolvedBangumiDashStream::from)
                .collect(),
            audio: dash
                .audio
                .into_iter()
                .chain(dash.dolby.into_iter().flat_map(|dolby| dolby.audio))
                .chain(dash.flac.map(|flac| flac.audio))
                .map(ResolvedBangumiDashStream::from)
                .collect(),
            subtitles,
        }
    }
}

impl From<DashTrack> for ResolvedBangumiDashStream {
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

pub fn bangumi_id_from_url(raw_url: &str) -> BdlResult<BangumiInputId> {
    let url = Url::parse(raw_url).map_err(|_| BdlError::InvalidInput {
        message: "无法识别番剧链接。".to_owned(),
    })?;
    let segments = path_segments(&url);

    query_u64(&url, "ep_id")
        .map(BangumiInputId::Episode)
        .or_else(|| query_u64(&url, "season_id").map(BangumiInputId::Season))
        .or_else(|| {
            segments
                .iter()
                .find_map(|segment| parse_prefixed_id(segment, "ep").map(BangumiInputId::Episode))
        })
        .or_else(|| {
            segments
                .iter()
                .find_map(|segment| parse_prefixed_id(segment, "ss").map(BangumiInputId::Season))
        })
        .ok_or_else(|| BdlError::InvalidInput {
            message: "番剧链接缺少 ss 或 ep ID。".to_owned(),
        })
}

fn map_episode_item(
    source_key: &str,
    part_key: &str,
    episode: ResolvedBangumiEpisode,
    owner_name: Option<String>,
    streams: Vec<MediaStream>,
    assets: Vec<DerivedAsset>,
) -> NormalizedItem {
    let item_key = format!("{source_key}:{}", episode.ep_id);

    NormalizedItem {
        id: ItemId(format!("item:{item_key}")),
        title: episode.title.clone(),
        owner_name,
        owner_mid: None,
        publish_date: episode.publish_date,
        cover_url: episode.cover_url.clone(),
        duration_seconds: episode.duration_seconds,
        parts: vec![NormalizedPart {
            id: PartId(format!("part:{part_key}")),
            title: episode.title,
            aid: Some(episode.aid),
            bvid: non_empty(episode.bvid),
            cid: Some(episode.cid),
            duration_seconds: episode.duration_seconds,
            streams,
            assets,
        }],
    }
}

fn bangumi_player_info_params(
    episode: &ResolvedBangumiEpisode,
) -> BdlResult<VideoPlayerInfoParams> {
    let cid = Cid::new(episode.cid).map_err(bpi_error)?;
    if !episode.bvid.trim().is_empty() {
        return episode
            .bvid
            .parse::<Bvid>()
            .map(|bvid| VideoPlayerInfoParams::from_bvid(bvid, cid))
            .map_err(bpi_error);
    }

    Aid::new(episode.aid)
        .map(|aid| VideoPlayerInfoParams::from_aid(aid, cid))
        .map_err(bpi_error)
}

fn archive_assets(cid: u64, subtitles: &[ResolvedBangumiSubtitle]) -> Vec<DerivedAsset> {
    let mut assets = vec![AssetKind::Cover.with_policy(FetchPolicy::OnDemand)];
    let subtitle_urls = subtitles
        .iter()
        .filter_map(|subtitle| normalize_asset_url(&subtitle.url))
        .collect::<Vec<_>>();
    if !subtitle_urls.is_empty() {
        assets.push(DerivedAsset::with_urls(
            AssetKind::Subtitle,
            FetchPolicy::OnDemand,
            "json",
            subtitle_urls,
            default_stream_headers(),
        ));
    }
    if let Ok(cid) = Cid::new(cid) {
        assets.push(DerivedAsset::with_urls(
            AssetKind::Danmaku,
            FetchPolicy::OnDemand,
            "xml",
            vec![DanmakuXmlListParams::new(cid).comment_xml_url()],
            default_stream_headers(),
        ));
    }
    assets.push(AssetKind::Nfo.with_policy(FetchPolicy::OnDemand));
    assets
}

fn normalize_asset_url(url: &str) -> Option<String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        None
    } else if trimmed.starts_with("//") {
        Some(format!("https:{trimmed}"))
    } else {
        Some(trimmed.to_owned())
    }
}

fn map_play_url(
    part_key: &str,
    play_url: ResolvedBangumiPlayUrl,
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
    stream: ResolvedBangumiDashStream,
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
    } else if lower.contains("hev")
        || lower.contains("hvc")
        || lower.contains("h265")
        || lower.starts_with("dvhe")
        || lower.starts_with("dvh1")
    {
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

fn bangumi_episode_title(
    show_title: &str,
    short_title: &str,
    long_title: &str,
    ep_id: u64,
) -> String {
    let prefix = first_non_empty(&[show_title, short_title]);
    let suffix = long_title.trim();

    if prefix.is_empty() && suffix.is_empty() {
        format!("EP {ep_id}")
    } else if prefix.is_empty() {
        suffix.to_owned()
    } else if suffix.is_empty() || prefix == suffix {
        prefix
    } else {
        format!("{prefix} - {suffix}")
    }
}

fn duration_seconds(value: u64) -> Option<u64> {
    if value == 0 {
        None
    } else if value > 24 * 60 * 60 {
        Some(value / 1000)
    } else {
        Some(value)
    }
}

fn first_non_empty(values: &[&str]) -> String {
    values
        .iter()
        .find_map(|value| {
            let trimmed = value.trim();
            (!trimmed.is_empty()).then(|| trimmed.to_owned())
        })
        .unwrap_or_default()
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
