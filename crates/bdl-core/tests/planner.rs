use std::path::PathBuf;

use chrono::Utc;

use bdl_core::ids::{GroupId, ItemId, PartId, SourceId};
use bdl_core::model::{
    AssetKind, FetchPolicy, HeaderPair, MediaKind, MediaStream, NormalizedGroup, NormalizedItem,
    NormalizedPart, NormalizedSourceTree, SourceKind, SourceSummary, StreamCodec, StreamQuality,
};
use bdl_core::planner::{ArchiveMode, DownloadOptions, plan_selected_parts};
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
    assert!(
        tasks[0]
            .output_path
            .ends_with("Fixture Video/Fixture Video - P1 - P1.mp4")
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

    assert!(
        tasks[0]
            .output_path
            .ends_with("Fixture Video/Fixture Video - P1 - P1.mp4")
    );
    assert!(
        tasks[1]
            .output_path
            .ends_with("Fixture Video/Fixture Video - P1 - P1 (1).mp4")
    );
    assert!(
        tasks[1].resources[0]
            .target_path
            .ends_with("Fixture Video/Fixture Video - P1 - P1 (1).video.m4s")
    );
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
                        AssetKind::Subtitle.with_policy(FetchPolicy::OnDemand),
                        AssetKind::Danmaku.with_policy(FetchPolicy::OnDemand),
                        AssetKind::Nfo.with_policy(FetchPolicy::OnDemand),
                    ],
                }],
            }],
            page: None,
        }],
    }
}
