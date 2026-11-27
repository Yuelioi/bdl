use async_trait::async_trait;
use bdl_core::error::BdlError;
use bdl_core::input::ClassifiedInput;
use bdl_core::model::{AssetKind, FetchPolicy, MediaKind, SourceKind};
use bdl_core::planner::{DownloadOptions, plan_selected_parts};
use bdl_core::resolver::bangumi::{
    BangumiApi, BangumiInputId, BangumiResolver, ResolvedBangumiDashStream, ResolvedBangumiEpisode,
    ResolvedBangumiPlayUrl, ResolvedBangumiSeason, ResolvedBangumiSubtitle, bangumi_id_from_url,
};
use bdl_core::resolver::{ResolveOptions, Resolver};

#[derive(Debug, Clone)]
struct FakeBangumiApi {
    season: ResolvedBangumiSeason,
    expected_id: BangumiInputId,
    play_url: ResolvedBangumiPlayUrl,
}

#[tokio::test]
async fn mobile_bangumi_share_targets_reach_season_and_episode_resolvers() -> Result<(), BdlError> {
    for (raw_url, expected_id) in [
        (
            "https://m.bilibili.com/bangumi/play/ss123?share_source=COPY",
            BangumiInputId::Season(123),
        ),
        (
            "https://m.bilibili.com/bangumi/play/ep456?share_source=COPY",
            BangumiInputId::Episode(456),
        ),
    ] {
        let tree = BangumiResolver::with_api(fake_api(expected_id))
            .resolve(
                bdl_core::input::classify_input(raw_url)?,
                ResolveOptions::default(),
            )
            .await?;
        assert_eq!(tree.source.kind, SourceKind::Bangumi);
        assert_eq!(tree.source.id.0, "bangumi:123");
        assert_eq!(tree.source.input, raw_url);
        assert!(tree.source.loaded_count > 0);
    }
    Ok(())
}

#[async_trait]
impl BangumiApi for FakeBangumiApi {
    async fn season(&self, id: BangumiInputId) -> Result<ResolvedBangumiSeason, BdlError> {
        assert_eq!(id, self.expected_id);
        Ok(self.season.clone())
    }

    async fn play_url(
        &self,
        episode: &ResolvedBangumiEpisode,
    ) -> Result<ResolvedBangumiPlayUrl, BdlError> {
        assert_eq!(episode.ep_id, 456);
        Ok(self.play_url.clone())
    }
}

#[tokio::test]
async fn bangumi_resolver_maps_season_without_streams() -> Result<(), BdlError> {
    let resolver = BangumiResolver::with_api(fake_api(BangumiInputId::Season(123)));
    let raw_url = "https://www.bilibili.com/bangumi/play/ss123";

    let tree = resolver
        .resolve(
            ClassifiedInput::Bangumi {
                raw_url: raw_url.to_owned(),
            },
            ResolveOptions::default(),
        )
        .await?;

    assert_eq!(tree.source.kind, SourceKind::Bangumi);
    assert_eq!(tree.source.id.0, "bangumi:123");
    assert_eq!(tree.source.input, raw_url);
    assert_eq!(tree.source.title, "fixture season");
    assert_eq!(tree.source.loaded_count, 2);
    assert_eq!(tree.source.total_count, Some(2));
    assert!(!tree.source.has_more);

    let group = tree.groups.first().expect("group should exist");
    assert_eq!(group.id.0, "group:bangumi:123");
    assert_eq!(group.kind, "bangumi");
    assert!(group.page.is_none());

    let item = group.items.first().expect("item should exist");
    assert_eq!(item.id.0, "item:bangumi:123:455");
    assert_eq!(item.title, "第1话 - opening");
    assert_eq!(item.owner_name.as_deref(), Some("fixture studio"));
    assert_eq!(item.publish_date.as_deref(), Some("2025-12-30"));

    let part = item.parts.first().expect("part should exist");
    assert_eq!(part.id.0, "part:bangumi:123:455:9001");
    assert_eq!(part.aid, Some(170001));
    assert_eq!(part.bvid.as_deref(), Some("BV1xx411c7mD"));
    assert_eq!(part.cid, Some(9001));
    assert!(part.streams.is_empty());
    assert_eq!(part.assets[0].kind, AssetKind::Cover);
    assert_eq!(part.assets[0].fetch_policy, FetchPolicy::OnDemand);

    Ok(())
}

