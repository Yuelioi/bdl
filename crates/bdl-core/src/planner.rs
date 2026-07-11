use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::{BdlError, BdlResult};
use crate::ids::PartId;
use crate::model::{
    AssetKind, DerivedAsset, MediaKind, MediaStream, NormalizedItem, NormalizedPart,
    NormalizedSourceTree, SourceKind, StreamCodec, StreamQuality,
};
use crate::naming::{
    DEFAULT_NAMING_TEMPLATE, DuplicateNamingStrategy, NamingContext, render_output_path,
    resolve_duplicate_path,
};
use crate::queue::{
    DownloadResource, DownloadResourceIntent, DownloadResourceKind, DownloadTask,
    DownloadTaskMediaSelection, DownloadTaskRefreshInput, DownloadTaskRefreshIntent,
    ResourceStatus, TaskStatus,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveMode {
    Fast,
    CompleteArchive,
    Custom,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadMediaMode {
    #[default]
    AudioVideo,
    VideoOnly,
    AudioOnly,
}

impl DownloadMediaMode {
    pub fn parse(value: &str) -> BdlResult<Self> {
        match value {
            "audio_video" => Ok(Self::AudioVideo),
            "video_only" => Ok(Self::VideoOnly),
            "audio_only" => Ok(Self::AudioOnly),
            other => Err(BdlError::Planning {
                message: format!("下载内容设置无效：`{other}`。"),
            }),
        }
    }

    const fn includes_video(self) -> bool {
        matches!(self, Self::AudioVideo | Self::VideoOnly)
    }

    const fn includes_audio(self) -> bool {
        matches!(self, Self::AudioVideo | Self::AudioOnly)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ArchiveAssetSelection {
    pub cover: bool,
    pub subtitles: bool,
    pub danmaku: bool,
    pub nfo: bool,
}

impl ArchiveAssetSelection {
    pub const fn all() -> Self {
        Self {
            cover: true,
            subtitles: true,
            danmaku: true,
            nfo: true,
        }
    }

    pub const fn none() -> Self {
        Self {
            cover: false,
            subtitles: false,
            danmaku: false,
            nfo: false,
        }
    }

    fn includes(self, intent: DownloadResourceIntent) -> bool {
        match intent {
            DownloadResourceIntent::Cover => self.cover,
            DownloadResourceIntent::Subtitle => self.subtitles,
            DownloadResourceIntent::Danmaku => self.danmaku,
            DownloadResourceIntent::Nfo => self.nfo,
            DownloadResourceIntent::Video | DownloadResourceIntent::Audio => false,
        }
    }
}

impl Default for ArchiveAssetSelection {
    fn default() -> Self {
        Self::all()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum StreamPreference {
    #[default]
    Best,
    Quality(u32),
}

impl StreamPreference {
    pub fn parse(value: &str, field_name: &str) -> BdlResult<Self> {
        let trimmed = value.trim();
        if trimmed.eq_ignore_ascii_case("best") {
            return Ok(Self::Best);
        }

        let quality = trimmed.parse::<u32>().map_err(|_| BdlError::Planning {
            message: format!("{field_name}设置无效：`{value}`。"),
        })?;

        if quality == 0 {
            return Err(BdlError::Planning {
                message: format!("{field_name}设置无效：清晰度必须大于 0。"),
            });
        }

        Ok(Self::Quality(quality))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MissingQualityPolicy {
    #[default]
    Lower,
    Skip,
    Ask,
}

impl MissingQualityPolicy {
    pub fn parse(value: &str) -> BdlResult<Self> {
        match value {
            "lower" => Ok(Self::Lower),
            "skip" => Ok(Self::Skip),
            "ask" => Ok(Self::Ask),
            other => Err(BdlError::Planning {
                message: format!("缺失清晰度策略无效：`{other}`。"),
            }),
        }
    }
}

pub fn parse_stream_codec(value: &str) -> BdlResult<StreamCodec> {
    match value {
        "auto" => Ok(StreamCodec::Auto),
        "avc" => Ok(StreamCodec::Avc),
        "hevc" => Ok(StreamCodec::Hevc),
        "av1" => Ok(StreamCodec::Av1),
        other => Err(BdlError::Planning {
            message: format!("视频编码设置无效：`{other}`。"),
        }),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadOptions {
    pub output_dir: PathBuf,
    pub archive_mode: ArchiveMode,
    pub media_mode: DownloadMediaMode,
    pub output_extension: String,
    pub naming_template: String,
    pub duplicate_naming_strategy: DuplicateNamingStrategy,
    pub video_quality: StreamPreference,
    pub audio_quality: StreamPreference,
    pub video_codec: StreamCodec,
    pub missing_quality_policy: MissingQualityPolicy,
    pub archive_assets: ArchiveAssetSelection,
}

impl DownloadOptions {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            output_dir,
            archive_mode: ArchiveMode::Fast,
            media_mode: DownloadMediaMode::default(),
            output_extension: "mp4".to_owned(),
            naming_template: DEFAULT_NAMING_TEMPLATE.to_owned(),
            duplicate_naming_strategy: DuplicateNamingStrategy::default(),
            video_quality: StreamPreference::default(),
            audio_quality: StreamPreference::default(),
            video_codec: StreamCodec::Auto,
            missing_quality_policy: MissingQualityPolicy::default(),
            archive_assets: ArchiveAssetSelection::all(),
        }
    }

    pub fn with_archive_mode(mut self, archive_mode: ArchiveMode) -> Self {
        self.archive_mode = archive_mode;
        self
    }
}

pub fn plan_selected_parts(
    tree: &NormalizedSourceTree,
    selected_part_ids: &[PartId],
    options: &DownloadOptions,
) -> BdlResult<Vec<DownloadTask>> {
    let mut reserved_paths = HashSet::new();
    let mut tasks = Vec::with_capacity(selected_part_ids.len());

    for part_id in selected_part_ids {
        let selected = find_part(tree, part_id).ok_or_else(|| BdlError::Planning {
            message: format!(
                "选中的分 P `{}` 未加载，请重新解析后再创建下载任务。",
                part_id.0
            ),
        })?;
        tasks.push(plan_part(tree, selected, options, &mut reserved_paths)?);
    }

    Ok(tasks)
}

struct SelectedPart<'a> {
    item: &'a NormalizedItem,
    part: &'a NormalizedPart,
    item_index: usize,
    part_index: usize,
}

fn find_part<'a>(tree: &'a NormalizedSourceTree, part_id: &PartId) -> Option<SelectedPart<'a>> {
    tree.groups
        .iter()
        .flat_map(|group| &group.items)
        .enumerate()
        .find_map(|(item_index, item)| {
            item.parts
                .iter()
                .enumerate()
                .find(|(_, part)| &part.id == part_id)
                .map(|(part_index, part)| SelectedPart {
                    item,
                    part,
                    item_index,
                    part_index,
                })
        })
}

fn plan_part(
    tree: &NormalizedSourceTree,
    selected: SelectedPart<'_>,
    options: &DownloadOptions,
    reserved_paths: &mut HashSet<PathBuf>,
) -> BdlResult<DownloadTask> {
    let item = selected.item;
    let part = selected.part;
    let task_id = format!("task:{}:{}", tree.source.id.0, part.id.0);
    let title = task_title(item, part);
    let video = if options.media_mode.includes_video() {
        Some(select_video_stream(part, options)?)
    } else {
        None
    };
    let audio = if options.media_mode.includes_audio() {
        Some(select_audio_stream(part, options)?)
    } else {
        None
    };
    let output_path = output_path_for(tree, &selected, video, audio, options, reserved_paths)?;

    let mut resources = Vec::with_capacity(2);
    if let Some(video) = video {
        resources.push(media_resource(
            &task_id,
            &output_path,
            DownloadResourceIntent::Video,
            DownloadResourceKind::Video,
            video,
        )?);
    }
    if let Some(audio) = audio {
        resources.push(media_resource(
            &task_id,
            &output_path,
            DownloadResourceIntent::Audio,
            DownloadResourceKind::Audio,
            audio,
        )?);
    }

    let archive_assets = match options.archive_mode {
        ArchiveMode::Fast => ArchiveAssetSelection::none(),
        ArchiveMode::CompleteArchive => ArchiveAssetSelection::all(),
        ArchiveMode::Custom => options.archive_assets,
    };
    if archive_assets != ArchiveAssetSelection::none() {
        resources.extend(complete_archive_resources(
            &task_id,
            &output_path,
            item,
            part,
            archive_assets,
        ));
    }

    Ok(DownloadTask {
        id: task_id,
        title,
        source_id: tree.source.id.0.clone(),
        status: TaskStatus::Waiting,
        resources,
        output_path,
        refresh_intent: media_refresh_intent(tree.source.kind, part, selected.part_index + 1),
        media_selection: DownloadTaskMediaSelection {
            video_quality: video
                .map(stream_quality_label)
                .unwrap_or_else(|| "none".to_owned()),
            audio_quality: audio
                .map(stream_quality_label)
                .unwrap_or_else(|| "none".to_owned()),
            video_codec: video.map(stream_codec_label).unwrap_or("none").to_owned(),
            container: options.output_extension.clone(),
        },
        scheduled_at: None,
        speed_limit_bytes_per_second: None,
    })
}

fn select_video_stream<'a>(
    part: &'a NormalizedPart,
    options: &DownloadOptions,
) -> BdlResult<&'a MediaStream> {
    let streams = streams_by_kind(part, MediaKind::Video);
    let codec_matches = if options.video_codec == StreamCodec::Auto {
        Vec::new()
    } else {
        streams
            .iter()
            .copied()
            .filter(|stream| stream.codec == options.video_codec)
            .collect::<Vec<_>>()
    };
    let candidates = if codec_matches.is_empty() {
        streams
    } else {
        codec_matches
    };

    select_stream_by_quality(
        candidates,
        options.video_quality,
        options.missing_quality_policy,
        "视频",
        &part.title,
    )
}

fn select_audio_stream<'a>(
    part: &'a NormalizedPart,
    options: &DownloadOptions,
) -> BdlResult<&'a MediaStream> {
    select_stream_by_quality(
        streams_by_kind(part, MediaKind::Audio),
        options.audio_quality,
        options.missing_quality_policy,
        "音频",
        &part.title,
    )
}

fn streams_by_kind(part: &NormalizedPart, kind: MediaKind) -> Vec<&MediaStream> {
    part.streams
        .iter()
        .filter(|stream| stream.kind == kind)
        .collect()
}

fn select_stream_by_quality<'a>(
    streams: Vec<&'a MediaStream>,
    preference: StreamPreference,
    missing_policy: MissingQualityPolicy,
    stream_label: &str,
    part_title: &str,
) -> BdlResult<&'a MediaStream> {
    if streams.is_empty() {
        return Err(BdlError::Planning {
            message: format!("`{part_title}` 缺少{stream_label}流，请重新解析后再试。"),
        });
    }

    match preference {
        StreamPreference::Best => streams
            .into_iter()
            .max_by_key(|stream| stream_selection_rank(stream))
            .ok_or_else(|| BdlError::Planning {
                message: format!("`{part_title}` 缺少{stream_label}流，请重新解析后再试。"),
            }),
        StreamPreference::Quality(target) => {
            select_target_quality(streams, target, missing_policy, stream_label, part_title)
        }
    }
}

