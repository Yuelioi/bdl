use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;

use bdl_core::ids::{GroupId, ItemId, PartId, SourceId};
use bdl_core::model::{
    AssetKind, DerivedAsset, FetchPolicy, HeaderPair, MediaKind, MediaStream, NormalizedGroup,
    NormalizedItem, NormalizedPart, NormalizedSourceTree, SourceKind, SourceSummary, StreamCodec,
    StreamQuality,
};
use bdl_core::naming::DuplicateNamingStrategy;
use bdl_core::planner::{
    ArchiveAssetSelection, ArchiveMode, DownloadOptions, MissingQualityPolicy, StreamPreference,
    plan_selected_parts,
};
use bdl_core::queue::DownloadTaskRefreshInput;
use bdl_core::queue::{DownloadResourceIntent, DownloadResourceKind, ResourceStatus, TaskStatus};

#[test]
fn plan_selected_parts_creates_one_task_for_one_selected_part() {
    let tree = fixture_tree(true);
    let options = DownloadOptions::new(PathBuf::from("downloads"));

    let tasks = plan_selected_parts(&tree, &[PartId("part:BV1:100".to_owned())], &options)
        .expect("selected part should plan");

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, "task:source:BV1:part:BV1:100");
    assert_eq!(tasks[0].source_id, "source:BV1");
    assert_eq!(tasks[0].status, TaskStatus::Waiting);
    assert!(tasks[0].output_path.ends_with("Fixture Video/P1 - P1.mp4"));
    assert_eq!(
        tasks[0]
            .refresh_intent
            .as_ref()
            .expect("video task should be refreshable")
            .input,
        DownloadTaskRefreshInput::VideoBvid {
            bvid: "BV1xx411c7mD".to_owned(),
        }
    );
}

#[test]
fn plan_selected_parts_fast_mode_creates_video_and_audio_resources_only() {
    let tree = fixture_tree(true);
    let options = DownloadOptions::new(PathBuf::from("downloads"));

    let tasks = plan_selected_parts(&tree, &[PartId("part:BV1:100".to_owned())], &options)
        .expect("selected part should plan");

    let intents: Vec<_> = tasks[0]
        .resources
        .iter()
        .map(|resource| resource.intent)
        .collect();
    assert_eq!(
        intents,
        vec![DownloadResourceIntent::Video, DownloadResourceIntent::Audio]
    );
    assert!(
        tasks[0]
            .resources
            .iter()
            .all(|resource| resource.status == ResourceStatus::Pending)
    );
    assert!(
        tasks[0]
            .resources
            .iter()
            .all(|resource| resource.kind != DownloadResourceKind::Asset)
    );
}

#[test]
fn plan_selected_parts_complete_archive_adds_derived_asset_intents() {
    let tree = fixture_tree(true);
    let options = DownloadOptions::new(PathBuf::from("downloads"))
        .with_archive_mode(ArchiveMode::CompleteArchive);

    let tasks = plan_selected_parts(&tree, &[PartId("part:BV1:100".to_owned())], &options)
        .expect("selected part should plan");

    let intents: Vec<_> = tasks[0]
        .resources
        .iter()
        .map(|resource| resource.intent)
        .collect();
    assert_eq!(
        intents,
        vec![
            DownloadResourceIntent::Video,
            DownloadResourceIntent::Audio,
            DownloadResourceIntent::Cover,
            DownloadResourceIntent::Subtitle,
            DownloadResourceIntent::Danmaku,
            DownloadResourceIntent::Nfo,
        ]
    );
}

#[test]
fn plan_selected_parts_complete_archive_uses_asset_urls_and_formats() {
    let tree = fixture_tree(true);
    let options = DownloadOptions::new(PathBuf::from("downloads"))
        .with_archive_mode(ArchiveMode::CompleteArchive);

    let tasks = plan_selected_parts(&tree, &[PartId("part:BV1:100".to_owned())], &options)
        .expect("complete archive should plan asset resources");

    let subtitle = resource_by_intent(&tasks[0], DownloadResourceIntent::Subtitle);
    assert_eq!(
        subtitle.current_urls,
        vec!["https://example.invalid/subtitle.json".to_owned()]
    );
    assert!(
        subtitle
            .target_path
            .ends_with("Fixture Video/P1 - P1.subtitle.json")
    );
    assert!(
        subtitle
            .headers
            .iter()
            .any(|header| header.name == "Referer")
    );

    let danmaku = resource_by_intent(&tasks[0], DownloadResourceIntent::Danmaku);
    assert_eq!(
        danmaku.current_urls,
        vec!["https://example.invalid/danmaku.xml".to_owned()]
    );
    assert!(
        danmaku
            .target_path
            .ends_with("Fixture Video/P1 - P1.danmaku.xml")
    );
}

