use crate::model::PageState;

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