fn select_target_quality<'a>(
    streams: Vec<&'a MediaStream>,
    target: u32,
    missing_policy: MissingQualityPolicy,
    stream_label: &str,
    part_title: &str,
) -> BdlResult<&'a MediaStream> {
    if let Some(exact) = streams
        .iter()
        .copied()
        .filter(|stream| stream_quality_rank(stream.quality) == target)
        .max_by_key(|stream| stream_selection_rank(stream))
    {
        return Ok(exact);
    }

    if missing_policy == MissingQualityPolicy::Lower {
        let lower_or_equal = streams
            .iter()
            .copied()
            .filter(|stream| stream_quality_rank(stream.quality) <= target)
            .max_by_key(|stream| stream_selection_rank(stream));

        return lower_or_equal
            .or_else(|| {
                streams
                    .iter()
                    .copied()
                    .min_by_key(|stream| stream_selection_rank(stream))
            })
            .ok_or_else(|| BdlError::Planning {
                message: format!("`{part_title}` 缺少{stream_label}流，请重新解析后再试。"),
            });
    }

    let action = match missing_policy {
        MissingQualityPolicy::Skip => "已按设置跳过创建任务",
        MissingQualityPolicy::Ask => "需要用户确认后再创建任务",
        MissingQualityPolicy::Lower => unreachable!("lower policy handled above"),
    };

    Err(BdlError::Planning {
        message: format!("`{part_title}` 没有 {target} 的{stream_label}流，{action}。"),
    })
}