#[test]
fn plan_selected_parts_custom_archive_uses_selected_asset_intents_only() {
    let tree = fixture_tree(true);
    let mut options =
        DownloadOptions::new(PathBuf::from("downloads")).with_archive_mode(ArchiveMode::Custom);
    options.archive_assets = ArchiveAssetSelection {
        cover: false,
        subtitles: true,
        danmaku: false,
        nfo: true,
    };

    let tasks = plan_selected_parts(&tree, &[PartId("part:BV1:100".to_owned())], &options)
        .expect("custom archive should plan selected asset resources");

    let intents = tasks[0]
        .resources
        .iter()
        .map(|resource| resource.intent)
        .collect::<Vec<_>>();
    assert_eq!(
        intents,
        vec![
            DownloadResourceIntent::Video,
            DownloadResourceIntent::Audio,
            DownloadResourceIntent::Subtitle,
            DownloadResourceIntent::Nfo,
        ]
    );
}

#[test]
fn plan_selected_parts_returns_actionable_error_when_audio_stream_missing() {
    let tree = fixture_tree(false);
    let options = DownloadOptions::new(PathBuf::from("downloads"));

    let error = plan_selected_parts(&tree, &[PartId("part:BV1:100".to_owned())], &options)
        .expect_err("missing audio should fail planning");

    assert!(error.to_string().contains("缺少音频流"));
    assert!(error.to_string().contains("重新解析"));
}

#[test]
fn plan_selected_parts_generates_stable_resource_ids_for_resume() {
    let tree = fixture_tree(true);
    let options = DownloadOptions::new(PathBuf::from("downloads"));

    let first = plan_selected_parts(&tree, &[PartId("part:BV1:100".to_owned())], &options)
        .expect("first plan should succeed");
    let second = plan_selected_parts(&tree, &[PartId("part:BV1:100".to_owned())], &options)
        .expect("second plan should succeed");

    assert_eq!(first[0].id, second[0].id);
    assert_eq!(first[0].resources[0].id, second[0].resources[0].id);
    assert_eq!(first[0].resources[1].id, second[0].resources[1].id);
}

#[test]
fn plan_selected_parts_adds_suffix_for_duplicate_output_paths() {
    let tree = fixture_tree(true);
    let options = DownloadOptions::new(PathBuf::from("downloads"));

    let tasks = plan_selected_parts(
        &tree,
        &[
            PartId("part:BV1:100".to_owned()),
            PartId("part:BV1:100".to_owned()),
        ],
        &options,
    )
    .expect("duplicate selection should still plan unique paths");

    assert!(tasks[0].output_path.ends_with("Fixture Video/P1 - P1.mp4"));
    assert!(
        tasks[1]
            .output_path
            .ends_with("Fixture Video/P1 - P1 (1).mp4")
    );
    assert!(
        tasks[1].resources[0]
            .target_path
            .ends_with("Fixture Video/P1 - P1 (1).video.m4s")
    );
}

#[test]
fn plan_selected_parts_can_overwrite_existing_files_without_batch_path_collisions() {
    let tree = fixture_tree(true);
    let output_dir = unique_temp_dir();
    let existing_output = output_dir.join("Fixture Video").join("P1 - P1.mp4");
    fs::create_dir_all(existing_output.parent().expect("output should have parent"))
        .expect("test output dir should be created");
    fs::write(&existing_output, b"existing").expect("existing output should be written");
    let mut options = DownloadOptions::new(output_dir.clone());
    options.duplicate_naming_strategy = DuplicateNamingStrategy::OverwriteExisting;

    let tasks = plan_selected_parts(
        &tree,
        &[
            PartId("part:BV1:100".to_owned()),
            PartId("part:BV1:100".to_owned()),
        ],
        &options,
    )
    .expect("overwrite mode should still plan duplicate selections");

    assert_eq!(tasks[0].output_path, existing_output);
    assert_eq!(
        tasks[1].output_path,
        output_dir.join("Fixture Video").join("P1 - P1 (1).mp4")
    );

    let _ = fs::remove_dir_all(output_dir);
}

