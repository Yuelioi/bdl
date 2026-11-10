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
async fn mp3_output_encodes_audio_instead_of_copying_aac() {
    let dir = temp_case_dir("mp3").await;
    let record_path = dir.join("args.txt");
    let ffmpeg = fake_ffmpeg(&dir, &record_path, 0, "").await;
    let mut request = mux_request(dir);
    request.video_path = None;
    request.output_path.set_extension("mp3");
    MediaMuxer::with_ffmpeg_path(ffmpeg)
        .mux(&request)
        .await
        .unwrap();
    let args = tokio::fs::read_to_string(record_path).await.unwrap();
    assert!(args.contains("libmp3lame"));
    assert!(args.contains("0:a:0"));
    assert!(!args.contains("copy"));
}

#[tokio::test]
#[ignore = "requires local FFmpeg and ffprobe"]
async fn real_mp3_output_has_mp3_codec_and_optional_cover() {
    let dir = temp_case_dir("real-mp3").await;
    let input = dir.join("original.m4s");
    let mut generator = std::process::Command::new("ffmpeg");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        generator.creation_flags(0x08000000);
    }
    let generated = generator
        .args([
            "-y",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:duration=3",
            "-c:a",
            "aac",
            "-movflags",
            "frag_keyframe+empty_moov",
            "-f",
            "mp4",
        ])
        .arg(&input)
        .output()
        .unwrap();
    assert!(generated.status.success());
    for extension in [None, Some("jpg"), Some("png")] {
        let cover_path = if let Some(extension) = extension {
            let path = dir.join(format!("cover.{extension}"));
            let mut image = std::process::Command::new("ffmpeg");
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                image.creation_flags(0x08000000);
            }
            let generated = image
                .args([
                    "-y",
                    "-f",
                    "lavfi",
                    "-i",
                    "color=c=pink:s=160x160",
                    "-frames:v",
                    "1",
                ])
                .arg(&path)
                .output()
                .unwrap();
            assert!(
                generated.status.success(),
                "{}",
                String::from_utf8_lossy(&generated.stderr)
            );
            Some(path)
        } else {
            None
        };
        let request = MuxRequest {
            video_path: None,
            audio_path: Some(input.clone()),
            output_path: dir.join(format!("converted-{}.mp3", extension.unwrap_or("audio"))),
            cover_path,
            subtitle_paths: Vec::new(),
        };
        MediaMuxer::new(MediaMuxerConfig::default())
            .unwrap()
            .mux(&request)
            .await
            .unwrap();
        let mut probe = std::process::Command::new("ffprobe");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            probe.creation_flags(0x08000000);
        }
        let output = probe
            .args([
                "-v",
                "error",
                "-show_streams",
                "-show_format",
                "-of",
                "json",
            ])
            .arg(&request.output_path)
            .output()
            .unwrap();
        assert!(output.status.success());
        let data: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(data["format"]["format_name"], "mp3");
        assert_eq!(
            data["streams"].as_array().unwrap().len(),
            if extension.is_some() { 2 } else { 1 }
        );
        assert_eq!(data["streams"][0]["codec_name"], "mp3");
        if let Some(extension) = extension {
            let cover = &data["streams"][1];
            assert_eq!(
                cover["codec_name"],
                if extension == "png" { "png" } else { "mjpeg" }
            );
            assert_eq!(cover["disposition"]["attached_pic"], 1);
            assert_eq!(cover["tags"]["comment"], "Cover (front)");
        }
        let duration: f64 = data["format"]["duration"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        assert!((2.9..3.2).contains(&duration));
        println!(
            "Verified MP3 codec and duration: {}",
            request.output_path.display()
        );
    }
}

