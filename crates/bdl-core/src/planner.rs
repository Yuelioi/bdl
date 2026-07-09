use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::{BdlError, BdlResult};
use crate::ids::PartId;
use crate::model::{MediaKind, MediaStream, NormalizedItem, NormalizedPart, NormalizedSourceTree};
use crate::naming::{DEFAULT_NAMING_TEMPLATE, NamingContext, render_output_path, unique_path};
use crate::queue::{
    DownloadResource, DownloadResourceIntent, DownloadResourceKind, DownloadTask, ResourceStatus,
    TaskStatus,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveMode {
    Fast,
    CompleteArchive,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadOptions {
    pub output_dir: PathBuf,
    pub archive_mode: ArchiveMode,
    pub output_extension: String,
    pub naming_template: String,
}

impl DownloadOptions {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            output_dir,
            archive_mode: ArchiveMode::Fast,
            output_extension: "mp4".to_owned(),
            naming_template: DEFAULT_NAMING_TEMPLATE.to_owned(),
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
    let video = select_stream(part, MediaKind::Video).ok_or_else(|| BdlError::Planning {
        message: format!("`{}` 缺少视频流，请重新解析后再试。", part.title),
    })?;
    let audio = select_stream(part, MediaKind::Audio).ok_or_else(|| BdlError::Planning {
        message: format!("`{}` 缺少音频流，请重新解析后再试。", part.title),
    })?;
    let output_path = output_path_for(tree, item, part, selected, video, options, reserved_paths)?;

    let mut resources = vec![
        media_resource(
            &task_id,
            &output_path,
            DownloadResourceIntent::Video,
            DownloadResourceKind::Video,
            video,
        )?,
        media_resource(
            &task_id,
            &output_path,
            DownloadResourceIntent::Audio,
            DownloadResourceKind::Audio,
            audio,
        )?,
    ];

    if options.archive_mode == ArchiveMode::CompleteArchive {
        resources.extend(complete_archive_resources(&task_id, &output_path, item));
    }

    Ok(DownloadTask {
        id: task_id,
        title,
        source_id: tree.source.id.0.clone(),
        status: TaskStatus::Waiting,
        resources,
        output_path,
    })
}

fn select_stream(part: &NormalizedPart, kind: MediaKind) -> Option<&MediaStream> {
    part.streams
        .iter()
        .filter(|stream| stream.kind == kind)
        .max_by_key(|stream| stream_quality_rank(stream.quality))
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

fn complete_archive_resources(
    task_id: &str,
    output_path: &Path,
    item: &NormalizedItem,
) -> Vec<DownloadResource> {
    [
        DownloadResourceIntent::Cover,
        DownloadResourceIntent::Subtitle,
        DownloadResourceIntent::Danmaku,
        DownloadResourceIntent::Nfo,
    ]
    .into_iter()
    .map(|intent| asset_resource(task_id, output_path, item, intent))
    .collect()
}

fn asset_resource(
    task_id: &str,
    output_path: &Path,
    item: &NormalizedItem,
    intent: DownloadResourceIntent,
) -> DownloadResource {
    let suffix = resource_suffix(intent);
    let target_path = sibling_resource_path(output_path, suffix, None);
    DownloadResource {
        id: format!("{task_id}:resource:{suffix}"),
        kind: DownloadResourceKind::Asset,
        intent,
        current_urls: asset_urls(item, intent),
        headers: Vec::new(),
        temp_path: temp_path_for(&target_path),
        target_path,
        status: ResourceStatus::Pending,
    }
}

fn output_path_for(
    tree: &NormalizedSourceTree,
    item: &NormalizedItem,
    part: &NormalizedPart,
    selected: SelectedPart<'_>,
    video: &MediaStream,
    options: &DownloadOptions,
    reserved_paths: &mut HashSet<PathBuf>,
) -> BdlResult<PathBuf> {
    let today = Utc::now().date_naive().to_string();
    let quality_label = stream_quality_label(video);
    let context = NamingContext {
        title: &item.title,
        part_title: &part.title,
        part_index: selected.part_index + 1,
        bvid: part.bvid.as_deref(),
        aid: part.aid,
        cid: part.cid,
        owner_name: item.owner_name.as_deref(),
        owner_mid: None,
        series_title: Some(&tree.source.title),
        season_index: None,
        episode_index: Some(selected.item_index + 1),
        collection_title: Some(&tree.source.title),
        index: Some(selected.item_index + 1),
        quality: Some(&quality_label),
        codec: Some(stream_codec_label(video)),
        date: Some(&today),
        ext: &options.output_extension,
    };
    let relative_path = render_output_path(&options.naming_template, &context)?;

    Ok(unique_path(
        options.output_dir.join(relative_path),
        reserved_paths,
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

fn asset_urls(item: &NormalizedItem, intent: DownloadResourceIntent) -> Vec<String> {
    match intent {
        DownloadResourceIntent::Cover => item.cover_url.iter().cloned().collect(),
        DownloadResourceIntent::Video
        | DownloadResourceIntent::Audio
        | DownloadResourceIntent::Subtitle
        | DownloadResourceIntent::Danmaku
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

fn stream_quality_rank(quality: crate::model::StreamQuality) -> u32 {
    match quality {
        crate::model::StreamQuality::Best => u32::MAX,
        crate::model::StreamQuality::Quality(value) => value,
    }
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