fn media_resource(
    task_id: &str,
    output_path: &Path,
    intent: DownloadResourceIntent,
    kind: DownloadResourceKind,
    stream: &MediaStream,
) -> BdlResult<DownloadResource> {
    if stream.urls.is_empty() {
        return Err(BdlError::Planning {
            message: format!("`{}` 没有可用下载地址，请重新解析后再试。", stream.id),
        });
    }

    let suffix = resource_suffix(intent);
    let target_path = sibling_resource_path(output_path, suffix, Some("m4s"));
    Ok(DownloadResource {
        id: format!("{task_id}:resource:{suffix}"),
        kind,
        intent,
        current_urls: stream.urls.clone(),
        headers: stream.headers.clone(),
        temp_path: temp_path_for(&target_path),
        target_path,
        status: ResourceStatus::Pending,
    })
}

fn media_refresh_intent(
    source_kind: SourceKind,
    part: &NormalizedPart,
    page_number: usize,
) -> Option<DownloadTaskRefreshIntent> {
    match source_kind {
        SourceKind::Bangumi => episode_refresh_intent(part, "bangumi")
            .or_else(|| video_refresh_intent(part, page_number)),
        SourceKind::Cheese => episode_refresh_intent(part, "cheese")
            .or_else(|| video_refresh_intent(part, page_number)),
        SourceKind::Video
        | SourceKind::Favorite
        | SourceKind::Collection
        | SourceKind::Series
        | SourceKind::Uploader
        | SourceKind::Unknown => video_refresh_intent(part, page_number)
            .or_else(|| episode_refresh_intent(part, "bangumi"))
            .or_else(|| episode_refresh_intent(part, "cheese")),
    }
}

