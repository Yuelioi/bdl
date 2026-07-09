use async_trait::async_trait;
use bdl_core::error::BdlError;
use bdl_core::input::ClassifiedInput;
use bdl_core::model::{AssetKind, FetchPolicy, MediaKind, SourceKind, StreamCodec, StreamQuality};
use bdl_core::resolver::video::{
    BpiVideoApi, ResolvedDashStream, ResolvedPlayUrl, ResolvedVideo, ResolvedVideoPage, VideoApi,
    VideoInputId, VideoResolver,
};
use bdl_core::resolver::{ResolveOptions, Resolver};

#[derive(Debug, Clone)]
struct FakeVideoApi {
    video: ResolvedVideo,
    play_url: ResolvedPlayUrl,
}

#[async_trait]
impl VideoApi for FakeVideoApi {
    async fn view(&self, id: &VideoInputId) -> Result<ResolvedVideo, BdlError> {
        assert_eq!(id, &VideoInputId::Bvid("BV1xx411c7mD".to_owned()));
        Ok(self.video.clone())
    }

    async fn play_url(&self, id: &VideoInputId, cid: u64) -> Result<ResolvedPlayUrl, BdlError> {
        assert_eq!(id, &VideoInputId::Bvid("BV1xx411c7mD".to_owned()));
        assert_eq!(cid, 62131);
        Ok(self.play_url.clone())
    }
}

#[tokio::test]
async fn video_resolver_resolves_bv_without_streams() -> Result<(), BdlError> {
    let resolver = VideoResolver::with_api(fake_api());

    let tree = resolver
        .resolve(
            ClassifiedInput::VideoBvid("BV1xx411c7mD".to_owned()),
            ResolveOptions {
                fetch_streams: false,
            },
        )
        .await?;

    assert_eq!(tree.source.kind, SourceKind::Video);
    assert_eq!(tree.source.id.0, "video:BV1xx411c7mD");
    assert_eq!(tree.source.input, "BV1xx411c7mD");
    assert_eq!(tree.source.title, "fixture video");
    assert_eq!(tree.source.loaded_count, 1);
    assert_eq!(tree.source.total_count, Some(1));
    assert!(!tree.source.has_more);

    let group = tree.groups.first().expect("group should exist");
    assert_eq!(group.id.0, "group:BV1xx411c7mD");
    assert_eq!(group.kind, "video");
    assert_eq!(group.title, "fixture video");

    let item = group.items.first().expect("item should exist");
    assert_eq!(item.id.0, "item:BV1xx411c7mD");
    assert_eq!(item.title, "fixture video");
    assert_eq!(item.owner_name.as_deref(), Some("fixture owner"));
    assert_eq!(
        item.cover_url.as_deref(),
        Some("https://example.invalid/cover.jpg")
    );
    assert_eq!(item.duration_seconds, Some(42));

    let part = item.parts.first().expect("part should exist");
    assert_eq!(part.id.0, "part:BV1xx411c7mD:62131");
    assert_eq!(part.title, "P1");
    assert_eq!(part.aid, Some(170001));
    assert_eq!(part.bvid.as_deref(), Some("BV1xx411c7mD"));
    assert_eq!(part.cid, Some(62131));
    assert!(part.streams.is_empty());
    assert_eq!(part.assets[0].kind, AssetKind::Cover);
    assert_eq!(part.assets[0].fetch_policy, FetchPolicy::OnDemand);

    Ok(())
}

#[tokio::test]
async fn video_resolver_maps_dash_streams_when_requested() -> Result<(), BdlError> {
    let resolver = VideoResolver::with_api(fake_api());

    let tree = resolver
        .resolve(
            ClassifiedInput::VideoBvid("BV1xx411c7mD".to_owned()),
            ResolveOptions {
                fetch_streams: true,
            },
        )
        .await?;

    let streams = &tree.groups[0].items[0].parts[0].streams;
    let video = streams
        .iter()
        .find(|stream| stream.kind == MediaKind::Video)
        .expect("video stream should exist");
    assert_eq!(video.quality, StreamQuality::Quality(80));
    assert_eq!(video.codec, StreamCodec::Avc);
    assert_eq!(video.bandwidth, Some(1_200_000));
    assert_eq!(
        video.urls,
        vec![
            "https://example.invalid/video.m4s".to_owned(),
            "https://example.invalid/video-backup.m4s".to_owned()
        ]
    );
    assert!(video.headers.iter().any(|header| header.name == "Referer"));

    let audio = streams
        .iter()
        .find(|stream| stream.kind == MediaKind::Audio)
        .expect("audio stream should exist");
    assert_eq!(audio.quality, StreamQuality::Quality(30280));
    assert_eq!(audio.codec, StreamCodec::Unknown);
    assert_eq!(audio.bandwidth, Some(192_000));

    Ok(())
}

#[tokio::test]
async fn video_resolver_rejects_non_video_input() {
    let resolver = VideoResolver::with_api(fake_api());

    let error = resolver
        .resolve(
            ClassifiedInput::Uploader { mid: 1000001 },
            ResolveOptions::default(),
        )
        .await
        .expect_err("uploader input should not resolve through video resolver");

    assert!(matches!(
        error,
        BdlError::UnsupportedSource { kind } if kind == "uploader"
    ));
}

#[tokio::test]
async fn live_video_resolver_resolves_bv_when_enabled() -> Result<(), BdlError> {
    if std::env::var("BDL_LIVE_TEST").ok().as_deref() != Some("1") {
        return Ok(());
    }

    let resolver = VideoResolver::with_api(BpiVideoApi::new()?);
    let tree = resolver
        .resolve(
            ClassifiedInput::VideoBvid("BV1xx411c7mD".to_owned()),
            ResolveOptions {
                fetch_streams: false,
            },
        )
        .await?;

    assert!(!tree.groups.is_empty());
    assert!(!tree.groups[0].items.is_empty());
    assert!(!tree.groups[0].items[0].parts.is_empty());
    Ok(())
}

fn fake_api() -> FakeVideoApi {
    FakeVideoApi {
        video: ResolvedVideo {
            aid: 170001,
            bvid: "BV1xx411c7mD".to_owned(),
            default_cid: 62131,
            title: "fixture video".to_owned(),
            owner_name: Some("fixture owner".to_owned()),
            cover_url: Some("https://example.invalid/cover.jpg".to_owned()),
            pages: vec![ResolvedVideoPage {
                cid: 62131,
                index: 1,
                title: "P1".to_owned(),
                duration_seconds: Some(42),
            }],
        },
        play_url: ResolvedPlayUrl {
            video: vec![ResolvedDashStream {
                id: 80,
                base_url: "https://example.invalid/video.m4s".to_owned(),
                backup_urls: vec!["https://example.invalid/video-backup.m4s".to_owned()],
                bandwidth: Some(1_200_000),
                codecs: "avc1.640032".to_owned(),
            }],
            audio: vec![ResolvedDashStream {
                id: 30280,
                base_url: "https://example.invalid/audio.m4s".to_owned(),
                backup_urls: Vec::new(),
                bandwidth: Some(192_000),
                codecs: "mp4a.40.2".to_owned(),
            }],
        },
    }
}
