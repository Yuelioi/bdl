use std::env;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Debug, thiserror::Error)]
pub enum MuxError {
    #[error("ffmpeg not found: {path}")]
    FfmpegNotFound { path: String },

    #[error("ffmpeg failed with code {code:?}: {stderr}")]
    CommandFailed { code: Option<i32>, stderr: String },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuxRequest {
    pub video_path: PathBuf,
    pub audio_path: PathBuf,
    pub output_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MediaMuxerConfig {
    pub ffmpeg_path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct MediaMuxer {
    ffmpeg_path: PathBuf,
}

impl MediaMuxer {
    pub fn new(config: MediaMuxerConfig) -> Result<Self, MuxError> {
        let ffmpeg_path = match config.ffmpeg_path {
            Some(path) => ensure_executable(path)?,
            None => find_in_path("ffmpeg")?,
        };

        Ok(Self { ffmpeg_path })
    }

    pub fn with_ffmpeg_path(path: PathBuf) -> Self {
        Self { ffmpeg_path: path }
    }

    pub fn ffmpeg_path(&self) -> &Path {
        &self.ffmpeg_path
    }

    pub async fn mux(&self, request: &MuxRequest) -> Result<(), MuxError> {
        let ffmpeg_path = ensure_executable(self.ffmpeg_path.clone())?;
        let output = Command::new(ffmpeg_path)
            .arg("-y")
            .arg("-i")
            .arg(&request.video_path)
            .arg("-i")
            .arg(&request.audio_path)
            .arg("-c")
            .arg("copy")
            .arg(&request.output_path)
            .output()
            .await?;

        if output.status.success() {
            return Ok(());
        }

        Err(MuxError::CommandFailed {
            code: output.status.code(),
            stderr: stderr_summary(&output.stderr),
        })
    }
}

fn ensure_executable(path: PathBuf) -> Result<PathBuf, MuxError> {
    if path.is_file() {
        Ok(path)
    } else {
        Err(MuxError::FfmpegNotFound {
            path: path.display().to_string(),
        })
    }
}

fn find_in_path(name: &str) -> Result<PathBuf, MuxError> {
    let Some(paths) = env::var_os("PATH") else {
        return Err(MuxError::FfmpegNotFound {
            path: name.to_owned(),
        });
    };

    for dir in env::split_paths(&paths) {
        for candidate in executable_candidates(&dir, name) {
            if candidate.is_file() {
                return Ok(candidate);
            }
        }
    }

    Err(MuxError::FfmpegNotFound {
        path: name.to_owned(),
    })
}

#[cfg(windows)]
fn executable_candidates(dir: &Path, name: &str) -> Vec<PathBuf> {
    vec![
        dir.join(format!("{name}.exe")),
        dir.join(format!("{name}.cmd")),
        dir.join(format!("{name}.bat")),
        dir.join(name),
    ]
}

#[cfg(not(windows))]
fn executable_candidates(dir: &Path, name: &str) -> Vec<PathBuf> {
    vec![dir.join(name)]
}

fn stderr_summary(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let trimmed = text.trim();
    if trimmed.len() <= 500 {
        trimmed.to_owned()
    } else {
        format!("{}...", &trimmed[..500])
    }
}