#[tokio::test]
async fn bangumi_resolver_fetches_streams_for_episode_input_only() -> Result<(), BdlError> {
    let resolver = BangumiResolver::with_api(fake_api(BangumiInputId::Episode(456)));

    let tree = resolver
        .resolve(
            ClassifiedInput::Bangumi {
                raw_url: "https://www.bilibili.com/bangumi/play/ep456".to_owned(),
            },
            ResolveOptions {
                fetch_streams: true,
            },
        )
        .await?;

    let first_part = &tree.groups[0].items[0].parts[0];
    let focused_part = &tree.groups[0].items[1].parts[0];

    assert!(first_part.streams.is_empty());
    assert_eq!(focused_part.id.0, "part:bangumi:123:456:9002");
    assert_eq!(focused_part.streams.len(), 2);
    assert!(
        focused_part
            .streams
            .iter()
            .any(|stream| stream.kind == MediaKind::Video)
    );
    assert!(
        focused_part
            .streams
            .iter()
            .any(|stream| stream.kind == MediaKind::Audio)
    );
    let danmaku = focused_part
        .assets
        .iter()
        .find(|asset| asset.kind == AssetKind::Danmaku)
        .expect("focused bangumi episode should expose its CID danmaku URL");
    assert_eq!(
        danmaku.urls,
        vec!["https://comment.bilibili.com/9002.xml".to_owned()]
    );
    let subtitle = focused_part
        .assets
        .iter()
        .find(|asset| asset.kind == AssetKind::Subtitle)
        .expect("focused bangumi episode should expose player-info subtitles");
    assert_eq!(
        subtitle.urls,
        vec!["https://subtitle.example.invalid/zh.json".to_owned()]
    );

    let mut movie_paths = Vec::new();
    for (movie_title, episode_title, task_title, file_name) in [
        ("变形金刚", "正片", "变形金刚 · 正片", "变形金刚 · 正片.mp4"),
        (
            "电影 - 修复版",
            "正片",
            "电影 - 修复版 · 正片",
            "电影 - 修复版 · 正片.mp4",
        ),
        ("同名电影", "同名电影", "同名电影", "同名电影.mp4"),
    ] {
        let mut api = fake_api(BangumiInputId::Episode(456));
        api.season.is_movie = true;
        api.season.title = movie_title.to_owned();
        api.season.episodes[1].title = episode_title.to_owned();
        let movie_tree = BangumiResolver::with_api(api)
            .resolve(
                ClassifiedInput::Bangumi {
                    raw_url: "https://www.bilibili.com/bangumi/play/ep456".to_owned(),
                },
                ResolveOptions {
                    fetch_streams: true,
                },
            )
            .await?;
        let item = &movie_tree.groups[0].items[1];
        let part = &item.parts[0];
        assert_eq!(item.title, task_title);
        assert_eq!(part.title, task_title);

        let mut options = DownloadOptions::new("downloads".into());
        let grouped = plan_selected_parts(&movie_tree, std::slice::from_ref(&part.id), &options)?;
        assert_eq!(
            grouped[0].output_path,
            options
                .output_dir
                .join(movie_title)
                .join(format!("{episode_title}.mp4"))
        );
        options.naming_template = "{part_title}".to_owned();
        let tasks = plan_selected_parts(&movie_tree, std::slice::from_ref(&part.id), &options)?;
        assert_eq!(tasks[0].title, task_title);
        assert!(tasks[0].output_path.ends_with(file_name));
        movie_paths.push(tasks[0].output_path.clone());
    }
    assert_ne!(movie_paths[0], movie_paths[1]);

    Ok(())
}

