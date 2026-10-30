//! Executes the immutable recipe and verifies every retained artifact before cleanup.
use crate::media_mux::MediaMuxBackend;
use bdl_core::error::IoResultExt;
use bdl_core::muxer::MuxRequest;
use bdl_core::queue::{DownloadResourceIntent as Intent, DownloadTask};
use bdl_core::{BdlError, BdlResult};
use std::path::{Path, PathBuf};
use tokio::fs;

pub(crate) async fn outputs_complete(task: &DownloadTask) -> BdlResult<bool> {
    if task.media_selection.artifacts.is_empty() {
        return Ok(false);
    }
    for artifact in &task.media_selection.artifacts {
        if !nonempty(&artifact.path).await? {
            return Ok(false);
        }
    }
    Ok(true)
}

async fn nonempty(path: &Path) -> BdlResult<bool> {
    match fs::metadata(path).await {
        Ok(meta) => Ok(meta.is_file() && meta.len() > 0),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).path_context("verify_artifact", path),
    }
}

async fn copy(source: &Path, output: &Path) -> BdlResult<()> {
    if source == output {
        return Ok(());
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .await
            .path_context("create_artifact_directory", parent)?;
    }
    let temporary = output.with_extension(format!(
        "{}.bdl-copy.tmp",
        output.extension().and_then(|e| e.to_str()).unwrap_or("bin")
    ));
    fs::copy(source, &temporary)
        .await
        .path_context("copy_artifact", &temporary)
        .map_err(|error| error.with_context(format!("source={}", source.display())))?;
    if !nonempty(&temporary).await? {
        return Err(BdlError::Planning {
            message: "保存内容为空，已保留原始素材。".into(),
        });
    }
    fs::rename(&temporary, output)
        .await
        .path_context("publish_artifact", output)
        .map_err(|error| error.with_context(format!("source={}", temporary.display())))?;
    Ok(())
}

pub(crate) async fn execute(
    task: &DownloadTask,
    mux: &MediaMuxBackend,
    ffmpeg: Option<PathBuf>,
) -> BdlResult<Vec<String>> {
    let workflow = task
        .media_selection
        .workflow
        .as_ref()
        .ok_or_else(|| BdlError::Planning {
            message: "任务缺少下载流程配置。".into(),
        })?;
    workflow.validate()?;
    let mut warnings = Vec::new();
    for (enabled, intent, label) in [
        (workflow.cover.enabled, Intent::Cover, "封面"),
        (workflow.subtitles.enabled, Intent::Subtitle, "字幕"),
        (workflow.danmaku.enabled, Intent::Danmaku, "弹幕"),
    ] {
        if enabled
            && !task
                .resources
                .iter()
                .any(|resource| resource.intent == intent)
        {
            warnings.push(format!("来源没有提供{label}，已跳过。"));
        }
    }
    for resource in &task.resources {
        if resource.intent == Intent::Nfo {
            crate::media_finalize::write_nfo(task, resource).await?;
        }
    }
    crate::media_finalize::prepare_sidecars(task).await?;
    for artifact in &task.media_selection.artifacts {
        let Some(id) = &artifact.resource_id else {
            continue;
        };
        let resource = task
            .resources
            .iter()
            .find(|resource| &resource.id == id)
            .ok_or_else(|| BdlError::Planning {
                message: "任务产物缺少对应素材。".into(),
            })?;
        if !resource.target_path.is_file() && nonempty(&artifact.path).await? {
            continue;
        }
        if resource.intent == Intent::Audio && !artifact.original {
            mux.mux(
                &MuxRequest {
                    video_path: None,
                    audio_path: Some(resource.target_path.clone()),
                    output_path: artifact.path.clone(),
                    cover_path: None,
                    subtitle_paths: Vec::new(),
                },
                ffmpeg.clone(),
            )
            .await?;
        } else {
            let source = if artifact.original {
                resource.target_path.clone()
            } else {
                crate::media_finalize::sidecar_output_path(task, resource)
            };
            copy(&source, &artifact.path).await?;
        }
    }
    if workflow.has_media_output() {
        let attachments = crate::media_finalize::select_mux_attachments(
            task,
            workflow.cover.enabled && workflow.cover.embed,
            workflow.subtitles.enabled && workflow.subtitles.embed,
        )
        .await;
        let path_for = |intent| {
            task.resources
                .iter()
                .find(|resource| resource.intent == intent)
                .map(|resource| resource.target_path.clone())
        };
        if (workflow.cover.enabled
            && workflow.cover.embed
            && path_for(Intent::Cover).is_some()
            && attachments.cover_path.is_none())
            || (workflow.subtitles.enabled
                && workflow.subtitles.embed
                && path_for(Intent::Subtitle).is_some()
                && attachments.subtitle_paths.is_empty())
        {
            return Err(BdlError::Planning {
                message: attachments.warnings.join("\n"),
            });
        }
        warnings.extend(attachments.warnings);
        mux.mux(
            &MuxRequest {
                video_path: path_for(Intent::Video),
                audio_path: path_for(Intent::Audio),
                output_path: task.output_path.clone(),
                cover_path: attachments.cover_path,
                subtitle_paths: attachments.subtitle_paths,
            },
            ffmpeg,
        )
        .await?;
    }
    if !outputs_complete(task).await? {
        return Err(BdlError::Planning {
            message: "部分输出文件未生成或为空，已保留素材以便重试。".into(),
        });
    }
    Ok(warnings)
}

