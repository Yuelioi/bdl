use bdl_core::error::IoResultExt;
use bdl_core::muxer::{supports_cover_embedding, supports_subtitle_embedding};
use bdl_core::queue::{DownloadResource, DownloadResourceIntent, DownloadTask, ResourceStatus};
use bdl_core::subtitles::{SubtitleFormat, bilibili_json_to_ass, bilibili_json_to_srt};
use bdl_core::{BdlError, BdlResult};
use std::path::PathBuf;
use tokio::fs;

pub(crate) struct MuxAttachmentSelection {
    pub(crate) cover_path: Option<PathBuf>,
    pub(crate) subtitle_paths: Vec<PathBuf>,
    pub(crate) warnings: Vec<String>,
}

pub(crate) async fn select_mux_attachments(
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
        let resources = task
            .resources
            .iter()
            .filter(|resource| {
                resource.intent == DownloadResourceIntent::Subtitle
                    && resource.status == ResourceStatus::Completed
                    && resource.target_path.is_file()
            })
            .collect::<Vec<_>>();
        if resources.is_empty() {
            selection
                .warnings
                .push("跳过字幕嵌入：没有已下载的字幕文件。".to_owned());
        }
        for resource in resources {
            let converted_path = sidecar_output_path(task, resource);
            let path = if converted_path.is_file() {
                &converted_path
            } else {
                &resource.target_path
            };
            if supports_subtitle_embedding(&task.output_path, path) {
                selection.subtitle_paths.push(path.clone());
            } else if path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
                && supports_subtitle_embedding(&task.output_path, &path.with_extension("srt"))
            {
                match convert_subtitle(path).await {
                    Ok(srt_path) => selection.subtitle_paths.push(srt_path),
                    Err(error) => selection
                        .warnings
                        .push(format!("跳过字幕嵌入：JSON 转 SRT 失败：{error}")),
                }
            } else {
                selection
                    .warnings
                    .push("跳过字幕嵌入：当前字幕格式或封装格式不支持。".to_owned());
            }
        }
    }

    selection
}

pub(crate) fn sidecar_output_path(task: &DownloadTask, resource: &DownloadResource) -> PathBuf {
    if let Some(workflow) = &task.media_selection.workflow {
        if let Some(artifact) = task.media_selection.artifacts.iter().find(|artifact| {
            artifact.resource_id.as_ref() == Some(&resource.id) && !artifact.original
        }) {
            return artifact.path.clone();
        }
        let format = match resource.intent {
            DownloadResourceIntent::Subtitle
                if workflow.subtitles.save || workflow.subtitles.embed =>
            {
                workflow.subtitles.format.as_str()
            }
            DownloadResourceIntent::Danmaku if workflow.danmaku.save => {
                workflow.danmaku.format.as_str()
            }
            _ => "original",
        };
        return if format == "original" {
            resource.target_path.clone()
        } else {
            resource.target_path.with_extension(format)
        };
    }
    let processing = task.media_selection.processing.unwrap_or_default();
    let extension = match resource.intent {
        DownloadResourceIntent::Subtitle => {
            processing.subtitle_format.map(|format| format.extension())
        }
        DownloadResourceIntent::Danmaku => {
            processing.danmaku_format.map(|format| format.extension())
        }
        _ => None,
    };
    extension.map_or_else(
        || resource.target_path.clone(),
        |ext| resource.target_path.with_extension(ext),
    )
}

