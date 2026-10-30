use std::sync::Arc;

use bdl_core::ids::PartId;
use bdl_core::model::{NormalizedSourceTree, SourceKind};
use bdl_core::planner::DownloadOptions;
use bdl_core::resolver::{ResolveOptions, Resolver};
use bdl_core::resolver::{bangumi::BangumiResolver, cheese::CheeseResolver, video::VideoResolver};
use bdl_core::settings::ParseRules;
use bdl_core::{BdlError, BdlResult};

use crate::parse_session::{
    hydrate_placeholder_part, merge_browsed_page, next_page_request, normalize_selected_part_ids,
    remap_selected_part_ids, selected_hydration_requests, should_continue_loading,
};
use crate::state::{AppState, PreparedSelection};

pub(crate) struct PreparationContext {
    pub operation_id: String,
    pub account_revision: u64,
    pub rules: ParseRules,
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
            rules: self.settings()?.parse_rules,
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
    ) -> BdlResult<(NormalizedSourceTree, Vec<String>)> {
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
                self.settings()?.parse_rules,
                || self.resolve_source_page(&tree, request),
                |_| 1,
                |phase| operation.phase(phase),
            ))
            .await?;
        let ids = page
            .groups
            .iter()
            .flat_map(|group| &group.items)
            .map(|item| item.id.0.clone())
            .collect();
        merge_browsed_page(&mut tree, page)?;
        Ok((tree, ids))
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
        force: bool,
        progress: impl Fn(usize, usize) -> BdlResult<()>,
    ) -> BdlResult<PreparedSelection> {
        let mut part_ids = normalize_selected_part_ids(&tree, selected);
        let mut requests = if force {
            crate::parse_session::selection_hydration_requests(&tree, &part_ids, true)?
        } else {
            selected_hydration_requests(&tree, &part_ids)?
        };
        let media = options.media_mode.includes_video() || options.media_mode.includes_audio();
        let subtitles = options.selected_archive_assets().subtitles
            || options
                .processing
                .is_some_and(|processing| processing.embed_subtitles);
        if !media && !subtitles {
            requests.retain(|request| request.target_cid.is_none());
        }
        let updated = !requests.is_empty();
        let total = requests.len();
        progress(0, total)?;
        let operation = self
            .parse_control
            .begin(&context.operation_id, "prepare_selection");
        for (index, request) in requests.into_iter().enumerate() {
            self.check_account_revision(context.account_revision)?;
            operation.target(format!("source_id={} part_id={} target_cid={:?} item={}/{} selected_parts={} force_refresh={force} media={media} subtitles={subtitles}", tree.source.id.0, request.part_id.0, request.target_cid, index + 1, total, part_ids.len()));
            let hydrated = operation
                .run(self.parse_pacer.run_retry(
                    context.rules,
                    || async {
                        let input = request.input.clone();
                        let resolve = ResolveOptions {
                            fetch_streams: media || subtitles,
                        };
                        match input.source_kind() {
                            SourceKind::Video => {
                                let resolver =
                                    VideoResolver::from_bpi_client(context.client.clone());
                                if media {
                                    match request.target_cid {
                                        Some(cid) => {
                                            resolver.resolve_target_streams(input, cid).await
                                        }
                                        None => resolver.resolve(input, resolve).await,
                                    }
                                } else {
                                    resolver
                                        .resolve_assets(input, request.target_cid, subtitles)
                                        .await
                                }
                            }
                            SourceKind::Bangumi => {
                                BangumiResolver::from_bpi_client(context.client.clone())
                                    .resolve(input, resolve)
                                    .await
                            }
                            SourceKind::Cheese => {
                                CheeseResolver::from_bpi_client(context.client.clone())
                                    .resolve(input, resolve)
                                    .await
                            }
                            _ => Err(BdlError::Planning {
                                message: "所选条目不是可下载的视频来源。".into(),
                            }),
                        }
                    },
                    |_| 0,
                    |phase| operation.phase(phase),
                ))
                .await?;
            self.check_account_revision(context.account_revision)?;
            let current =
                normalize_selected_part_ids(&tree, std::slice::from_ref(&request.part_id));
            let current_id = current.first().ok_or_else(|| BdlError::Planning {
                message: "所选条目已移除。".into(),
            })?;
            let hydrated_ids =
                hydrate_placeholder_part(&mut tree, current_id, request.target_cid, hydrated)?;
            remap_selected_part_ids(&mut part_ids, &request.part_id, &hydrated_ids);
            progress(index + 1, total)?;
        }
        self.check_account_revision(context.account_revision)?;
        Ok(PreparedSelection {
            tree,
            part_ids,
            tree_updated: updated,
        })
    }
}
