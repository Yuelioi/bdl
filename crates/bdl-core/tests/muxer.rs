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
async fn media_muxer_probe_reports_path_and_version() {
    let dir = temp_case_dir("probe").await;
    let record_path = dir.join("args.txt");
    let ffmpeg = fake_ffmpeg(&dir, &record_path, 0, "").await;
    let muxer = MediaMuxer::new(MediaMuxerConfig {
        ffmpeg_path: Some(ffmpeg.clone()),
    })
    .expect("configured fake ffmpeg should be accepted");

    let probe = muxer.probe().await.expect("fake ffmpeg should be probed");

    assert_eq!(probe.path, ffmpeg);
    assert_eq!(probe.version, "ffmpeg version bdl-test");
}

#[tokio::test]
async fn media_muxer_probe_rejects_non_ffmpeg_executable() {
    let dir = temp_case_dir("probe-impostor").await;
    let executable = fake_version_program(&dir, "not-ffmpeg version 1.0").await;
    let muxer = MediaMuxer::with_ffmpeg_path(executable);

    let error = muxer
        .probe()
        .await
        .expect_err("a successful non-FFmpeg executable must be rejected");

    assert!(
        matches!(
            error,
            MuxError::CommandFailed { ref stderr, .. }
                if stderr.contains("unexpected FFmpeg version banner")
        ),
        "unexpected probe error: {error:?}"
    );
}

#[tokio::test]
async fn media_muxer_supports_video_only_output() {
    let dir = temp_case_dir("video-only").await;
    let record_path = dir.join("args.txt");
    let ffmpeg = fake_ffmpeg(&dir, &record_path, 0, "").await;
    let mut request = mux_request(dir.clone());
    request.audio_path = None;

    MediaMuxer::with_ffmpeg_path(ffmpeg)
        .mux(&request)
        .await
        .expect("fake ffmpeg should succeed");

    let args = tokio::fs::read_to_string(record_path).await.unwrap();
    assert!(args.contains("video.m4s"));
    assert!(!args.contains("audio.m4s"));
    assert!(args.contains("output.mp4"));
}

#[tokio::test]
async fn media_muxer_supports_audio_only_output() {
    let dir = temp_case_dir("audio-only").await;
    let record_path = dir.join("args.txt");
    let ffmpeg = fake_ffmpeg(&dir, &record_path, 0, "").await;
    let mut request = mux_request(dir.clone());
    request.video_path = None;

    MediaMuxer::with_ffmpeg_path(ffmpeg)
        .mux(&request)
        .await
        .expect("fake ffmpeg should succeed");

    let args = tokio::fs::read_to_string(record_path).await.unwrap();
    assert!(!args.contains("video.m4s"));
    assert!(args.contains("audio.m4s"));
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
        video_path: Some(dir.join("video.m4s")),
        audio_path: Some(dir.join("audio.m4s")),
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
        "@echo off\r\necho %* > \"{}\"\r\necho ffmpeg version bdl-test\r\n{}exit /B {exit_code}\r\n",
        record_path.display(),
        stderr_line
    );
    tokio::fs::write(&path, script).await.unwrap();
    path
}

#[cfg(windows)]
async fn fake_version_program(dir: &Path, banner: &str) -> PathBuf {
    let path = dir.join("version-program.cmd");
    let script = format!("@echo off\r\necho {banner}\r\nexit /B 0\r\n");
    tokio::fs::write(&path, script).await.unwrap();
    path
}

#[cfg(not(windows))]
async fn fake_ffmpeg(dir: &Path, record_path: &Path, exit_code: i32, stderr: &str) -> PathBuf {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    let path = dir.join("ffmpeg");
    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$*\" > '{}'\nprintf '%s\\n' 'ffmpeg version bdl-test'\nprintf '%s\\n' '{}' >&2\nexit {exit_code}\n",
        record_path.display(),
        stderr
    );
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(script.as_bytes()).unwrap();
    let mut permissions = file.metadata().unwrap().permissions();
    permissions.set_mode(0o755);
    file.set_permissions(permissions).unwrap();
    drop(file);
    wait_until_executable(&path).await;
    path
}

#[cfg(not(windows))]
async fn fake_version_program(dir: &Path, banner: &str) -> PathBuf {
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    let path = dir.join("version-program");
    let script = format!("#!/bin/sh\nprintf '%s\\n' '{banner}'\nexit 0\n");
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(script.as_bytes()).unwrap();
    let mut permissions = file.metadata().unwrap().permissions();
    permissions.set_mode(0o755);
    file.set_permissions(permissions).unwrap();
    drop(file);
    wait_until_executable(&path).await;
    path
}

#[cfg(not(windows))]
async fn wait_until_executable(path: &Path) {
    let mut last_error = None;

    for _ in 0..50 {
        match tokio::process::Command::new(path)
            .arg("-version")
            .output()
            .await
        {
            Ok(_) => return,
            Err(error) if error.raw_os_error() == Some(26) => {
                last_error = Some(error);
                tokio::time::sleep(std::time::Duration::from_millis(2)).await;
            }
            Err(error) => panic!("failed to execute test fixture {}: {error}", path.display()),
        }
    }

    panic!(
        "test fixture {} stayed busy: {}",
        path.display(),
        last_error.unwrap()
    );
}