pub(crate) async fn prepare_sidecars(task: &DownloadTask) -> BdlResult<()> {
    for resource in task
        .resources
        .iter()
        .filter(|resource| resource.status == ResourceStatus::Completed)
        .filter(|resource| {
            matches!(
                resource.intent,
                DownloadResourceIntent::Subtitle | DownloadResourceIntent::Danmaku
            )
        })
    {
        let output = sidecar_output_path(task, resource);
        let source = &resource.target_path;
        if output == *source || (!source.is_file() && output.is_file()) || !source.is_file() {
            continue;
        }
        if source.extension() == output.extension() {
            fs::copy(source, &output)
                .await
                .path_context("copy_sidecar", &output)?;
            continue;
        }
        let raw = fs::read_to_string(source)
            .await
            .path_context("read_sidecar", source)?;
        let content = match resource.intent {
            DownloadResourceIntent::Subtitle
                if source.extension().is_some_and(|ext| ext == "json") =>
            {
                let converted = match task
                    .media_selection
                    .workflow
                    .as_ref()
                    .map(|workflow| {
                        if workflow.subtitles.format == "ass" {
                            SubtitleFormat::Ass
                        } else {
                            SubtitleFormat::Srt
                        }
                    })
                    .or_else(|| {
                        task.media_selection
                            .processing
                            .and_then(|options| options.subtitle_format)
                    })
                    .unwrap_or_default()
                {
                    SubtitleFormat::Srt => bilibili_json_to_srt(&raw),
                    SubtitleFormat::Ass => bilibili_json_to_ass(&raw),
                };
                converted.map_err(|error| BdlError::Planning {
                    message: format!("字幕转换失败：{error}"),
                })?
            }
            DownloadResourceIntent::Danmaku
                if source.extension().is_some_and(|ext| ext == "xml") =>
            {
                match output.extension().and_then(|ext| ext.to_str()) {
                    Some("html") => bdl_core::danmaku::xml_to_html(&raw)?,
                    Some("srt") => bdl_core::danmaku::xml_to_subtitle(
                        &raw,
                        bdl_core::danmaku::DanmakuFormat::Srt,
                    )?,
                    Some("ass") => bdl_core::danmaku::xml_to_subtitle(
                        &raw,
                        bdl_core::danmaku::DanmakuFormat::Ass,
                    )?,
                    _ => {
                        return Err(BdlError::Planning {
                            message: "弹幕转换格式无效。".into(),
                        });
                    }
                }
            }
            _ => {
                return Err(BdlError::Planning {
                    message: format!("无法将 {} 转换为所选格式。", source.display()),
                });
            }
        };
        fs::write(&output, content)
            .await
            .path_context("write_sidecar", &output)?;
    }
    Ok(())
}

pub(crate) async fn cleanup_converted_sources(task: &DownloadTask) -> BdlResult<()> {
    if !fs::metadata(&task.output_path)
        .await
        .is_ok_and(|meta| meta.is_file() && meta.len() > 0)
    {
        return Ok(());
    }
    for resource in &task.resources {
        let output = sidecar_output_path(task, resource);
        if output != resource.target_path && output.is_file() && resource.target_path.is_file() {
            fs::remove_file(&resource.target_path)
                .await
                .path_context("cleanup_converted_source", &resource.target_path)?;
        }
    }
    Ok(())
}

async fn convert_subtitle(
    path: &std::path::Path,
) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let json = fs::read_to_string(path)
        .await
        .path_context("read_subtitle", path)?;
    let srt = bilibili_json_to_srt(&json)?;
    let output = path.with_extension("srt");
    fs::write(&output, srt).await?;
    Ok(output)
}

pub(crate) async fn cleanup_embedded_sources(
    task: &DownloadTask,
    selection: &MuxAttachmentSelection,
) -> Vec<String> {
    let Some(keep) = task
        .media_selection
        .processing
        .and_then(|options| options.archive_assets)
    else {
        return Vec::new();
    };
    // Only remove temporary embedding sources after a nonempty final file exists.
    if !fs::metadata(&task.output_path)
        .await
        .is_ok_and(|meta| meta.is_file() && meta.len() > 0)
    {
        return Vec::new();
    }
    let mut warnings = Vec::new();
    for resource in &task.resources {
        let path = &resource.target_path;
        let converted = sidecar_output_path(task, resource);
        let legacy_srt = path.with_extension("srt");
        let paths = match resource.intent {
            DownloadResourceIntent::Cover
                if !keep.cover && selection.cover_path.as_ref() == Some(path) =>
            {
                vec![path]
            }
            DownloadResourceIntent::Subtitle
                if !keep.subtitles
                    && selection.subtitle_paths.iter().any(|subtitle| {
                        subtitle == path || subtitle == &converted || subtitle == &legacy_srt
                    }) =>
            {
                if selection.subtitle_paths.contains(&converted) && path != &converted {
                    vec![path, &converted]
                } else if selection.subtitle_paths.contains(&legacy_srt) && path != &legacy_srt {
                    vec![path, &legacy_srt]
                } else {
                    vec![path]
                }
            }
            _ => continue,
        };
        for path in paths {
            if let Err(error) = fs::remove_file(path).await
                && error.kind() != std::io::ErrorKind::NotFound
            {
                warnings.push(format!(
                    "嵌入成功，清理临时素材失败：{}",
                    bdl_core::error::file_io_error("cleanup_embedded_source", path, error)
                ));
            }
        }
    }
    warnings
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
        fs::create_dir_all(parent)
            .await
            .path_context("create_nfo_directory", parent)?;
    }
    fs::write(&resource.target_path, nfo_content(task)?)
        .await
        .path_context("write_nfo", &resource.target_path)?;
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

