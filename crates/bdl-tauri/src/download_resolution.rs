use std::sync::Arc;

use bdl_core::ids::PartId;
use bdl_core::input::ClassifiedInput;
use bdl_core::model::{NormalizedPart, NormalizedSourceTree, SourceKind};
use bdl_core::planner::DownloadOptions;
use bdl_core::resolver::{ResolveOptions, Resolver};
use bdl_core::resolver::{bangumi::BangumiResolver, cheese::CheeseResolver, video::VideoResolver};
use bdl_core::{BdlError, BdlResult};

use crate::parse_session::{PartHydrationRequest, selection_hydration_requests};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SelectionPreparation {
    Enqueue,
    MissingStreams,
    Refresh,
    PostDetails,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct DownloadResolution {
    stage: SelectionPreparation,
    media: bool,
    subtitles: bool,
}

pub(crate) enum ResolvedDownload {
    Part(NormalizedPart),
    Source(NormalizedSourceTree),
}

impl DownloadResolution {
    pub(crate) fn for_selection(stage: SelectionPreparation, options: &DownloadOptions) -> Self {
        if stage == SelectionPreparation::PostDetails {
            return Self {
                stage,
                media: false,
                subtitles: false,
            };
        }
        Self {
            stage,
            media: options.media_mode.includes_video() || options.media_mode.includes_audio(),
            subtitles: options.selected_archive_assets().subtitles
                || options
                    .processing
                    .is_some_and(|processing| processing.embed_subtitles),
        }
    }

    pub(crate) fn for_url_refresh() -> Self {
        Self {
            stage: SelectionPreparation::Refresh,
            media: true,
            subtitles: false,
        }
    }

    pub(crate) fn requests(
        self,
        tree: &NormalizedSourceTree,
        selected: &[PartId],
    ) -> BdlResult<Vec<PartHydrationRequest>> {
        if self.stage == SelectionPreparation::Enqueue && tree.source.kind.queues_whole_posts() {
            return Ok(Vec::new());
        }
        selection_hydration_requests(tree, selected, |part| self.needs_request(part))
    }

    fn needs_request(self, part: &NormalizedPart) -> bool {
        if part.cid.is_none() {
            return true;
        }
        if !self.media && !self.subtitles {
            return false;
        }
        match self.stage {
            SelectionPreparation::Enqueue => false,
            SelectionPreparation::MissingStreams => part.streams.is_empty(),
            SelectionPreparation::Refresh => true,
            SelectionPreparation::PostDetails => true,
        }
    }

    pub(crate) async fn resolve_part(
        self,
        client: Arc<bpi_rs::BpiClient>,
        request: &PartHydrationRequest,
        part: Option<&NormalizedPart>,
    ) -> BdlResult<ResolvedDownload> {
        let input = request.input.clone();
        if self.stage == SelectionPreparation::Refresh
            && input.source_kind() == SourceKind::Video
            && request.target_cid.is_some()
        {
            let mut part = part.cloned().ok_or_else(|| BdlError::Planning {
                message: "下载任务缺少所选分 P。".into(),
            })?;
            VideoResolver::from_bpi_client(client)
                .load_part_playback(input, &mut part, self.media, self.subtitles)
                .await?;
            return Ok(ResolvedDownload::Part(part));
        }
        self.resolve_source(client, input, request.target_cid)
            .await
            .map(ResolvedDownload::Source)
    }

    pub(crate) async fn resolve_source(
        self,
        client: Arc<bpi_rs::BpiClient>,
        input: ClassifiedInput,
        target_cid: Option<u64>,
    ) -> BdlResult<NormalizedSourceTree> {
        let options = ResolveOptions {
            fetch_streams: self.media || self.subtitles,
        };
        match input.source_kind() {
            SourceKind::Video => {
                VideoResolver::from_bpi_client(client)
                    .resolve_with_requirements(input, self.media, self.subtitles, target_cid)
                    .await
            }
            SourceKind::Bangumi => {
                BangumiResolver::from_bpi_client(client)
                    .resolve(input, options)
                    .await
            }
            SourceKind::Cheese => {
                CheeseResolver::from_bpi_client(client)
                    .resolve(input, options)
                    .await
            }
            _ => Err(BdlError::Planning {
                message: "所选条目不是可下载的视频来源。".into(),
            }),
        }
    }
}
