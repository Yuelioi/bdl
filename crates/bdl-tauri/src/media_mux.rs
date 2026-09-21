use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
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
        self.inner.mux(request, ffmpeg_path).await
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