fn nfo_content(task: &DownloadTask) -> BdlResult<String> {
    let options = task
        .media_selection
        .workflow
        .as_ref()
        .map(|w| w.nfo_options.clone())
        .unwrap_or_default();
    let cover = task
        .media_selection
        .artifacts
        .iter()
        .find(|a| a.intent == Some(DownloadResourceIntent::Cover))
        .and_then(|a| a.path.file_name())
        .map(|v| v.to_string_lossy().into_owned());
    bdl_core::nfo::render(
        &options,
        &task.media_selection.nfo_metadata,
        &task.title,
        cover.as_deref(),
    )
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

        let nfo = nfo_content(&task).unwrap();

        assert!(nfo.contains("A&amp;B &lt;C&gt;"));
        assert!(!nfo.contains("<source>"));
    }

    #[tokio::test]
    async fn requested_but_unavailable_attachments_produce_specific_warnings() {
        let resources = vec![
            resource(DownloadResourceIntent::Cover),
            resource(DownloadResourceIntent::Subtitle),
        ];

        let selection = select_mux_attachments(&task(resources), true, true).await;

        assert_eq!(selection.warnings.len(), 2);
        assert!(selection.warnings[0].contains("封面"));
        assert!(selection.warnings[1].contains("字幕"));
        assert!(selection.cover_path.is_none());
        assert!(selection.subtitle_paths.is_empty());
    }

    #[tokio::test]
    async fn mkv_selects_cover_and_converts_json_without_changing_downloaded_payload() {
        let dir = std::env::temp_dir().join(format!("bdl-attachments-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        let json = r#"{"body":[{"from":0.38,"to":6.22,"content":"测试字幕"}]}"#;
        let mut cover = resource(DownloadResourceIntent::Cover);
        cover.target_path = dir.join("image.jpg");
        cover.status = ResourceStatus::Completed;
        tokio::fs::write(&cover.target_path, b"fixture image")
            .await
            .unwrap();
        let mut subtitle = resource(DownloadResourceIntent::Subtitle);
        subtitle.target_path = dir.join("download.subtitle.json");
        subtitle.status = ResourceStatus::Completed;
        tokio::fs::write(&subtitle.target_path, json).await.unwrap();
        let mut task = task(vec![cover.clone(), subtitle.clone()]);
        task.output_path = dir.join("output.mkv");

        let disabled = select_mux_attachments(&task, false, false).await;
        assert!(disabled.cover_path.is_none());
        assert!(disabled.subtitle_paths.is_empty());
        assert!(!subtitle.target_path.with_extension("srt").exists());

        let selection = select_mux_attachments(&task, true, true).await;
        assert!(selection.warnings.is_empty());
        assert_eq!(selection.cover_path, Some(cover.target_path));
        assert_eq!(
            selection.subtitle_paths,
            vec![subtitle.target_path.with_extension("srt")]
        );
        assert_eq!(
            tokio::fs::read_to_string(&selection.subtitle_paths[0])
                .await
                .unwrap(),
            "1\n00:00:00,380 --> 00:00:06,220\n测试字幕\n\n"
        );
        assert_eq!(
            tokio::fs::read_to_string(&subtitle.target_path)
                .await
                .unwrap(),
            json
        );

        // A stale converted sidecar must not hide a malformed current payload.
        tokio::fs::write(&subtitle.target_path, "broken json")
            .await
            .unwrap();
        let broken = select_mux_attachments(&task, true, true).await;
        assert!(broken.subtitle_paths.is_empty());
        assert!(broken.cover_path.is_some());
        assert!(broken.warnings[0].contains("JSON 转 SRT 失败"));
        tokio::fs::remove_dir_all(dir).await.unwrap();
    }

    #[tokio::test]
    async fn embedded_source_cleanup_respects_explicit_sidecars_and_legacy_tasks() {
        use bdl_core::{planner::ArchiveAssetSelection, queue::TaskProcessingOptions};
        let dir =
            std::env::temp_dir().join(format!("bdl-embedding-cleanup-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        let mut cover = resource(DownloadResourceIntent::Cover);
        cover.target_path = dir.join("cover.jpg");
        cover.status = ResourceStatus::Completed;
        let mut subtitle = resource(DownloadResourceIntent::Subtitle);
        subtitle.target_path = dir.join("subtitle.json");
        subtitle.status = ResourceStatus::Completed;
        tokio::fs::write(&cover.target_path, "image").await.unwrap();
        tokio::fs::write(
            &subtitle.target_path,
            r#"{"body":[{"from":0,"to":1,"content":"测试"}]}"#,
        )
        .await
        .unwrap();
        let mut task = task(vec![cover.clone(), subtitle.clone()]);
        task.output_path = dir.join("output.mkv");
        let selection = select_mux_attachments(&task, true, true).await;
        // A legacy task has no explicit standalone-file policy: retain its files.
        tokio::fs::write(&task.output_path, "nonempty muxed fixture")
            .await
            .unwrap();
        assert!(
            super::cleanup_embedded_sources(&task, &selection)
                .await
                .is_empty()
        );
        assert!(cover.target_path.exists());
        task.media_selection.processing = Some(TaskProcessingOptions {
            embed_cover: true,
            embed_subtitles: true,
            archive_assets: Some(ArchiveAssetSelection {
                cover: true,
                ..ArchiveAssetSelection::none()
            }),
            ..Default::default()
        });
        // A missing final output must never trigger source deletion.
        tokio::fs::remove_file(&task.output_path).await.unwrap();
        super::cleanup_embedded_sources(&task, &selection).await;
        assert!(subtitle.target_path.exists());
        tokio::fs::write(&task.output_path, "nonempty muxed fixture")
            .await
            .unwrap();
        assert!(
            super::cleanup_embedded_sources(&task, &selection)
                .await
                .is_empty()
        );
        assert!(cover.target_path.exists());
        assert!(!subtitle.target_path.exists());
        assert!(!subtitle.target_path.with_extension("srt").exists());
        tokio::fs::remove_dir_all(dir).await.unwrap();
    }

    fn task(resources: Vec<DownloadResource>) -> DownloadTask {
        DownloadTask {
            id: "task:fixture".to_owned(),
            title: "A&B <C>".to_owned(),
            source_id: "video:\"source\"".to_owned(),
            status: TaskStatus::Completed,
            resources,
            output_path: PathBuf::from("downloads/A&B <C>.mp4"),
            export_target: None,
            refresh_intent: None,
            media_selection: DownloadTaskMediaSelection::default(),
            scheduled_at: None,
            speed_limit_bytes_per_second: None,
        }
    }

    #[tokio::test]
    async fn danmaku_subtitle_formats_write_real_outputs_and_recover_after_source_cleanup() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.task")
            .join(format!("danmaku-subtitles-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        for format in [
            bdl_core::danmaku::DanmakuFormat::Srt,
            bdl_core::danmaku::DanmakuFormat::Ass,
        ] {
            let mut resource = resource(DownloadResourceIntent::Danmaku);
            resource.target_path = dir.join(format!("{}.xml", format.extension()));
            resource.status = ResourceStatus::Completed;
            tokio::fs::write(
                &resource.target_path,
                r#"<i><d p="2.5,1,25,16777215">中文弹幕</d></i>"#,
            )
            .await
            .unwrap();
            let mut task = task(vec![resource.clone()]);
            task.media_selection.processing = Some(bdl_core::queue::TaskProcessingOptions {
                danmaku_format: Some(format),
                ..Default::default()
            });
            let restored: DownloadTask =
                serde_json::from_str(&serde_json::to_string(&task).unwrap()).unwrap();
            super::prepare_sidecars(&restored).await.unwrap();
            let output = super::sidecar_output_path(&restored, &resource);
            let content = tokio::fs::read_to_string(&output).await.unwrap();
            assert!(content.contains("中文弹幕"));
            assert!(
                content.contains(if format == bdl_core::danmaku::DanmakuFormat::Srt {
                    "00:00:02,500 --> 00:00:06,500"
                } else {
                    "Dialogue: 0,0:00:02.50,0:00:10.50"
                })
            );
            assert!(!content.contains("<i>"));
            assert!(resource.target_path.is_file());
            tokio::fs::remove_file(&resource.target_path).await.unwrap();
            super::prepare_sidecars(&restored).await.unwrap();
            assert_eq!(tokio::fs::read_to_string(&output).await.unwrap(), content);
        }
        tokio::fs::remove_dir_all(&dir).await.unwrap();
    }

    #[tokio::test]
    async fn selected_formats_survive_reload_convert_embed_and_export_without_raw_sidecars() {
        let dir = std::env::temp_dir().join(format!("bdl-sidecars-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        let mut subtitle = resource(DownloadResourceIntent::Subtitle);
        subtitle.target_path = dir.join("字幕.json");
        subtitle.status = ResourceStatus::Completed;
        let mut danmaku = resource(DownloadResourceIntent::Danmaku);
        danmaku.target_path = dir.join("弹幕.xml");
        danmaku.status = ResourceStatus::Completed;
        tokio::fs::write(
            &subtitle.target_path,
            r#"{"body":[{"from":1,"to":2,"content":"中文"}]}"#,
        )
        .await
        .unwrap();
        tokio::fs::write(
            &danmaku.target_path,
            "<i><d p=\"1,1,25,16777215\">弹幕</d></i>",
        )
        .await
        .unwrap();
        let mut task = task(vec![subtitle.clone(), danmaku.clone()]);
        task.output_path = dir.join("output.mkv");
        task.media_selection.processing = Some(bdl_core::queue::TaskProcessingOptions {
            subtitle_format: Some(bdl_core::subtitles::SubtitleFormat::Ass),
            danmaku_format: Some(bdl_core::danmaku::DanmakuFormat::Html),
            ..Default::default()
        });
        let restored: DownloadTask =
            serde_json::from_str(&serde_json::to_string(&task).unwrap()).unwrap();
        super::prepare_sidecars(&restored).await.unwrap();
        let selected = select_mux_attachments(&restored, false, true).await;
        assert_eq!(
            selected.subtitle_paths,
            vec![subtitle.target_path.with_extension("ass")]
        );
        assert!(selected.warnings.is_empty());
        assert_eq!(
            super::sidecar_output_path(&restored, &danmaku),
            danmaku.target_path.with_extension("html")
        );
        super::cleanup_converted_sources(&restored).await.unwrap();
        assert!(subtitle.target_path.exists());
        tokio::fs::write(&restored.output_path, "completed media")
            .await
            .unwrap();
        super::cleanup_converted_sources(&restored).await.unwrap();
        assert!(!subtitle.target_path.exists());
        assert!(!danmaku.target_path.exists());
        // Recovery keeps the chosen files even after temporary JSON/XML are gone.
        super::prepare_sidecars(&restored).await.unwrap();
        assert!(subtitle.target_path.with_extension("ass").exists());
        assert!(danmaku.target_path.with_extension("html").exists());
        tokio::fs::remove_dir_all(dir).await.unwrap();
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