#[tokio::test]
async fn bangumi_resolver_omits_subtitle_asset_when_player_info_has_none() -> Result<(), BdlError> {
    let mut api = fake_api(BangumiInputId::Episode(456));
    api.play_url.subtitles.clear();
    let tree = BangumiResolver::with_api(api)
        .resolve(
            ClassifiedInput::Bangumi {
                raw_url: "https://www.bilibili.com/bangumi/play/ep456".to_owned(),
            },
            ResolveOptions {
                fetch_streams: true,
            },
        )
        .await?;
    let focused_part = &tree.groups[0].items[1].parts[0];

    assert!(
        focused_part
            .assets
            .iter()
            .all(|asset| asset.kind != AssetKind::Subtitle)
    );
    assert!(
        focused_part
            .assets
            .iter()
            .any(|asset| asset.kind == AssetKind::Danmaku)
    );
    Ok(())
}

#[test]
fn bangumi_id_from_url_accepts_ss_ep_and_query() -> Result<(), BdlError> {
    assert_eq!(
        bangumi_id_from_url("https://www.bilibili.com/bangumi/play/ss123")?,
        BangumiInputId::Season(123)
    );
    assert_eq!(
        bangumi_id_from_url("https://www.bilibili.com/bangumi/play/ep456")?,
        BangumiInputId::Episode(456)
    );
    assert_eq!(
        bangumi_id_from_url("https://www.bilibili.com/bangumi/play/ss123?ep_id=456")?,
        BangumiInputId::Episode(456)
    );
    Ok(())
}

fn fake_api(expected_id: BangumiInputId) -> FakeBangumiApi {
    FakeBangumiApi {
        season: ResolvedBangumiSeason {
            season_id: 123,
            title: "fixture season".to_owned(),
            is_movie: false,
            owner_name: Some("fixture studio".to_owned()),
            cover_url: Some("https://example.invalid/season.jpg".to_owned()),
            total_count: Some(2),
            focused_ep_id: None,
            episodes: vec![
                ResolvedBangumiEpisode {
                    aid: 170001,
                    bvid: "BV1xx411c7mD".to_owned(),
                    cid: 9001,
                    ep_id: 455,
                    title: "第1话 - opening".to_owned(),
                    publish_date: Some("2025-12-30".to_owned()),
                    cover_url: Some("https://example.invalid/ep1.jpg".to_owned()),
                    duration_seconds: Some(1200),
                },
                ResolvedBangumiEpisode {
                    aid: 170002,
                    bvid: "BV1yy411c7mD".to_owned(),
                    cid: 9002,
                    ep_id: 456,
                    title: "第2话 - ending".to_owned(),
                    publish_date: Some("2025-12-31".to_owned()),
                    cover_url: Some("https://example.invalid/ep2.jpg".to_owned()),
                    duration_seconds: Some(1201),
                },
            ],
        },
        expected_id,
        play_url: ResolvedBangumiPlayUrl {
            video: vec![dash_stream(80, "video", "avc1.640032")],
            audio: vec![dash_stream(30280, "audio", "mp4a.40.2")],
            subtitles: vec![ResolvedBangumiSubtitle {
                lan: "zh-CN".to_owned(),
                lan_doc: "中文（简体）".to_owned(),
                url: "//subtitle.example.invalid/zh.json".to_owned(),
            }],
        },
    }
}

fn dash_stream(id: u64, label: &str, codecs: &str) -> ResolvedBangumiDashStream {
    ResolvedBangumiDashStream {
        id,
        base_url: format!("https://example.invalid/{label}.m4s"),
        backup_urls: vec![format!("https://backup.example.invalid/{label}.m4s")],
        bandwidth: Some(1_000_000),
        codecs: codecs.to_owned(),
    }
}