#[tokio::test]
#[ignore = "requires FFmpeg and BDL_REAL_MUX_VIDEO pointing to a local video"]
async fn real_ass_subtitle_embeds_and_roundtrips_chinese_cues() {
    let dir = temp_case_dir("real-ass").await;
    let media = PathBuf::from(std::env::var_os("BDL_REAL_MUX_VIDEO").unwrap());
    let subtitle = dir.join("subtitle.ass");
    tokio::fs::write(
        &subtitle,
        bdl_core::subtitles::bilibili_json_to_ass(
            r#"{"body":[{"from":0.38,"to":2.22,"content":"中文第一行\n第二行"}]}"#,
        )
        .unwrap(),
    )
    .await
    .unwrap();
    let request = MuxRequest {
        video_path: Some(media.clone()),
        audio_path: Some(media),
        output_path: dir.join("verified-ass.mkv"),
        cover_path: None,
        subtitle_paths: vec![subtitle],
    };
    MediaMuxer::new(MediaMuxerConfig::default())
        .unwrap()
        .mux(&request)
        .await
        .unwrap();
    let mut probe = std::process::Command::new("ffprobe");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        probe.creation_flags(0x08000000);
    }
    let output = probe
        .args(["-v", "error", "-show_streams", "-of", "json"])
        .arg(&request.output_path)
        .output()
        .unwrap();
    assert!(output.status.success());
    let data: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(
        data["streams"]
            .as_array()
            .unwrap()
            .iter()
            .any(|stream| stream["codec_name"] == "ass")
    );
    let mut extract = std::process::Command::new("ffmpeg");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        extract.creation_flags(0x08000000);
    }
    let srt = extract
        .args(["-v", "error", "-i"])
        .arg(&request.output_path)
        .args(["-map", "0:s:0", "-f", "srt", "-"])
        .output()
        .unwrap();
    assert!(srt.status.success());
    let text = String::from_utf8_lossy(&srt.stdout);
    assert!(text.contains("00:00:00,380 --> 00:00:02,220"));
    assert!(text.contains("中文第一行\n第二行"));
    println!(
        "Verified embedded ASS and Chinese cue roundtrip: {}",
        request.output_path.display()
    );
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

#[tokio::test]
async fn media_muxer_embeds_mkv_cover_as_attachment_and_keeps_srt() {
    for extension in ["jpg", "png"] {
        let dir = temp_case_dir("mkv-attachments").await;
        let record_path = dir.join("args.txt");
        let ffmpeg = fake_ffmpeg(&dir, &record_path, 0, "").await;
        let mut request = mux_request(dir.clone());
        request.output_path = dir.join("output.mkv");
        request.cover_path = Some(dir.join(format!("thumbnail.{extension}")));
        request.subtitle_paths = vec![dir.join("subtitle.srt")];
        MediaMuxer::with_ffmpeg_path(ffmpeg)
            .mux(&request)
            .await
            .unwrap();
        let args = tokio::fs::read_to_string(record_path).await.unwrap();
        assert!(args.contains("-attach"));
        assert!(args.contains(&format!("thumbnail.{extension}")));
        assert!(args.contains(&format!("filename=cover.{extension}")));
        assert!(args.contains(if extension == "png" {
            "mimetype=image/png"
        } else {
            "mimetype=image/jpeg"
        }));
        assert!(args.contains("subtitle.srt"));
        assert!(args.contains("-map 2:0"));
        assert!(!args.contains("attached_pic"));
        assert!(!args.contains("mov_text"));
    }
}

/// Run explicitly with local media paths; never modifies the input files.
#[tokio::test]
#[ignore = "requires local FFmpeg/ffprobe and BDL_REAL_MUX_* fixture paths"]
async fn media_muxer_real_mkv_json_subtitle_and_cover() {
    let dir = temp_case_dir("real-mkv").await;
    let video = PathBuf::from(std::env::var_os("BDL_REAL_MUX_VIDEO").unwrap());
    let cover = PathBuf::from(std::env::var_os("BDL_REAL_MUX_COVER").unwrap());
    let json = tokio::fs::read_to_string(PathBuf::from(
        std::env::var_os("BDL_REAL_MUX_SUBTITLE").unwrap(),
    ))
    .await
    .unwrap();
    let srt = bdl_core::subtitles::bilibili_json_to_srt(&json).unwrap();
    let subtitle = dir.join("subtitle.srt");
    tokio::fs::write(&subtitle, srt).await.unwrap();
    let request = MuxRequest {
        video_path: Some(video.clone()),
        audio_path: Some(video),
        cover_path: Some(cover),
        subtitle_paths: vec![subtitle],
        output_path: dir.join("verified.mkv"),
    };
    MediaMuxer::new(MediaMuxerConfig::default())
        .unwrap()
        .mux(&request)
        .await
        .unwrap();
    let mut probe = std::process::Command::new("ffprobe");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        probe.creation_flags(0x08000000);
    }
    let output = probe
        .args(["-v", "error", "-show_streams", "-of", "json"])
        .arg(&request.output_path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let data: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let streams = data["streams"].as_array().unwrap();
    assert!(
        streams
            .iter()
            .any(|s| s["codec_name"] == "h264" && s["disposition"]["attached_pic"] == 0)
    );
    assert!(streams.iter().any(|s| s["codec_name"] == "aac"));
    assert!(
        streams
            .iter()
            .any(|s| s["codec_name"] == "subrip" && s["codec_type"] == "subtitle")
    );
    assert!(streams.iter().any(|s| s["disposition"]["attached_pic"] == 1
        && s["tags"]["filename"] == "cover.jpg"
        && s["tags"]["mimetype"] == "image/jpeg"));
    println!(
        "Verified real MKV subtitle and cover: {}",
        request.output_path.display()
    );
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
