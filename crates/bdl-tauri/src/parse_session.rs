//! Pure parse-session rules for source paging and initial workspace expansion.
//!
//! Network resolution stays in [`crate::state::AppState`]. This module owns the
//! deterministic rules that turn resolver pages into one stable source tree.

use std::collections::HashSet;

use bdl_core::ids::PartId;
use bdl_core::input::{ClassifiedInput, classify_input};
use bdl_core::model::{
    NormalizedItem, NormalizedPart, NormalizedSourceTree, PageState, SourceKind,
};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PartHydrationRequest {
    pub(crate) part_id: PartId,
    pub(crate) input: ClassifiedInput,
    pub(crate) target_cid: Option<u64>,
}

pub(crate) fn selected_hydration_requests(
    tree: &NormalizedSourceTree,
    selected_part_ids: &[PartId],
) -> BdlResult<Vec<PartHydrationRequest>> {
    let mut seen = HashSet::new();
    let mut requests = Vec::new();

    for part_id in selected_part_ids {
        if !seen.insert(part_id.clone()) {
            continue;
        }
        let Some(part) = find_part(tree, part_id) else {
            continue;
        };
        if part.cid.is_some() && !part.streams.is_empty() {
            continue;
        }
        requests.push(PartHydrationRequest {
            part_id: part_id.clone(),
            input: hydration_input_for_part(tree.source.kind, part, part_id)?,
            target_cid: part.cid,
        });
    }

    Ok(requests)
}

pub(crate) fn normalize_selected_part_ids(
    tree: &NormalizedSourceTree,
    selected_part_ids: &[PartId],
) -> Vec<PartId> {
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();

    for part_id in selected_part_ids {
        if find_part(tree, part_id).is_some() {
            if seen.insert(part_id.clone()) {
                normalized.push(part_id.clone());
            }
            continue;
        }

        let hydrated_parts = list_placeholder_bvid(part_id)
            .and_then(|bvid| {
                tree.groups
                    .iter()
                    .flat_map(|group| &group.items)
                    .find(|item| {
                        item.parts
                            .iter()
                            .any(|part| part.bvid.as_deref() == Some(bvid))
                    })
            })
            .map(|item| item.parts.iter().map(|part| part.id.clone()));

        if let Some(hydrated_parts) = hydrated_parts {
            normalized.extend(hydrated_parts.filter(|part_id| seen.insert(part_id.clone())));
        } else if seen.insert(part_id.clone()) {
            normalized.push(part_id.clone());
        }
    }

    normalized
}

pub(crate) fn hydrate_placeholder_part(
    tree: &mut NormalizedSourceTree,
    placeholder_id: &PartId,
    target_cid: Option<u64>,
    hydrated: NormalizedSourceTree,
) -> BdlResult<Vec<PartId>> {
    let mut hydrated_item = hydrated_item_for_target(hydrated, target_cid)?;
    let hydrated_part_ids = selected_hydrated_part_ids(&hydrated_item.parts, target_cid)?;
    if hydrated_part_ids.is_empty() {
        return Err(BdlError::Planning {
            message: format!("选中的分 P `{}` 没有可下载分 P。", placeholder_id.0),
        });
    }

    for group in &mut tree.groups {
        let Some(item_index) = group
            .items
            .iter()
            .position(|item| item.parts.iter().any(|part| &part.id == placeholder_id))
        else {
            continue;
        };

        let existing_item = &mut group.items[item_index];
        if let Some(target_cid) = target_cid
            && let Some(existing_part_index) = existing_item
                .parts
                .iter()
                .position(|part| part.cid == Some(target_cid))
        {
            let hydrated_part_index = hydrated_item
                .parts
                .iter()
                .position(|part| part.cid == Some(target_cid))
                .ok_or_else(|| BdlError::Planning {
                    message: format!("视频解析结果缺少 CID `{target_cid}`，无法创建下载任务。"),
                })?;
            let mut hydrated_part = hydrated_item.parts.swap_remove(hydrated_part_index);
            let existing_part_id = existing_item.parts[existing_part_index].id.clone();
            hydrated_part.id = existing_part_id.clone();
            merge_missing_item_metadata(existing_item, &hydrated_item);
            existing_item.parts[existing_part_index] = hydrated_part;
            return Ok(vec![existing_part_id]);
        }

        hydrated_item.id = existing_item.id.clone();
        merge_missing_item_metadata(&mut hydrated_item, existing_item);
        *existing_item = hydrated_item;
        return Ok(hydrated_part_ids);
    }

    Err(BdlError::Planning {
        message: format!(
            "选中的分 P `{}` 未加载，请重新解析后再试。",
            placeholder_id.0
        ),
    })
}

pub(crate) fn remap_selected_part_ids(
    selected_part_ids: &mut Vec<PartId>,
    placeholder_id: &PartId,
    hydrated_part_ids: &[PartId],
) {
    let mut remapped = Vec::with_capacity(selected_part_ids.len() + hydrated_part_ids.len());
    for part_id in selected_part_ids.drain(..) {
        if &part_id == placeholder_id {
            remapped.extend(hydrated_part_ids.iter().cloned());
        } else {
            remapped.push(part_id);
        }
    }
    *selected_part_ids = remapped;
}

