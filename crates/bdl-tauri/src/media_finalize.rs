use bdl_core::muxer::{supports_cover_embedding, supports_subtitle_embedding};
use bdl_core::queue::{DownloadResource, DownloadResourceIntent, DownloadTask, ResourceStatus};
use bdl_core::{BdlError, BdlResult};
use std::path::PathBuf;
use tokio::fs;

pub(crate) struct MuxAttachmentSelection {
    pub(crate) cover_path: Option<PathBuf>,
    pub(crate) subtitle_paths: Vec<PathBuf>,
    pub(crate) warnings: Vec<String>,
}

pub(crate) fn select_mux_attachments(
    task: &DownloadTask,
    embed_cover: bool,
    embed_subtitles: bool,
) -> MuxAttachmentSelection {
    let mut selection = MuxAttachmentSelection {
        cover_path: None,
        subtitle_paths: Vec::new(),
        warnings: Vec::new(),
    };

    if embed_cover && task_has_resource_intent(task, DownloadResourceIntent::Cover) {
        match completed_resource_by_intent(task, DownloadResourceIntent::Cover) {
            Some(resource)
                if supports_cover_embedding(&task.output_path, &resource.target_path) =>
            {
                selection.cover_path = Some(resource.target_path.clone());
            }
            Some(_) => selection
                .warnings
                .push("跳过封面嵌入：当前封面格式或封装格式不支持。".to_owned()),
            None => selection
                .warnings
                .push("跳过封面嵌入：没有已下载的封面文件。".to_owned()),
        }
    }

    if embed_subtitles && task_has_resource_intent(task, DownloadResourceIntent::Subtitle) {
        match completed_resource_by_intent(task, DownloadResourceIntent::Subtitle) {
            Some(resource)
                if supports_subtitle_embedding(&task.output_path, &resource.target_path) =>
            {
                selection.subtitle_paths.push(resource.target_path.clone());
            }
            Some(_) => selection
                .warnings
                .push("跳过字幕嵌入：当前字幕格式或封装格式不支持。".to_owned()),
            None => selection
                .warnings
                .push("跳过字幕嵌入：没有已下载的字幕文件。".to_owned()),
        }
    }

    selection
}

pub(crate) fn completed_resource_by_intent(
    task: &DownloadTask,
    intent: DownloadResourceIntent,
) -> Option<&DownloadResource> {
    task.resources.iter().find(|resource| {
        resource.intent == intent
            && resource.status == ResourceStatus::Completed
            && resource.target_path.is_file()
    })
}

pub(crate) async fn write_nfo(task: &DownloadTask, resource: &DownloadResource) -> BdlResult<()> {
    if let Some(parent) = resource.target_path.parent() {
        fs::create_dir_all(parent).await.map_err(BdlError::from)?;
    }
    fs::write(&resource.target_path, nfo_content(task))
        .await
        .map_err(BdlError::from)?;
    Ok(())
}

pub(crate) fn task_has_resource_intent(
    task: &DownloadTask,
    intent: DownloadResourceIntent,
) -> bool {
    task.resources
        .iter()
        .any(|resource| resource.intent == intent)
}

fn nfo_content(task: &DownloadTask) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<movie>\n\
  <title>{}</title>\n\
  <source>{}</source>\n\
  <filename>{}</filename>\n\
</movie>\n",
        escape_xml(&task.title),
        escape_xml(&task.source_id),
        escape_xml(&task.output_path.to_string_lossy())
    )
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::{nfo_content, select_mux_attachments};
    use bdl_core::queue::{
        DownloadResource, DownloadResourceIntent, DownloadResourceKind, DownloadTask,
        DownloadTaskMediaSelection, ResourceStatus, TaskStatus,
    };
    use std::path::PathBuf;

    #[test]
    fn nfo_content_escapes_xml_sensitive_fields() {
        let task = task(Vec::new());

        let nfo = nfo_content(&task);

        assert!(nfo.contains("A&amp;B &lt;C&gt;"));
        assert!(nfo.contains("video:&quot;source&quot;"));
    }

    #[test]
    fn requested_but_unavailable_attachments_produce_specific_warnings() {
        let resources = vec![
            resource(DownloadResourceIntent::Cover),
            resource(DownloadResourceIntent::Subtitle),
        ];

        let selection = select_mux_attachments(&task(resources), true, true);

        assert_eq!(selection.warnings.len(), 2);
        assert!(selection.warnings[0].contains("封面"));
        assert!(selection.warnings[1].contains("字幕"));
        assert!(selection.cover_path.is_none());
        assert!(selection.subtitle_paths.is_empty());
    }

    fn task(resources: Vec<DownloadResource>) -> DownloadTask {
        DownloadTask {
            id: "task:fixture".to_owned(),
            title: "A&B <C>".to_owned(),
            source_id: "video:\"source\"".to_owned(),
            status: TaskStatus::Completed,
            resources,
            output_path: PathBuf::from("downloads/A&B <C>.mp4"),
            refresh_intent: None,
            media_selection: DownloadTaskMediaSelection::default(),
            scheduled_at: None,
            speed_limit_bytes_per_second: None,
        }
    }

    fn resource(intent: DownloadResourceIntent) -> DownloadResource {
        DownloadResource {
            id: format!("resource:{intent:?}"),
            kind: DownloadResourceKind::Asset,
            intent,
            current_urls: vec!["https://example.invalid/asset".to_owned()],
            headers: Vec::new(),
            target_path: PathBuf::from("missing.asset"),
            temp_path: PathBuf::from("missing.asset.bdlpart"),
            status: ResourceStatus::Pending,
        }
    }
}
