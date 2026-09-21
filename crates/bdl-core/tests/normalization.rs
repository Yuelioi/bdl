use bdl_core::ids::{GroupId, ItemId, PartId, SourceId};
use bdl_core::model::{
    AssetKind, FetchPolicy, HeaderPair, MediaKind, MediaStream, NormalizedGroup, NormalizedItem,
    NormalizedPart, NormalizedSourceTree, PageState, SourceKind, SourceSummary, StreamCodec,
    StreamQuality,
};
use chrono::{TimeZone, Utc};
use serde_json::json;

#[test]
fn normalized_source_tree_serializes_stable_shape() {
    let tree = NormalizedSourceTree {
        source: SourceSummary {
            id: SourceId::from("src_1"),
            kind: SourceKind::Video,
            input: "BV1xx411c7mD".into(),
            title: "Example".into(),
            loaded_count: 1,
            total_count: Some(2),
            has_more: true,
        },
        groups: vec![NormalizedGroup {
            id: GroupId::from("grp_1"),
            kind: "video".into(),
            title: "Example".into(),
            items: vec![NormalizedItem {
                id: ItemId::from("item_1"),
                title: "Example".into(),
                owner_name: Some("owner".into()),
                owner_mid: Some(42),
                publish_date: Some("2026-07-08".into()),
                cover_url: None,
                duration_seconds: None,
                parts: vec![NormalizedPart {
                    id: PartId::from("part_1"),
                    title: "P1".into(),
                    aid: Some(1),
                    bvid: Some("BV1xx411c7mD".into()),
                    cid: Some(2),
                    duration_seconds: None,
                    streams: vec![
                        MediaStream {
                            id: "stream_video_720".into(),
                            kind: MediaKind::Video,
                            quality: StreamQuality::Quality(720),
                            codec: StreamCodec::Hevc,
                            bandwidth: Some(4_000_000),
                            urls: vec!["https://example.test/video.m4s".into()],
                            headers: vec![HeaderPair {
                                name: "referer".into(),
                                value: "https://www.bilibili.com".into(),
                            }],
                            acquired_at: Utc
                                .with_ymd_and_hms(2026, 7, 9, 8, 30, 0)
                                .single()
                                .expect("valid timestamp"),
                        },
                        MediaStream {
                            id: "stream_audio_best".into(),
                            kind: MediaKind::Audio,
                            quality: StreamQuality::Best,
                            codec: StreamCodec::Auto,
                            bandwidth: None,
                            urls: vec!["https://example.test/audio.m4s".into()],
                            headers: vec![],
                            acquired_at: Utc
                                .with_ymd_and_hms(2026, 7, 9, 8, 31, 0)
                                .single()
                                .expect("valid timestamp"),
                        },
                    ],
                    assets: vec![AssetKind::Cover.with_policy(FetchPolicy::OnDemand)],
                }],
            }],
            page: Some(PageState {
                page_number: 1,
                page_size: 20,
                loaded_count: 1,
                total_count: None,
                has_more: false,
            }),
        }],
    };

    let json = serde_json::to_value(&tree).expect("tree serializes");
    assert_eq!(
        json,
        json!({
            "source": {
                "id": "src_1",
                "kind": "video",
                "input": "BV1xx411c7mD",
                "title": "Example",
                "loaded_count": 1,
                "total_count": 2,
                "has_more": true
            },
            "groups": [
                {
                    "id": "grp_1",
                    "kind": "video",
                    "title": "Example",
                    "items": [
                        {
                            "id": "item_1",
                            "title": "Example",
                            "owner_name": "owner",
                            "owner_mid": 42,
                            "publish_date": "2026-07-08",
                            "cover_url": null,
                            "duration_seconds": null,
                            "parts": [
                                {
                                    "id": "part_1",
                                    "title": "P1",
                                    "aid": 1,
                                    "bvid": "BV1xx411c7mD",
                                    "cid": 2,
                                    "duration_seconds": null,
                                    "streams": [
                                        {
                                            "id": "stream_video_720",
                                            "kind": "video",
                                            "quality": {
                                                "kind": "quality",
                                                "value": 720
                                            },
                                            "codec": "hevc",
                                            "bandwidth": 4000000,
                                            "urls": ["https://example.test/video.m4s"],
                                            "headers": [
                                                {
                                                    "name": "referer",
                                                    "value": "https://www.bilibili.com"
                                                }
                                            ],
                                            "acquired_at": "2026-07-09T08:30:00Z"
                                        },
                                        {
                                            "id": "stream_audio_best",
                                            "kind": "audio",
                                            "quality": {
                                                "kind": "best"
                                            },
                                            "codec": "auto",
                                            "bandwidth": null,
                                            "urls": ["https://example.test/audio.m4s"],
                                            "headers": [],
                                            "acquired_at": "2026-07-09T08:31:00Z"
                                        }
                                    ],
                                    "assets": [
                                        {
                                            "kind": "cover",
                                            "format": null,
                                            "fetch_policy": "on_demand",
                                            "urls": [],
                                            "headers": []
                                        }
                                    ]
                                }
                            ]
                        }
                    ],
                    "page": {
                        "page_number": 1,
                        "page_size": 20,
                        "loaded_count": 1,
                        "total_count": null,
                        "has_more": false
                    }
                }
            ]
        })
    );
}

#[test]
fn stream_quality_uses_tagged_frontend_safe_shape() {
    assert_eq!(
        serde_json::to_value(StreamQuality::Best).unwrap(),
        json!({ "kind": "best" })
    );
    assert_eq!(
        serde_json::to_value(StreamQuality::Quality(720)).unwrap(),
        json!({ "kind": "quality", "value": 720 })
    );
    assert_eq!(
        serde_json::to_string(&StreamCodec::Hevc).unwrap(),
        "\"hevc\""
    );
    assert_eq!(
        serde_json::to_string(&MediaKind::Audio).unwrap(),
        "\"audio\""
    );
}