fn list_placeholder_bvid(part_id: &PartId) -> Option<&str> {
    let mut segments = part_id.0.strip_prefix("part:")?.split(':');
    let kind = segments.next()?;
    if !matches!(kind, "favorite" | "uploader" | "collection" | "series") {
        return None;
    }
    segments.next_back().filter(|value| value.starts_with("BV"))
}

pub(crate) fn find_part<'a>(
    tree: &'a NormalizedSourceTree,
    part_id: &PartId,
) -> Option<&'a NormalizedPart> {
    tree.groups
        .iter()
        .flat_map(|group| &group.items)
        .flat_map(|item| &item.parts)
        .find(|part| &part.id == part_id)
}

fn hydration_input_for_part(
    kind: SourceKind,
    part: &NormalizedPart,
    part_id: &PartId,
) -> BdlResult<ClassifiedInput> {
    match kind {
        SourceKind::Bangumi => {
            let ep_id = episode_id_from_part_id(part_id, "bangumi")?;
            Ok(ClassifiedInput::Bangumi {
                raw_url: format!("https://www.bilibili.com/bangumi/play/ep{ep_id}"),
            })
        }
        SourceKind::Cheese => {
            let ep_id = episode_id_from_part_id(part_id, "cheese")?;
            Ok(ClassifiedInput::Cheese {
                raw_url: format!("https://www.bilibili.com/cheese/play/ep{ep_id}"),
            })
        }
        _ => part
            .bvid
            .clone()
            .map(ClassifiedInput::VideoBvid)
            .ok_or_else(|| BdlError::Planning {
                message: format!("选中的分 P `{}` 缺少 BV ID，无法补齐下载流。", part_id.0),
            }),
    }
}

fn episode_id_from_part_id(part_id: &PartId, expected_kind: &'static str) -> BdlResult<u64> {
    let mut parts = part_id
        .0
        .strip_prefix("part:")
        .into_iter()
        .flat_map(|value| value.split(':'));
    let kind = parts.next().ok_or_else(|| BdlError::Planning {
        message: format!("选中的分 P `{}` 缺少来源类型。", part_id.0),
    })?;
    if kind != expected_kind {
        return Err(BdlError::Planning {
            message: format!("选中的分 P `{}` 不是 {expected_kind} 来源。", part_id.0),
        });
    }
    parse_positive_u64(parts.next()).ok_or_else(|| BdlError::Planning {
        message: format!("选中的分 P `{}` 缺少 season ID。", part_id.0),
    })?;
    parse_positive_u64(parts.next()).ok_or_else(|| BdlError::Planning {
        message: format!("选中的分 P `{}` 缺少 ep ID。", part_id.0),
    })
}

fn parse_positive_u64(value: Option<&str>) -> Option<u64> {
    value?.parse::<u64>().ok().filter(|value| *value > 0)
}

fn selected_hydrated_part_ids(
    hydrated_parts: &[NormalizedPart],
    target_cid: Option<u64>,
) -> BdlResult<Vec<PartId>> {
    if let Some(target_cid) = target_cid {
        return hydrated_parts
            .iter()
            .find(|part| part.cid == Some(target_cid))
            .map(|part| vec![part.id.clone()])
            .ok_or_else(|| BdlError::Planning {
                message: format!("视频解析结果缺少 CID `{target_cid}`，无法创建下载任务。"),
            });
    }
    Ok(hydrated_parts.iter().map(|part| part.id.clone()).collect())
}

fn hydrated_item_for_target(
    hydrated: NormalizedSourceTree,
    target_cid: Option<u64>,
) -> BdlResult<NormalizedItem> {
    let mut items = hydrated
        .groups
        .into_iter()
        .flat_map(|group| group.items)
        .collect::<Vec<_>>();
    if let Some(target_cid) = target_cid {
        let item_index = items
            .iter()
            .position(|item| item.parts.iter().any(|part| part.cid == Some(target_cid)))
            .ok_or_else(|| BdlError::Planning {
                message: format!("解析结果缺少 CID `{target_cid}`，无法创建下载任务。"),
            })?;
        return Ok(items.swap_remove(item_index));
    }
    items.into_iter().next().ok_or_else(|| BdlError::Planning {
        message: "解析结果为空，无法创建下载任务。".to_owned(),
    })
}

fn merge_missing_item_metadata(target: &mut NormalizedItem, fallback: &NormalizedItem) {
    if target.owner_name.is_none() {
        target.owner_name.clone_from(&fallback.owner_name);
    }
    if target.cover_url.is_none() {
        target.cover_url.clone_from(&fallback.cover_url);
    }
    if target.duration_seconds.is_none() {
        target.duration_seconds = fallback.duration_seconds;
    }
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
