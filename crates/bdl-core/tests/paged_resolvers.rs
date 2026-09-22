use bdl_core::model::NormalizedSourceTree;
use bdl_core::resolver::paged::append_source_page;
use bdl_core::resolver::paged::{PageRequest, PagedSourceKind, effective_page_size, page_state};

fn page(number: u32, ids: &[u32], has_more: bool, total: Option<usize>) -> NormalizedSourceTree {
    serde_json::from_value(serde_json::json!({
        "source": {"id": "favorite:1", "kind": "favorite", "input": "original", "title": "List", "loaded_count": ids.len(), "total_count": total, "has_more": has_more},
        "groups": [{"id": "group:favorite:1", "kind": "favorite", "title": "List",
            "page": {"page_number": number, "page_size": 2, "loaded_count": ids.len(), "total_count": total, "has_more": has_more},
            "items": ids.iter().map(|id| serde_json::json!({"id": format!("item:{id}"), "title": format!("Video {id}"), "owner_name": null, "cover_url": null, "duration_seconds": null, "parts": []})).collect::<Vec<_>>()
        }]
    })).unwrap()
}

#[test]
fn pages_deduplicate_and_finish_at_cumulative_total() {
    let mut tree = page(1, &[1, 2], true, Some(4));
    append_source_page(&mut tree, page(2, &[2, 3], true, Some(4))).unwrap();
    assert_eq!(tree.source.loaded_count, 3);
    assert!(tree.source.has_more);
    append_source_page(&mut tree, page(3, &[4], true, Some(4))).unwrap();
    assert!(!tree.source.has_more);
    assert_eq!(tree.groups[0].page.as_ref().unwrap().loaded_count, 4);
    assert_eq!(tree.source.input, "original");
    assert_eq!(
        tree.groups[0]
            .items
            .iter()
            .map(|item| item.id.0.as_str())
            .collect::<Vec<_>>(),
        vec!["item:1", "item:2", "item:3", "item:4"]
    );
}

#[test]
fn unknown_totals_end_on_empty_terminal_page() {
    let mut tree = page(1, &[1, 2], true, None);
    append_source_page(&mut tree, page(2, &[], false, None)).unwrap();
    assert_eq!(tree.source.loaded_count, 2);
    assert!(!tree.source.has_more);
}

#[test]
fn stalled_or_foreign_pages_fail_without_mutating_tree() {
    let original = page(1, &[1, 2], true, None);
    let mut foreign = page(2, &[3], false, None);
    foreign.source.id.0 = "favorite:2".into();
    for next in [
        page(2, &[1, 2], true, None),
        page(2, &[], true, None),
        page(1, &[3], true, None),
        foreign,
    ] {
        let mut tree = original.clone();
        assert!(append_source_page(&mut tree, next).is_err());
        assert_eq!(tree, original);
    }
}

#[test]
fn effective_page_size_uses_api_max_when_smaller_than_global_limit() {
    assert_eq!(effective_page_size(PagedSourceKind::Favorite, Some(50)), 50);
}

#[test]
fn effective_page_size_allows_api_max_equal_to_global_limit() {
    assert_eq!(
        effective_page_size(PagedSourceKind::UploaderVideos, Some(100)),
        100
    );
}

#[test]
fn effective_page_size_caps_api_max_above_global_limit() {
    assert_eq!(
        effective_page_size(PagedSourceKind::Collection, Some(200)),
        100
    );
}

#[test]
fn effective_page_size_uses_endpoint_default_when_api_max_is_unknown() {
    assert_eq!(effective_page_size(PagedSourceKind::Favorite, None), 20);
    assert_eq!(
        effective_page_size(PagedSourceKind::UploaderVideos, None),
        30
    );
    assert_eq!(effective_page_size(PagedSourceKind::Collection, None), 20);
    assert_eq!(effective_page_size(PagedSourceKind::Series, None), 20);
    assert_eq!(
        effective_page_size(PagedSourceKind::CheeseEpisodes, None),
        30
    );
}

#[test]
fn page_request_next_keeps_page_size_and_advances_page_number() {
    let first = PageRequest::first(PagedSourceKind::UploaderVideos, Some(50));
    let next = first.next();

    assert_eq!(
        next,
        PageRequest {
            page_number: 2,
            page_size: 50,
        }
    );
}

#[test]
fn page_state_marks_has_more_when_loaded_count_is_below_total() {
    let request = PageRequest::first(PagedSourceKind::Favorite, Some(20));
    let state = page_state(request, 20, Some(45));

    assert!(state.has_more);
}

#[test]
fn page_state_marks_complete_when_loaded_count_reaches_total() {
    let request = PageRequest::first(PagedSourceKind::Favorite, Some(20));
    let state = page_state(request, 45, Some(45));

    assert!(!state.has_more);
}
