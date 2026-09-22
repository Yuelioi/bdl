use std::path::{Path, PathBuf};

use async_trait::async_trait;
use bdl_core::muxer::MuxRequest;
use bdl_core::{BdlError, BdlResult};
use bdl_tauri::media_mux::{MediaMuxBackend, MediaMuxBackendImpl};
use serde::Serialize;
use tauri::{
    Manager, Runtime,
    plugin::{Builder, PluginHandle, TauriPlugin},
};

const PLUGIN_IDENTIFIER: &str = "com.yueli.bdl.media";

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("bdl-mobile-media")
        .setup(|app, api| {
            let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "MediaMuxPlugin")?;
            app.manage(MediaMuxBackend::from_impl(AndroidMediaMuxBackend {
                handle,
            }));
            Ok(())
        })
        .build()
}

struct AndroidMediaMuxBackend<R: Runtime> {
    handle: PluginHandle<R>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MuxPayload {
    video_path: Option<String>,
    audio_path: Option<String>,
    output_path: String,
    format_name: String,
}

#[async_trait]
impl<R: Runtime> MediaMuxBackendImpl for AndroidMediaMuxBackend<R> {
    async fn mux(&self, request: &MuxRequest, _ffmpeg_path: Option<PathBuf>) -> BdlResult<()> {
        if request.cover_path.is_some() || !request.subtitle_paths.is_empty() {
            return Err(BdlError::Platform {
                message: "Android 内置 FFmpeg 当前暂不开放嵌入封面或字幕。".to_owned(),
            });
        }
        let format_name =
            output_format_name(&request.output_path).ok_or_else(|| BdlError::Platform {
                message: "Android 内置 FFmpeg 当前支持 MP4 与 MKV 封装。".to_owned(),
            })?;
        if request.video_path.is_none() && request.audio_path.is_none() {
            return Err(BdlError::Platform {
                message: "Android 媒体合并缺少视频或音频输入。".to_owned(),
            });
        }

        let payload = MuxPayload {
            video_path: request
                .video_path
                .as_deref()
                .map(android_path)
                .transpose()?,
            audio_path: request
                .audio_path
                .as_deref()
                .map(android_path)
                .transpose()?,
            output_path: android_path(&request.output_path)?,
            format_name: format_name.to_owned(),
        };

        self.handle
            .run_mobile_plugin::<serde_json::Value>("mux", payload)
            .map(|_| ())
            .map_err(|error| BdlError::Platform {
                message: format!("Android 媒体合并失败：{error}"),
            })
    }
}

fn android_path(path: &Path) -> BdlResult<String> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| BdlError::Platform {
            message: "Android 媒体文件路径不是有效 UTF-8。".to_owned(),
        })
}

fn output_format_name(path: &Path) -> Option<&'static str> {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("mp4" | "m4v" | "mov") => Some("mp4"),
        Some("mkv") => Some("matroska"),
        _ => None,
    }
}