fn video_refresh_intent(
    part: &NormalizedPart,
    page_number: usize,
) -> Option<DownloadTaskRefreshIntent> {
    let cid = part.cid?;
    let input = if let Some(bvid) = part.bvid.as_ref().filter(|value| !value.is_empty()) {
        DownloadTaskRefreshInput::VideoBvid { bvid: bvid.clone() }
    } else {
        DownloadTaskRefreshInput::VideoAid { aid: part.aid? }
    };

    Some(DownloadTaskRefreshIntent {
        input,
        cid,
        page_number: u32::try_from(page_number).ok(),
    })
}

fn episode_refresh_intent(part: &NormalizedPart, kind: &str) -> Option<DownloadTaskRefreshIntent> {
    let cid = part.cid?;
    let part_key = part.id.0.strip_prefix("part:")?;
    let mut segments = part_key.split(':');
    if segments.next()? != kind {
        return None;
    }

    let _season_id = parse_positive_u64(segments.next()?)?;
    let ep_id = parse_positive_u64(segments.next()?)?;
    let input = match kind {
        "bangumi" => DownloadTaskRefreshInput::BangumiEpisode { ep_id },
        "cheese" => DownloadTaskRefreshInput::CheeseEpisode { ep_id },
        _ => return None,
    };

    Some(DownloadTaskRefreshIntent {
        input,
        cid,
        page_number: None,
    })
}

