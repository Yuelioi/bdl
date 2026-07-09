use async_trait::async_trait;
use bdl_core::error::BdlError;
use bdl_core::input::ClassifiedInput;
use bdl_core::model::{AssetKind, FetchPolicy, SourceKind};
use bdl_core::resolver::favorite::{
    FavoriteApi, FavoriteResolver, ResolvedFavoritePage, ResolvedFavoriteVideo,
    favorite_media_id_from_url,
};
use bdl_core::resolver::paged::PageRequest;
use bdl_core::resolver::{ResolveOptions, Resolver};

#[derive(Debug, Clone)]
struct FakeFavoriteApi {
    page: ResolvedFavoritePage,
    expected_request: PageRequest,
}

#[async_trait]
impl FavoriteApi for FakeFavoriteApi {
    async fn list_detail(
        &self,
        media_id: u64,
        request: PageRequest,
    ) -> Result<ResolvedFavoritePage, BdlError> {
        assert_eq!(media_id, 1052622027);
        assert_eq!(request, self.expected_request);
        Ok(self.page.clone())
    }
}

#[tokio::test]
async fn favorite_resolver_maps_first_page() -> Result<(), BdlError> {
    let resolver = FavoriteResolver::with_api(fake_api());
    let raw_url = "https://space.bilibili.com/12345/favlist?fid=1052622027";

    let tree = resolver
        .resolve(
            ClassifiedInput::Favorite {
                raw_url: raw_url.to_owned(),
            },
            ResolveOptions::default(),
        )
        .await?;

    assert_eq!(tree.source.kind, SourceKind::Favorite);
    assert_eq!(tree.source.id.0, "favorite:1052622027");
    assert_eq!(tree.source.input, raw_url);
    assert_eq!(tree.source.title, "fixture favorite");
    assert_eq!(tree.source.loaded_count, 1);
    assert_eq!(tree.source.total_count, Some(28));
    assert!(tree.source.has_more);

    let group = tree.groups.first().expect("group should exist");
    assert_eq!(group.id.0, "group:favorite:1052622027");
    assert_eq!(group.kind, "favorite");
    assert_eq!(group.page.as_ref().map(|page| page.page_size), Some(20));

    let item = group.items.first().expect("item should exist");
    assert_eq!(item.id.0, "item:favorite:1052622027:BV1xx411c7mD");
    assert_eq!(item.title, "favorite video");
    assert_eq!(item.owner_name.as_deref(), Some("fixture owner"));
    assert_eq!(item.duration_seconds, Some(62));

    let part = item.parts.first().expect("part should exist");
    assert_eq!(part.id.0, "part:favorite:1052622027:BV1xx411c7mD");
    assert_eq!(part.aid, Some(371494037));
    assert_eq!(part.bvid.as_deref(), Some("BV1xx411c7mD"));
    assert_eq!(part.cid, None);
    assert!(part.streams.is_empty());
    assert_eq!(part.assets[0].kind, AssetKind::Cover);
    assert_eq!(part.assets[0].fetch_policy, FetchPolicy::OnDemand);

    Ok(())
}

#[tokio::test]
async fn favorite_resolver_resolves_requested_page() -> Result<(), BdlError> {
    let resolver = FavoriteResolver::with_api(FakeFavoriteApi {
        page: ResolvedFavoritePage {
            request: PageRequest {
                page_number: 2,
                page_size: 20,
            },
            has_more: false,
            ..fake_page()
        },
        expected_request: PageRequest {
            page_number: 2,
            page_size: 20,
        },
    });

    let tree = resolver
        .resolve_page(
            1052622027,
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
    assert!(!tree.source.has_more);
    Ok(())
}

#[test]
fn favorite_media_id_from_url_accepts_fid_or_media_id() -> Result<(), BdlError> {
    assert_eq!(
        favorite_media_id_from_url("https://space.bilibili.com/1/favlist?fid=1052622027")?,
        1052622027
    );
    assert_eq!(
        favorite_media_id_from_url("https://space.bilibili.com/1/favlist?media_id=1052622028")?,
        1052622028
    );
    Ok(())
}

fn fake_api() -> FakeFavoriteApi {
    FakeFavoriteApi {
        page: fake_page(),
        expected_request: PageRequest {
            page_number: 1,
            page_size: 20,
        },
    }
}

fn fake_page() -> ResolvedFavoritePage {
    ResolvedFavoritePage {
        media_id: 1052622027,
        title: "fixture favorite".to_owned(),
        owner_name: Some("fixture folder owner".to_owned()),
        request: PageRequest {
            page_number: 1,
            page_size: 20,
        },
        total_count: Some(28),
        has_more: true,
        videos: vec![ResolvedFavoriteVideo {
            aid: 371494037,
            bvid: "BV1xx411c7mD".to_owned(),
            title: "favorite video".to_owned(),
            owner_mid: 1001,
            owner_name: Some("fixture owner".to_owned()),
            cover_url: Some("https://example.invalid/media-cover.jpg".to_owned()),
            duration_seconds: Some(62),
        }],
    }
}
