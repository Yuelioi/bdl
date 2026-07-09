use std::ffi::OsString;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{BdlError, BdlResult};
use crate::ids::PartId;
use crate::model::{MediaKind, MediaStream, NormalizedItem, NormalizedPart, NormalizedSourceTree};
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
}

impl DownloadOptions {
    pub fn new(output_dir: PathBuf) -> Self {
        Self {
            output_dir,
            archive_mode: ArchiveMode::Fast,
            output_extension: "mp4".to_owned(),
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
    selected_part_ids
        .iter()
        .map(|part_id| {
            let selected = find_part(tree, part_id).ok_or_else(|| BdlError::Planning {
                message: format!(
                    "选中的分 P `{}` 未加载，请重新解析后再创建下载任务。",
                    part_id.0
                ),
            })?;
            plan_part(tree, selected.item, selected.part, options)
        })
        .collect()
}

struct SelectedPart<'a> {
    item: &'a NormalizedItem,
    part: &'a NormalizedPart,
}

fn find_part<'a>(tree: &'a NormalizedSourceTree, part_id: &PartId) -> Option<SelectedPart<'a>> {
    tree.groups
        .iter()
        .flat_map(|group| &group.items)
        .find_map(|item| {
            item.parts
                .iter()
                .find(|part| &part.id == part_id)
                .map(|part| SelectedPart { item, part })
        })
}

fn plan_part(
    tree: &NormalizedSourceTree,
    item: &NormalizedItem,
    part: &NormalizedPart,
    options: &DownloadOptions,
) -> BdlResult<DownloadTask> {
    let task_id = format!("task:{}:{}", tree.source.id.0, part.id.0);
    let title = task_title(item, part);
    let safe_base = sanitize_path_segment(&title);
    let output_path = options
        .output_dir
        .join(format!("{safe_base}.{}", options.output_extension));

    let video = select_stream(part, MediaKind::Video).ok_or_else(|| BdlError::Planning {
        message: format!("`{}` 缺少视频流，请重新解析后再试。", part.title),
    })?;
    let audio = select_stream(part, MediaKind::Audio).ok_or_else(|| BdlError::Planning {
        message: format!("`{}` 缺少音频流，请重新解析后再试。", part.title),
    })?;

    let mut resources = vec![
        media_resource(
            &task_id,
            &options.output_dir,
            &safe_base,
            DownloadResourceIntent::Video,
            DownloadResourceKind::Video,
            video,
        )?,
        media_resource(
            &task_id,
            &options.output_dir,
            &safe_base,
            DownloadResourceIntent::Audio,
            DownloadResourceKind::Audio,
            audio,
        )?,
    ];

    if options.archive_mode == ArchiveMode::CompleteArchive {
        resources.extend(complete_archive_resources(
            &task_id,
            &options.output_dir,
            &safe_base,
            item,
        ));
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
    output_dir: &Path,
    safe_base: &str,
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
    let target_path = output_dir.join(format!("{safe_base}.{suffix}.m4s"));
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
    output_dir: &Path,
    safe_base: &str,
    item: &NormalizedItem,
) -> Vec<DownloadResource> {
    [
        DownloadResourceIntent::Cover,
        DownloadResourceIntent::Subtitle,
        DownloadResourceIntent::Danmaku,
        DownloadResourceIntent::Nfo,
    ]
    .into_iter()
    .map(|intent| asset_resource(task_id, output_dir, safe_base, item, intent))
    .collect()
}

fn asset_resource(
    task_id: &str,
    output_dir: &Path,
    safe_base: &str,
    item: &NormalizedItem,
    intent: DownloadResourceIntent,
) -> DownloadResource {
    let suffix = resource_suffix(intent);
    let target_path = output_dir.join(format!("{safe_base}.{suffix}"));
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

fn sanitize_path_segment(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
            {
                '_'
            } else {
                ch
            }
        })
        .collect();

    let trimmed = sanitized.trim().trim_matches('.').to_owned();
    if trimmed.is_empty() {
        "untitled".to_owned()
    } else {
        trimmed
    }
}