pub(crate) async fn cleanup(task: &DownloadTask) -> Vec<String> {
    let mut warnings = Vec::new();
    if !task.media_selection.outputs_verified {
        return warnings;
    }
    match outputs_complete(task).await {
        Ok(true) => {}
        Ok(false) => return warnings,
        Err(error) => {
            warnings.push(format!("清理前无法验证成品，已保留素材：{error}"));
            return warnings;
        }
    }
    // The verified manifest is persisted before removing any input.
    let retained = task
        .media_selection
        .artifacts
        .iter()
        .map(|a| &a.path)
        .collect::<std::collections::HashSet<_>>();
    for resource in &task.resources {
        for path in [
            resource.target_path.clone(),
            crate::media_finalize::sidecar_output_path(task, resource),
        ] {
            if retained.contains(&path) {
                continue;
            }
            match fs::remove_file(&path).await {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => warnings.push(format!(
                    "输出已生成，临时素材清理失败：{}",
                    bdl_core::error::file_io_error("cleanup_source", &path, error)
                )),
            }
        }
    }
    warnings
}

/// Persist each Android export receipt before starting the next file.
pub(crate) async fn export_manifest(
    mut task: DownloadTask,
    storage: crate::mobile_storage::MobileStorage,
    mut persist: impl FnMut(&DownloadTask) -> BdlResult<()>,
) -> BdlResult<DownloadTask> {
    use bdl_core::queue::{DownloadExportTarget, portable_relative_path, retarget_task_path};
    let Some(DownloadExportTarget::DocumentTree {
        tree_uri,
        relative_path,
        duplicate_naming_strategy,
        document_uri,
    }) = task.export_target.clone()
    else {
        return Ok(task);
    };
    let already_exported = document_uri.is_some();
    let primary = if let Some(uri) = document_uri {
        crate::mobile_storage::ExportResult {
            relative_path,
            document_uri: uri,
            outcome: crate::mobile_storage::ExportOutcome::Exported,
        }
    } else {
        let source = task.output_path.clone();
        let target = task.export_target.clone().unwrap();
        let backend = storage.clone();
        tokio::task::spawn_blocking(move || backend.export_file(&source, &target))
            .await
            .map_err(|error| BdlError::Platform {
                message: format!("Android 导出任务异常结束：{error}"),
            })??
    };
    task.export_target = Some(DownloadExportTarget::DocumentTree {
        tree_uri: tree_uri.clone(),
        relative_path: primary.relative_path.clone(),
        duplicate_naming_strategy,
        document_uri: Some(primary.document_uri.clone()),
    });
    for artifact in &mut task.media_selection.artifacts {
        if artifact.path == task.output_path {
            artifact.document_uri = Some(primary.document_uri.clone());
        }
    }
    if !already_exported {
        record_export_ownership(&mut task, &primary);
    }
    persist(&task)?;
    let exported_output = PathBuf::from(primary.relative_path);
    for index in 0..task.media_selection.artifacts.len() {
        if task.media_selection.artifacts[index].document_uri.is_some() {
            continue;
        }
        let source = task.media_selection.artifacts[index].path.clone();
        let relative = retarget_task_path(&source, &task.output_path, &exported_output)?;
        let target = DownloadExportTarget::DocumentTree {
            tree_uri: tree_uri.clone(),
            relative_path: portable_relative_path(&relative)?,
            duplicate_naming_strategy,
            document_uri: None,
        };
        let backend = storage.clone();
        let exported = tokio::task::spawn_blocking(move || backend.export_file(&source, &target))
            .await
            .map_err(|error| BdlError::Platform {
                message: format!("Android 导出任务异常结束：{error}"),
            })??;
        record_export_ownership(&mut task, &exported);
        task.media_selection.artifacts[index].document_uri = Some(exported.document_uri);
        persist(&task)?;
    }
    Ok(task)
}

