use async_trait::async_trait;
use bdl_core::error::BdlError;
use bdl_core::input::ClassifiedInput;
use bdl_core::model::{AssetKind, FetchPolicy, MediaKind, SourceKind};
use bdl_core::resolver::cheese::{
    CheeseApi, CheeseInputId, CheeseResolver, ResolvedCheeseDashStream, ResolvedCheeseEpisode,
    ResolvedCheesePage, ResolvedCheesePlayUrl, cheese_id_from_url,
};
use bdl_core::resolver::paged::PageRequest;
use bdl_core::resolver::{ResolveOptions, Resolver};

#[derive(Debug, Clone)]
struct FakeCheeseApi {
    page: ResolvedCheesePage,
    expected_id: CheeseInputId,
    expected_request: PageRequest,
    play_url: ResolvedCheesePlayUrl,
}

#[async_trait]
impl CheeseApi for FakeCheeseApi {
    async fn page(
        &self,
        id: CheeseInputId,
        request: PageRequest,
    ) -> Result<ResolvedCheesePage, BdlError> {
        assert_eq!(id, self.expected_id);
        assert_eq!(request, self.expected_request);
        Ok(self.page.clone())
    }

    async fn play_url(
        &self,
        episode: &ResolvedCheeseEpisode,
    ) -> Result<ResolvedCheesePlayUrl, BdlError> {
        assert_eq!(episode.ep_id, 456);
        Ok(self.play_url.clone())
    }
}

#[tokio::test]
async fn cheese_resolver_maps_first_page_without_streams() -> Result<(), BdlError> {
    let resolver = CheeseResolver::with_api(fake_api(
        CheeseInputId::Season(123),
        PageRequest {
            page_number: 1,
            page_size: 100,
        },
    ));
    let raw_url = "https://www.bilibili.com/cheese/play/ss123";

    let tree = resolver
        .resolve(
            ClassifiedInput::Cheese {
                raw_url: raw_url.to_owned(),
            },
            ResolveOptions::default(),
        )
        .await?;

    assert_eq!(tree.source.kind, SourceKind::Cheese);
    assert_eq!(tree.source.id.0, "cheese:123");
    assert_eq!(tree.source.input, raw_url);
    assert_eq!(tree.source.title, "fixture course");
    assert_eq!(tree.source.loaded_count, 2);
    assert_eq!(tree.source.total_count, Some(12));
    assert!(tree.source.has_more);

    let group = tree.groups.first().expect("group should exist");
    assert_eq!(group.id.0, "group:cheese:123");
    assert_eq!(group.kind, "cheese");
    assert_eq!(group.page.as_ref().map(|page| page.page_size), Some(100));

    let item = group.items.first().expect("item should exist");
    assert_eq!(item.id.0, "item:cheese:123:455");
    assert_eq!(item.title, "P1 - intro");
    assert_eq!(item.owner_name.as_deref(), Some("fixture teacher"));
    assert_eq!(item.publish_date.as_deref(), Some("2025-12-31"));

    let part = item.parts.first().expect("part should exist");
    assert_eq!(part.id.0, "part:cheese:123:455:9001");
    assert_eq!(part.aid, Some(170001));
    assert_eq!(part.bvid, None);
    assert_eq!(part.cid, Some(9001));
    assert!(part.streams.is_empty());
    assert_eq!(part.assets[0].kind, AssetKind::Cover);
    assert_eq!(part.assets[0].fetch_policy, FetchPolicy::OnDemand);

    Ok(())
}

#[tokio::test]
async fn cheese_resolver_fetches_streams_for_episode_input_only() -> Result<(), BdlError> {
    let resolver = CheeseResolver::with_api(fake_api(
        CheeseInputId::Episode(456),
        PageRequest {
            page_number: 1,
            page_size: 100,
        },
    ));

    let tree = resolver
        .resolve(
            ClassifiedInput::Cheese {
                raw_url: "https://www.bilibili.com/cheese/play/ep456".to_owned(),
            },
            ResolveOptions {
                fetch_streams: true,
            },
        )
        .await?;

    let first_part = &tree.groups[0].items[0].parts[0];
    let focused_part = &tree.groups[0].items[1].parts[0];

    assert!(first_part.streams.is_empty());
    assert_eq!(focused_part.id.0, "part:cheese:123:456:9002");
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

    Ok(())
}