fn complete_archive_resources(
    task_id: &str,
    output_path: &Path,
    item: &NormalizedItem,
    part: &NormalizedPart,
    selection: ArchiveAssetSelection,
) -> Vec<DownloadResource> {
    [
        DownloadResourceIntent::Cover,
        DownloadResourceIntent::Subtitle,
        DownloadResourceIntent::Danmaku,
        DownloadResourceIntent::Nfo,
    ]
    .into_iter()
    .filter(|intent| selection.includes(*intent))
    .map(|intent| asset_resource(task_id, output_path, item, part, intent))
    .collect()
}

fn asset_resource(
    task_id: &str,
    output_path: &Path,
    item: &NormalizedItem,
    part: &NormalizedPart,
    intent: DownloadResourceIntent,
) -> DownloadResource {
    let suffix = resource_suffix(intent);
    let asset = part_asset(part, intent);
    let current_urls = asset_urls(item, asset, intent);
    let inferred_format = asset
        .and_then(|asset| asset.format.clone())
        .or_else(|| infer_asset_extension(intent, &current_urls));
    let target_path = sibling_resource_path(output_path, suffix, inferred_format.as_deref());
    DownloadResource {
        id: format!("{task_id}:resource:{suffix}"),
        kind: DownloadResourceKind::Asset,
        intent,
        current_urls,
        headers: asset.map(|asset| asset.headers.clone()).unwrap_or_default(),
        temp_path: temp_path_for(&target_path),
        target_path,
        status: ResourceStatus::Pending,
    }
}

fn infer_asset_extension(intent: DownloadResourceIntent, urls: &[String]) -> Option<String> {
    if intent != DownloadResourceIntent::Cover {
        return None;
    }

    urls.iter().find_map(|value| {
        let url = Url::parse(value).ok()?;
        let extension = Path::new(url.path())
            .extension()?
            .to_str()?
            .to_ascii_lowercase();
        matches!(
            extension.as_str(),
            "jpg" | "jpeg" | "png" | "webp" | "avif" | "gif"
        )
        .then_some(extension)
    })
}

fn output_path_for(
    tree: &NormalizedSourceTree,
    selected: &SelectedPart<'_>,
    video: Option<&MediaStream>,
    audio: Option<&MediaStream>,
    options: &DownloadOptions,
    reserved_paths: &mut HashSet<PathBuf>,
) -> BdlResult<PathBuf> {
    let item = selected.item;
    let part = selected.part;
    let today = Utc::now().date_naive().to_string();
    let representative_stream = video.or(audio).ok_or_else(|| BdlError::Planning {
        message: format!("`{}` 没有可下载的媒体流。", part.title),
    })?;
    let quality_label = stream_quality_label(representative_stream);
    let codec_label = video.map(stream_codec_label).unwrap_or("none");
    let context = NamingContext {
        title: &item.title,
        part_title: &part.title,
        part_index: selected.part_index + 1,
        bvid: part.bvid.as_deref(),
        aid: part.aid,
        cid: part.cid,
        owner_name: item.owner_name.as_deref(),
        owner_mid: item.owner_mid,
        series_title: Some(&tree.source.title),
        season_index: None,
        episode_index: Some(selected.item_index + 1),
        collection_title: Some(&tree.source.title),
        index: Some(selected.item_index + 1),
        quality: Some(&quality_label),
        codec: Some(codec_label),
        date: Some(&today),
        ext: &options.output_extension,
    };
    let relative_path = render_output_path(&options.naming_template, &context)?;

    Ok(resolve_duplicate_path(
        options.output_dir.join(relative_path),
        reserved_paths,
        options.duplicate_naming_strategy,
    ))
}

