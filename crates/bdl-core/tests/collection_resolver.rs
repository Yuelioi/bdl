use async_trait::async_trait;
use bdl_core::error::BdlError;
use bdl_core::input::ClassifiedInput;
use bdl_core::model::{AssetKind, FetchPolicy, SourceKind};
use bdl_core::resolver::collection::{
    CollectionApi, CollectionInputIds, CollectionResolver, ResolvedArchiveVideo,
    ResolvedCollectionPage, ResolvedSeriesPage, SeriesApi, SeriesInputIds, SeriesResolver,
    collection_ids_from_url, series_ids_from_url,
};
use bdl_core::resolver::paged::PageRequest;
use bdl_core::resolver::{ResolveOptions, Resolver};

#[derive(Debug, Clone)]
struct FakeCollectionApi {
    page: ResolvedCollectionPage,
    expected_ids: CollectionInputIds,
    expected_request: PageRequest,
}

#[derive(Debug, Clone)]
struct FakeSeriesApi {
    page: ResolvedSeriesPage,
    expected_ids: SeriesInputIds,
    expected_request: PageRequest,
}

#[async_trait]
impl CollectionApi for FakeCollectionApi {
    async fn collection_page(
        &self,
        ids: CollectionInputIds,
        request: PageRequest,
    ) -> Result<ResolvedCollectionPage, BdlError> {
        assert_eq!(ids, self.expected_ids);
        assert_eq!(request, self.expected_request);
        Ok(self.page.clone())
    }
}

#[async_trait]
impl SeriesApi for FakeSeriesApi {
    async fn series_page(
        &self,
        ids: SeriesInputIds,
        request: PageRequest,
    ) -> Result<ResolvedSeriesPage, BdlError> {
        assert_eq!(ids, self.expected_ids);
        assert_eq!(request, self.expected_request);
        Ok(self.page.clone())
    }
}

#[tokio::test]
async fn collection_resolver_maps_first_page() -> Result<(), BdlError> {
    let resolver = CollectionResolver::with_api(fake_collection_api());
    let raw_url = "https://www.bilibili.com/medialist/play/1001?season_id=678";

    let tree = resolver
        .resolve(
            ClassifiedInput::Collection {
                raw_url: raw_url.to_owned(),
            },
            ResolveOptions::default(),
        )
        .await?;

    assert_eq!(tree.source.kind, SourceKind::Collection);
    assert_eq!(tree.source.id.0, "collection:1001:678");
    assert_eq!(tree.source.input, raw_url);
    assert_eq!(tree.source.title, "fixture collection");
    assert_eq!(tree.source.loaded_count, 1);
    assert_eq!(tree.source.total_count, Some(38));
    assert!(tree.source.has_more);

    let group = tree.groups.first().expect("group should exist");
    assert_eq!(group.id.0, "group:collection:1001:678");
    assert_eq!(group.kind, "collection");
    assert_eq!(group.page.as_ref().map(|page| page.page_size), Some(20));

    let item = group.items.first().expect("item should exist");
    assert_eq!(item.id.0, "item:collection:1001:678:BV1xx411c7mD");
    assert_eq!(item.title, "collection video");
    assert_eq!(item.duration_seconds, Some(62));

    let part = item.parts.first().expect("part should exist");
    assert_eq!(part.id.0, "part:collection:1001:678:BV1xx411c7mD");
    assert_eq!(part.aid, Some(170001));
    assert_eq!(part.bvid.as_deref(), Some("BV1xx411c7mD"));
    assert_eq!(part.cid, None);
    assert!(part.streams.is_empty());
    assert_eq!(part.assets[0].kind, AssetKind::Cover);
    assert_eq!(part.assets[0].fetch_policy, FetchPolicy::OnDemand);

    Ok(())
}

#[tokio::test]
async fn series_resolver_maps_first_page_without_mid_in_input() -> Result<(), BdlError> {
    let resolver = SeriesResolver::with_api(FakeSeriesApi {
        expected_ids: SeriesInputIds {
            mid: None,
            series_id: 987,
        },
        ..fake_series_api()
    });
    let raw_url = "https://www.bilibili.com/video/series?series_id=987";

    let tree = resolver
        .resolve(
            ClassifiedInput::Series {
                raw_url: raw_url.to_owned(),
            },
            ResolveOptions::default(),
        )
        .await?;

    assert_eq!(tree.source.kind, SourceKind::Series);
    assert_eq!(tree.source.id.0, "series:1001:987");
    assert_eq!(tree.source.input, raw_url);
    assert_eq!(tree.source.title, "fixture series");
    assert_eq!(tree.source.total_count, Some(12));

    let item = tree.groups[0].items.first().expect("item should exist");
    assert_eq!(item.id.0, "item:series:1001:987:BV1yy411c7mD");
    assert_eq!(item.owner_name.as_deref(), Some("fixture creator"));

    Ok(())
}

