use crate::error::{BdlError, BdlResult};
use crate::model::{NormalizedSourceTree, PageState};
use std::collections::HashSet;

/// Merge a metadata page without losing list identity or silently accepting stalled paging.
pub fn append_source_page(
    existing: &mut NormalizedSourceTree,
    next: NormalizedSourceTree,
) -> BdlResult<()> {
    let invalid = |message: &str| BdlError::Planning {
        message: message.to_owned(),
    };
    if existing.source.id != next.source.id || existing.source.kind != next.source.kind {
        return Err(invalid("page source does not match"));
    }
    if next.groups.len() != 1 {
        return Err(invalid("expected one group in a paged source"));
    }
    let mut next_group = next
        .groups
        .into_iter()
        .next()
        .expect("group length checked");
    let mut page = next_group
        .page
        .take()
        .ok_or_else(|| invalid("next page has no page state"))?;
    let group = existing
        .groups
        .iter_mut()
        .find(|group| group.id == next_group.id)
        .ok_or_else(|| invalid("page group does not match"))?;
    let previous = group
        .page
        .as_ref()
        .ok_or_else(|| invalid("source has no page state"))?;
    if previous.page_number.checked_add(1) != Some(page.page_number)
        || previous.page_size != page.page_size
    {
        return Err(invalid("page sequence does not advance correctly"));
    }
    let mut seen = group
        .items
        .iter()
        .map(|item| item.id.clone())
        .collect::<HashSet<_>>();
    let additions = next_group
        .items
        .into_iter()
        .filter(|item| seen.insert(item.id.clone()))
        .collect::<Vec<_>>();
    if additions.is_empty() && next.source.has_more {
        return Err(invalid(
            "paging made no progress while the server reports more items; retry later",
        ));
    }
    group.items.extend(additions);
    page.loaded_count = group.items.len();
    page.total_count = next.source.total_count.or(existing.source.total_count);
    page.has_more = next.source.has_more
        && page
            .total_count
            .is_none_or(|total| page.loaded_count < total);
    existing.source.has_more = page.has_more;
    existing.source.total_count = page.total_count;
    group.page = Some(page);
    existing.source.loaded_count = existing.groups.iter().map(|group| group.items.len()).sum();
    Ok(())
}

pub const MAX_PAGE_SIZE: u32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PagedSourceKind {
    Favorite,
    UploaderVideos,
    Collection,
    Series,
    CheeseEpisodes,
}

impl PagedSourceKind {
    pub fn default_page_size(self) -> u32 {
        match self {
            Self::Favorite => 20,
            Self::UploaderVideos => 30,
            Self::Collection | Self::Series => 20,
            Self::CheeseEpisodes => 30,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageRequest {
    pub page_number: u32,
    pub page_size: u32,
}

impl PageRequest {
    pub fn first(kind: PagedSourceKind, api_max_page_size: Option<u32>) -> Self {
        Self {
            page_number: 1,
            page_size: effective_page_size(kind, api_max_page_size),
        }
    }

    pub fn next(self) -> Self {
        Self {
            page_number: self.page_number.saturating_add(1),
            page_size: self.page_size,
        }
    }
}

pub fn effective_page_size(kind: PagedSourceKind, api_max_page_size: Option<u32>) -> u32 {
    api_max_page_size
        .unwrap_or_else(|| kind.default_page_size())
        .clamp(1, MAX_PAGE_SIZE)
}

pub fn page_state(
    request: PageRequest,
    loaded_count: usize,
    total_count: Option<usize>,
) -> PageState {
    let has_more = total_count
        .map(|total| loaded_count < total)
        .unwrap_or_else(|| loaded_count >= request.page_size as usize);

    PageState {
        page_number: request.page_number,
        page_size: request.page_size,
        loaded_count,
        total_count,
        has_more,
    }
}
