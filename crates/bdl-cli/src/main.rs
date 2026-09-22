use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
mod progress;
use bdl_core::resolver::pacing::{ResolvePolicy, is_source_restriction};
use progress::Progress;

use anyhow::{Context, bail};
use bdl_core::fetcher::{FetchConfig, Fetcher, ReqwestFetcher};
use bdl_core::ids::PartId;
use bdl_core::model::{NormalizedSourceTree, SourceKind};
use bdl_core::muxer::{MediaMuxer, MediaMuxerConfig, MuxError, MuxRequest};
use bdl_core::planner::{
    ArchiveAssetSelection, DownloadMediaMode, DownloadOptions, MissingQualityPolicy,
    StreamPreference, parse_stream_codec, plan_selected_parts,
};
use bdl_core::queue::{DownloadResourceIntent, DownloadTask};
use bdl_core::resolver::ResolveOptions;
use bdl_core::resolver::source::SourceResolver;
use clap::{Parser, Subcommand, ValueEnum};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(
    name = "bdl",
    version,
    about = "BDL command-line downloader for Bilibili"
)]
struct Cli {
    /// Emit machine-readable JSON on stdout.
    #[arg(long, global = true)]
    json: bool,

    /// Read the Bilibili Cookie header from a UTF-8 file.
    ///
    /// When omitted, BDL_COOKIE is used if it is present.
    #[arg(long, global = true, value_name = "PATH")]
    cookie_file: Option<PathBuf>,

    /// Metadata/completion checkpoint (never contains cookies or media URLs).
    #[arg(long, global = true)]
    state: Option<PathBuf>,
    /// Resume a matching checkpoint; does not automatically retry restrictions.
    #[arg(long, global = true, requires = "state")]
    resume: bool,
    /// Maximum resolver operations this invocation (one operation can make several HTTP calls).
    #[arg(long, global = true, default_value_t = 50, value_parser = clap::value_parser!(u32).range(1..=1000))]
    max_operations: u32,
    /// Maximum actual resolver HTTP attempts this invocation; a shared hourly cap also applies.
    #[arg(long, global = true, default_value_t = 100, value_parser = clap::value_parser!(u32).range(1..=1000))]
    max_http_requests: u32,
    /// Minimum seconds between HTTP request starts (0.5 = at most two per second).
    #[arg(long, global = true, default_value_t = 0.5, value_parser = parse_interval)]
    interval_seconds: f64,

    #[command(subcommand)]
    command: Command,
}