#[test]
fn plan_selected_parts_uses_configured_video_and_audio_quality() {
    let mut tree = fixture_tree(true);
    let part = fixture_part_mut(&mut tree);
    part.streams.push(media_stream(
        MediaKind::Video,
        StreamQuality::Quality(64),
        StreamCodec::Avc,
        "https://example.invalid/video-64.m4s",
    ));
    part.streams.push(media_stream(
        MediaKind::Audio,
        StreamQuality::Quality(30216),
        StreamCodec::Unknown,
        "https://example.invalid/audio-30216.m4s",
    ));
    let mut options = DownloadOptions::new(PathBuf::from("downloads"));
    options.video_quality = StreamPreference::Quality(64);
    options.audio_quality = StreamPreference::Quality(30216);

    let tasks = plan_selected_parts(&tree, &[PartId("part:BV1:100".to_owned())], &options)
        .expect("configured stream qualities should plan");

    assert_eq!(
        tasks[0].resources[0].current_urls,
        vec!["https://example.invalid/video-64.m4s"]
    );
    assert_eq!(
        tasks[0].resources[1].current_urls,
        vec!["https://example.invalid/audio-30216.m4s"]
    );
    assert_eq!(tasks[0].media_selection.video_quality, "64");
    assert_eq!(tasks[0].media_selection.audio_quality, "30216");
    assert_eq!(tasks[0].media_selection.container, "mp4");
}

#[test]
fn plan_selected_parts_prefers_configured_video_codec_when_available() {
    let mut tree = fixture_tree(true);
    fixture_part_mut(&mut tree).streams.push(media_stream(
        MediaKind::Video,
        StreamQuality::Quality(80),
        StreamCodec::Hevc,
        "https://example.invalid/video-hevc.m4s",
    ));
    let mut options = DownloadOptions::new(PathBuf::from("downloads"));
    options.video_codec = StreamCodec::Hevc;

    let tasks = plan_selected_parts(&tree, &[PartId("part:BV1:100".to_owned())], &options)
        .expect("configured codec should plan");

    assert_eq!(
        tasks[0].resources[0].current_urls,
        vec!["https://example.invalid/video-hevc.m4s"]
    );
    assert_eq!(tasks[0].media_selection.video_codec, "hevc");
}

#[test]
fn plan_selected_parts_prefers_episode_refresh_intent_for_bangumi_sources() {
    let mut tree = fixture_tree(true);
    tree.source.kind = SourceKind::Bangumi;
    let part = fixture_part_mut(&mut tree);
    part.id = PartId("part:bangumi:123:456:789".to_owned());
    part.cid = Some(789);

    let tasks = plan_selected_parts(
        &tree,
        &[PartId("part:bangumi:123:456:789".to_owned())],
        &DownloadOptions::new(PathBuf::from("downloads")),
    )
    .expect("bangumi source should plan");

    let refresh_intent = tasks[0]
        .refresh_intent
        .as_ref()
        .expect("bangumi task should be refreshable");
    assert_eq!(
        refresh_intent.input,
        DownloadTaskRefreshInput::BangumiEpisode { ep_id: 456 }
    );
    assert_eq!(refresh_intent.cid, 789);
}

#[test]
fn plan_selected_parts_chooses_lower_quality_when_target_is_unavailable() {
    let tree = fixture_tree(true);
    let mut options = DownloadOptions::new(PathBuf::from("downloads"));
    options.video_quality = StreamPreference::Quality(120);
    options.missing_quality_policy = MissingQualityPolicy::Lower;

    let tasks = plan_selected_parts(&tree, &[PartId("part:BV1:100".to_owned())], &options)
        .expect("lower policy should choose available quality");

    assert_eq!(
        tasks[0].resources[0].current_urls,
        vec!["https://example.invalid/video.m4s"]
    );
}

#[test]
fn plan_selected_parts_rejects_missing_quality_when_policy_requires_user_action() {
    let tree = fixture_tree(true);
    let mut options = DownloadOptions::new(PathBuf::from("downloads"));
    options.video_quality = StreamPreference::Quality(120);
    options.missing_quality_policy = MissingQualityPolicy::Ask;

    let error = plan_selected_parts(&tree, &[PartId("part:BV1:100".to_owned())], &options)
        .expect_err("ask policy should block missing target quality");

    assert!(error.to_string().contains("没有 120"));
    assert!(error.to_string().contains("需要用户确认"));
}