pub(crate) fn record_export_ownership(
    task: &mut DownloadTask,
    result: &crate::mobile_storage::ExportResult,
) {
    let uris = match result.outcome {
        crate::mobile_storage::ExportOutcome::Exported => {
            &mut task.media_selection.owned_document_uris
        }
        crate::mobile_storage::ExportOutcome::SkippedExisting => {
            &mut task.media_selection.preserved_document_uris
        }
    };
    if !uris.contains(&result.document_uri) {
        uris.push(result.document_uri.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_core::queue::{DownloadResource, DownloadResourceKind, ResourceStatus, TaskStatus};
    use bdl_core::workflow::{DownloadArtifact, DownloadWorkflow};

    async fn subtitle_task(format: &str, retain: bool) -> DownloadTask {
        let dir =
            std::env::temp_dir().join(format!("bdl-workflow-subtitle-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).await.unwrap();
        let source = dir.join("素材.subtitle.json");
        fs::write(
            &source,
            r#"{"body":[{"from":1.25,"to":3.5,"content":"线条与色彩\n中文字幕"}]}"#,
        )
        .await
        .unwrap();
        let mut workflow = bdl_core::workflow::legacy_builtin_presets()
            .remove(2)
            .workflow;
        workflow.subtitles.format = format.into();
        workflow.subtitles.retain_original = retain;
        let output = dir.join(format!(
            "字幕.{}",
            if format == "original" { "json" } else { format }
        ));
        let mut artifacts = vec![DownloadArtifact {
            path: output.clone(),
            resource_id: Some("task:subtitle".into()),
            intent: Some(Intent::Subtitle),
            original: format == "original",
            document_uri: None,
        }];
        if retain && format != "original" {
            artifacts.push(DownloadArtifact {
                path: source.clone(),
                resource_id: Some("task:subtitle".into()),
                intent: Some(Intent::Subtitle),
                original: true,
                document_uri: None,
            });
        }
        DownloadTask {
            id: "task".into(),
            title: "线条与色彩".into(),
            source_id: "fixture".into(),
            status: TaskStatus::Muxing,
            resources: vec![DownloadResource {
                id: "task:subtitle".into(),
                kind: DownloadResourceKind::Asset,
                intent: Intent::Subtitle,
                current_urls: Vec::new(),
                headers: Vec::new(),
                target_path: source.clone(),
                temp_path: source.with_extension("bdlpart"),
                status: ResourceStatus::Completed,
            }],
            output_path: output,
            export_target: None,
            refresh_intent: None,
            scheduled_at: None,
            speed_limit_bytes_per_second: None,
            media_selection: bdl_core::queue::DownloadTaskMediaSelection {
                nfo_metadata: Default::default(),
                workflow: Some(workflow),
                artifacts,
                ..Default::default()
            },
        }
    }

    #[tokio::test]
    #[ignore = "live Bilibili probe: set BDL_DANMAKU_LIVE_DIR, BDL_DANMAKU_LIVE_BVIDS and BDL_TEST_FFMPEG"]
    async fn live_danmaku_videos_download_and_publish_srt_ass() {
        use bdl_core::fetcher::{FetchConfig, Fetcher, ReqwestFetcher};
        use bdl_core::input::ClassifiedInput;
        use bdl_core::planner::{DownloadOptions, plan_selected_parts};
        use bdl_core::resolver::video::VideoResolver;
        use bdl_core::resolver::{ResolveOptions, Resolver};

        let root = PathBuf::from(std::env::var("BDL_DANMAKU_LIVE_DIR").unwrap());
        let bvids = std::env::var("BDL_DANMAKU_LIVE_BVIDS").unwrap();
        let ffmpeg = PathBuf::from(std::env::var("BDL_TEST_FFMPEG").unwrap());
        let fetcher = ReqwestFetcher::with_config(FetchConfig {
            max_retries: 0,
            ..Default::default()
        })
        .unwrap()
        .with_stop_on_restriction();
        let mut receipts = Vec::new();
        for bvid in bvids.split(',') {
            let tree = VideoResolver::new()
                .unwrap()
                .resolve(
                    ClassifiedInput::VideoBvid(bvid.into()),
                    ResolveOptions {
                        fetch_streams: true,
                    },
                )
                .await
                .unwrap();
            let item = &tree.groups[0].items[0];
            let part = &item.parts[0];
            let mut downloaded: Option<DownloadTask> = None;
            for format in ["srt", "ass"] {
                let mut workflow = DownloadWorkflow {
                    quality: "16".into(),
                    codec: "avc".into(),
                    ..Default::default()
                };
                workflow.video.retain_original = true;
                workflow.audio.retain_original = true;
                workflow.danmaku.enabled = true;
                workflow.danmaku.format = format.into();
                workflow.danmaku.retain_original = true;
                let mut options = DownloadOptions::new(root.join(bvid).join(format));
                options.naming_template = "{bvid}.{ext}".into();
                workflow.apply(&mut options).unwrap();
                let mut task = plan_selected_parts(&tree, std::slice::from_ref(&part.id), &options)
                    .unwrap()
                    .remove(0);
                fs::create_dir_all(task.output_path.parent().unwrap())
                    .await
                    .unwrap();
                for resource in &mut task.resources {
                    if let Some(previous) = &downloaded {
                        let source = previous
                            .resources
                            .iter()
                            .find(|old| old.intent == resource.intent)
                            .unwrap();
                        fs::copy(&source.target_path, &resource.target_path)
                            .await
                            .unwrap();
                    } else {
                        fetcher.fetch(resource, None).await.unwrap();
                    }
                    resource.status = ResourceStatus::Completed;
                }
                // Exercise the persisted workflow snapshot, not a standalone converter.
                let mut restored: DownloadTask =
                    serde_json::from_str(&serde_json::to_string(&task).unwrap()).unwrap();
                execute(&restored, &MediaMuxBackend::desktop(), Some(ffmpeg.clone()))
                    .await
                    .unwrap();
                assert!(outputs_complete(&restored).await.unwrap());
                let source = restored
                    .resources
                    .iter()
                    .find(|resource| resource.intent == Intent::Danmaku)
                    .unwrap();
                let output = crate::media_finalize::sidecar_output_path(&restored, source);
                let raw = fs::read_to_string(&source.target_path).await.unwrap();
                let content = fs::read_to_string(&output).await.unwrap();
                assert!(raw.contains("<d "));
                let cues = if format == "srt" {
                    assert!(content.contains(" --> "));
                    content.matches(" --> ").count()
                } else {
                    assert!(content.contains("[Events]"));
                    content.matches("Dialogue: ").count()
                };
                assert!(cues > 0);
                assert!(!content.contains("<i>"));
                restored.media_selection.outputs_verified = true;
                assert!(cleanup(&restored).await.is_empty());
                assert!(source.target_path.is_file());
                receipts.push(serde_json::json!({
                    "bvid": bvid, "title": item.title, "cid": part.cid,
                    "duration_seconds": item.duration_seconds, "format": format,
                    "xml_comments": raw.matches("<d ").count(), "cues": cues,
                    "video": restored.output_path, "subtitle": output, "xml": source.target_path,
                }));
                println!("{bvid} {format}: {cues} cues, media and retained XML verified");
                if downloaded.is_none() {
                    downloaded = Some(restored);
                }
            }
            // A small public sample set; do not make this probe a bulk downloader.
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        fs::write(
            root.join("receipt.json"),
            serde_json::to_vec_pretty(&receipts).unwrap(),
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn subtitle_only_formats_are_real_files_without_ffmpeg_and_retention_is_explicit() {
        for format in ["srt", "ass", "original"] {
            for retain in [false, true] {
                let mut task = subtitle_task(format, retain).await;
                execute(&task, &MediaMuxBackend::unsupported(), None)
                    .await
                    .unwrap();
                let content = fs::read_to_string(&task.output_path).await.unwrap();
                assert!(content.contains("线条与色彩"));
                match format {
                    "srt" => assert!(content.contains("00:00:01,250 --> 00:00:03,500")),
                    "ass" => assert!(content.contains("[Events]")),
                    _ => assert!(serde_json::from_str::<serde_json::Value>(&content).is_ok()),
                }
                assert!(outputs_complete(&task).await.unwrap());
                cleanup(&task).await;
                assert!(
                    task.resources[0].target_path.exists(),
                    "cleanup requires a persisted verification milestone"
                );
                task.media_selection.outputs_verified = true;
                cleanup(&task).await;
                assert_eq!(
                    task.resources[0].target_path.exists(),
                    retain && format != "original"
                );
                assert!(outputs_complete(&task).await.unwrap());
            }
        }
    }

    #[tokio::test]
    async fn interrupted_multi_file_export_resumes_without_duplicate_files() {
        use crate::mobile_storage::{
            ExportOutcome, ExportResult, MobileStorage, MobileStorageBackend,
        };
        use bdl_core::queue::DownloadExportTarget;
        use bdl_core::settings::DocumentTreeDirectory;
        use std::sync::{Arc, Mutex};
        #[derive(Default)]
        struct ExportState {
            calls: Vec<String>,
            interrupted: bool,
        }
        struct Storage(Arc<Mutex<ExportState>>);
        impl MobileStorageBackend for Storage {
            fn save_diagnostic_file(&self, _: &Path) -> BdlResult<bool> {
                unreachable!()
            }
            fn write_clipboard_text(&self, _: &str) -> BdlResult<()> {
                unreachable!()
            }
            fn pick_document_tree(&self) -> BdlResult<DocumentTreeDirectory> {
                unreachable!()
            }
            fn read_clipboard_text(&self) -> BdlResult<String> {
                unreachable!()
            }
            fn save_image_to_gallery(&self, _: &str, _: &str, _: &str) -> BdlResult<String> {
                unreachable!()
            }
            fn open_exported_file(&self, _: &DownloadExportTarget) -> BdlResult<()> {
                unreachable!()
            }
            fn open_export_directory(&self, _: &DownloadExportTarget) -> BdlResult<()> {
                unreachable!()
            }
            fn export_file(
                &self,
                _: &Path,
                target: &DownloadExportTarget,
            ) -> BdlResult<ExportResult> {
                let mut state = self.0.lock().unwrap();
                if state.calls.len() == 1 && !state.interrupted {
                    state.interrupted = true;
                    return Err(BdlError::Platform {
                        message: "export interrupted".into(),
                    });
                }
                let DownloadExportTarget::DocumentTree { relative_path, .. } = target;
                let relative = if state.calls.is_empty() {
                    "collection/字幕 (1).srt".into()
                } else {
                    relative_path.clone()
                };
                state.calls.push(relative.clone());
                Ok(ExportResult {
                    document_uri: format!("content://fixture/{relative}"),
                    relative_path: relative,
                    outcome: ExportOutcome::Exported,
                })
            }
        }
        let state = Arc::new(Mutex::new(ExportState::default()));
        let backend = MobileStorage::from_backend(Storage(state.clone()));
        let mut task = subtitle_task("srt", true).await;
        let source = task.output_path.with_file_name("字幕.subtitle.json");
        fs::rename(&task.resources[0].target_path, &source)
            .await
            .unwrap();
        task.resources[0].target_path = source.clone();
        task.media_selection.artifacts[1].path = source;
        task.export_target = Some(DownloadExportTarget::DocumentTree {
            tree_uri: "content://fixture/tree".into(),
            relative_path: "collection/字幕.srt".into(),
            duplicate_naming_strategy: bdl_core::naming::DuplicateNamingStrategy::AppendSuffix,
            document_uri: None,
        });
        let mut persisted = Vec::new();
        assert!(
            export_manifest(task, backend.clone(), |snapshot| {
                persisted.push(snapshot.clone());
                Ok(())
            })
            .await
            .is_err()
        );
        let checkpoint = persisted.last().unwrap().clone();
        assert!(
            checkpoint.media_selection.artifacts[0]
                .document_uri
                .is_some()
        );
        assert!(
            checkpoint.media_selection.artifacts[1]
                .document_uri
                .is_none()
        );
        let result = export_manifest(checkpoint, backend, |snapshot| {
            persisted.push(snapshot.clone());
            Ok(())
        })
        .await
        .unwrap();
        assert!(
            result
                .media_selection
                .artifacts
                .iter()
                .all(|a| a.document_uri.is_some())
        );
        assert_eq!(
            state.lock().unwrap().calls,
            [
                "collection/字幕 (1).srt",
                "collection/字幕 (1).subtitle.json"
            ]
        );
    }

    #[tokio::test]
    async fn failed_conversion_preserves_sources_and_cannot_be_recovered_as_complete() {
        let task = subtitle_task("srt", false).await;
        fs::write(&task.resources[0].target_path, "invalid json")
            .await
            .unwrap();
        assert!(
            execute(&task, &MediaMuxBackend::unsupported(), None)
                .await
                .is_err()
        );
        assert!(task.resources[0].target_path.exists());
        assert!(!outputs_complete(&task).await.unwrap());
        assert!(
            !crate::commands::has_recoverable_completed_output(&task)
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn existing_unverified_outputs_do_not_skip_requested_work() {
        let task = subtitle_task("srt", false).await;
        fs::write(&task.output_path, "preexisting partial output")
            .await
            .unwrap();
        assert!(
            !crate::commands::has_recoverable_completed_output(&task)
                .await
                .unwrap()
        );
    }

    #[test]
    fn empty_and_incompatible_recipes_are_rejected() {
        let mut workflow = DownloadWorkflow::default();
        workflow.video.enabled = false;
        workflow.audio.enabled = false;
        assert!(workflow.validate().is_err());
        workflow.subtitles.enabled = true;
        workflow.subtitles.embed = true;
        assert!(workflow.validate().is_err());
    }

    #[tokio::test]
    #[ignore = "requires BDL_WORKFLOW_FIXTURE_DIR with generated media and BDL_TEST_FFMPEG"]
    async fn real_media_recipes_publish_every_planned_output() {
        use bdl_core::ids::PartId;
        use bdl_core::planner::{DownloadOptions, plan_selected_parts};
        let root = PathBuf::from(std::env::var_os("BDL_WORKFLOW_FIXTURE_DIR").unwrap());
        let ffmpeg = PathBuf::from(std::env::var_os("BDL_TEST_FFMPEG").unwrap());
        let stream = |kind, codec, quality| {
            serde_json::json!({
                "id": kind, "kind": kind, "quality": { "kind": "quality", "value": quality },
                "codec": codec, "bandwidth": 1024, "urls": ["https://example.invalid/media"],
                "headers": [], "acquired_at": chrono::Utc::now()
            })
        };
        let asset = |kind, format| {
            serde_json::json!({
                "kind": kind, "format": format, "fetch_policy": "on_demand",
                "urls": ["https://example.invalid/asset"], "headers": []
            })
        };
        let tree = serde_json::from_value(serde_json::json!({
            "source": { "id": "source:fixture", "kind": "video", "input": "BVfixture", "title": "配方验证", "loaded_count": 1, "total_count": 1, "has_more": false },
            "groups": [{ "id": "group:fixture", "kind": "video", "title": "配方验证", "page": null, "items": [{
                "id": "item:fixture", "title": "线条与色彩", "owner_name": "fixture", "cover_url": null, "duration_seconds": 2,
                "parts": [{ "id": "part:fixture", "title": "线条与色彩", "aid": 1, "bvid": "BVfixture", "cid": 1,
                    "streams": [stream("video", "avc", 80), stream("audio", "unknown", 30280)],
                    "assets": [asset("cover", "jpg"), asset("subtitle", "json"), asset("danmaku", "xml")]
                }]
            }] }]
        })).unwrap();
        let presets = bdl_core::workflow::legacy_builtin_presets();
        let mut raw = presets[1].workflow.clone();
        raw.audio.format = "m4s".into();
        let mut mp3 = presets[1].workflow.clone();
        mp3.audio.retain_original = true;
        let mut archive = presets[5].workflow.clone();
        archive.audio.save = true;
        archive.audio.format = "mp3".into();
        archive.audio.retain_original = true;
        archive.subtitles.retain_original = true;
        archive.danmaku.format = "html".into();
        archive.danmaku.retain_original = true;
        let mut embedded = presets[5].workflow.clone();
        embedded.cover.save = false;
        embedded.subtitles.save = false;
        embedded.danmaku.enabled = false;
        embedded.nfo = false;
        let mut assets = presets[2].workflow.clone();
        assets.subtitles.format = "ass".into();
        assets.cover.enabled = true;
        assets.danmaku.enabled = true;
        assets.danmaku.format = "html".into();
        assets.nfo = true;
        let mut manifests = Vec::new();
        for (name, workflow, native) in [
            ("raw-audio", raw, false),
            ("mp3-audio", mp3, true),
            ("archive", archive, true),
            ("embedded-only", embedded, true),
            ("assets-only", assets, false),
        ] {
            let mut options = DownloadOptions::new(root.join(name));
            options.naming_template = "{title}.{ext}".into();
            workflow.apply(&mut options).unwrap();
            let mut task = plan_selected_parts(&tree, &[PartId("part:fixture".into())], &options)
                .unwrap()
                .remove(0);
            fs::create_dir_all(task.output_path.parent().unwrap())
                .await
                .unwrap();
            for resource in &mut task.resources {
                match resource.intent {
                    Intent::Video | Intent::Audio | Intent::Cover => {
                        let name = match resource.intent {
                            Intent::Video => "video.m4s",
                            Intent::Audio => "audio.m4s",
                            _ => "cover.jpg",
                        };
                        fs::copy(root.join(name), &resource.target_path)
                            .await
                            .unwrap();
                    }
                    Intent::Subtitle => fs::write(
                        &resource.target_path,
                        r#"{"body":[{"from":0.25,"to":1.5,"content":"线条与色彩"}]}"#,
                    )
                    .await
                    .unwrap(),
                    Intent::Danmaku => fs::write(
                        &resource.target_path,
                        r#"<i><d p="1,1,25,16777215">测试弹幕</d></i>"#,
                    )
                    .await
                    .unwrap(),
                    Intent::Nfo => {}
                }
                resource.status = ResourceStatus::Completed;
            }
            let backend = if native {
                MediaMuxBackend::desktop()
            } else {
                MediaMuxBackend::unsupported()
            };
            execute(&task, &backend, Some(ffmpeg.clone()))
                .await
                .unwrap();
            task.media_selection.outputs_verified = true;
            assert!(cleanup(&task).await.is_empty());
            assert!(outputs_complete(&task).await.unwrap());
            assert!(
                crate::commands::has_recoverable_completed_output(&task)
                    .await
                    .unwrap()
            );
            if !workflow.audio.retain_original && workflow.audio.enabled {
                assert!(
                    !task
                        .resources
                        .iter()
                        .find(|r| r.intent == Intent::Audio)
                        .unwrap()
                        .target_path
                        .exists()
                );
            }
            manifests.push(task);
        }
        fs::write(
            root.join("manifests.json"),
            serde_json::to_vec_pretty(&manifests).unwrap(),
        )
        .await
        .unwrap();
    }
}
