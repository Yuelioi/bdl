use std::sync::Arc;

use bdl_core::ids::PartId;
use bdl_core::model::NormalizedSourceTree;
use bdl_core::planner::DownloadOptions;
use bdl_core::{BdlError, BdlResult};

use crate::download_resolution::{DownloadResolution, ResolvedDownload, SelectionPreparation};
use crate::parse_pacing::CooldownBehavior;
use crate::parse_session::{
    find_part, hydrate_placeholder_part, merge_browsed_page, next_page_request,
    normalize_selected_part_ids, remap_selected_part_ids, should_continue_loading,
};
use crate::state::{AppState, PreparedSelection};

pub(crate) struct PreparationContext {
    pub operation_id: String,
    pub account_revision: u64,
    pub cooldown: CooldownBehavior,
    pub client: Arc<bpi_rs::BpiClient>,
}

impl AppState {
    pub(crate) fn preparation_context(
        &self,
        operation_id: String,
    ) -> BdlResult<PreparationContext> {
        Ok(PreparationContext {
            operation_id,
            account_revision: self.account_revision(),
            cooldown: CooldownBehavior::Report,
            client: self.shared_resolver_client()?,
        })
    }

    pub async fn load_more(
        &self,
        mut tree: NormalizedSourceTree,
        operation_id: &str,
    ) -> BdlResult<NormalizedSourceTree> {
        self.append_next_page(&mut tree, operation_id).await?;
        Ok(tree)
    }

    pub async fn load_page(
        &self,
        mut tree: NormalizedSourceTree,
        page_number: u32,
        operation_id: &str,
    ) -> BdlResult<(NormalizedSourceTree, Vec<String>, bool)> {
        if page_number == 0 {
            return Err(BdlError::Planning {
                message: "页码必须大于 0。".into(),
            });
        }
        let request = bdl_core::resolver::paged::PageRequest {
            page_number,
            page_size: 20,
        };
        let operation = self.parse_control.begin(operation_id, "load_page");
        operation.target(format!(
            "source_id={} page_number={page_number} page_size=20",
            tree.source.id.0
        ));
        let page = operation
            .run(self.parse_pacer.run_retry(
                Some(self.settings()?.parse_rules),
                CooldownBehavior::Report,
                || self.resolve_source_page(&tree, request),
                |_| true,
                |phase| operation.phase(phase),
            ))
            .await?;
        let ids = page
            .groups
            .iter()
            .flat_map(|group| &group.items)
            .map(|item| item.id.0.clone())
            .collect();
        let has_more = page.source.has_more;
        merge_browsed_page(&mut tree, page)?;
        Ok((tree, ids, has_more))
    }

    pub async fn load_all(
        &self,
        mut tree: NormalizedSourceTree,
        limit: Option<usize>,
        operation_id: &str,
    ) -> BdlResult<NormalizedSourceTree> {
        while should_continue_loading(tree.source.has_more, tree.source.loaded_count, limit) {
            let before = next_page_request(&tree)?;
            self.append_next_page(&mut tree, operation_id).await?;
            if tree.source.has_more && next_page_request(&tree)? == before {
                break;
            }
        }
        Ok(tree)
    }

    pub(crate) async fn prepare_selection_owned(
        &self,
        mut tree: NormalizedSourceTree,
        selected: &[PartId],
        options: &DownloadOptions,
        context: &PreparationContext,
        mode: SelectionPreparation,
        progress: impl Fn(usize, usize) -> BdlResult<()>,
    ) -> BdlResult<PreparedSelection> {
        let mut part_ids = normalize_selected_part_ids(&tree, selected);
        let resolution = DownloadResolution::for_selection(mode, options);
        let requests = resolution.requests(&tree, &part_ids)?;
        let mut prepared_for_enqueue = std::collections::HashSet::new();
        let updated = !requests.is_empty();
        let total = requests.len();
        progress(0, total)?;
        let operation = self
            .parse_control
            .begin(&context.operation_id, "prepare_selection");
        for (index, request) in requests.into_iter().enumerate() {
            self.check_account_revision(context.account_revision)?;
            operation.target(format!("source_id={} part_id={} target_cid={:?} item={}/{} selected_parts={} resolution={resolution:?}", tree.source.id.0, request.part_id.0, request.target_cid, index + 1, total, part_ids.len()));
            let resolved = operation
                .run(self.parse_pacer.run_retry(
                    None,
                    context.cooldown,
                    || {
                        resolution.resolve_part(
                            context.client.clone(),
                            &request,
                            find_part(&tree, &request.part_id),
                        )
                    },
                    |_| false,
                    |phase| operation.phase(phase),
                ))
                .await?;
            self.check_account_revision(context.account_revision)?;
            let hydrated_ids = match resolved {
                ResolvedDownload::Part(updated) => {
                    let part = tree
                        .groups
                        .iter_mut()
                        .flat_map(|group| &mut group.items)
                        .flat_map(|item| &mut item.parts)
                        .find(|part| part.id == updated.id)
                        .ok_or_else(|| BdlError::Planning {
                            message: "所选条目已移除。".into(),
                        })?;
                    *part = updated;
                    vec![part.id.clone()]
                }
                ResolvedDownload::Source(hydrated) => {
                    let current =
                        normalize_selected_part_ids(&tree, std::slice::from_ref(&request.part_id));
                    let current_id = current.first().ok_or_else(|| BdlError::Planning {
                        message: "所选条目已移除。".into(),
                    })?;
                    hydrate_placeholder_part(&mut tree, current_id, request.target_cid, hydrated)?
                }
            };
            if mode == SelectionPreparation::Enqueue {
                prepared_for_enqueue.extend(hydrated_ids.iter().cloned());
            }
            remap_selected_part_ids(&mut part_ids, &request.part_id, &hydrated_ids);
            progress(index + 1, total)?;
        }
        self.check_account_revision(context.account_revision)?;
        Ok(PreparedSelection {
            tree,
            part_ids,
            tree_updated: updated,
            prepared_for_enqueue,
        })
    }
}