fn parse_interval(value: &str) -> Result<f64, String> {
    let seconds: f64 = value
        .parse()
        .map_err(|_| "expected seconds between 0.5 and 120")?;
    if !seconds.is_finite() || !(0.5..=120.0).contains(&seconds) {
        return Err("expected seconds between 0.5 and 120".into());
    }
    Ok(seconds)
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Parse a Bilibili input into BDL's normalized source tree.
    Parse {
        input: String,

        /// Load all metadata pages instead of only the first page.
        #[arg(long, conflicts_with = "with_streams")]
        all: bool,

        /// Fetch playable media streams while parsing.
        #[arg(long, conflicts_with = "state")]
        with_streams: bool,
    },

    /// Download one or more Bilibili inputs.
    Download {
        /// Explicitly download every item/part (requires --state).
        #[arg(long, conflicts_with_all = ["items", "parts"])]
        all: bool,
        /// One or more BV/AV IDs or Bilibili URLs.
        #[arg(required = true, num_args = 1..)]
        inputs: Vec<String>,

        /// Download only these one-based video part numbers, e.g. 1,2.
        #[arg(long, value_delimiter = ',', value_parser = clap::value_parser!(u32).range(1..))]
        parts: Vec<u32>,

        /// Download only these one-based list item numbers, e.g. 1,2.
        #[arg(long, value_delimiter = ',', value_parser = clap::value_parser!(u32).range(1..))]
        items: Vec<u32>,

        /// Output directory.
        #[arg(short, long, value_name = "DIR")]
        output: PathBuf,

        /// Video quality: best, sdr, or a numeric Bilibili qn such as 80/120/125.
        #[arg(long, default_value = "best")]
        quality: String,

        /// Audio quality: best or a numeric Bilibili audio quality id.
        #[arg(long, default_value = "best")]
        audio_quality: String,

        /// Preferred video codec.
        #[arg(long, value_enum, default_value_t = CliCodec::Auto)]
        codec: CliCodec,

        /// Download audio+video, video only, or audio only.
        #[arg(long, value_enum, default_value_t = CliMediaMode::AudioVideo)]
        media_mode: CliMediaMode,

        /// Final media container.
        #[arg(long, value_enum, default_value_t = CliContainer::Mp4)]
        container: CliContainer,

        /// What to do when the requested quality is unavailable.
        #[arg(long, value_enum, default_value_t = CliMissingQuality::Lower)]
        missing_quality: CliMissingQuality,

        /// Use a specific FFmpeg executable instead of searching PATH.
        #[arg(long, value_name = "PATH")]
        ffmpeg: Option<PathBuf>,
    },

    /// Validate the configured login Cookie shape.
    VerifyCookie,

    /// Check whether FFmpeg is available.
    FfmpegCheck {
        /// Use a specific FFmpeg executable instead of searching PATH.
        #[arg(long, value_name = "PATH")]
        ffmpeg: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliCodec {
    Auto,
    Avc,
    Hevc,
    Av1,
}

impl CliCodec {
    const fn as_core_value(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Avc => "avc",
            Self::Hevc => "hevc",
            Self::Av1 => "av1",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliMediaMode {
    AudioVideo,
    VideoOnly,
    AudioOnly,
}

impl From<CliMediaMode> for DownloadMediaMode {
    fn from(value: CliMediaMode) -> Self {
        match value {
            CliMediaMode::AudioVideo => Self::AudioVideo,
            CliMediaMode::VideoOnly => Self::VideoOnly,
            CliMediaMode::AudioOnly => Self::AudioOnly,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliContainer {
    Mp4,
    Mkv,
}

impl CliContainer {
    const fn extension(self) -> &'static str {
        match self {
            Self::Mp4 => "mp4",
            Self::Mkv => "mkv",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliMissingQuality {
    Lower,
    Skip,
}

impl From<CliMissingQuality> for MissingQualityPolicy {
    fn from(value: CliMissingQuality) -> Self {
        match value {
            CliMissingQuality::Lower => Self::Lower,
            CliMissingQuality::Skip => Self::Skip,
        }
    }
}

#[derive(Debug, Serialize)]
struct DownloadCommandResult {
    downloads: Vec<DownloadInputResult>,
}

#[derive(Debug, Serialize)]
struct DownloadInputResult {
    input: String,
    source_title: String,
    outputs: Vec<String>,
}

#[derive(Debug, Serialize)]
struct FfmpegCheckResult {
    available: bool,
    path: Option<String>,
    version: Option<String>,
}

struct DownloadRequest {
    all: bool,
    inputs: Vec<String>,
    parts: Vec<u32>,
    items: Vec<u32>,
    output: PathBuf,
    quality: String,
    audio_quality: String,
    codec: CliCodec,
    media_mode: CliMediaMode,
    container: CliContainer,
    missing_quality: CliMissingQuality,
    ffmpeg: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = env::args_os().collect::<Vec<_>>();
    let json_requested = args
        .iter()
        .take_while(|arg| *arg != "--")
        .any(|arg| arg == "--json");
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            if json_requested && error.use_stderr() {
                print_error(&error.to_string(), true);
                return ExitCode::from(2);
            }
            error.exit();
        }
    };
    let json = cli.json;
    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            print_error(&format!("{error:#}"), json);
            ExitCode::FAILURE
        }
    }
}

fn print_error(message: &str, json: bool) {
    if json {
        eprintln!("{}", serde_json::json!({ "ok": false, "error": message }));
    } else {
        eprintln!("error: {message}");
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    let cookie = load_cookie(cli.cookie_file.as_deref())?;
    if matches!(
        &cli.command,
        Command::Parse { all: true, .. } | Command::Download { all: true, .. }
    ) && cli.state.is_none()
    {
        bail!(
            "--all requires --state <path> so interrupted enumeration can resume without starting over"
        );
    }
    if let Command::Download {
        all: false,
        items,
        parts,
        ..
    } = &cli.command
        && items.is_empty()
        && parts.is_empty()
    {
        bail!(
            "select a bounded download with --items or --parts, or explicitly use --all --state <path>"
        );
    }
    let signature = format!("cwd={:?};{:?}", env::current_dir()?, cli.command);
    let mut progress = Progress::open(cli.state, cli.resume, signature)?;
    let http_guard = std::sync::Arc::new(bdl_core::resolver::http_guard::HttpGuard::new(
        bdl_core::resolver::http_guard::HttpGuard::user_state_path()?,
        std::time::Duration::from_secs_f64(cli.interval_seconds),
        cli.max_http_requests,
    ));
    let resolver = SourceResolver::from_optional_cookie(cookie.as_deref())?
        .with_policy(ResolvePolicy {
            interval: std::time::Duration::ZERO,
            max_operations: cli.max_operations,
        })
        .with_http_guard(http_guard.clone());
    let result = match cli.command {
        Command::Parse {
            input,
            all,
            with_streams,
        } => {
            parse_command(
                &input,
                all,
                with_streams,
                &resolver,
                &mut progress,
                cli.json,
            )
            .await
        }
        Command::Download {
            all,
            inputs,
            parts,
            items,
            output,
            quality,
            audio_quality,
            codec,
            media_mode,
            container,
            missing_quality,
            ffmpeg,
        } => {
            download_command(
                DownloadRequest {
                    all,
                    inputs,
                    parts,
                    items,
                    output,
                    quality,
                    audio_quality,
                    codec,
                    media_mode,
                    container,
                    missing_quality,
                    ffmpeg,
                },
                &resolver,
                &mut progress,
                cli.json,
            )
            .await
        }
        Command::VerifyCookie => verify_cookie_command(cookie.as_deref(), cli.json),
        Command::FfmpegCheck { ffmpeg } => ffmpeg_check_command(ffmpeg, cli.json).await,
    };
    if result
        .as_ref()
        .is_err_and(|error| is_source_restriction(&format!("{error:#}")))
    {
        http_guard.record_restriction().await?;
        progress.restrict()?;
    }
    result
}

async fn parse_command(
    input: &str,
    all: bool,
    with_streams: bool,
    resolver: &SourceResolver,
    progress: &mut Progress,
    json: bool,
) -> anyhow::Result<()> {
    let mut tree = if let Some(tree) = progress.source(input) {
        tree
    } else {
        resolver
            .resolve(
                input,
                ResolveOptions {
                    fetch_streams: with_streams,
                },
            )
            .await?
    };
    progress.remember(input, &tree)?;
    if all {
        load_pages(resolver, progress, input, &mut tree, None).await?;
    }
    if json {
        println!("{}", serde_json::to_string_pretty(&tree)?);
    } else {
        print_source_summary(&tree);
    }
    Ok(())
}

async fn download_command(
    request: DownloadRequest,
    resolver: &SourceResolver,
    progress: &mut Progress,
    json: bool,
) -> anyhow::Result<()> {
    let options = download_options(&request)?;
    let fetcher = ReqwestFetcher::with_config(FetchConfig {
        max_retries: 0,
        ..FetchConfig::default()
    })?
    .with_stop_on_restriction();
    let muxer = MediaMuxer::new(MediaMuxerConfig {
        ffmpeg_path: request.ffmpeg.clone(),
    })?;
    let mut downloads = Vec::with_capacity(request.inputs.len());

    for input in &request.inputs {
        let mut tree = if let Some(tree) = progress.source(input) {
            tree
        } else {
            resolver
                .resolve(
                    input,
                    ResolveOptions {
                        fetch_streams: false,
                    },
                )
                .await
                .with_context(|| format!("failed to parse {input}"))?
        };
        progress.remember(input, &tree)?;
        if !request.parts.is_empty() && tree.source.kind != SourceKind::Video {
            bail!(
                "--parts supports ordinary multi-part videos; use --items for lists and episodes"
            );
        }
        let limit = if request.all {
            None
        } else {
            Some(request.items.iter().max().copied().unwrap_or(1) as usize)
        };
        load_pages(resolver, progress, input, &mut tree, limit).await?;
        let positions = tree
            .groups
            .iter()
            .enumerate()
            .flat_map(|(g, group)| (0..group.items.len()).map(move |i| (g, i)))
            .collect::<Vec<_>>();
        let selected = select_item_positions(positions.len(), &request.items)?;
        if selected.is_empty() {
            bail!("resolved source {input} has no items");
        }
        let mut outputs = Vec::new();
        for position in selected {
            let (g, i) = positions[position];
            let item_id = tree.groups[g].items[i].id.clone();
            if tree.groups[g].items[i]
                .parts
                .iter()
                .any(|part| part.cid.is_none())
            {
                resolver.expand_item(&mut tree, &item_id).await?;
                progress.remember(input, &tree)?;
            }
            let ids = tree.groups[g].items[i]
                .parts
                .iter()
                .map(|part| part.id.clone())
                .collect::<Vec<_>>();
            let selected_part_ids = select_part_ids(&ids, &request.parts)?;
            for part_id in selected_part_ids {
                let task_key = format!("task:{}:{}", tree.source.id.0, part_id.0);
                if let Some(output) = progress.completed(&task_key) {
                    outputs.push(output);
                    continue;
                }
                hydrate_part(resolver, &mut tree, g, i, &part_id).await?;
                let selected_part_ids = vec![part_id];
                let tasks = plan_selected_parts(&tree, &selected_part_ids, &options)
                    .with_context(|| format!("failed to plan download for {input}"))?;
                for mut task in tasks {
                    if let Err(error) = fetch_media(&fetcher, &task).await {
                        if !is_expired_media(&error.to_string()) {
                            return Err(error);
                        }
                        hydrate_part(resolver, &mut tree, g, i, &selected_part_ids[0]).await?;
                        let refreshed = plan_selected_parts(&tree, &selected_part_ids, &options)?;
                        task = refreshed
                            .into_iter()
                            .find(|candidate| {
                                candidate.id == task.id && candidate.output_path == task.output_path
                            })
                            .context("refreshed media no longer matches the planned output")?;
                        fetch_media(&fetcher, &task)
                            .await
                            .context("download failed after one media URL refresh")?;
                    }
                    let video = task
                        .resources
                        .iter()
                        .find(|resource| resource.intent == DownloadResourceIntent::Video);
                    let audio = task
                        .resources
                        .iter()
                        .find(|resource| resource.intent == DownloadResourceIntent::Audio);
                    if video.is_none() && audio.is_none() {
                        bail!(
                            "planned task {} has no downloadable media tracks",
                            task.title
                        );
                    }
                    muxer
                        .mux(&MuxRequest {
                            video_path: video.map(|resource| resource.target_path.clone()),
                            audio_path: audio.map(|resource| resource.target_path.clone()),
                            output_path: task.output_path.clone(),
                            cover_path: None,
                            subtitle_paths: Vec::new(),
                        })
                        .await?;

                    let metadata = tokio::fs::metadata(&task.output_path)
                        .await
                        .context("FFmpeg finished without a readable output file")?;
                    if !metadata.is_file() || metadata.len() == 0 {
                        bail!("FFmpeg finished without a non-empty media file");
                    }

                    progress.finish(task.id.clone(), &task.output_path)?;

                    cleanup_raw_media(
                        video.map(|resource| resource.target_path.as_path()),
                        &task.output_path,
                    )
                    .await;
                    cleanup_raw_media(
                        audio.map(|resource| resource.target_path.as_path()),
                        &task.output_path,
                    )
                    .await;

                    let output = task.output_path.display().to_string();
                    if !json {
                        println!("downloaded {output}");
                    }
                    outputs.push(output);
                }
            }
            // Keep metadata, never expiring stream URLs, for explicit resume.
            progress.remember(input, &tree)?;
        }
        downloads.push(DownloadInputResult {
            input: input.clone(),
            source_title: tree.source.title,
            outputs,
        });
    }

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&DownloadCommandResult { downloads })?
        );
    }
    Ok(())
}

async fn hydrate_part(
    resolver: &SourceResolver,
    tree: &mut NormalizedSourceTree,
    g: usize,
    i: usize,
    part_id: &PartId,
) -> anyhow::Result<()> {
    let item_id = tree.groups[g].items[i].id.clone();
    if matches!(tree.source.kind, SourceKind::Bangumi | SourceKind::Cheese) {
        resolver.hydrate_item(tree, &item_id).await?;
    } else {
        let part = tree.groups[g].items[i]
            .parts
            .iter_mut()
            .find(|p| &p.id == part_id)
            .context("selected part missing")?;
        let media_input = part
            .bvid
            .clone()
            .or_else(|| part.aid.map(|id| format!("av{id}")))
            .context("part has no media identity")?;
        let hydrated = resolver
            .resolve_video_target_streams(&media_input, part.cid.context("part has no CID")?)
            .await?;
        *part = hydrated
            .groups
            .into_iter()
            .flat_map(|g| g.items)
            .flat_map(|i| i.parts)
            .find(|p| &p.id == part_id)
            .context("part disappeared")?;
    }
    Ok(())
}

async fn fetch_media(fetcher: &ReqwestFetcher, task: &DownloadTask) -> anyhow::Result<()> {
    for resource in &task.resources {
        if matches!(
            resource.intent,
            DownloadResourceIntent::Video | DownloadResourceIntent::Audio
        ) {
            fetcher.fetch(resource, None).await?;
        }
    }
    Ok(())
}

fn is_expired_media(message: &str) -> bool {
    !is_source_restriction(message)
        && ["HTTP 404", "HTTP 410"]
            .iter()
            .any(|status| message.contains(status))
}

async fn load_pages(
    resolver: &SourceResolver,
    progress: &mut Progress,
    input: &str,
    tree: &mut NormalizedSourceTree,
    limit: Option<usize>,
) -> anyhow::Result<()> {
    while tree.source.has_more && limit.is_none_or(|limit| tree.source.loaded_count < limit) {
        resolver.load_more(tree).await?;
        progress.remember(input, tree)?;
    }
    Ok(())
}

fn download_options(request: &DownloadRequest) -> anyhow::Result<DownloadOptions> {
    let mut options = DownloadOptions::new(request.output.clone());
    options.video_quality = StreamPreference::parse_video(&request.quality)?;
    options.audio_quality = StreamPreference::parse(&request.audio_quality, "音频质量")?;
    options.video_codec = parse_stream_codec(request.codec.as_core_value())?;
    options.media_mode = request.media_mode.into();
    options.output_extension = request.container.extension().to_owned();
    options.missing_quality_policy = request.missing_quality.into();
    options.archive_assets = ArchiveAssetSelection::none();
    Ok(options)
}

fn select_item_positions(count: usize, numbers: &[u32]) -> anyhow::Result<Vec<usize>> {
    for number in numbers {
        if *number == 0 || *number as usize > count {
            bail!("item {number} is out of range; source has {count} items");
        }
    }
    Ok((0..count)
        .filter(|index| numbers.is_empty() || numbers.contains(&((*index + 1) as u32)))
        .collect())
}

fn verify_cookie_command(cookie: Option<&str>, json: bool) -> anyhow::Result<()> {
    let cookie = cookie.context("set BDL_COOKIE or pass --cookie-file <path>")?;
    let has_sessdata = cookie
        .split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .any(|(name, value)| name == "SESSDATA" && !value.trim().is_empty());
    if !has_sessdata {
        bail!("cookie does not contain SESSDATA");
    }

    if json {
        println!("{}", serde_json::json!({ "valid": true }));
    } else {
        println!("cookie: login cookie shape ok");
    }
    Ok(())
}

async fn ffmpeg_check_command(ffmpeg: Option<PathBuf>, json: bool) -> anyhow::Result<()> {
    match MediaMuxer::new(MediaMuxerConfig {
        ffmpeg_path: ffmpeg,
    }) {
        Ok(muxer) => match muxer.probe().await {
            Ok(probe) => {
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&FfmpegCheckResult {
                            available: true,
                            path: Some(probe.path.display().to_string()),
                            version: Some(probe.version),
                        })?
                    );
                } else {
                    println!("ffmpeg: found {}", probe.path.display());
                    println!("{}", probe.version);
                }
                Ok(())
            }
            Err(error) => Err(error.into()),
        },
        Err(MuxError::FfmpegNotFound { .. }) => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&FfmpegCheckResult {
                        available: false,
                        path: None,
                        version: None,
                    })?
                );
            } else {
                println!("ffmpeg: missing");
            }
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

