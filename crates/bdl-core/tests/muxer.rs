use std::path::{Path, PathBuf};

use bdl_core::muxer::{MediaMuxer, MediaMuxerConfig, MuxError, MuxRequest};
use uuid::Uuid;

#[tokio::test]
async fn media_muxer_returns_not_found_for_missing_configured_ffmpeg() {
    let missing = temp_case_dir("missing").await.join("does-not-exist-ffmpeg");

    let error = MediaMuxer::with_ffmpeg_path(missing)
        .mux(&mux_request(temp_case_dir("missing-request").await))
        .await
        .expect_err("missing ffmpeg should fail");

    assert!(matches!(error, MuxError::FfmpegNotFound { .. }));
}

#[tokio::test]
async fn media_muxer_uses_configured_ffmpeg_path() {
    let dir = temp_case_dir("configured").await;
    let record_path = dir.join("args.txt");
    let ffmpeg = fake_ffmpeg(&dir, &record_path, 0, "").await;
    let request = mux_request(dir.clone());

    let muxer = MediaMuxer::new(MediaMuxerConfig {
        ffmpeg_path: Some(ffmpeg.clone()),
    })
    .expect("configured fake ffmpeg should be accepted");

    assert_eq!(muxer.ffmpeg_path(), ffmpeg.as_path());
    muxer
        .mux(&request)
        .await
        .expect("fake ffmpeg should succeed");

    let args = tokio::fs::read_to_string(record_path).await.unwrap();
    assert!(args.contains("-y"));
    assert!(args.contains("-i"));
    assert!(args.contains("video.m4s"));
    assert!(args.contains("audio.m4s"));
    assert!(args.contains("-c"));
    assert!(args.contains("copy"));
    assert!(args.contains("output.mp4"));
}

#[tokio::test]
async fn media_muxer_returns_exit_code_and_stderr_when_ffmpeg_fails() {
    let dir = temp_case_dir("failed").await;
    let record_path = dir.join("args.txt");
    let ffmpeg = fake_ffmpeg(&dir, &record_path, 7, "mux failed").await;
    let request = mux_request(dir);

    let error = MediaMuxer::with_ffmpeg_path(ffmpeg)
        .mux(&request)
        .await
        .expect_err("fake ffmpeg should fail");

    assert!(matches!(
        error,
        MuxError::CommandFailed {
            code: Some(7),
            stderr
        } if stderr.contains("mux failed")
    ));
}

#[tokio::test]
async fn media_muxer_embeds_supported_cover_for_mp4() {
    let dir = temp_case_dir("cover").await;
    let record_path = dir.join("args.txt");
    let ffmpeg = fake_ffmpeg(&dir, &record_path, 0, "").await;
    let mut request = mux_request(dir.clone());
    request.cover_path = Some(dir.join("cover.jpg"));

    MediaMuxer::with_ffmpeg_path(ffmpeg)
        .mux(&request)
        .await
        .expect("fake ffmpeg should succeed");

    let args = tokio::fs::read_to_string(record_path).await.unwrap();
    assert!(args.contains("cover.jpg"));
    assert!(args.contains("-disposition:v:1"));
    assert!(args.contains("attached_pic"));
}

#[tokio::test]
async fn media_muxer_embeds_supported_srt_subtitle_for_mp4() {
    let dir = temp_case_dir("subtitle").await;
    let record_path = dir.join("args.txt");
    let ffmpeg = fake_ffmpeg(&dir, &record_path, 0, "").await;
    let mut request = mux_request(dir.clone());
    request.subtitle_paths = vec![dir.join("subtitle.srt")];

    MediaMuxer::with_ffmpeg_path(ffmpeg)
        .mux(&request)
        .await
        .expect("fake ffmpeg should succeed");

    let args = tokio::fs::read_to_string(record_path).await.unwrap();
    assert!(args.contains("subtitle.srt"));
    assert!(args.contains("-c:s"));
    assert!(args.contains("mov_text"));
}

#[tokio::test]
async fn media_muxer_skips_unsupported_json_subtitle_embedding() {
    let dir = temp_case_dir("json-subtitle").await;
    let record_path = dir.join("args.txt");
    let ffmpeg = fake_ffmpeg(&dir, &record_path, 0, "").await;
    let mut request = mux_request(dir.clone());
    request.subtitle_paths = vec![dir.join("subtitle.json")];

    MediaMuxer::with_ffmpeg_path(ffmpeg)
        .mux(&request)
        .await
        .expect("fake ffmpeg should succeed");

    let args = tokio::fs::read_to_string(record_path).await.unwrap();
    assert!(!args.contains("subtitle.json"));
    assert!(args.contains("-c"));
    assert!(args.contains("copy"));
}

fn mux_request(dir: PathBuf) -> MuxRequest {
    MuxRequest {
        video_path: dir.join("video.m4s"),
        audio_path: dir.join("audio.m4s"),
        output_path: dir.join("output.mp4"),
        cover_path: None,
        subtitle_paths: Vec::new(),
    }
}

async fn temp_case_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("bdl-muxer-{name}-{}", Uuid::new_v4()));
    tokio::fs::create_dir_all(&dir).await.unwrap();
    dir
}

#[cfg(windows)]
async fn fake_ffmpeg(dir: &Path, record_path: &Path, exit_code: i32, stderr: &str) -> PathBuf {
    let path = dir.join("ffmpeg.cmd");
    let stderr_line = if stderr.is_empty() {
        String::new()
    } else {
        format!("echo {stderr} 1>&2\r\n")
    };
    let script = format!(
        "@echo off\r\necho %* > \"{}\"\r\n{}exit /B {exit_code}\r\n",
        record_path.display(),
        stderr_line
    );
    tokio::fs::write(&path, script).await.unwrap();
    path
}

#[cfg(not(windows))]
async fn fake_ffmpeg(dir: &Path, record_path: &Path, exit_code: i32, stderr: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let path = dir.join("ffmpeg");
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" > '{}'\nprintf '%s\\n' '{}' >&2\nexit {exit_code}\n",
        record_path.display(),
        stderr
    );
    tokio::fs::write(&path, script).await.unwrap();
    let mut permissions = tokio::fs::metadata(&path).await.unwrap().permissions();
    permissions.set_mode(0o755);
    tokio::fs::set_permissions(&path, permissions)
        .await
        .unwrap();
    path
}
