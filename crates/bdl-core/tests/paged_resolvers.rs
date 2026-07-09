use bdl_core::resolver::paged::{PageRequest, PagedSourceKind, effective_page_size, page_state};

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