fn fixture_tree(include_audio: bool) -> NormalizedSourceTree {
    let mut streams = vec![MediaStream {
        id: "stream:video:80".to_owned(),
        kind: MediaKind::Video,
        quality: StreamQuality::Quality(80),
        codec: StreamCodec::Avc,
        bandwidth: Some(1_200_000),
        urls: vec!["https://example.invalid/video.m4s".to_owned()],
        headers: vec![HeaderPair {
            name: "Referer".to_owned(),
            value: "https://www.bilibili.com/".to_owned(),
        }],
        acquired_at: Utc::now(),
    }];

    if include_audio {
        streams.push(MediaStream {
            id: "stream:audio:30280".to_owned(),
            kind: MediaKind::Audio,
            quality: StreamQuality::Quality(30280),
            codec: StreamCodec::Unknown,
            bandwidth: Some(192_000),
            urls: vec!["https://example.invalid/audio.m4s".to_owned()],
            headers: vec![HeaderPair {
                name: "Referer".to_owned(),
                value: "https://www.bilibili.com/".to_owned(),
            }],
            acquired_at: Utc::now(),
        });
    }

    NormalizedSourceTree {
        source: SourceSummary {
            id: SourceId("source:BV1".to_owned()),
            kind: SourceKind::Video,
            input: "BV1xx411c7mD".to_owned(),
            title: "Fixture Video".to_owned(),
            loaded_count: 1,
            total_count: Some(1),
            has_more: false,
        },
        groups: vec![NormalizedGroup {
            id: GroupId("group:BV1".to_owned()),
            kind: "video".to_owned(),
            title: "Fixture Video".to_owned(),
            items: vec![NormalizedItem {
                id: ItemId("item:BV1".to_owned()),
                title: "Fixture Video".to_owned(),
                owner_name: Some("owner".to_owned()),
                cover_url: Some("https://example.invalid/cover.jpg".to_owned()),
                duration_seconds: Some(42),
                parts: vec![NormalizedPart {
                    id: PartId("part:BV1:100".to_owned()),
                    title: "P1".to_owned(),
                    aid: Some(1),
                    bvid: Some("BV1xx411c7mD".to_owned()),
                    cid: Some(100),
                    streams,
                    assets: vec![
                        AssetKind::Cover.with_policy(FetchPolicy::OnDemand),
                        DerivedAsset::with_urls(
                            AssetKind::Subtitle,
                            FetchPolicy::OnDemand,
                            "json",
                            vec!["https://example.invalid/subtitle.json".to_owned()],
                            asset_headers(),
                        ),
                        DerivedAsset::with_urls(
                            AssetKind::Danmaku,
                            FetchPolicy::OnDemand,
                            "xml",
                            vec!["https://example.invalid/danmaku.xml".to_owned()],
                            asset_headers(),
                        ),
                        AssetKind::Nfo.with_policy(FetchPolicy::OnDemand),
                    ],
                }],
            }],
            page: None,
        }],
    }
}

fn media_stream(
    kind: MediaKind,
    quality: StreamQuality,
    codec: StreamCodec,
    url: &str,
) -> MediaStream {
    MediaStream {
        id: format!("stream:{kind:?}:{url}"),
        kind,
        quality,
        codec,
        bandwidth: Some(1_000_000),
        urls: vec![url.to_owned()],
        headers: vec![HeaderPair {
            name: "Referer".to_owned(),
            value: "https://www.bilibili.com/".to_owned(),
        }],
        acquired_at: Utc::now(),
    }
}

fn fixture_part_mut(tree: &mut NormalizedSourceTree) -> &mut NormalizedPart {
    &mut tree.groups[0].items[0].parts[0]
}

fn resource_by_intent(
    task: &bdl_core::queue::DownloadTask,
    intent: DownloadResourceIntent,
) -> &bdl_core::queue::DownloadResource {
    task.resources
        .iter()
        .find(|resource| resource.intent == intent)
        .expect("resource should exist")
}

fn asset_headers() -> Vec<HeaderPair> {
    vec![HeaderPair {
        name: "Referer".to_owned(),
        value: "https://www.bilibili.com/".to_owned(),
    }]
}

fn unique_temp_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();

    std::env::temp_dir().join(format!("bdl-planner-{nanos}"))
}
