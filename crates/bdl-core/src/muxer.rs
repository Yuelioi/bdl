use std::env;
use std::ffi::OsString;
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
    pub cover_path: Option<PathBuf>,
    pub subtitle_paths: Vec<PathBuf>,
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
        let mut command = Command::new(ffmpeg_path);
        for arg in ffmpeg_args(request) {
            command.arg(arg);
        }
        let output = command.output().await?;

        if output.status.success() {
            return Ok(());
        }

        Err(MuxError::CommandFailed {
            code: output.status.code(),
            stderr: stderr_summary(&output.stderr),
        })
    }
}

fn ffmpeg_args(request: &MuxRequest) -> Vec<OsString> {
    let cover_path = request
        .cover_path
        .as_deref()
        .filter(|path| supports_cover_embedding(&request.output_path, path));
    let subtitle_paths = request
        .subtitle_paths
        .iter()
        .filter(|path| supports_subtitle_embedding(&request.output_path, path))
        .map(PathBuf::as_path)
        .collect::<Vec<_>>();

    if cover_path.is_none() && subtitle_paths.is_empty() {
        return basic_mux_args(request);
    }

    let mut args = vec![
        os("-y"),
        os("-i"),
        request.video_path.clone().into_os_string(),
    ];
    args.extend([os("-i"), request.audio_path.clone().into_os_string()]);

    let mut next_input_index = 2usize;
    let cover_input_index = cover_path.map(|path| {
        args.extend([os("-i"), path.as_os_str().to_owned()]);
        let index = next_input_index;
        next_input_index += 1;
        index
    });

    let mut subtitle_input_indices = Vec::with_capacity(subtitle_paths.len());
    for subtitle_path in subtitle_paths {
        args.extend([os("-i"), subtitle_path.as_os_str().to_owned()]);
        subtitle_input_indices.push(next_input_index);
        next_input_index += 1;
    }

    args.extend([os("-map"), os("0:v:0"), os("-map"), os("1:a:0")]);
    if let Some(index) = cover_input_index {
        args.extend([os("-map"), os(format!("{index}:v:0"))]);
    }
    for index in &subtitle_input_indices {
        args.extend([os("-map"), os(format!("{index}:0"))]);
    }

    if subtitle_input_indices.is_empty() || !is_mp4_like(&request.output_path) {
        args.extend([os("-c"), os("copy")]);
    } else {
        args.extend([
            os("-c:v"),
            os("copy"),
            os("-c:a"),
            os("copy"),
            os("-c:s"),
            os("mov_text"),
        ]);
    }

    if cover_input_index.is_some() {
        args.extend([os("-disposition:v:1"), os("attached_pic")]);
    }

    args.push(request.output_path.clone().into_os_string());
    args
}

fn basic_mux_args(request: &MuxRequest) -> Vec<OsString> {
    vec![
        os("-y"),
        os("-i"),
        request.video_path.clone().into_os_string(),
        os("-i"),
        request.audio_path.clone().into_os_string(),
        os("-c"),
        os("copy"),
        request.output_path.clone().into_os_string(),
    ]
}

pub fn supports_cover_embedding(output_path: &Path, cover_path: &Path) -> bool {
    is_mp4_like(output_path)
        && matches!(
            lower_extension(cover_path).as_deref(),
            Some("jpg" | "jpeg" | "png")
        )
}

pub fn supports_subtitle_embedding(output_path: &Path, subtitle_path: &Path) -> bool {
    let subtitle_extension = lower_extension(subtitle_path);
    if is_mp4_like(output_path) {
        return matches!(subtitle_extension.as_deref(), Some("srt" | "vtt"));
    }
    if is_mkv(output_path) {
        return matches!(
            subtitle_extension.as_deref(),
            Some("srt" | "ass" | "ssa" | "vtt")
        );
    }
    false
}

fn is_mp4_like(path: &Path) -> bool {
    matches!(
        lower_extension(path).as_deref(),
        Some("mp4" | "m4v" | "mov")
    )
}

fn is_mkv(path: &Path) -> bool {
    matches!(lower_extension(path).as_deref(), Some("mkv"))
}

fn lower_extension(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
}

fn os(value: impl Into<OsString>) -> OsString {
    value.into()
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