#[tokio::test]
async fn collection_resolver_resolves_requested_page() -> Result<(), BdlError> {
    let resolver = CollectionResolver::with_api(FakeCollectionApi {
        page: ResolvedCollectionPage {
            request: PageRequest {
                page_number: 2,
                page_size: 20,
            },
            videos: vec![archive_video("BV1zz411c7mD", "collection page 2")],
            ..fake_collection_page()
        },
        expected_ids: CollectionInputIds {
            mid: 1001,
            season_id: 678,
        },
        expected_request: PageRequest {
            page_number: 2,
            page_size: 20,
        },
    });

    let tree = resolver
        .resolve_page(
            CollectionInputIds {
                mid: 1001,
                season_id: 678,
            },
            PageRequest {
                page_number: 2,
                page_size: 20,
            },
        )
        .await?;

    assert_eq!(
        tree.groups[0].page.as_ref().map(|page| page.page_number),
        Some(2)
    );
    assert_eq!(
        tree.groups[0].items[0].id.0,
        "item:collection:1001:678:BV1zz411c7mD"
    );
    Ok(())
}

#[tokio::test]
async fn series_resolver_resolves_requested_page() -> Result<(), BdlError> {
    let resolver = SeriesResolver::with_api(FakeSeriesApi {
        page: ResolvedSeriesPage {
            request: PageRequest {
                page_number: 2,
                page_size: 20,
            },
            videos: vec![archive_video("BV1ww411c7mD", "series page 2")],
            ..fake_series_page()
        },
        expected_ids: SeriesInputIds {
            mid: Some(1001),
            series_id: 987,
        },
        expected_request: PageRequest {
            page_number: 2,
            page_size: 20,
        },
    });

    let tree = resolver
        .resolve_page(
            SeriesInputIds {
                mid: Some(1001),
                series_id: 987,
            },
            PageRequest {
                page_number: 2,
                page_size: 20,
            },
        )
        .await?;

    assert_eq!(
        tree.groups[0].page.as_ref().map(|page| page.page_number),
        Some(2)
    );
    assert_eq!(
        tree.groups[0].items[0].id.0,
        "item:series:1001:987:BV1ww411c7mD"
    );
    Ok(())
}

#[test]
fn collection_ids_from_url_accepts_medialist_and_space_list_urls() -> Result<(), BdlError> {
    assert_eq!(
        collection_ids_from_url("https://www.bilibili.com/medialist/play/1001?season_id=678")?,
        CollectionInputIds {
            mid: 1001,
            season_id: 678,
        }
    );
    assert_eq!(
        collection_ids_from_url("https://space.bilibili.com/1001/lists/679?type=season")?,
        CollectionInputIds {
            mid: 1001,
            season_id: 679,
        }
    );
    assert_eq!(
        collection_ids_from_url(
            "https://space.bilibili.com/1001/favlist?fid=680&ftype=collect&ctype=21"
        )?,
        CollectionInputIds {
            mid: 1001,
            season_id: 680,
        }
    );
    Ok(())
}

#[test]
fn series_ids_from_url_accepts_query_and_space_list_urls() -> Result<(), BdlError> {
    assert_eq!(
        series_ids_from_url("https://www.bilibili.com/video/series?series_id=987")?,
        SeriesInputIds {
            mid: None,
            series_id: 987,
        }
    );
    assert_eq!(
        series_ids_from_url("https://space.bilibili.com/1001/lists/988?type=series")?,
        SeriesInputIds {
            mid: Some(1001),
            series_id: 988,
        }
    );
    Ok(())
}

fn fake_collection_api() -> FakeCollectionApi {
    FakeCollectionApi {
        page: fake_collection_page(),
        expected_ids: CollectionInputIds {
            mid: 1001,
            season_id: 678,
        },
        expected_request: PageRequest {
            page_number: 1,
            page_size: 20,
        },
    }
}

fn fake_series_api() -> FakeSeriesApi {
    FakeSeriesApi {
        page: fake_series_page(),
        expected_ids: SeriesInputIds {
            mid: Some(1001),
            series_id: 987,
        },
        expected_request: PageRequest {
            page_number: 1,
            page_size: 20,
        },
    }
}

fn fake_collection_page() -> ResolvedCollectionPage {
    ResolvedCollectionPage {
        mid: 1001,
        season_id: 678,
        title: "fixture collection".to_owned(),
        owner_name: None,
        request: PageRequest {
            page_number: 1,
            page_size: 20,
        },
        total_count: Some(38),
        videos: vec![archive_video("BV1xx411c7mD", "collection video")],
    }
}

fn fake_series_page() -> ResolvedSeriesPage {
    ResolvedSeriesPage {
        mid: 1001,
        series_id: 987,
        title: "fixture series".to_owned(),
        owner_name: Some("fixture creator".to_owned()),
        request: PageRequest {
            page_number: 1,
            page_size: 20,
        },
        total_count: Some(12),
        videos: vec![ResolvedArchiveVideo {
            owner_name: Some("fixture creator".to_owned()),
            ..archive_video("BV1yy411c7mD", "series video")
        }],
    }
}

fn archive_video(bvid: &str, title: &str) -> ResolvedArchiveVideo {
    ResolvedArchiveVideo {
        aid: 170001,
        bvid: bvid.to_owned(),
        title: title.to_owned(),
        owner_mid: 1001,
        owner_name: None,
        cover_url: Some("https://example.invalid/cover.jpg".to_owned()),
        duration_seconds: Some(62),
    }
}