#[tokio::test]
async fn cheese_resolver_resolves_requested_page() -> Result<(), BdlError> {
    let resolver = CheeseResolver::with_api(FakeCheeseApi {
        page: ResolvedCheesePage {
            request: PageRequest {
                page_number: 2,
                page_size: 30,
            },
            episodes: vec![ResolvedCheeseEpisode {
                ep_id: 457,
                cid: 9003,
                title: "P3 - advanced".to_owned(),
                ..episode(3, 457, 9003, "advanced")
            }],
            ..fake_page()
        },
        expected_id: CheeseInputId::Season(123),
        expected_request: PageRequest {
            page_number: 2,
            page_size: 30,
        },
        play_url: fake_play_url(),
    });

    let tree = resolver
        .resolve_page(
            123,
            PageRequest {
                page_number: 2,
                page_size: 30,
            },
        )
        .await?;

    assert_eq!(
        tree.groups[0].page.as_ref().map(|page| page.page_number),
        Some(2)
    );
    assert_eq!(tree.groups[0].items[0].id.0, "item:cheese:123:457");

    Ok(())
}

#[test]
fn cheese_id_from_url_accepts_ss_ep_and_query() -> Result<(), BdlError> {
    assert_eq!(
        cheese_id_from_url("https://www.bilibili.com/cheese/play/ss123")?,
        CheeseInputId::Season(123)
    );
    assert_eq!(
        cheese_id_from_url("https://www.bilibili.com/cheese/play/ep456")?,
        CheeseInputId::Episode(456)
    );
    assert_eq!(
        cheese_id_from_url("https://www.bilibili.com/cheese/play/ss123?ep_id=456")?,
        CheeseInputId::Episode(456)
    );
    Ok(())
}

fn fake_api(expected_id: CheeseInputId, expected_request: PageRequest) -> FakeCheeseApi {
    FakeCheeseApi {
        page: fake_page(),
        expected_id,
        expected_request,
        play_url: fake_play_url(),
    }
}

fn fake_page() -> ResolvedCheesePage {
    ResolvedCheesePage {
        season_id: 123,
        title: "fixture course".to_owned(),
        owner_name: Some("fixture teacher".to_owned()),
        cover_url: Some("https://example.invalid/course.jpg".to_owned()),
        request: PageRequest {
            page_number: 1,
            page_size: 100,
        },
        total_count: Some(12),
        has_more: true,
        focused_ep_id: None,
        episodes: vec![
            episode(1, 455, 9001, "intro"),
            episode(2, 456, 9002, "basics"),
        ],
    }
}

fn episode(index: u32, ep_id: u64, cid: u64, title: &str) -> ResolvedCheeseEpisode {
    ResolvedCheeseEpisode {
        aid: 170000 + u64::from(index),
        cid,
        ep_id,
        index,
        title: format!("P{index} - {title}"),
        publish_date: Some("2025-12-31".to_owned()),
        duration_seconds: Some(600 + u64::from(index)),
    }
}

fn fake_play_url() -> ResolvedCheesePlayUrl {
    ResolvedCheesePlayUrl {
        video: vec![dash_stream(80, "video", "avc1.640032")],
        audio: vec![dash_stream(30280, "audio", "mp4a.40.2")],
    }
}

fn dash_stream(id: u64, label: &str, codecs: &str) -> ResolvedCheeseDashStream {
    ResolvedCheeseDashStream {
        id,
        base_url: format!("https://example.invalid/{label}.m4s"),
        backup_urls: vec![format!("https://backup.example.invalid/{label}.m4s")],
        bandwidth: Some(1_000_000),
        codecs: codecs.to_owned(),
    }
}
