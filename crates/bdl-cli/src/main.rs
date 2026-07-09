use std::env;
use std::path::PathBuf;

use anyhow::{Context, bail};
use bdl_core::fetcher::{Fetcher, ReqwestFetcher};
use bdl_core::ids::PartId;
use bdl_core::input::classify_input;
use bdl_core::model::NormalizedSourceTree;
use bdl_core::muxer::{MediaMuxer, MediaMuxerConfig, MuxError, MuxRequest};
use bdl_core::planner::{DownloadOptions, plan_selected_parts};
use bdl_core::queue::DownloadResourceIntent;
use bdl_core::resolver::video::VideoResolver;
use bdl_core::resolver::{ResolveOptions, Resolver};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let Some(command) = args.first().map(String::as_str) else {
        print_usage();
        return Ok(());
    };

    match command {
        "parse" => parse_command(&args[1..]).await,
        "download" => download_command(&args[1..]).await,
        "verify-cookie" => verify_cookie_command(),
        "ffmpeg-check" => ffmpeg_check_command(),
        "-h" | "--help" | "help" => {
            print_usage();
            Ok(())
        }
        other => bail!("unknown command `{other}`"),
    }
}

async fn parse_command(args: &[String]) -> anyhow::Result<()> {
    let input = positional(args, 0).context("usage: bdl-cli parse <input> --json")?;
    let tree = resolve_input(input, false).await?;
    println!("{}", serde_json::to_string_pretty(&tree)?);
    Ok(())
}

async fn download_command(args: &[String]) -> anyhow::Result<()> {
    let input = positional(args, 0)
        .context("usage: bdl-cli download <input> --output <dir> --quality best")?;
    let output_dir = option_value(args, "--output")
        .map(PathBuf::from)
        .context("missing --output <dir>")?;
    let quality = option_value(args, "--quality").unwrap_or("best");
    if quality != "best" {
        bail!("only --quality best is supported in this vertical slice");
    }

    let tree = resolve_input(input, true).await?;
    let selected_part_ids = all_part_ids(&tree);
    if selected_part_ids.is_empty() {
        bail!("resolved source has no loaded parts");
    }

    let tasks = plan_selected_parts(&tree, &selected_part_ids, &DownloadOptions::new(output_dir))?;
    let fetcher = ReqwestFetcher::new()?;
    let muxer = MediaMuxer::new(MediaMuxerConfig::default())?;

    for task in tasks {
        let video = task
            .resources
            .iter()
            .find(|resource| resource.intent == DownloadResourceIntent::Video)
            .context("planned task is missing video resource")?;
        let audio = task
            .resources
            .iter()
            .find(|resource| resource.intent == DownloadResourceIntent::Audio)
            .context("planned task is missing audio resource")?;

        fetcher.fetch(video, None).await?;
        fetcher.fetch(audio, None).await?;
        muxer
            .mux(&MuxRequest {
                video_path: Some(video.target_path.clone()),
                audio_path: Some(audio.target_path.clone()),
                output_path: task.output_path.clone(),
                cover_path: None,
                subtitle_paths: Vec::new(),
            })
            .await?;
        println!("downloaded {}", task.output_path.display());
    }

    Ok(())
}

fn verify_cookie_command() -> anyhow::Result<()> {
    let cookie = env::var("BDL_COOKIE").context("set BDL_COOKIE to verify a cookie string")?;
    let has_sessdata = cookie
        .split(';')
        .any(|pair| pair.trim_start().starts_with("SESSDATA="));
    if !has_sessdata {
        bail!("cookie does not contain SESSDATA");
    }

    println!("cookie: login cookie shape ok");
    Ok(())
}

fn ffmpeg_check_command() -> anyhow::Result<()> {
    match MediaMuxer::new(MediaMuxerConfig::default()) {
        Ok(muxer) => {
            println!("ffmpeg: found {}", muxer.ffmpeg_path().display());
            Ok(())
        }
        Err(MuxError::FfmpegNotFound { .. }) => {
            println!("ffmpeg: missing");
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

async fn resolve_input(input: &str, fetch_streams: bool) -> anyhow::Result<NormalizedSourceTree> {
    let classified = classify_input(input)?;
    let resolver = VideoResolver::new()?;
    let tree = resolver
        .resolve(classified, ResolveOptions { fetch_streams })
        .await?;
    Ok(tree)
}

fn all_part_ids(tree: &NormalizedSourceTree) -> Vec<PartId> {
    tree.groups
        .iter()
        .flat_map(|group| &group.items)
        .flat_map(|item| &item.parts)
        .map(|part| part.id.clone())
        .collect()
}

fn positional(args: &[String], index: usize) -> Option<&str> {
    args.iter()
        .filter(|arg| !arg.starts_with("--"))
        .nth(index)
        .map(String::as_str)
}

fn option_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.windows(2).find_map(|window| {
        (window[0] == name && !window[1].starts_with("--")).then_some(window[1].as_str())
    })
}

fn print_usage() {
    eprintln!(
        "usage:\n  bdl-cli parse <input> --json\n  bdl-cli download <input> --output <dir> --quality best\n  bdl-cli verify-cookie\n  bdl-cli ffmpeg-check"
    );
}
