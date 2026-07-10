use async_trait::async_trait;
use bdl_core::error::BdlError;
use bdl_core::input::ClassifiedInput;
use bdl_core::model::{AssetKind, FetchPolicy, MediaKind, SourceKind, StreamCodec, StreamQuality};
use bdl_core::resolver::video::{
    BpiVideoApi, ResolvedDashStream, ResolvedPlayUrl, ResolvedPlayerInfo, ResolvedSubtitle,
    ResolvedVideo, ResolvedVideoPage, VideoApi, VideoInputId, VideoResolver,
};
use bdl_core::resolver::{ResolveOptions, Resolver};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
struct FakeVideoApi {
    video: ResolvedVideo,
    play_url: ResolvedPlayUrl,
    player_info: ResolvedPlayerInfo,
}

#[derive(Debug, Clone)]
struct CountingVideoApi {
    calls: Arc<Mutex<Vec<(&'static str, u64)>>>,
}

#[async_trait]
impl VideoApi for CountingVideoApi {
    async fn view(&self, _id: &VideoInputId) -> Result<ResolvedVideo, BdlError> {
        Ok(ResolvedVideo {
            aid: 170001,
            bvid: "BV1xx411c7mD".to_owned(),
            default_cid: 62131,
            title: "large fixture".to_owned(),
            owner_name: None,
            cover_url: None,
            pages: (62131..62321)
                .enumerate()
                .map(|(index, cid)| ResolvedVideoPage {
                    cid,
                    index: index as u32 + 1,
                    title: format!("P{}", index + 1),
                    duration_seconds: Some(30),
                })
                .collect(),
        })
    }

    async fn play_url(&self, _id: &VideoInputId, cid: u64) -> Result<ResolvedPlayUrl, BdlError> {
        self.calls
            .lock()
            .expect("calls should lock")
            .push(("play", cid));
        Ok(ResolvedPlayUrl::default())
    }

    async fn player_info(
        &self,
        _id: &VideoInputId,
        cid: u64,
    ) -> Result<ResolvedPlayerInfo, BdlError> {
        self.calls
            .lock()
            .expect("calls should lock")
            .push(("player", cid));
        Ok(ResolvedPlayerInfo::default())
    }
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

    async fn player_info(
        &self,
        id: &VideoInputId,
        cid: u64,
    ) -> Result<ResolvedPlayerInfo, BdlError> {
        assert_eq!(id, &VideoInputId::Bvid("BV1xx411c7mD".to_owned()));
        assert_eq!(cid, 62131);
        Ok(self.player_info.clone())
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
async fn video_resolver_hydrates_only_the_selected_cid() -> Result<(), BdlError> {
    let calls = Arc::new(Mutex::new(Vec::new()));
    let resolver = VideoResolver::with_api(CountingVideoApi {
        calls: Arc::clone(&calls),
    });

    let tree = resolver
        .resolve_target_streams(ClassifiedInput::VideoBvid("BV1xx411c7mD".to_owned()), 62132)
        .await?;

    assert_eq!(
        calls.lock().expect("calls should lock").as_slice(),
        &[("play", 62132), ("player", 62132)]
    );
    let parts = &tree.groups[0].items[0].parts;
    assert_eq!(parts.len(), 190);
    assert!(parts[0].streams.is_empty());
    assert!(parts[2].streams.is_empty());
    Ok(())
}

#[tokio::test]
async fn video_resolver_maps_archive_asset_urls_when_streams_are_requested() -> Result<(), BdlError>
{
    let resolver = VideoResolver::with_api(fake_api());

    let tree = resolver
        .resolve(
            ClassifiedInput::VideoBvid("BV1xx411c7mD".to_owned()),
            ResolveOptions {
                fetch_streams: true,
            },
        )
        .await?;

    let assets = &tree.groups[0].items[0].parts[0].assets;
    let subtitle = assets
        .iter()
        .find(|asset| asset.kind == AssetKind::Subtitle)
        .expect("subtitle asset should exist");
    assert_eq!(subtitle.format.as_deref(), Some("json"));
    assert_eq!(
        subtitle.urls,
        vec!["https://subtitle.example.invalid/zh.json".to_owned()]
    );
    assert!(
        subtitle
            .headers
            .iter()
            .any(|header| header.name == "Referer")
    );

    let danmaku = assets
        .iter()
        .find(|asset| asset.kind == AssetKind::Danmaku)
        .expect("danmaku asset should exist");
    assert_eq!(danmaku.format.as_deref(), Some("xml"));
    assert_eq!(
        danmaku.urls,
        vec!["https://comment.bilibili.com/62131.xml".to_owned()]
    );

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
        player_info: ResolvedPlayerInfo {
            subtitles: vec![ResolvedSubtitle {
                lan: "zh-CN".to_owned(),
                lan_doc: "中文".to_owned(),
                url: "//subtitle.example.invalid/zh.json".to_owned(),
            }],
        },
    }
}
