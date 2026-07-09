use bdl_core::model::{
    AssetKind, FetchPolicy, MediaKind, NormalizedGroup, NormalizedItem, NormalizedPart,
    NormalizedSourceTree, SourceKind, SourceSummary, StreamCodec, StreamQuality,
};

#[test]
fn normalized_source_tree_serializes_stable_shape() {
    let tree = NormalizedSourceTree {
        source: SourceSummary {
            id: "src_1".into(),
            kind: SourceKind::Video,
            input: "BV1xx411c7mD".into(),
            title: "Example".into(),
            loaded_count: 1,
            total_count: Some(1),
            has_more: false,
        },
        groups: vec![NormalizedGroup {
            id: "grp_1".into(),
            kind: "video".into(),
            title: "Example".into(),
            items: vec![NormalizedItem {
                id: "item_1".into(),
                title: "Example".into(),
                owner_name: Some("owner".into()),
                cover_url: None,
                duration_seconds: Some(120),
                parts: vec![NormalizedPart {
                    id: "part_1".into(),
                    title: "P1".into(),
                    aid: Some(1),
                    bvid: Some("BV1xx411c7mD".into()),
                    cid: Some(2),
                    streams: vec![],
                    assets: vec![AssetKind::Cover.with_policy(FetchPolicy::OnDemand)],
                }],
            }],
            page: None,
        }],
    };

    let json = serde_json::to_value(&tree).expect("tree serializes");
    assert_eq!(json["source"]["kind"], "video");
    assert_eq!(
        json["groups"][0]["items"][0]["parts"][0]["assets"][0]["kind"],
        "cover"
    );
}

#[test]
fn stream_quality_and_codec_are_frontend_safe_strings() {
    assert_eq!(
        serde_json::to_string(&StreamQuality::Best).unwrap(),
        "\"best\""
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
