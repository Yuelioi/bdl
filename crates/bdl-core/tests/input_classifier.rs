use bdl_core::input::{ClassifiedInput, classify_input};
use bdl_core::model::SourceKind;

#[test]
fn classifies_bv_and_av_ids() {
    assert_eq!(
        classify_input("BV1xx411c7mD").unwrap(),
        ClassifiedInput::VideoBvid("BV1xx411c7mD".into())
    );

    assert_eq!(
        classify_input("av170001").unwrap(),
        ClassifiedInput::VideoAid(170001)
    );
}

#[test]
fn classifies_video_url() {
    let input = "https://www.bilibili.com/video/BV1xx411c7mD/?spm_id_from=333.1007";
    assert_eq!(
        classify_input(input).unwrap(),
        ClassifiedInput::VideoBvid("BV1xx411c7mD".into())
    );
}

#[test]
fn classifies_av_video_url() {
    assert_eq!(
        classify_input("https://www.bilibili.com/video/av170001/?spm_id_from=333.1007").unwrap(),
        ClassifiedInput::VideoAid(170001)
    );
}

#[test]
fn classifies_short_url_as_unknown_source_kind() {
    let input = "http://b23.tv/abc123";
    let classified = classify_input(input).unwrap();

    assert_eq!(classified, ClassifiedInput::ShortUrl(input.into()));
    assert_eq!(classified.source_kind(), SourceKind::Unknown);
}

#[test]
fn classifies_supported_url_kinds_without_extracting_final_ids() {
    assert_eq!(
        classify_input("https://space.bilibili.com/12345/video")
            .unwrap()
            .source_kind(),
        SourceKind::Uploader
    );
    assert_eq!(
        classify_input("https://space.bilibili.com/12345/favlist?fid=678")
            .unwrap()
            .source_kind(),
        SourceKind::Favorite
    );
    assert_eq!(
        classify_input("https://www.bilibili.com/bangumi/play/ss123")
            .unwrap()
            .source_kind(),
        SourceKind::Bangumi
    );
    assert_eq!(
        classify_input("https://www.bilibili.com/cheese/play/ss456")
            .unwrap()
            .source_kind(),
        SourceKind::Cheese
    );
}

#[test]
fn extracts_mid_from_plain_uploader_space_url() {
    assert_eq!(
        classify_input("https://space.bilibili.com/12345").unwrap(),
        ClassifiedInput::Uploader { mid: 12345 }
    );
}

#[test]
fn preserves_raw_urls_for_supported_source_urls() {
    let favorite = "https://space.bilibili.com/12345/favlist?fid=678";
    let collection = "https://www.bilibili.com/medialist/play/12345?season_id=678";
    let series = "https://space.bilibili.com/12345/lists/987?type=series&series_id=987";
    let bangumi = "https://www.bilibili.com/bangumi/play/ss123";
    let cheese = "https://www.bilibili.com/cheese/play/ss456";

    assert_eq!(
        classify_input(favorite).unwrap(),
        ClassifiedInput::Favorite {
            raw_url: favorite.into()
        }
    );
    assert_eq!(
        classify_input(collection).unwrap(),
        ClassifiedInput::Collection {
            raw_url: collection.into()
        }
    );
    assert_eq!(
        classify_input(series).unwrap(),
        ClassifiedInput::Series {
            raw_url: series.into()
        }
    );
    assert_eq!(
        classify_input(bangumi).unwrap(),
        ClassifiedInput::Bangumi {
            raw_url: bangumi.into()
        }
    );
    assert_eq!(
        classify_input(cheese).unwrap(),
        ClassifiedInput::Cheese {
            raw_url: cheese.into()
        }
    );
}

#[test]
fn rejects_invalid_or_overlong_bvid_tokens() {
    assert!(classify_input("BV1xx411c7mDextra").is_err());
    assert!(classify_input("https://www.bilibili.com/video/BV1xx411c7mDextra").is_err());
}

#[test]
fn rejects_false_positives_from_non_bilibili_urls() {
    let inputs = [
        "https://example.com/video/av170001",
        "https://example.com/video/BV1xx411c7mD",
        "https://example.com/watch?next=https://space.bilibili.com/12345/video",
        "https://example.com/watch?season_id=678",
        "https://example.com/series/987",
        "https://example.com/bangumi/play/ss123",
        "https://example.com/cheese/play/ss456",
    ];

    for input in inputs {
        assert!(classify_input(input).is_err(), "{input} should be rejected");
    }
}

#[test]
fn rejects_unknown_input_with_actionable_message() {
    let err = classify_input("not a bilibili thing").unwrap_err();
    assert!(err.to_string().contains("无法识别这个输入"));
}
