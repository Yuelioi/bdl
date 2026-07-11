//! Pure parse-session rules for source paging and initial workspace expansion.
//!
//! Network resolution stays in [`crate::state::AppState`]. This module owns the
//! deterministic rules that turn resolver pages into one stable source tree.

use std::collections::HashSet;

use bdl_core::input::{ClassifiedInput, classify_input};
use bdl_core::model::{NormalizedSourceTree, PageState, SourceKind};
use bdl_core::resolver::collection::{CollectionInputIds, SeriesInputIds};
use bdl_core::resolver::paged::PageRequest;
use bdl_core::{BdlError, BdlResult};

pub(crate) fn should_continue_loading(
    has_more: bool,
    loaded_count: usize,
    limit: Option<usize>,
) -> bool {
    has_more && limit.is_none_or(|limit| loaded_count < limit.max(1))
}

pub(crate) fn should_expand_initial_source(kind: SourceKind, has_more: bool) -> bool {
    has_more && kind != SourceKind::Uploader
}

pub(crate) fn uploader_mid(tree: &NormalizedSourceTree) -> BdlResult<u64> {
    match classify_input(&tree.source.input)? {
        ClassifiedInput::Uploader { mid } => Ok(mid),
        other => Err(BdlError::UnsupportedSource {
            kind: source_kind_name(other.source_kind()).to_owned(),
        }),
    }
}

pub(crate) fn favorite_media_id(tree: &NormalizedSourceTree) -> BdlResult<u64> {
    tree.source
        .id
        .0
        .strip_prefix("favorite:")
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| BdlError::Planning {
            message: format!("来源 `{}` 缺少收藏夹 ID。", tree.source.id.0),
        })
}

pub(crate) fn collection_ids(tree: &NormalizedSourceTree) -> BdlResult<CollectionInputIds> {
    let (mid, season_id) = parse_two_part_source_id(&tree.source.id.0, "collection")?;
    Ok(CollectionInputIds { mid, season_id })
}

pub(crate) fn series_ids(tree: &NormalizedSourceTree) -> BdlResult<SeriesInputIds> {
    let (mid, series_id) = parse_two_part_source_id(&tree.source.id.0, "series")?;
    Ok(SeriesInputIds {
        mid: Some(mid),
        series_id,
    })
}

pub(crate) fn cheese_season_id(tree: &NormalizedSourceTree) -> BdlResult<u64> {
    tree.source
        .id
        .0
        .strip_prefix("cheese:")
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| BdlError::Planning {
            message: format!("来源 `{}` 缺少课程 season ID。", tree.source.id.0),
        })
}

fn parse_two_part_source_id(source_id: &str, prefix: &str) -> BdlResult<(u64, u64)> {
    let mut parts = source_id
        .strip_prefix(prefix)
        .and_then(|value| value.strip_prefix(':'))
        .into_iter()
        .flat_map(|value| value.split(':'));
    let first = parts
        .next()
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| BdlError::Planning {
            message: format!("来源 `{source_id}` 缺少 {prefix} mid。"),
        })?;
    let second = parts
        .next()
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| BdlError::Planning {
            message: format!("来源 `{source_id}` 缺少 {prefix} ID。"),
        })?;

    Ok((first, second))
}

pub(crate) fn next_page_request(tree: &NormalizedSourceTree) -> BdlResult<PageRequest> {
    let page = tree
        .groups
        .iter()
        .find_map(|group| group.page.clone())
        .ok_or_else(|| BdlError::Planning {
            message: format!("来源 `{}` 没有分页状态。", tree.source.id.0),
        })?;

    if !page.has_more {
        return Err(BdlError::Planning {
            message: format!("来源 `{}` 没有更多可解析内容。", tree.source.title),
        });
    }

    Ok(PageRequest {
        page_number: page.page_number,
        page_size: page.page_size,
    }
    .next())
}

pub(crate) fn append_source_page(
    existing: &mut NormalizedSourceTree,
    next_page: NormalizedSourceTree,
) -> BdlResult<()> {
    if existing.source.id != next_page.source.id {
        return Err(BdlError::Planning {
            message: format!(
                "分页来源不匹配：`{}` != `{}`。",
                existing.source.id.0, next_page.source.id.0
            ),
        });
    }

    let mut next_group = next_page
        .groups
        .into_iter()
        .next()
        .ok_or_else(|| BdlError::Planning {
            message: "分页解析结果为空。".to_owned(),
        })?;
    let next_page_state = next_group.page.take().ok_or_else(|| BdlError::Planning {
        message: "分页解析结果缺少分页状态。".to_owned(),
    })?;

    let group = existing
        .groups
        .iter_mut()
        .find(|group| group.id == next_group.id)
        .ok_or_else(|| BdlError::Planning {
            message: format!("来源 `{}` 缺少目标分组。", existing.source.id.0),
        })?;

    let mut seen_ids = group
        .items
        .iter()
        .map(|item| item.id.clone())
        .collect::<HashSet<_>>();
    group.items.extend(
        next_group
            .items
            .into_iter()
            .filter(|item| seen_ids.insert(item.id.clone())),
    );

    let total_count = next_page.source.total_count.or(existing.source.total_count);
    let has_more = next_page.source.has_more
        && total_count
            .map(|total| group.items.len() < total)
            .unwrap_or(next_page_state.loaded_count >= next_page_state.page_size as usize);
    group.page = Some(PageState {
        page_number: next_page_state.page_number,
        page_size: next_page_state.page_size,
        loaded_count: group.items.len(),
        total_count,
        has_more,
    });
    existing.source.loaded_count = existing.groups.iter().map(|group| group.items.len()).sum();
    existing.source.total_count = total_count;
    existing.source.has_more = has_more;

    Ok(())
}

fn source_kind_name(kind: SourceKind) -> &'static str {
    match kind {
        SourceKind::Video => "video",
        SourceKind::Bangumi => "bangumi",
        SourceKind::Cheese => "cheese",
        SourceKind::Favorite => "favorite",
        SourceKind::Collection => "collection",
        SourceKind::Series => "series",
        SourceKind::Uploader => "uploader",
        SourceKind::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_two_part_source_id, should_continue_loading};

    #[test]
    fn load_limit_is_clamped_to_one() {
        assert!(!should_continue_loading(true, 1, Some(0)));
    }

    #[test]
    fn two_part_source_id_rejects_wrong_prefix() {
        let error = parse_two_part_source_id("series:42:7", "collection")
            .expect_err("a mismatched source kind must be rejected");

        assert!(error.to_string().contains("collection mid"));
    }
}
