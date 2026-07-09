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
fn rejects_unknown_input_with_actionable_message() {
    let err = classify_input("not a bilibili thing").unwrap_err();
    assert!(err.to_string().contains("无法识别这个输入"));
}
