use crate::error::{BdlError, BdlResult};
use crate::model::{NormalizedSourceTree, PageState, SourceKind};
use std::collections::HashSet;

/// Merge a metadata page without losing list identity or silently accepting stalled paging.
pub fn append_source_page(
    existing: &mut NormalizedSourceTree,
    next: NormalizedSourceTree,
) -> BdlResult<()> {
    if existing.source.kind == SourceKind::Collection {
        return merge_collection_page(existing, next, true);
    }
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

/// Collection sections share one source-level paging cursor. Merge every group and
/// preserve hydrated items when a video reappears or later receives section metadata.
pub fn merge_collection_page(
    existing: &mut NormalizedSourceTree,
    next: NormalizedSourceTree,
    sequential: bool,
) -> BdlResult<()> {
    let invalid = |message: &str| BdlError::Planning {
        message: message.into(),
    };
    if existing.source.id != next.source.id || existing.source.kind != next.source.kind {
        return Err(invalid("page source does not match"));
    }
    let mut cursor = next
        .groups
        .iter()
        .find_map(|g| g.page.clone())
        .ok_or_else(|| invalid("next page has no page state"))?;
    if sequential {
        let previous = existing
            .groups
            .iter()
            .find_map(|g| g.page.as_ref())
            .ok_or_else(|| invalid("source has no page state"))?;
        if previous.page_number.checked_add(1) != Some(cursor.page_number)
            || previous.page_size != cursor.page_size
        {
            return Err(invalid("page sequence does not advance correctly"));
        }
    }
    let mut merged = existing.clone();
    let before = merged.source.loaded_count;
    let mut cached_page = next.source.loaded_count > 0;
    let has_sections = next.groups.iter().any(|g| g.kind == "collection_section");
    for mut incoming in next.groups {
        let destination =
            if let Some(index) = merged.groups.iter().position(|g| g.id == incoming.id) {
                index
            } else {
                let mut group = incoming.clone();
                group.items.clear();
                merged.groups.push(group);
                merged.groups.len() - 1
            };
        if incoming.kind == "collection_section" {
            merged.groups[destination].title = incoming.title.clone();
        }
        for item in incoming.items.drain(..) {
            let location = merged.groups.iter().enumerate().find_map(|(g, group)| {
                group
                    .items
                    .iter()
                    .position(|v| v.id == item.id)
                    .map(|i| (g, i))
            });
            cached_page &= item.nfo_metadata.index > 0
                && location.is_some_and(|(g, i)| {
                    merged.groups[g].items[i].nfo_metadata.index == item.nfo_metadata.index
                });
            match location {
                Some((g, i))
                    if g != destination
                        && incoming.kind == "collection_section"
                        && merged.groups[g].kind != "collection_section" =>
                {
                    let hydrated = merged.groups[g].items.remove(i);
                    merged.groups[destination].items.push(hydrated);
                }
                Some(_) => {}
                None => merged.groups[destination].items.push(item),
            }
        }
    }
    if has_sections {
        for group in &mut merged.groups {
            if group.kind != "collection_section" {
                group.title = "其他视频".into();
            }
        }
    }
    let loaded_count = merged.groups.iter().map(|g| g.items.len()).sum();
    if sequential && loaded_count == before && next.source.has_more && !cached_page {
        return Err(invalid(
            "paging made no progress while the server reports more items; retry later",
        ));
    }
    cursor.loaded_count = loaded_count;
    cursor.total_count = next.source.total_count.or(existing.source.total_count);
    cursor.has_more =
        next.source.has_more && cursor.total_count.is_none_or(|total| loaded_count < total);
    for group in &mut merged.groups {
        group.page = Some(cursor.clone());
    }
    merged.source.loaded_count = loaded_count;
    merged.source.total_count = cursor.total_count;
    merged.source.has_more = cursor.has_more;
    *existing = merged;
    Ok(())
}

pub const MAX_PAGE_SIZE: u32 = 100;

pub fn page_item_index(request: PageRequest, offset: usize) -> usize {
    (request.page_number.saturating_sub(1) as usize)
        .saturating_mul(request.page_size as usize)
        .saturating_add(offset)
        .saturating_add(1)
}

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
