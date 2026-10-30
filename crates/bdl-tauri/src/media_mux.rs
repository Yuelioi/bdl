use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use bdl_core::error::IoResultExt;
use bdl_core::muxer::{MediaMuxer, MediaMuxerConfig, MuxRequest};
use bdl_core::{BdlError, BdlResult};

#[async_trait]
pub trait MediaMuxBackendImpl: Send + Sync {
    async fn mux(&self, request: &MuxRequest, ffmpeg_path: Option<PathBuf>) -> BdlResult<()>;
}

#[derive(Clone)]
pub struct MediaMuxBackend {
    inner: Arc<dyn MediaMuxBackendImpl>,
}

impl MediaMuxBackend {
    pub fn from_impl<T>(backend: T) -> Self
    where
        T: MediaMuxBackendImpl + 'static,
    {
        Self {
            inner: Arc::new(backend),
        }
    }

    pub fn desktop() -> Self {
        Self::from_impl(DesktopMediaMuxBackend)
    }

    pub fn unsupported() -> Self {
        Self::from_impl(UnsupportedMediaMuxBackend)
    }

    pub fn platform_default() -> Self {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            Self::unsupported()
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            Self::desktop()
        }
    }

    pub async fn mux(&self, request: &MuxRequest, ffmpeg_path: Option<PathBuf>) -> BdlResult<()> {
        if request.video_path.is_none()
            && request
                .output_path
                .extension()
                .is_some_and(|ext| ext == "m4s")
        {
            let source = request.audio_path.as_ref().ok_or(BdlError::Planning {
                message: "缺少原始音频输入。".to_owned(),
            })?;
            if source != &request.output_path {
                // Publish only a complete copy so interrupted saves cannot be recovered as finished.
                let temporary = request.output_path.with_extension("m4s.bdl-copy.tmp");
                tokio::fs::copy(source, &temporary)
                    .await
                    .path_context("copy_raw_audio", &temporary)?;
                tokio::fs::rename(&temporary, &request.output_path)
                    .await
                    .path_context("publish_raw_audio", &request.output_path)?;
            }
            return Ok(());
        }
        self.inner.mux(request, ffmpeg_path).await.map_err(|error| {
            error.with_context(format!(
                "[stage=media_process] video={:?} audio={:?} output={} ffmpeg_backend={}",
                request.video_path,
                request.audio_path,
                request.output_path.display(),
                if cfg!(target_os = "android") {
                    "android_native"
                } else {
                    "desktop"
                },
            ))
        })
    }
}

struct DesktopMediaMuxBackend;

#[async_trait]
impl MediaMuxBackendImpl for DesktopMediaMuxBackend {
    async fn mux(&self, request: &MuxRequest, ffmpeg_path: Option<PathBuf>) -> BdlResult<()> {
        let muxer = MediaMuxer::new(MediaMuxerConfig { ffmpeg_path }).map_err(BdlError::from)?;
        muxer.mux(request).await.map_err(BdlError::from)
    }
}

struct UnsupportedMediaMuxBackend;

#[async_trait]
impl MediaMuxBackendImpl for UnsupportedMediaMuxBackend {
    async fn mux(&self, _request: &MuxRequest, _ffmpeg_path: Option<PathBuf>) -> BdlResult<()> {
        Err(BdlError::Platform {
            message: "当前平台尚未提供原生媒体合并后端。".to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn raw_audio_is_byte_identical_without_a_mux_backend_or_ffmpeg() {
        let dir = std::env::temp_dir().join(format!("bdl-raw-audio-{}", uuid::Uuid::new_v4()));
        tokio::fs::create_dir_all(&dir).await.unwrap();
        let input = dir.join("source.audio.m4s");
        let output = dir.join("音频.m4s");
        let bytes = [0, 255, 1, 2, 3, 0, 42];
        tokio::fs::write(&input, bytes).await.unwrap();
        let request = MuxRequest {
            video_path: None,
            audio_path: Some(input.clone()),
            output_path: output.clone(),
            cover_path: None,
            subtitle_paths: Vec::new(),
        };
        MediaMuxBackend::unsupported()
            .mux(&request, Some(dir.join("missing-ffmpeg")))
            .await
            .unwrap();
        assert_eq!(tokio::fs::read(&output).await.unwrap(), bytes);
        assert!(input.exists());
        assert!(!output.with_extension("m4s.bdl-copy.tmp").exists());
        let missing_request = MuxRequest {
            audio_path: Some(dir.join("missing.m4s")),
            ..request
        };
        assert!(
            MediaMuxBackend::unsupported()
                .mux(&missing_request, None)
                .await
                .is_err()
        );
        assert_eq!(tokio::fs::read(output).await.unwrap(), bytes);
        tokio::fs::remove_dir_all(dir).await.unwrap();
    }
}