fn sibling_resource_path(output_path: &Path, suffix: &str, extension: Option<&str>) -> PathBuf {
    let stem = output_path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("untitled");
    let file_name = match extension {
        Some(extension) => format!("{stem}.{suffix}.{extension}"),
        None => format!("{stem}.{suffix}"),
    };

    output_path.with_file_name(file_name)
}

fn part_asset(part: &NormalizedPart, intent: DownloadResourceIntent) -> Option<&DerivedAsset> {
    let kind = match intent {
        DownloadResourceIntent::Cover => AssetKind::Cover,
        DownloadResourceIntent::Subtitle => AssetKind::Subtitle,
        DownloadResourceIntent::Danmaku => AssetKind::Danmaku,
        DownloadResourceIntent::Nfo => AssetKind::Nfo,
        DownloadResourceIntent::Video | DownloadResourceIntent::Audio => return None,
    };

    part.assets.iter().find(|asset| asset.kind == kind)
}

fn asset_urls(
    item: &NormalizedItem,
    asset: Option<&DerivedAsset>,
    intent: DownloadResourceIntent,
) -> Vec<String> {
    match intent {
        DownloadResourceIntent::Cover => asset
            .map(|asset| asset.urls.clone())
            .unwrap_or_default()
            .into_iter()
            .chain(item.cover_url.iter().cloned())
            .collect(),
        DownloadResourceIntent::Subtitle | DownloadResourceIntent::Danmaku => {
            asset.map(|asset| asset.urls.clone()).unwrap_or_default()
        }
        DownloadResourceIntent::Video
        | DownloadResourceIntent::Audio
        | DownloadResourceIntent::Nfo => Vec::new(),
    }
}

fn temp_path_for(target_path: &Path) -> PathBuf {
    let mut value = OsString::from(target_path.as_os_str());
    value.push(".bdlpart");
    PathBuf::from(value)
}

fn task_title(item: &NormalizedItem, part: &NormalizedPart) -> String {
    if part.title.trim().is_empty() || part.title == item.title {
        item.title.clone()
    } else {
        format!("{} - {}", item.title, part.title)
    }
}

fn resource_suffix(intent: DownloadResourceIntent) -> &'static str {
    match intent {
        DownloadResourceIntent::Video => "video",
        DownloadResourceIntent::Audio => "audio",
        DownloadResourceIntent::Cover => "cover",
        DownloadResourceIntent::Subtitle => "subtitle",
        DownloadResourceIntent::Danmaku => "danmaku",
        DownloadResourceIntent::Nfo => "nfo",
    }
}

fn stream_selection_rank(stream: &MediaStream) -> (u32, u64) {
    (
        stream_quality_rank(stream.quality),
        stream.bandwidth.unwrap_or_default(),
    )
}

fn stream_quality_rank(quality: StreamQuality) -> u32 {
    match quality {
        crate::model::StreamQuality::Best => u32::MAX,
        crate::model::StreamQuality::Quality(value) => value,
    }
}

fn parse_positive_u64(value: &str) -> Option<u64> {
    value.parse::<u64>().ok().filter(|value| *value > 0)
}

fn stream_quality_label(stream: &MediaStream) -> String {
    match stream.quality {
        crate::model::StreamQuality::Best => "best".to_owned(),
        crate::model::StreamQuality::Quality(value) => value.to_string(),
    }
}

fn stream_codec_label(stream: &MediaStream) -> &'static str {
    match stream.codec {
        crate::model::StreamCodec::Auto => "auto",
        crate::model::StreamCodec::Avc => "avc",
        crate::model::StreamCodec::Hevc => "hevc",
        crate::model::StreamCodec::Av1 => "av1",
        crate::model::StreamCodec::Unknown => "unknown",
    }
}
