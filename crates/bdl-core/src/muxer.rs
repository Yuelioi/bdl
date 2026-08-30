use std::env;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::process::hidden_command;

#[derive(Debug, thiserror::Error)]
pub enum MuxError {
    #[error("ffmpeg not found: {path}")]
    FfmpegNotFound { path: String },

    #[error("missing media input")]
    MissingMediaInput,

    #[error("ffmpeg failed with code {code:?}: {stderr}")]
    CommandFailed { code: Option<i32>, stderr: String },

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MuxRequest {
    pub video_path: Option<PathBuf>,
    pub audio_path: Option<PathBuf>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FfmpegProbe {
    pub path: PathBuf,
    pub version: String,
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

    pub async fn probe(&self) -> Result<FfmpegProbe, MuxError> {
        let ffmpeg_path = ensure_executable(self.ffmpeg_path.clone())?;
        let output = hidden_command(&ffmpeg_path)
            .arg("-version")
            .output()
            .await?;
        if !output.status.success() {
            return Err(MuxError::CommandFailed {
                code: output.status.code(),
                stderr: stderr_summary(&output.stderr),
            });
        }

        let version = String::from_utf8_lossy(&output.stdout)
            .lines()
            .next()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .ok_or_else(|| MuxError::CommandFailed {
                code: output.status.code(),
                stderr: "ffmpeg returned no version information".to_owned(),
            })?
            .to_owned();

        if !version.to_ascii_lowercase().starts_with("ffmpeg version") {
            return Err(MuxError::CommandFailed {
                code: output.status.code(),
                stderr: format!("unexpected FFmpeg version banner: {version}"),
            });
        }

        Ok(FfmpegProbe {
            path: ffmpeg_path,
            version,
        })
    }

    pub async fn mux(&self, request: &MuxRequest) -> Result<(), MuxError> {
        let ffmpeg_path = ensure_executable(self.ffmpeg_path.clone())?;
        let mut command = hidden_command(ffmpeg_path);
        for arg in ffmpeg_args(request)? {
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

fn ffmpeg_args(request: &MuxRequest) -> Result<Vec<OsString>, MuxError> {
    if request.video_path.is_none() && request.audio_path.is_none() {
        return Err(MuxError::MissingMediaInput);
    }

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

    let mut args = vec![os("-y")];
    let mut next_input_index = 0usize;
    let video_input_index = push_optional_input(
        &mut args,
        request.video_path.as_ref(),
        &mut next_input_index,
    );
    let audio_input_index = push_optional_input(
        &mut args,
        request.audio_path.as_ref(),
        &mut next_input_index,
    );
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

    if let Some(index) = video_input_index {
        args.extend([os("-map"), os(format!("{index}:v:0"))]);
    }
    if let Some(index) = audio_input_index {
        args.extend([os("-map"), os(format!("{index}:a:0"))]);
    }
    if let Some(index) = cover_input_index {
        args.extend([os("-map"), os(format!("{index}:v:0"))]);
    }
    for index in &subtitle_input_indices {
        args.extend([os("-map"), os(format!("{index}:0"))]);
    }

    if subtitle_input_indices.is_empty() || !is_mp4_like(&request.output_path) {
        args.extend([os("-c"), os("copy")]);
    } else {
        if video_input_index.is_some() || cover_input_index.is_some() {
            args.extend([os("-c:v"), os("copy")]);
        }
        if audio_input_index.is_some() {
            args.extend([os("-c:a"), os("copy")]);
        }
        args.extend([os("-c:s"), os("mov_text")]);
    }

    if cover_input_index.is_some() {
        let cover_stream_index = if video_input_index.is_some() { 1 } else { 0 };
        args.extend([
            os(format!("-disposition:v:{cover_stream_index}")),
            os("attached_pic"),
        ]);
    }

    args.push(request.output_path.clone().into_os_string());
    Ok(args)
}

fn basic_mux_args(request: &MuxRequest) -> Result<Vec<OsString>, MuxError> {
    let mut args = vec![os("-y")];
    let mut next_input_index = 0usize;
    push_optional_input(
        &mut args,
        request.video_path.as_ref(),
        &mut next_input_index,
    );
    push_optional_input(
        &mut args,
        request.audio_path.as_ref(),
        &mut next_input_index,
    );
    args.extend([
        os("-c"),
        os("copy"),
        request.output_path.clone().into_os_string(),
    ]);
    Ok(args)
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

fn push_optional_input(
    args: &mut Vec<OsString>,
    path: Option<&PathBuf>,
    next_input_index: &mut usize,
) -> Option<usize> {
    let path = path?;
    args.extend([os("-i"), path.as_os_str().to_owned()]);
    let index = *next_input_index;
    *next_input_index += 1;
    Some(index)
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
    if let Some(paths) = env::var_os("PATH") {
        for dir in env::split_paths(&paths) {
            for candidate in executable_candidates(&dir, name) {
                if candidate.is_file() {
                    return Ok(candidate);
                }
            }
        }
    }

    for candidate in platform_executable_candidates(name) {
        if candidate.is_file() {
            return Ok(candidate);
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

#[cfg(target_os = "macos")]
fn platform_executable_candidates(name: &str) -> Vec<PathBuf> {
    [
        "/opt/homebrew/bin",
        "/usr/local/bin",
        "/opt/local/bin",
        "/usr/local/opt/ffmpeg/bin",
    ]
    .into_iter()
    .map(|dir| Path::new(dir).join(name))
    .collect()
}

#[cfg(not(target_os = "macos"))]
fn platform_executable_candidates(_name: &str) -> Vec<PathBuf> {
    Vec::new()
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

#[cfg(test)]
mod tests {
    #[cfg(target_os = "macos")]
    #[test]
    fn macos_ffmpeg_fallbacks_cover_common_package_managers() {
        use std::path::PathBuf;

        assert_eq!(
            super::platform_executable_candidates("ffmpeg"),
            vec![
                PathBuf::from("/opt/homebrew/bin/ffmpeg"),
                PathBuf::from("/usr/local/bin/ffmpeg"),
                PathBuf::from("/opt/local/bin/ffmpeg"),
                PathBuf::from("/usr/local/opt/ffmpeg/bin/ffmpeg"),
            ]
        );
    }
}
