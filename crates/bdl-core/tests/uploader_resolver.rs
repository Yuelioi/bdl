use async_trait::async_trait;
use bdl_core::error::BdlError;
use bdl_core::input::ClassifiedInput;
use bdl_core::model::{AssetKind, FetchPolicy, SourceKind};
use bdl_core::resolver::paged::PageRequest;
use bdl_core::resolver::uploader::{
    ResolvedUploaderPage, ResolvedUploaderVideo, UploaderApi, UploaderResolver,
};
use bdl_core::resolver::{ResolveOptions, Resolver};

#[derive(Debug, Clone)]
struct FakeUploaderApi {
    page: ResolvedUploaderPage,
    expected_request: PageRequest,
}

#[async_trait]
impl UploaderApi for FakeUploaderApi {
    async fn uploaded_videos(
        &self,
        mid: u64,
        request: PageRequest,
    ) -> Result<ResolvedUploaderPage, BdlError> {
        assert_eq!(mid, 1001);
        assert_eq!(request, self.expected_request);
        Ok(self.page.clone())
    }
}

#[tokio::test]
async fn uploader_resolver_maps_first_uploaded_video_page() -> Result<(), BdlError> {
    let resolver = UploaderResolver::with_api(fake_api());

    let tree = resolver
        .resolve(
            ClassifiedInput::Uploader { mid: 1001 },
            ResolveOptions {
                fetch_streams: true,
            },
        )
        .await?;

    assert_eq!(tree.source.kind, SourceKind::Uploader);
    assert_eq!(tree.source.id.0, "uploader:1001:videos");
    assert_eq!(tree.source.input, "https://space.bilibili.com/1001/video");
    assert_eq!(tree.source.title, "fixture owner 的投稿");
    assert_eq!(tree.source.loaded_count, 1);
    assert_eq!(tree.source.total_count, Some(45));
    assert!(tree.source.has_more);

    let group = tree.groups.first().expect("group should exist");
    assert_eq!(group.id.0, "group:uploader:1001:videos");
    assert_eq!(group.kind, "uploader_videos");
    assert_eq!(group.title, "fixture owner 的投稿");

    let page = group.page.as_ref().expect("page state should exist");
    assert_eq!(page.page_number, 1);
    assert_eq!(page.page_size, 30);
    assert_eq!(page.loaded_count, 1);
    assert_eq!(page.total_count, Some(45));
    assert!(page.has_more);

    let item = group.items.first().expect("item should exist");
    assert_eq!(item.id.0, "item:uploader:1001:BV1xx411c7mD");
    assert_eq!(item.title, "fixture upload");
    assert_eq!(item.owner_name.as_deref(), Some("fixture owner"));
    assert_eq!(item.publish_date.as_deref(), Some("2025-12-31"));
    assert_eq!(
        item.cover_url.as_deref(),
        Some("https://example.invalid/cover.jpg")
    );
    assert_eq!(item.duration_seconds, Some(62));

    let part = item.parts.first().expect("part should exist");
    assert_eq!(part.id.0, "part:uploader:1001:BV1xx411c7mD");
    assert_eq!(part.title, "fixture upload");
    assert_eq!(part.aid, Some(170001));
    assert_eq!(part.bvid.as_deref(), Some("BV1xx411c7mD"));
    assert_eq!(part.cid, None);
    assert!(part.streams.is_empty());
    assert_eq!(part.assets[0].kind, AssetKind::Cover);
    assert_eq!(part.assets[0].fetch_policy, FetchPolicy::OnDemand);

    Ok(())
}

#[tokio::test]
async fn uploader_resolver_resolves_requested_uploaded_video_page() -> Result<(), BdlError> {
    let resolver = UploaderResolver::with_api(FakeUploaderApi {
        page: ResolvedUploaderPage {
            request: PageRequest {
                page_number: 2,
                page_size: 30,
            },
            videos: vec![ResolvedUploaderVideo {
                aid: 170002,
                bvid: "BV1yy411c7mD".to_owned(),
                title: "fixture upload page 2".to_owned(),
                owner_mid: 1001,
                owner_name: Some("fixture owner".to_owned()),
                publish_date: Some("2026-01-01".to_owned()),
                cover_url: None,
                duration_seconds: None,
            }],
            ..fake_page()
        },
        expected_request: PageRequest {
            page_number: 2,
            page_size: 30,
        },
    });

    let tree = resolver
        .resolve_page(
            1001,
            PageRequest {
                page_number: 2,
                page_size: 30,
            },
        )
        .await?;

    assert_eq!(tree.source.loaded_count, 1);
    assert_eq!(
        tree.groups[0].page.as_ref().map(|page| page.page_number),
        Some(2)
    );
    assert_eq!(
        tree.groups[0].items[0].id.0,
        "item:uploader:1001:BV1yy411c7mD"
    );
    Ok(())
}

#[tokio::test]
async fn uploader_resolver_rejects_non_uploader_input() {
    let resolver = UploaderResolver::with_api(fake_api());

    let error = resolver
        .resolve(
            ClassifiedInput::VideoBvid("BV1xx411c7mD".to_owned()),
            ResolveOptions::default(),
        )
        .await
        .expect_err("video input should not resolve through uploader resolver");

    assert!(matches!(
        error,
        BdlError::UnsupportedSource { kind } if kind == "video"
    ));
}

fn fake_api() -> FakeUploaderApi {
    FakeUploaderApi {
        page: fake_page(),
        expected_request: PageRequest {
            page_number: 1,
            page_size: 30,
        },
    }
}

fn fake_page() -> ResolvedUploaderPage {
    ResolvedUploaderPage {
        mid: 1001,
        owner_name: Some("fixture owner".to_owned()),
        request: PageRequest {
            page_number: 1,
            page_size: 30,
        },
        total_count: Some(45),
        videos: vec![ResolvedUploaderVideo {
            aid: 170001,
            bvid: "BV1xx411c7mD".to_owned(),
            title: "fixture upload".to_owned(),
            owner_mid: 1001,
            owner_name: Some("fixture owner".to_owned()),
            publish_date: Some("2025-12-31".to_owned()),
            cover_url: Some("https://example.invalid/cover.jpg".to_owned()),
            duration_seconds: Some(62),
        }],
    }
}