fn load_cookie(cookie_file: Option<&Path>) -> anyhow::Result<Option<String>> {
    let cookie = if let Some(path) = cookie_file {
        Some(
            std::fs::read_to_string(path)
                .with_context(|| format!("failed to read cookie file {}", path.display()))?,
        )
    } else {
        env::var("BDL_COOKIE").ok()
    };
    Ok(cookie
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty()))
}

fn select_part_ids(ids: &[PartId], numbers: &[u32]) -> anyhow::Result<Vec<PartId>> {
    if numbers.is_empty() {
        return Ok(ids.to_vec());
    }
    for number in numbers {
        if *number == 0 || *number as usize > ids.len() {
            bail!(
                "part {number} is out of range; source has {} parts",
                ids.len()
            );
        }
    }
    Ok(ids
        .iter()
        .enumerate()
        .filter(|(index, _)| numbers.contains(&((*index + 1) as u32)))
        .map(|(_, id)| id.clone())
        .collect())
}

fn print_source_summary(tree: &NormalizedSourceTree) {
    println!("{}", tree.source.title);
    println!("kind: {:?}", tree.source.kind);
    match tree.source.total_count {
        Some(total) => println!("loaded: {} / {total}", tree.source.loaded_count),
        None => println!("loaded: {}", tree.source.loaded_count),
    }
    if tree.source.has_more {
        println!("more: yes");
    }
}

