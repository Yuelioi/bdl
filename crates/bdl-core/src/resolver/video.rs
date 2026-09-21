use async_trait::async_trait;
use bpi_rs::BpiClient;
use bpi_rs::danmaku::DanmakuXmlListParams;
use bpi_rs::ids::{Aid, Bvid, Cid};
use bpi_rs::models::{Fnval, VideoQuality};
use bpi_rs::video::videostream_url::DashStream;
use bpi_rs::video::{VideoPlayUrlParams, VideoPlayerInfoParams};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
const VIDEO_VIEW_API: &str = "https://api.bilibili.com/x/web-interface/view";
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
    pub owner_mid: Option<u64>,
    pub publish_date: Option<String>,
    pub cover_url: Option<String>,
    pub pages: Vec<ResolvedVideoPage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoCollectionRef {
    pub mid: u64,
    pub season_id: u64,
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedPlayerInfo {
    pub subtitles: Vec<ResolvedSubtitle>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSubtitle {
    pub lan: String,
    pub lan_doc: String,
    pub url: String,
}

#[async_trait]
pub trait VideoApi: Send + Sync {
    async fn view(&self, id: &VideoInputId) -> BdlResult<ResolvedVideo>;

    async fn play_url(&self, id: &VideoInputId, cid: u64) -> BdlResult<ResolvedPlayUrl>;

    async fn collection_hint(&self, _id: &VideoInputId) -> BdlResult<Option<VideoCollectionRef>> {
        Ok(None)
    }

    async fn player_info(&self, _id: &VideoInputId, _cid: u64) -> BdlResult<ResolvedPlayerInfo> {
        Ok(ResolvedPlayerInfo::default())
    }
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

    pub fn from_bpi_client(client: impl Into<Arc<BpiClient>>) -> Self {
        Self::with_api(BpiVideoApi::from_client(client))
    }
}

impl<A> VideoResolver<A> {
    pub fn with_api(api: A) -> Self {
        Self { api }
    }
}

impl<A> VideoResolver<A>
where
    A: VideoApi,
{
    pub async fn collection_hint(
        &self,
        input: &ClassifiedInput,
    ) -> BdlResult<Option<VideoCollectionRef>> {
        let input_id = VideoInputId::from_classified(input.clone())?;
        self.api.collection_hint(&input_id).await
    }

    pub async fn resolve_target_streams(
        &self,
        input: ClassifiedInput,
        target_cid: u64,
    ) -> BdlResult<NormalizedSourceTree> {
        self.resolve_with_stream_target(input, true, Some(target_cid))
            .await
    }

    async fn resolve_with_stream_target(
        &self,
        input: ClassifiedInput,
        fetch_streams: bool,
        target_cid: Option<u64>,
    ) -> BdlResult<NormalizedSourceTree> {
        let input_id = VideoInputId::from_classified(input)?;
        let input_label = input_id.input_label();
        let video = self.api.view(&input_id).await?;
        let canonical_key = canonical_video_key(&video);
        let pages = normalized_pages(&video);

        let mut parts = Vec::with_capacity(pages.len());
        for page in pages {
            let part_key = format!("{canonical_key}:{}", page.cid);
            let should_fetch_streams =
                fetch_streams && target_cid.is_none_or(|cid| cid == page.cid);
            let (streams, player_info) = if should_fetch_streams {
                let play_url = self.api.play_url(&input_id, page.cid).await?;
                let player_info = self.api.player_info(&input_id, page.cid).await?;
                (map_play_url(&part_key, play_url, Utc::now()), player_info)
            } else {
                (Vec::new(), ResolvedPlayerInfo::default())
            };

            parts.push(NormalizedPart {
                id: PartId(format!("part:{part_key}")),
                title: page_title(&video.title, &page),
                aid: Some(video.aid),
                bvid: Some(video.bvid.clone()),
                cid: Some(page.cid),
                duration_seconds: page.duration_seconds,
                streams,
                assets: archive_assets(page.cid, &player_info),
            });
        }

        let item = NormalizedItem {
            id: ItemId(format!("item:{canonical_key}")),
            title: video.title.clone(),
            owner_name: video.owner_name.clone(),
            owner_mid: video.owner_mid,
            publish_date: video.publish_date.clone(),
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
        self.resolve_with_stream_target(input, options.fetch_streams, None)
            .await
    }
}

pub struct BpiVideoApi {
    client: Arc<BpiClient>,
    view_snapshots: Mutex<HashMap<String, VideoViewSnapshot>>,
}

impl BpiVideoApi {
    pub fn new() -> BdlResult<Self> {
        BpiClient::new()
            .map(Self::from_client)
            .map_err(|error| BdlError::Bpi(error.to_string()))
    }

    pub fn from_client(client: impl Into<Arc<BpiClient>>) -> Self {
        Self {
            client: client.into(),
            view_snapshots: Mutex::new(HashMap::new()),
        }
    }

    async fn view_snapshot(&self, id: &VideoInputId) -> BdlResult<VideoViewSnapshot> {
        let key = id.input_label();
        if let Some(snapshot) = self
            .view_snapshots
            .lock()
            .map_err(|_| BdlError::Bpi("video view cache lock poisoned".to_owned()))?
            .get(&key)
            .cloned()
        {
            return Ok(snapshot);
        }

        let response = self
            .client
            .get(VIDEO_VIEW_API)
            .query(&video_view_query(id))
            .send()
            .await
            .map_err(|error| BdlError::Bpi(error.to_string()))?
            .error_for_status()
            .map_err(|error| BdlError::Bpi(error.to_string()))?;
        let snapshot = response
            .json::<VideoViewResponse>()
            .await
            .map_err(|error| BdlError::Bpi(error.to_string()))?
            .into_snapshot()?;

        self.view_snapshots
            .lock()
            .map_err(|_| BdlError::Bpi("video view cache lock poisoned".to_owned()))?
            .insert(key, snapshot.clone());

        Ok(snapshot)
    }
}

#[async_trait]
impl VideoApi for BpiVideoApi {
    async fn view(&self, id: &VideoInputId) -> BdlResult<ResolvedVideo> {
        Ok(self.view_snapshot(id).await?.video)
    }

    async fn play_url(&self, id: &VideoInputId, cid: u64) -> BdlResult<ResolvedPlayUrl> {
        let data = self
            .client
            .video()
            .play_url(play_url_params(id, cid)?)
            .await
            .map_err(|error| BdlError::Bpi(error.to_string()))?;

        Ok(data
            .dash
            .map(ResolvedPlayUrl::from_dash)
            .unwrap_or_default())
    }

    async fn collection_hint(&self, id: &VideoInputId) -> BdlResult<Option<VideoCollectionRef>> {
        Ok(self.view_snapshot(id).await?.collection_hint)
    }

    async fn player_info(&self, id: &VideoInputId, cid: u64) -> BdlResult<ResolvedPlayerInfo> {
        let data = self
            .client
            .video()
            .player_info_v2(player_info_params(id, cid)?)
            .await
            .map_err(|error| BdlError::Bpi(error.to_string()))?;

        Ok(ResolvedPlayerInfo {
            subtitles: data
                .subtitle
                .map(|info| {
                    info.subtitles
                        .into_iter()
                        .filter_map(|subtitle| {
                            normalize_asset_url(&subtitle.subtitle_url).map(|url| {
                                ResolvedSubtitle {
                                    lan: subtitle.lan,
                                    lan_doc: subtitle.lan_doc,
                                    url,
                                }
                            })
                        })
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

#[derive(Debug, Deserialize)]
struct VideoViewResponse {
    code: i64,
    #[serde(default)]
    message: String,
    data: Option<VideoViewData>,
}

impl VideoViewResponse {
    fn into_snapshot(self) -> BdlResult<VideoViewSnapshot> {
        if self.code != 0 {
            return Err(BdlError::Bpi(format!(
                "video view returned code {}: {}",
                self.code, self.message
            )));
        }

        let data = self
            .data
            .ok_or_else(|| BdlError::Bpi("video view returned no data".to_owned()))?;
        let owner_name = data
            .owner
            .as_ref()
            .and_then(|owner| non_empty(owner.name.clone()));
        let owner_mid = data
            .owner
            .as_ref()
            .map(|owner| owner.mid)
            .filter(|mid| *mid > 0);
        let video = ResolvedVideo {
            aid: data.aid,
            bvid: data.bvid,
            default_cid: data.cid,
            title: data.title,
            owner_name,
            owner_mid,
            publish_date: (data.pubdate > 0)
                .then_some(data.pubdate)
                .and_then(bilibili_publish_date),
            cover_url: non_empty(data.pic),
            pages: data
                .pages
                .into_iter()
                .map(|page| ResolvedVideoPage {
                    cid: page.cid,
                    index: page.page,
                    title: page.part,
                    duration_seconds: Some(page.duration),
                })
                .collect(),
        };
        Ok(VideoViewSnapshot {
            video,
            collection_hint: data.ugc_season.map(|season| VideoCollectionRef {
                mid: season.mid,
                season_id: season.id,
            }),
        })
    }
}

#[derive(Debug, Deserialize)]
struct VideoViewData {
    aid: u64,
    #[serde(default)]
    bvid: String,
    cid: u64,
    title: String,
    #[serde(default)]
    pic: String,
    #[serde(default)]
    pubdate: u64,
    owner: Option<VideoViewOwner>,
    #[serde(default)]
    pages: Vec<VideoViewPage>,
    ugc_season: Option<VideoCollectionHintSeason>,
}

#[derive(Debug, Deserialize)]
struct VideoViewOwner {
    mid: u64,
    #[serde(default)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct VideoViewPage {
    cid: u64,
    page: u32,
    #[serde(default)]
    part: String,
    duration: u64,
}

#[derive(Debug, Deserialize)]
struct VideoCollectionHintSeason {
    id: u64,
    mid: u64,
}

#[derive(Debug, Clone)]
struct VideoViewSnapshot {
    video: ResolvedVideo,
    collection_hint: Option<VideoCollectionRef>,
}

impl ResolvedPlayUrl {
    fn from_dash(dash: bpi_rs::video::videostream_url::DashInfo) -> Self {
        Self {
            video: dash
                .video
                .into_iter()
                .map(ResolvedDashStream::from)
                .collect(),
            audio: dash
                .audio
                .into_iter()
                .chain(
                    dash.dolby
                        .and_then(|dolby| dolby.audio)
                        .into_iter()
                        .flatten(),
                )
                .chain(dash.flac.and_then(|flac| flac.audio))
                .map(ResolvedDashStream::from)
                .collect(),
        }
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

fn video_view_query(id: &VideoInputId) -> Vec<(&'static str, String)> {
    match id {
        VideoInputId::Aid(aid) => vec![("aid", aid.to_string())],
        VideoInputId::Bvid(bvid) => vec![("bvid", bvid.clone())],
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
        .quality(u64::from(VideoQuality::P8K.as_u32()))
        .format_flags(u64::from(
            (Fnval::DASH
                | Fnval::FOURK
                | Fnval::EIGHTK
                | Fnval::HDR
                | Fnval::DOLBY_AUDIO
                | Fnval::DOLBY_VISION
                | Fnval::AV1)
                .bits(),
        ))
        .format_version(0)
        .fourk(true)
        .high_quality(true))
}

fn player_info_params(id: &VideoInputId, cid: u64) -> BdlResult<VideoPlayerInfoParams> {
    let cid = Cid::new(cid).map_err(|error| BdlError::Bpi(error.to_string()))?;
    match id {
        VideoInputId::Aid(aid) => Aid::new(*aid)
            .map(|aid| VideoPlayerInfoParams::from_aid(aid, cid))
            .map_err(|error| BdlError::Bpi(error.to_string())),
        VideoInputId::Bvid(bvid) => bvid
            .parse::<Bvid>()
            .map(|bvid| VideoPlayerInfoParams::from_bvid(bvid, cid))
            .map_err(|error| BdlError::Bpi(error.to_string())),
    }
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

fn archive_assets(cid: u64, player_info: &ResolvedPlayerInfo) -> Vec<DerivedAsset> {
    let mut assets = vec![AssetKind::Cover.with_policy(FetchPolicy::OnDemand)];

    if let Some(asset) = subtitle_asset(player_info) {
        assets.push(asset);
    }
    if let Some(asset) = danmaku_asset(cid) {
        assets.push(asset);
    }
    assets.push(AssetKind::Nfo.with_policy(FetchPolicy::OnDemand));

    assets
}

fn subtitle_asset(player_info: &ResolvedPlayerInfo) -> Option<DerivedAsset> {
    let urls = player_info
        .subtitles
        .iter()
        .filter_map(|subtitle| normalize_asset_url(&subtitle.url))
        .collect::<Vec<_>>();

    (!urls.is_empty()).then(|| {
        DerivedAsset::with_urls(
            AssetKind::Subtitle,
            FetchPolicy::OnDemand,
            "json",
            urls,
            default_stream_headers(),
        )
    })
}

fn danmaku_asset(cid: u64) -> Option<DerivedAsset> {
    Cid::new(cid).ok().map(|cid| {
        DerivedAsset::with_urls(
            AssetKind::Danmaku,
            FetchPolicy::OnDemand,
            "xml",
            vec![DanmakuXmlListParams::new(cid).comment_xml_url()],
            default_stream_headers(),
        )
    })
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
    use super::{VideoCollectionRef, VideoViewResponse};

    #[test]
    fn media_preferences_request_all_supported_video_formats() {
        use bpi_rs::ids::{Aid, Cid};
        use bpi_rs::video::VideoPlayUrlParams;
        let expected = VideoPlayUrlParams::from_aid(Aid::new(1).unwrap(), Cid::new(2).unwrap())
            .quality(127)
            .format_flags(4048)
            .format_version(0)
            .fourk(true)
            .high_quality(true);
        assert_eq!(
            super::play_url_params(&super::VideoInputId::Aid(1), 2).unwrap(),
            expected
        );
    }

    #[test]
    fn media_preferences_collects_dolby_and_flac_tracks() {
        let track = |id| serde_json::json!({"id": id, "baseUrl": "https://example.invalid/track", "backupUrl": [], "bandwidth": 1000, "mimeType": "audio/mp4", "codecs": "mp4a"});
        let dash = serde_json::from_value(serde_json::json!({
            "video": [], "audio": [track(30280)], "duration": 1,
            "dolby": {"type": 1, "audio": [track(30250)]},
            "flac": {"audio": track(30251)}
        }))
        .unwrap();
        let resolved = super::ResolvedPlayUrl::from_dash(dash);
        assert_eq!(
            resolved
                .audio
                .iter()
                .map(|stream| stream.id)
                .collect::<Vec<_>>(),
            vec![30280, 30250, 30251]
        );
        assert_eq!(
            super::stream_codec("dvhe.05.06"),
            crate::model::StreamCodec::Hevc
        );
    }

    #[test]
    fn video_view_collection_hint_reads_ugc_season_identity() {
        let payload: VideoViewResponse = serde_json::from_str(
            r#"{
                "code": 0,
                "message": "0",
                "data": {
                    "aid": 1,
                    "bvid": "BV1test",
                    "cid": 2,
                    "title": "云端镜像 01",
                    "pic": "https://example.invalid/cover.jpg",
                    "pubdate": 1767196800,
                    "owner": {"mid": 592988861, "name": "测试UP"},
                    "pages": [{"cid": 2, "page": 1, "part": "P1", "duration": 120}],
                    "ugc_season": {
                        "id": 8122710,
                        "mid": 592988861,
                        "title": "云端镜像"
                    }
                }
            }"#,
        )
        .expect("fixture should deserialize");
        let snapshot = payload.into_snapshot().expect("payload should succeed");

        assert_eq!(
            snapshot.collection_hint,
            Some(VideoCollectionRef {
                mid: 592988861,
                season_id: 8122710,
            })
        );
        assert_eq!(snapshot.video.publish_date.as_deref(), Some("2026-01-01"));
        assert_eq!(snapshot.video.owner_name.as_deref(), Some("测试UP"));
        assert_eq!(snapshot.video.pages[0].duration_seconds, Some(120));
    }

    #[test]
    fn video_view_collection_hint_allows_plain_video() {
        let payload: VideoViewResponse = serde_json::from_str(
            r#"{"code":0,"message":"0","data":{"aid":1,"bvid":"BV1plain","cid":2,"title":"plain","pic":"","pubdate":0,"owner":null,"pages":[],"ugc_season":null}}"#,
        )
        .expect("fixture should deserialize");
        let snapshot = payload.into_snapshot().expect("payload should succeed");

        assert_eq!(snapshot.collection_hint, None);
        assert_eq!(snapshot.video.publish_date, None);
    }
}