async fn cleanup_raw_media(path: Option<&Path>, output_path: &Path) {
    let Some(path) = path.filter(|path| *path != output_path) else {
        return;
    };
    match tokio::fs::remove_file(path).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => eprintln!("warning: failed to remove {}: {error}", path.display()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_only_targets_expired_media_not_restrictions_or_resource_ids() {
        assert!(is_expired_media("请求资源长度失败: HTTP 404 Not Found"));
        assert!(is_expired_media("下载请求失败: HTTP 410 Gone"));
        assert!(!is_expired_media("HTTP 403 Forbidden"));
        assert!(!is_expired_media("HTTP 429 Too Many Requests"));
        assert!(!is_expired_media("resource:404 timed out"));
    }

    #[tokio::test]
    async fn resume_completed_part_uses_checkpoint_without_resolving_input() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let state = env::temp_dir().join(format!("bdl-resume-{}-{stamp}.json", std::process::id()));
        let media = state.with_extension("mp4");
        std::fs::write(&media, b"completed media fixture").unwrap();
        let exe = env::current_exe().unwrap();
        let cli = Cli::try_parse_from([
            "bdl",
            "download",
            "intentionally-invalid-input",
            "--parts",
            "1",
            "-o",
            ".",
            "--json",
            "--ffmpeg",
            exe.to_str().unwrap(),
            "--state",
            state.to_str().unwrap(),
            "--resume",
        ])
        .unwrap();
        let signature = format!("cwd={:?};{:?}", env::current_dir().unwrap(), cli.command);
        let mut progress = Progress::open(Some(state.clone()), false, signature).unwrap();
        let tree: NormalizedSourceTree = serde_json::from_value(serde_json::json!({
            "source":{"id":"video:one","kind":"video","input":"intentionally-invalid-input","title":"title","loaded_count":1,"total_count":1,"has_more":false},
            "groups":[{"id":"group:one","kind":"video","title":"title","page":null,"items":[{
                "id":"item:one","title":"title","owner_name":null,"cover_url":null,"duration_seconds":1,
                "parts":[{"id":"part:one","title":"part","aid":1,"bvid":"one","cid":2,"duration_seconds":1,"streams":[],"assets":[]}]
            }]}]
        })).unwrap();
        progress
            .remember("intentionally-invalid-input", &tree)
            .unwrap();
        progress
            .finish("task:video:one:part:one".into(), &media)
            .unwrap();
        drop(progress);
        run(cli).await.unwrap();
        std::fs::remove_file(state).unwrap();
        std::fs::remove_file(media).unwrap();
    }

    #[test]
    fn list_selection_is_ordered_deduplicated_and_bounded() {
        assert_eq!(
            select_item_positions(30, &[21, 2, 21]).unwrap(),
            vec![1, 20]
        );
        assert_eq!(select_item_positions(3, &[]).unwrap(), vec![0, 1, 2]);
        assert!(select_item_positions(3, &[4]).is_err());
        assert!(select_item_positions(3, &[0]).is_err());
        let cli = Cli::try_parse_from([
            "bdl",
            "download",
            "https://space.bilibili.com/123",
            "-o",
            "downloads",
            "--items",
            "1,2",
        ])
        .unwrap();
        assert!(matches!(cli.command, Command::Download { items, .. } if items == vec![1, 2]));
        let cli = Cli::try_parse_from(["bdl", "parse", "BV1ZA411g7Sb", "--all", "--json"]).unwrap();
        assert!(matches!(cli.command, Command::Parse { all: true, .. }));
    }

    #[test]
    fn part_selection_preserves_source_order_and_rejects_out_of_range() {
        let ids = (1..=3)
            .map(|n| PartId(format!("part:{n}")))
            .collect::<Vec<_>>();
        assert_eq!(select_part_ids(&ids, &[2, 1, 2]).unwrap(), ids[..2]);
        assert_eq!(select_part_ids(&ids, &[]).unwrap(), ids);
        assert!(select_part_ids(&ids, &[0]).is_err());
        assert!(select_part_ids(&ids, &[4]).is_err());
        assert!(
            Cli::try_parse_from([
                "bdl",
                "download",
                "BV1ZA411g7Sb",
                "-o",
                "downloads",
                "--parts",
                "0"
            ])
            .is_err()
        );
    }

    #[test]
    fn parse_command_accepts_global_json_after_subcommand() {
        let cli = Cli::try_parse_from(["bdl", "parse", "BV1xx411c7mD", "--json"])
            .expect("CLI should parse");
        assert!(cli.json);
        assert!(matches!(cli.command, Command::Parse { .. }));
    }

    #[test]
    fn download_command_accepts_multiple_inputs_and_media_options() {
        let cli = Cli::try_parse_from([
            "bdl",
            "download",
            "BV1xx411c7mD",
            "BV1Q541167Qg",
            "--output",
            "downloads",
            "--quality",
            "80",
            "--codec",
            "hevc",
            "--media-mode",
            "video-only",
            "--container",
            "mkv",
        ])
        .expect("CLI should parse");

        let Command::Download {
            inputs,
            quality,
            codec,
            media_mode,
            container,
            ..
        } = cli.command
        else {
            panic!("expected download command");
        };
        assert_eq!(inputs.len(), 2);
        assert_eq!(quality, "80");
        assert!(matches!(codec, CliCodec::Hevc));
        assert!(matches!(media_mode, CliMediaMode::VideoOnly));
        assert!(matches!(container, CliContainer::Mkv));
    }
}
