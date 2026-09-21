use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use bdl_core::account::{
    AccountLibraryFolderKind, AccountLibraryPage, AccountSummary, ImportedCookie,
    account_library_page, verify_cookie_session,
};
use bdl_core::fetcher::FetchCancelToken;
use bdl_core::ids::{PartId, SourceId};
use bdl_core::input::{ClassifiedInput, classify_input};
use bdl_core::model::{
    MediaKind, MediaStream, NormalizedPart, NormalizedSourceTree, SourceKind, StreamCodec,
    StreamQuality,
};
use bdl_core::naming::DuplicateNamingStrategy;
use bdl_core::queue::{
    DownloadResourceIntent, DownloadTask, DownloadTaskRefreshInput, DownloadTaskRefreshIntent,
    DuplicateTaskPolicy, QueueLogEntry, ResourceStatus, TaskStatus,
};
use bdl_core::resolver::bangumi::BangumiResolver;
use bdl_core::resolver::cheese::CheeseResolver;
use bdl_core::resolver::collection::{CollectionResolver, SeriesResolver};
use bdl_core::resolver::favorite::FavoriteResolver;
use bdl_core::resolver::uploader::UploaderResolver;
use bdl_core::resolver::video::VideoResolver;
use bdl_core::resolver::{ResolveOptions, Resolver};
use bdl_core::settings::{AppSettings, validate_speed_limit};
use bdl_core::storage::TaskStorage;
use bdl_core::{BdlError, BdlResult};
use chrono::{DateTime, Utc};
use tokio::sync::Notify;

#[cfg(test)]
use crate::parse_session::{PartHydrationRequest, find_part};
use crate::parse_session::{
    append_source_page, cheese_season_id, collection_ids, favorite_media_id,
    hydrate_placeholder_part, next_page_request, normalize_selected_part_ids,
    remap_selected_part_ids, selected_hydration_requests, series_ids, should_continue_loading,
    should_expand_initial_source, uploader_mid,
};
pub use crate::queue_coordinator::{
    DuplicateTaskEnqueueResult, DuplicateTaskMatch, StartupRecoverySnapshot,
};
use crate::queue_coordinator::{
    append_new_tasks, dedupe_tasks_by_id, duplicate_task_matches, ignores_status_transition,
    prepare_duplicate_copies, prepare_startup_recovery, reserve_queued_paths,
    reset_interrupted_resources, reset_task_for_retry,
};
use crate::secure_store::SecureStore;

pub struct AppState {
    parse_pacer: crate::parse_pacing::ParsePacer,
    parse_control: crate::parse_control::ParseControl,
    resolver_client: Mutex<Option<(u64, Arc<bpi_rs::BpiClient>)>>,
    parse_sources: Mutex<HashMap<SourceId, NormalizedSourceTree>>,
    queue: Mutex<Vec<DownloadTask>>,
    storage: Mutex<TaskStorage>,
    settings: Mutex<SettingsSnapshot>,
    settings_path: PathBuf,
    data_dir: PathBuf,
    account: Mutex<AccountSnapshot>,
    account_cookie: Mutex<AccountCookieCache>,
    account_session_revision: AtomicU64,
    queue_worker_active: AtomicBool,
    queue_changed: Notify,
    queue_cancellations: Mutex<HashMap<String, FetchCancelToken>>,
    startup_recovery: Mutex<StartupRecoverySnapshot>,
    secure_store: SecureStore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AccountCookieCache {
    Unloaded,
    Loaded(Option<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedSelection {
    pub tree: NormalizedSourceTree,
    pub part_ids: Vec<PartId>,
    pub tree_updated: bool,
}

impl AppState {
    pub fn new(
        preferred_data_dir: impl Into<PathBuf>,
        default_download_dir: impl Into<PathBuf>,
    ) -> BdlResult<Self> {
        let preferred_data_dir = preferred_data_dir.into();
        let legacy_data_dir = std::env::current_dir()?.join(".bdl");
        let default_data_dir = select_startup_data_dir(&preferred_data_dir, &legacy_data_dir);
        let settings_path = default_data_dir.join("settings.json");
        let mut settings = load_settings(&settings_path)?;
        if settings.download_dir.is_none() {
            settings.download_dir =
                Some(default_download_dir.into().to_string_lossy().into_owned());
        }
        let data_dir = settings
            .data_dir
            .as_deref()
            .map(PathBuf::from)
            .unwrap_or(default_data_dir);
        let mut storage = TaskStorage::open(data_dir.join("tasks.sqlite"))?;
        let mut queue = storage.load_tasks()?;
        let startup_recovery = prepare_startup_recovery(&mut queue, settings.startup_auto_recovery);
        if !startup_recovery.task_ids.is_empty() {
            storage.replace_tasks(&queue)?;
        }
        let account = load_account_snapshot(&storage)?;
        let account_cookie = if account.logged_in {
            AccountCookieCache::Unloaded
        } else {
            AccountCookieCache::Loaded(None)
        };
        let secure_store = SecureStore::new(&data_dir);

        Ok(Self {
            parse_pacer: Default::default(),
            parse_control: Default::default(),
            resolver_client: Default::default(),
            parse_sources: Mutex::new(HashMap::new()),
            queue: Mutex::new(queue),
            storage: Mutex::new(storage),
            settings: Mutex::new(settings),
            settings_path,
            data_dir,
            account: Mutex::new(account),
            account_cookie: Mutex::new(account_cookie),
            account_session_revision: AtomicU64::new(0),
            queue_worker_active: AtomicBool::new(false),
            queue_changed: Notify::new(),
            queue_cancellations: Mutex::new(HashMap::new()),
            startup_recovery: Mutex::new(startup_recovery),
            secure_store,
        })
    }

    pub async fn parse_source(
        &self,
        input: &str,
        fetch_streams: bool,
    ) -> BdlResult<NormalizedSourceTree> {
        self.parse_source_with_options(input, fetch_streams, false)
            .await
    }

    async fn parse_source_with_options(
        &self,
        input: &str,
        fetch_streams: bool,
        expand_video_collection: bool,
    ) -> BdlResult<NormalizedSourceTree> {
        let classified = classify_input(input)?;
        let options = ResolveOptions { fetch_streams };
        let tree = self
            .parse_pacer
            .run_retry(
                self.settings()?.parse_rules,
                || {
                    let classified = classified.clone();
                    async move {
                        let tree = match classified.source_kind() {
                            SourceKind::Video if expand_video_collection => {
                                let resolver = self.video_resolver()?;
                                if let Some(collection) =
                                    resolver.collection_hint(&classified).await?
                                {
                                    let raw_url = format!(
                                        "https://space.bilibili.com/{}/lists/{}?type=season",
                                        collection.mid, collection.season_id
                                    );
                                    self.collection_resolver()?
                                        .resolve(ClassifiedInput::Collection { raw_url }, options)
                                        .await?
                                } else {
                                    resolver.resolve(classified, options).await?
                                }
                            }
                            SourceKind::Video => {
                                self.video_resolver()?.resolve(classified, options).await?
                            }
                            SourceKind::Favorite => {
                                self.favorite_resolver()?
                                    .resolve(classified, options)
                                    .await?
                            }
                            SourceKind::Uploader => {
                                self.uploader_resolver()?
                                    .resolve(classified, options)
                                    .await?
                            }
                            SourceKind::Collection => {
                                self.collection_resolver()?
                                    .resolve(classified, options)
                                    .await?
                            }
                            SourceKind::Series => {
                                self.series_resolver()?.resolve(classified, options).await?
                            }
                            SourceKind::Bangumi => {
                                self.bangumi_resolver()?
                                    .resolve(classified, options)
                                    .await?
                            }
                            SourceKind::Cheese => {
                                self.cheese_resolver()?.resolve(classified, options).await?
                            }
                            _ => self.video_resolver()?.resolve(classified, options).await?,
                        };
                        Ok(tree)
                    }
                },
                |tree| {
                    usize::from(matches!(
                        tree.source.kind,
                        SourceKind::Favorite
                            | SourceKind::Uploader
                            | SourceKind::Collection
                            | SourceKind::Series
                            | SourceKind::Cheese
                    ))
                },
                |_| {},
            )
            .await?;
        self.parse_sources
            .lock()
            .map_err(|_| state_poisoned("parse_sources"))?
            .insert(tree.source.id.clone(), tree.clone());
        Ok(tree)
    }

    pub async fn parse_source_all(
        &self,
        input: &str,
        fetch_streams: bool,
    ) -> BdlResult<NormalizedSourceTree> {
        let tree = self.parse_source(input, fetch_streams).await?;
        if tree.source.has_more {
            self.load_all(&tree.source.id, None).await
        } else {
            Ok(tree)
        }
    }

    pub async fn parse_source_for_workspace(
        &self,
        input: &str,
        fetch_streams: bool,
        expand_video_collection: bool,
    ) -> BdlResult<NormalizedSourceTree> {
        let tree = self
            .parse_source_with_options(input, fetch_streams, expand_video_collection)
            .await?;
        if should_expand_initial_source(tree.source.kind, tree.source.has_more) {
            self.load_all(&tree.source.id, None).await
        } else {
            Ok(tree)
        }
    }

    pub fn close_source(&self, source_id: &SourceId) -> BdlResult<bool> {
        self.cancel_parse(&source_id.0);
        Ok(self
            .parse_sources
            .lock()
            .map_err(|_| state_poisoned("parse_sources"))?
            .remove(source_id)
            .is_some())
    }

    pub fn source_snapshot(&self, source_id: &SourceId) -> BdlResult<NormalizedSourceTree> {
        self.parse_sources
            .lock()
            .map_err(|_| state_poisoned("parse_sources"))?
            .get(source_id)
            .cloned()
            .ok_or_else(|| BdlError::Planning {
                message: format!("解析源 `{}` 不存在，请重新解析。", source_id.0),
            })
    }

    pub async fn refresh_source(&self, source_id: &SourceId) -> BdlResult<NormalizedSourceTree> {
        let current = self.source_snapshot(source_id)?;
        self.parse_source_for_workspace(&current.source.input, false, false)
            .await
    }

    pub async fn load_more(&self, source_id: &SourceId) -> BdlResult<NormalizedSourceTree> {
        let mut tree = self.source_snapshot(source_id)?;
        self.append_next_page(&mut tree).await?;
        self.store_source_tree(&tree)?;

        Ok(tree)
    }

    pub async fn load_all(
        &self,
        source_id: &SourceId,
        limit: Option<usize>,
    ) -> BdlResult<NormalizedSourceTree> {
        let mut tree = self.source_snapshot(source_id)?;
        while should_continue_loading(tree.source.has_more, tree.source.loaded_count, limit) {
            let loaded_before = tree.source.loaded_count;
            self.append_next_page(&mut tree).await?;
            self.store_source_tree(&tree)?;
            if tree.source.loaded_count == loaded_before {
                break;
            }
        }

        self.store_source_tree(&tree)?;

        Ok(tree)
    }

    pub async fn prepare_selection(
        &self,
        source_id: &SourceId,
        selected_part_ids: &[PartId],
    ) -> BdlResult<PreparedSelection> {
        let mut tree = self.source_snapshot(source_id)?;
        let selected_part_ids = normalize_selected_part_ids(&tree, selected_part_ids);
        let requests = selected_hydration_requests(&tree, &selected_part_ids)?;
        if requests.is_empty() {
            return Ok(PreparedSelection {
                tree,
                part_ids: selected_part_ids,
                tree_updated: false,
            });
        }

        let mut part_ids = selected_part_ids;
        let operation = self.parse_control.begin(&source_id.0);

        for request in requests {
            let hydrated = operation
                .run(self.parse_pacer.run_retry(
                    self.settings()?.parse_rules,
                    || {
                        self.resolve_media_input_with_streams_unpaced(
                            request.input.clone(),
                            request.target_cid,
                        )
                    },
                    |_| 0,
                    |phase| operation.phase(phase),
                ))
                .await?;
            let hydrated_part_ids = hydrate_placeholder_part(
                &mut tree,
                &request.part_id,
                request.target_cid,
                hydrated,
            )?;
            remap_selected_part_ids(&mut part_ids, &request.part_id, &hydrated_part_ids);
            self.store_source_tree(&tree)?;
        }

        self.parse_sources
            .lock()
            .map_err(|_| state_poisoned("parse_sources"))?
            .insert(tree.source.id.clone(), tree.clone());

        Ok(PreparedSelection {
            tree,
            part_ids,
            tree_updated: true,
        })
    }

    pub fn enqueue_tasks_with_duplicate_policy(
        &self,
        mut tasks: Vec<DownloadTask>,
        policy: DuplicateTaskPolicy,
        naming_strategy: DuplicateNamingStrategy,
    ) -> BdlResult<DuplicateTaskEnqueueResult> {
        let mut queue = self.queue.lock().map_err(|_| state_poisoned("queue"))?;
        let removed_existing_duplicates = dedupe_tasks_by_id(&mut queue);
        let duplicates = duplicate_task_matches(&tasks, &queue);

        if policy == DuplicateTaskPolicy::Ask && !duplicates.is_empty() {
            if removed_existing_duplicates {
                self.persist_queue(&queue)?;
            }
            return Ok(DuplicateTaskEnqueueResult {
                inserted: Vec::new(),
                duplicates,
                requires_confirmation: true,
                skipped_existing: 0,
            });
        }

        match policy {
            DuplicateTaskPolicy::Skip => {
                let duplicate_ids = duplicates
                    .iter()
                    .map(|duplicate| duplicate.proposed_task_id.as_str())
                    .collect::<HashSet<_>>();
                tasks.retain(|task| !duplicate_ids.contains(task.id.as_str()));
            }
            DuplicateTaskPolicy::Create => {
                tasks = prepare_duplicate_copies(tasks, &queue)?;
            }
            DuplicateTaskPolicy::Ask => {}
        }

        let planned_count = tasks.len();
        let tasks = reserve_queued_paths(tasks, &queue, naming_strategy)?;
        let skipped_existing = planned_count - tasks.len();
        let inserted = append_new_tasks(&mut queue, tasks);
        if removed_existing_duplicates || !inserted.is_empty() {
            self.persist_queue(&queue)?;
        }
        Ok(DuplicateTaskEnqueueResult {
            inserted,
            duplicates,
            requires_confirmation: false,
            skipped_existing,
        })
    }

    pub fn try_start_queue_worker(&self) -> bool {
        !self.queue_worker_active.swap(true, Ordering::SeqCst)
    }

    pub fn finish_queue_worker(&self) {
        self.queue_worker_active.store(false, Ordering::SeqCst);
    }

    pub fn notify_queue_changed(&self) {
        self.queue_changed.notify_one();
    }

    pub async fn wait_for_queue_change(&self) {
        self.queue_changed.notified().await;
    }

    pub fn take_next_startable_task(&self) -> BdlResult<Option<DownloadTask>> {
        self.take_next_startable_task_at(Utc::now())
    }

    pub fn take_next_startable_task_at(
        &self,
        now: DateTime<Utc>,
    ) -> BdlResult<Option<DownloadTask>> {
        let mut queue = self.queue.lock().map_err(|_| state_poisoned("queue"))?;
        // A paused attempt may still be unwinding after an immediate resume.
        let cancellations = self
            .queue_cancellations
            .lock()
            .map_err(|_| state_poisoned("queue_cancellations"))?;
        let Some(task_index) = queue
            .iter()
            .position(|task| task.can_start_at(now) && !cancellations.contains_key(&task.id))
        else {
            return Ok(None);
        };

        queue[task_index].status = TaskStatus::Downloading;
        queue[task_index].scheduled_at = None;
        let task = queue[task_index].clone();
        self.persist_queue(&queue)?;
        Ok(Some(task))
    }

    pub fn has_startable_task(&self) -> BdlResult<bool> {
        Ok(self
            .queue
            .lock()
            .map_err(|_| state_poisoned("queue"))?
            .iter()
            .any(|task| task.can_start_at(Utc::now())))
    }

    pub fn next_scheduled_at(&self) -> BdlResult<Option<DateTime<Utc>>> {
        Ok(self
            .queue
            .lock()
            .map_err(|_| state_poisoned("queue"))?
            .iter()
            .filter(|task| task.status == TaskStatus::Waiting)
            .filter_map(|task| task.scheduled_at)
            .min())
    }

    pub fn set_task_schedule(
        &self,
        task_id: &str,
        scheduled_at: Option<DateTime<Utc>>,
    ) -> BdlResult<DownloadTask> {
        let mut queue = self.queue.lock().map_err(|_| state_poisoned("queue"))?;
        let task = queue
            .iter_mut()
            .find(|task| task.id == task_id)
            .ok_or_else(|| BdlError::Planning {
                message: format!("任务 `{task_id}` 不存在。"),
            })?;
        if !matches!(task.status, TaskStatus::Waiting | TaskStatus::Paused) {
            return Err(BdlError::Planning {
                message: "只有排队中或已暂停的任务可以设置开始时间。".to_owned(),
            });
        }

        task.status = TaskStatus::Waiting;
        task.scheduled_at = scheduled_at;
        reset_interrupted_resources(&mut task.resources);
        let updated = task.clone();
        self.persist_queue(&queue)?;
        Ok(updated)
    }

    pub fn set_task_speed_limit(
        &self,
        task_id: &str,
        speed_limit_bytes_per_second: Option<u64>,
    ) -> BdlResult<DownloadTask> {
        validate_speed_limit(speed_limit_bytes_per_second, "单任务下载限速")?;
        let mut queue = self.queue.lock().map_err(|_| state_poisoned("queue"))?;
        let task = queue
            .iter_mut()
            .find(|task| task.id == task_id)
            .ok_or_else(|| BdlError::Planning {
                message: format!("任务 `{task_id}` 不存在。"),
            })?;
        if !matches!(
            task.status,
            TaskStatus::Waiting | TaskStatus::Paused | TaskStatus::Failed
        ) {
            return Err(BdlError::Planning {
                message: "请先暂停任务，再修改单任务下载限速。".to_owned(),
            });
        }

        task.speed_limit_bytes_per_second = speed_limit_bytes_per_second;
        let updated = task.clone();
        self.persist_queue(&queue)?;
        self.notify_queue_changed();
        Ok(updated)
    }

    pub fn update_task_status(&self, task_id: &str, status: TaskStatus) -> BdlResult<DownloadTask> {
        let mut queue = self.queue.lock().map_err(|_| state_poisoned("queue"))?;
        let task_index = queue
            .iter()
            .position(|task| task.id == task_id)
            .ok_or_else(|| BdlError::Planning {
                message: format!("任务 `{task_id}` 不存在。"),
            })?;

        let current = queue[task_index].status;
        if ignores_status_transition(current, status) {
            return Ok(queue[task_index].clone());
        }

        queue[task_index].status = status;
        if status.can_start() {
            queue[task_index].scheduled_at = None;
            reset_interrupted_resources(&mut queue[task_index].resources);
        } else if status == TaskStatus::Paused || status.is_terminal() {
            queue[task_index].scheduled_at = None;
        }
        let task = queue[task_index].clone();
        self.persist_queue(&queue)?;
        if matches!(status, TaskStatus::Paused | TaskStatus::Cancelled) {
            self.cancel_running_task(task_id)?;
        }
        Ok(task)
    }

    pub fn register_task_cancel_token(&self, task_id: &str) -> BdlResult<FetchCancelToken> {
        let token = FetchCancelToken::new();
        self.queue_cancellations
            .lock()
            .map_err(|_| state_poisoned("queue_cancellations"))?
            .insert(task_id.to_owned(), token.clone());
        Ok(token)
    }

    pub fn cancel_running_task(&self, task_id: &str) -> BdlResult<bool> {
        let token = self
            .queue_cancellations
            .lock()
            .map_err(|_| state_poisoned("queue_cancellations"))?
            .get(task_id)
            .cloned();
        if let Some(token) = token {
            token.cancel();
            return Ok(true);
        }

        Ok(false)
    }

    pub fn clear_task_cancel_token(&self, task_id: &str) -> BdlResult<()> {
        self.queue_cancellations
            .lock()
            .map_err(|_| state_poisoned("queue_cancellations"))?
            .remove(task_id);
        Ok(())
    }

    pub fn update_resource_status(
        &self,
        task_id: &str,
        resource_id: &str,
        status: ResourceStatus,
    ) -> BdlResult<DownloadTask> {
        let mut queue = self.queue.lock().map_err(|_| state_poisoned("queue"))?;
        let task_index = queue
            .iter()
            .position(|task| task.id == task_id)
            .ok_or_else(|| BdlError::Planning {
                message: format!("任务 `{task_id}` 不存在。"),
            })?;
        let resource = queue[task_index]
            .resources
            .iter_mut()
            .find(|resource| resource.id == resource_id)
            .ok_or_else(|| BdlError::Planning {
                message: format!("资源 `{resource_id}` 不存在。"),
            })?;

        resource.status = status;
        let task = queue[task_index].clone();
        self.persist_queue(&queue)?;
        Ok(task)
    }

    pub fn retry_task(&self, task_id: &str) -> BdlResult<DownloadTask> {
        let mut queue = self.queue.lock().map_err(|_| state_poisoned("queue"))?;
        let task_index = queue
            .iter()
            .position(|task| task.id == task_id)
            .ok_or_else(|| BdlError::Planning {
                message: format!("任务 `{task_id}` 不存在。"),
            })?;

        reset_task_for_retry(&mut queue[task_index]);

        let task = queue[task_index].clone();
        self.persist_queue(&queue)?;
        Ok(task)
    }

    pub(crate) fn finish_task_attempt(
        &self,
        task_id: &str,
        next: TaskStatus,
        cancel_token: &FetchCancelToken,
    ) -> BdlResult<Option<DownloadTask>> {
        let mut queue = self.queue.lock().map_err(|_| state_poisoned("queue"))?;
        let Some(task) = queue.iter_mut().find(|task| task.id == task_id) else {
            return Ok(None);
        };
        // Check and transition under the same lock as pause/cancel. A late
        // completion must also leave an explicitly resumed/scheduled task alone.
        let can_finish = match (task.status, next) {
            (TaskStatus::Downloading, TaskStatus::Waiting | TaskStatus::Failed) => {
                !cancel_token.is_cancelled()
            }
            (TaskStatus::Muxing, TaskStatus::Failed) => true,
            _ => false,
        };
        if !can_finish {
            return Ok(None);
        }
        if next == TaskStatus::Waiting {
            reset_task_for_retry(task);
        } else {
            task.status = next;
        }
        let updated = task.clone();
        self.persist_queue(&queue)?;
        Ok(Some(updated))
    }

    pub async fn refresh_task_media_urls(&self, task_id: &str) -> BdlResult<DownloadTask> {
        let task = self.task_snapshot(task_id)?;
        let refresh_ids = task_media_refresh_ids(&task)?;
        let refreshed = self
            .resolve_media_input_with_streams(refresh_ids.input, Some(refresh_ids.cid))
            .await?;
        let part =
            find_part_by_cid(&refreshed, refresh_ids.cid).ok_or_else(|| BdlError::Planning {
                message: format!(
                    "刷新下载地址失败：视频解析结果缺少 CID `{}`。",
                    refresh_ids.cid
                ),
            })?;
        let needs_video = task
            .resources
            .iter()
            .any(|resource| resource.intent == DownloadResourceIntent::Video);
        let needs_audio = task
            .resources
            .iter()
            .any(|resource| resource.intent == DownloadResourceIntent::Audio);
        let video = if needs_video {
            Some(
                select_task_stream(
                    part,
                    MediaKind::Video,
                    &task.media_selection.video_quality,
                    Some(&task.media_selection.video_codec),
                )
                .ok_or_else(|| BdlError::Planning {
                    message: format!("刷新下载地址失败：`{}` 缺少视频流。", part.title),
                })?,
            )
        } else {
            None
        };
        let audio = if needs_audio {
            Some(
                select_task_stream(
                    part,
                    MediaKind::Audio,
                    &task.media_selection.audio_quality,
                    None,
                )
                .ok_or_else(|| BdlError::Planning {
                    message: format!("刷新下载地址失败：`{}` 缺少音频流。", part.title),
                })?,
            )
        } else {
            None
        };

        self.replace_task_media_urls(task_id, video, audio)
    }

    pub fn task_status(&self, task_id: &str) -> BdlResult<TaskStatus> {
        self.queue
            .lock()
            .map_err(|_| state_poisoned("queue"))?
            .iter()
            .find(|task| task.id == task_id)
            .map(|task| task.status)
            .ok_or_else(|| BdlError::Planning {
                message: format!("任务 `{task_id}` 不存在。"),
            })
    }

    pub fn remove_task(&self, task_id: &str) -> BdlResult<bool> {
        let _ = self.cancel_running_task(task_id)?;
        let mut queue = self.queue.lock().map_err(|_| state_poisoned("queue"))?;
        let original_len = queue.len();
        queue.retain(|task| task.id != task_id);
        let removed = queue.len() != original_len;
        if removed {
            self.persist_queue(&queue)?;
        }
        Ok(removed)
    }

    pub fn queue_snapshot(&self) -> BdlResult<Vec<DownloadTask>> {
        Ok(self
            .queue
            .lock()
            .map_err(|_| state_poisoned("queue"))?
            .clone())
    }

    pub fn task_snapshot(&self, task_id: &str) -> BdlResult<DownloadTask> {
        self.queue
            .lock()
            .map_err(|_| state_poisoned("queue"))?
            .iter()
            .find(|task| task.id == task_id)
            .cloned()
            .ok_or_else(|| BdlError::Planning {
                message: format!("任务 `{task_id}` 不存在。"),
            })
    }

    fn replace_task_media_urls(
        &self,
        task_id: &str,
        video: Option<&MediaStream>,
        audio: Option<&MediaStream>,
    ) -> BdlResult<DownloadTask> {
        let mut queue = self.queue.lock().map_err(|_| state_poisoned("queue"))?;
        let task_index = queue
            .iter()
            .position(|task| task.id == task_id)
            .ok_or_else(|| BdlError::Planning {
                message: format!("任务 `{task_id}` 不存在。"),
            })?;

        for resource in &mut queue[task_index].resources {
            let stream = match resource.intent {
                DownloadResourceIntent::Video => video,
                DownloadResourceIntent::Audio => audio,
                DownloadResourceIntent::Cover
                | DownloadResourceIntent::Subtitle
                | DownloadResourceIntent::Danmaku
                | DownloadResourceIntent::Nfo => None,
            };

            if let Some(stream) = stream {
                resource.current_urls.clone_from(&stream.urls);
                resource.headers.clone_from(&stream.headers);
            }
        }

        let task = queue[task_index].clone();
        self.persist_queue(&queue)?;
        Ok(task)
    }

    pub fn append_task_log(&self, entry: QueueLogEntry) -> BdlResult<QueueLogEntry> {
        self.storage
            .lock()
            .map_err(|_| state_poisoned("storage"))?
            .append_task_log(&entry)?;
        Ok(entry)
    }

    pub fn task_logs(&self, task_id: &str, limit: usize) -> BdlResult<Vec<QueueLogEntry>> {
        self.storage
            .lock()
            .map_err(|_| state_poisoned("storage"))?
            .load_task_logs(task_id, limit)
    }

    pub fn record_completed_task(&self, task: &DownloadTask) -> BdlResult<()> {
        let logs = self.task_logs(&task.id, 1000)?;
        self.storage
            .lock()
            .map_err(|_| state_poisoned("storage"))?
            .save_completed_record(task, &logs)?;
        Ok(())
    }

    pub fn settings(&self) -> BdlResult<SettingsSnapshot> {
        Ok(self
            .settings
            .lock()
            .map_err(|_| state_poisoned("settings"))?
            .clone())
    }

    pub fn data_dir(&self) -> PathBuf {
        self.data_dir.clone()
    }

    pub fn startup_recovery(&self) -> BdlResult<StartupRecoverySnapshot> {
        Ok(self
            .startup_recovery
            .lock()
            .map_err(|_| state_poisoned("startup_recovery"))?
            .clone())
    }

    pub fn clear_startup_recovery(&self) -> BdlResult<StartupRecoverySnapshot> {
        let mut startup_recovery = self
            .startup_recovery
            .lock()
            .map_err(|_| state_poisoned("startup_recovery"))?;
        startup_recovery.task_ids.clear();
        Ok(startup_recovery.clone())
    }

    pub fn update_settings(&self, settings: SettingsSnapshot) -> BdlResult<SettingsSnapshot> {
        let settings = settings.normalized();
        settings.validate()?;
        save_settings(&self.settings_path, &settings)?;
        *self
            .settings
            .lock()
            .map_err(|_| state_poisoned("settings"))? = settings;
        self.notify_queue_changed();
        self.settings()
    }

    pub fn account(&self) -> BdlResult<AccountSnapshot> {
        Ok(self
            .account
            .lock()
            .map_err(|_| state_poisoned("account"))?
            .clone())
    }

    pub fn import_cookie(&self, raw_cookie: &str) -> BdlResult<AccountSnapshot> {
        let imported_cookie = ImportedCookie::parse(raw_cookie)?;
        let _resolver = VideoResolver::from_cookie(imported_cookie.as_header())?;
        let account = AccountSummary::from_imported_cookie(&imported_cookie);

        self.secure_store.save_cookie(imported_cookie.as_header())?;
        self.storage
            .lock()
            .map_err(|_| state_poisoned("storage"))?
            .save_account_summary(&account)?;
        *self
            .account_cookie
            .lock()
            .map_err(|_| state_poisoned("account_cookie"))? =
            AccountCookieCache::Loaded(Some(imported_cookie.as_header().to_owned()));
        *self.account.lock().map_err(|_| state_poisoned("account"))? = account.clone();
        self.account_session_revision.fetch_add(1, Ordering::SeqCst);

        Ok(account)
    }

    pub fn logout(&self) -> BdlResult<AccountSnapshot> {
        self.clear_account_state()
    }

    pub async fn verify_account(&self) -> BdlResult<AccountSnapshot> {
        let Some(raw_cookie) = self.account_cookie()? else {
            if self.account()?.logged_in {
                return self.clear_account_state();
            }
            return self.account();
        };

        let account = verify_cookie_session(&raw_cookie).await?;
        if account.logged_in {
            self.storage
                .lock()
                .map_err(|_| state_poisoned("storage"))?
                .save_account_summary(&account)?;
            *self.account.lock().map_err(|_| state_poisoned("account"))? = account.clone();
            return Ok(account);
        }

        self.clear_account_state()
    }

    pub async fn account_library(
        &self,
        kind: AccountLibraryFolderKind,
        page: u32,
        page_size: u32,
    ) -> BdlResult<AccountLibraryPage> {
        if page == 0 || !(1..=50).contains(&page_size) {
            return Err(BdlError::Account {
                message: "内容库分页参数无效。".to_owned(),
            });
        }
        let revision = self.account_session_revision.load(Ordering::SeqCst);
        let cookie = self.account_cookie()?.ok_or_else(|| BdlError::Account {
            message: "请先登录，再浏览账号内容。".to_owned(),
        })?;
        let mid = ImportedCookie::parse(cookie.clone())?
            .dede_user_id()
            .and_then(|value| value.parse::<u64>().ok())
            .ok_or_else(|| BdlError::Account {
                message: "账号信息尚未验证，请刷新登录状态后重试。".to_owned(),
            })?;
        if revision != self.account_session_revision.load(Ordering::SeqCst) {
            return Err(account_session_changed());
        }

        let result = account_library_page(&cookie, mid, kind, page, page_size).await?;
        if revision != self.account_session_revision.load(Ordering::SeqCst) {
            return Err(account_session_changed());
        }
        Ok(result)
    }

    fn clear_account_state(&self) -> BdlResult<AccountSnapshot> {
        self.secure_store.clear_cookie()?;
        self.storage
            .lock()
            .map_err(|_| state_poisoned("storage"))?
            .clear_account_summary()?;
        *self
            .account_cookie
            .lock()
            .map_err(|_| state_poisoned("account_cookie"))? = AccountCookieCache::Loaded(None);
        *self.account.lock().map_err(|_| state_poisoned("account"))? = AccountSnapshot::default();
        self.account_session_revision.fetch_add(1, Ordering::SeqCst);

        self.account()
    }

    fn persist_queue(&self, queue: &[DownloadTask]) -> BdlResult<()> {
        self.storage
            .lock()
            .map_err(|_| state_poisoned("storage"))?
            .replace_tasks(queue)
    }

    fn store_source_tree(&self, tree: &NormalizedSourceTree) -> BdlResult<()> {
        self.parse_sources
            .lock()
            .map_err(|_| state_poisoned("parse_sources"))?
            .insert(tree.source.id.clone(), tree.clone());
        Ok(())
    }

    async fn append_next_page(&self, tree: &mut NormalizedSourceTree) -> BdlResult<()> {
        let operation = self.parse_control.begin(&tree.source.id.0);
        let next = operation
            .run(self.parse_pacer.run_retry(
                self.settings()?.parse_rules,
                || {
                    let mut next = tree.clone();
                    async move {
                        self.append_next_page_unpaced(&mut next).await?;
                        Ok(next)
                    }
                },
                |_| 1,
                |phase| operation.phase(phase),
            ))
            .await?;
        *tree = next;
        Ok(())
    }

    async fn append_next_page_unpaced(&self, tree: &mut NormalizedSourceTree) -> BdlResult<()> {
        match tree.source.kind {
            SourceKind::Favorite => {
                let media_id = favorite_media_id(tree)?;
                let request = next_page_request(tree)?;
                let next_page = self
                    .favorite_resolver()?
                    .resolve_page(media_id, request)
                    .await?;
                append_source_page(tree, next_page)
            }
            SourceKind::Uploader => {
                let mid = uploader_mid(tree)?;
                let request = next_page_request(tree)?;
                let next_page = self.uploader_resolver()?.resolve_page(mid, request).await?;
                append_source_page(tree, next_page)
            }
            SourceKind::Collection => {
                let ids = collection_ids(tree)?;
                let request = next_page_request(tree)?;
                let next_page = self
                    .collection_resolver()?
                    .resolve_page(ids, request)
                    .await?;
                append_source_page(tree, next_page)
            }
            SourceKind::Series => {
                let ids = series_ids(tree)?;
                let request = next_page_request(tree)?;
                let next_page = self.series_resolver()?.resolve_page(ids, request).await?;
                append_source_page(tree, next_page)
            }
            SourceKind::Cheese => {
                let season_id = cheese_season_id(tree)?;
                let request = next_page_request(tree)?;
                let next_page = self
                    .cheese_resolver()?
                    .resolve_page(season_id, request)
                    .await?;
                append_source_page(tree, next_page)
            }
            kind => Err(BdlError::UnsupportedSource {
                kind: source_kind_name(kind).to_owned(),
            }),
        }
    }

    async fn resolve_media_input_with_streams(
        &self,
        input: ClassifiedInput,
        target_cid: Option<u64>,
    ) -> BdlResult<NormalizedSourceTree> {
        self.parse_pacer
            .run_retry(
                self.settings()?.parse_rules,
                || self.resolve_media_input_with_streams_unpaced(input.clone(), target_cid),
                |_| 0,
                |_| {},
            )
            .await
    }

    async fn resolve_media_input_with_streams_unpaced(
        &self,
        input: ClassifiedInput,
        target_cid: Option<u64>,
    ) -> BdlResult<NormalizedSourceTree> {
        let options = ResolveOptions {
            fetch_streams: true,
        };

        match input.source_kind() {
            SourceKind::Video => match target_cid {
                Some(cid) => {
                    self.video_resolver()?
                        .resolve_target_streams(input, cid)
                        .await
                }
                None => self.video_resolver()?.resolve(input, options).await,
            },
            SourceKind::Bangumi => self.bangumi_resolver()?.resolve(input, options).await,
            SourceKind::Cheese => self.cheese_resolver()?.resolve(input, options).await,
            kind => Err(BdlError::UnsupportedSource {
                kind: source_kind_name(kind).to_owned(),
            }),
        }
    }

    pub fn cancel_parse(&self, source_id: &str) {
        self.parse_control.cancel(source_id);
    }

    pub fn parse_progress(&self, source_id: &str) -> crate::parse_control::ParseProgress {
        self.parse_control.progress(source_id)
    }

    fn shared_resolver_client(&self) -> BdlResult<Arc<bpi_rs::BpiClient>> {
        let revision = self.account_session_revision.load(Ordering::SeqCst);
        let mut cached = self
            .resolver_client
            .lock()
            .map_err(|_| state_poisoned("resolver_client"))?;
        if let Some((_, client)) = cached
            .as_ref()
            .filter(|(cached_revision, _)| *cached_revision == revision)
        {
            return Ok(client.clone());
        }
        let mut builder = bpi_rs::BpiClient::builder().timeout(std::time::Duration::from_secs(20));
        if let Some(cookie) = self.account_cookie()? {
            builder = builder.cookie(cookie);
        }
        let client = Arc::new(
            builder
                .build()
                .map_err(|error| BdlError::Bpi(error.to_string()))?,
        );
        *cached = Some((revision, client.clone()));
        Ok(client)
    }

    fn video_resolver(&self) -> BdlResult<VideoResolver> {
        Ok(VideoResolver::from_bpi_client(
            self.shared_resolver_client()?,
        ))
    }

    fn uploader_resolver(&self) -> BdlResult<UploaderResolver> {
        Ok(UploaderResolver::from_bpi_client(
            self.shared_resolver_client()?,
        ))
    }

    fn favorite_resolver(&self) -> BdlResult<FavoriteResolver> {
        Ok(FavoriteResolver::from_bpi_client(
            self.shared_resolver_client()?,
        ))
    }

    fn collection_resolver(&self) -> BdlResult<CollectionResolver> {
        Ok(CollectionResolver::from_bpi_client(
            self.shared_resolver_client()?,
        ))
    }

    fn series_resolver(&self) -> BdlResult<SeriesResolver> {
        Ok(SeriesResolver::from_bpi_client(
            self.shared_resolver_client()?,
        ))
    }

    fn bangumi_resolver(&self) -> BdlResult<BangumiResolver> {
        Ok(BangumiResolver::from_bpi_client(
            self.shared_resolver_client()?,
        ))
    }

    fn cheese_resolver(&self) -> BdlResult<CheeseResolver> {
        Ok(CheeseResolver::from_bpi_client(
            self.shared_resolver_client()?,
        ))
    }

    fn account_cookie(&self) -> BdlResult<Option<String>> {
        let revision = self.account_session_revision.load(Ordering::SeqCst);
        {
            let cache = self
                .account_cookie
                .lock()
                .map_err(|_| state_poisoned("account_cookie"))?;
            if let AccountCookieCache::Loaded(cookie) = &*cache {
                return Ok(cookie.clone());
            }
        }

        let loaded = load_persisted_account_cookie(&self.secure_store)?;
        let mut cache = self
            .account_cookie
            .lock()
            .map_err(|_| state_poisoned("account_cookie"))?;

        if revision != self.account_session_revision.load(Ordering::SeqCst) {
            return match &*cache {
                AccountCookieCache::Loaded(cookie) => Ok(cookie.clone()),
                AccountCookieCache::Unloaded => Ok(None),
            };
        }

        match &*cache {
            AccountCookieCache::Loaded(cookie) => Ok(cookie.clone()),
            AccountCookieCache::Unloaded => {
                *cache = AccountCookieCache::Loaded(loaded.clone());
                Ok(loaded)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TaskMediaRefreshIds {
    input: ClassifiedInput,
    cid: u64,
}

pub type SettingsSnapshot = AppSettings;
pub type AccountSnapshot = AccountSummary;

fn state_poisoned(name: &'static str) -> BdlError {
    BdlError::Planning {
        message: format!("state lock `{name}` is poisoned"),
    }
}

fn account_session_changed() -> BdlError {
    BdlError::Account {
        message: "账号已切换，请重新加载内容库。".to_owned(),
    }
}

fn select_startup_data_dir(preferred: &std::path::Path, legacy: &std::path::Path) -> PathBuf {
    if has_persisted_state(preferred) || !has_persisted_state(legacy) {
        preferred.to_path_buf()
    } else {
        legacy.to_path_buf()
    }
}

fn has_persisted_state(path: &std::path::Path) -> bool {
    path.join("settings.json").is_file() || path.join("tasks.sqlite").is_file()
}

fn load_settings(path: &PathBuf) -> BdlResult<SettingsSnapshot> {
    match fs::read_to_string(path) {
        Ok(raw) => Ok(serde_json::from_str::<SettingsSnapshot>(&raw)?.normalized()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(SettingsSnapshot::default().normalized())
        }
        Err(error) => Err(error.into()),
    }
}

fn save_settings(path: &PathBuf, settings: &SettingsSnapshot) -> BdlResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, serde_json::to_string_pretty(settings)?)?;
    Ok(())
}

fn load_account_snapshot(storage: &TaskStorage) -> BdlResult<AccountSnapshot> {
    Ok(storage
        .load_account_summary()?
        .filter(|account| account.logged_in)
        .unwrap_or_default())
}

fn load_persisted_account_cookie(secure_store: &SecureStore) -> BdlResult<Option<String>> {
    let Some(raw_cookie) = secure_store.load_cookie()? else {
        return Ok(None);
    };

    match ImportedCookie::parse(&raw_cookie) {
        Ok(_) => Ok(Some(raw_cookie)),
        Err(_) => {
            secure_store.clear_cookie()?;
            Ok(None)
        }
    }
}

fn task_media_refresh_ids(task: &DownloadTask) -> BdlResult<TaskMediaRefreshIds> {
    if let Some(refresh_intent) = &task.refresh_intent {
        return Ok(task_media_refresh_ids_from_intent(refresh_intent));
    }

    legacy_task_media_refresh_ids(task)
}

fn task_media_refresh_ids_from_intent(
    refresh_intent: &DownloadTaskRefreshIntent,
) -> TaskMediaRefreshIds {
    let input = match &refresh_intent.input {
        DownloadTaskRefreshInput::VideoBvid { bvid } => ClassifiedInput::VideoBvid(bvid.clone()),
        DownloadTaskRefreshInput::VideoAid { aid } => ClassifiedInput::VideoAid(*aid),
        DownloadTaskRefreshInput::BangumiEpisode { ep_id } => ClassifiedInput::Bangumi {
            raw_url: format!("https://www.bilibili.com/bangumi/play/ep{ep_id}"),
        },
        DownloadTaskRefreshInput::CheeseEpisode { ep_id } => ClassifiedInput::Cheese {
            raw_url: format!("https://www.bilibili.com/cheese/play/ep{ep_id}"),
        },
    };

    TaskMediaRefreshIds {
        input,
        cid: refresh_intent.cid,
    }
}

fn legacy_task_media_refresh_ids(task: &DownloadTask) -> BdlResult<TaskMediaRefreshIds> {
    let part_segment = task
        .id
        .split_once(":part:")
        .map(|(_, part_segment)| part_segment)
        .ok_or_else(|| BdlError::Planning {
            message: format!("任务 `{}` 缺少可刷新媒体标识。", task.title),
        })?;

    if let Some(ids) = episode_task_media_refresh_ids(part_segment, "bangumi")? {
        return Ok(ids);
    }

    if let Some(ids) = episode_task_media_refresh_ids(part_segment, "cheese")? {
        return Ok(ids);
    }

    let mut parts = part_segment.split(':');
    let video_key = parts
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| BdlError::Planning {
            message: format!("任务 `{}` 缺少视频 ID，无法刷新下载地址。", task.title),
        })?;
    let cid = parts
        .next()
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| BdlError::Planning {
            message: format!("任务 `{}` 缺少 CID，无法刷新下载地址。", task.title),
        })?;
    let input = match classify_input(video_key)? {
        ClassifiedInput::VideoAid(aid) => ClassifiedInput::VideoAid(aid),
        ClassifiedInput::VideoBvid(bvid) => ClassifiedInput::VideoBvid(bvid),
        other => {
            return Err(BdlError::UnsupportedSource {
                kind: source_kind_name(other.source_kind()).to_owned(),
            });
        }
    };

    Ok(TaskMediaRefreshIds { input, cid })
}

fn episode_task_media_refresh_ids(
    part_segment: &str,
    kind: &'static str,
) -> BdlResult<Option<TaskMediaRefreshIds>> {
    let mut parts = part_segment.split(':');
    if parts.next() != Some(kind) {
        return Ok(None);
    }

    let _season_id = parse_positive_u64(parts.next()).ok_or_else(|| BdlError::Planning {
        message: format!("任务缺少 {kind} season ID，无法刷新下载地址。"),
    })?;
    let ep_id = parse_positive_u64(parts.next()).ok_or_else(|| BdlError::Planning {
        message: format!("任务缺少 {kind} ep ID，无法刷新下载地址。"),
    })?;
    let cid = parse_positive_u64(parts.next()).ok_or_else(|| BdlError::Planning {
        message: format!("任务缺少 {kind} CID，无法刷新下载地址。"),
    })?;
    let raw_url = format!("https://www.bilibili.com/{kind}/play/ep{ep_id}");
    let input = match kind {
        "bangumi" => ClassifiedInput::Bangumi { raw_url },
        "cheese" => ClassifiedInput::Cheese { raw_url },
        _ => {
            return Err(BdlError::UnsupportedSource {
                kind: kind.to_owned(),
            });
        }
    };

    Ok(Some(TaskMediaRefreshIds { input, cid }))
}

fn parse_positive_u64(value: Option<&str>) -> Option<u64> {
    value?.parse::<u64>().ok().filter(|value| *value > 0)
}

fn find_part_by_cid(tree: &NormalizedSourceTree, cid: u64) -> Option<&NormalizedPart> {
    tree.groups
        .iter()
        .flat_map(|group| &group.items)
        .flat_map(|item| &item.parts)
        .find(|part| part.cid == Some(cid))
}

fn select_task_stream<'a>(
    part: &'a NormalizedPart,
    kind: MediaKind,
    quality_label: &str,
    codec_label: Option<&str>,
) -> Option<&'a MediaStream> {
    let streams = part
        .streams
        .iter()
        .filter(|stream| stream.kind == kind)
        .collect::<Vec<_>>();
    if streams.is_empty() {
        return None;
    }

    let codec_matches = codec_label
        .filter(|label| !matches!(*label, "auto" | "none" | "unknown"))
        .map(|label| {
            streams
                .iter()
                .copied()
                .filter(|stream| stream_codec_label(stream) == label)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let candidates = if codec_matches.is_empty() {
        streams
    } else {
        codec_matches
    };

    if let Some(target_quality) = parse_quality_label(quality_label)
        && let Some(exact) = candidates
            .iter()
            .copied()
            .filter(|stream| stream_quality_rank(stream.quality) == target_quality)
            .max_by_key(|stream| stream_selection_rank(stream))
    {
        return Some(exact);
    }

    candidates
        .into_iter()
        .max_by_key(|stream| stream_selection_rank(stream))
}

fn parse_quality_label(value: &str) -> Option<u32> {
    value.parse::<u32>().ok().filter(|value| *value > 0)
}

fn stream_selection_rank(stream: &MediaStream) -> (u32, u64) {
    (
        stream_quality_rank(stream.quality),
        stream.bandwidth.unwrap_or_default(),
    )
}

fn stream_quality_rank(quality: StreamQuality) -> u32 {
    match quality {
        StreamQuality::Best => u32::MAX,
        StreamQuality::Quality(value) => value,
    }
}

fn stream_codec_label(stream: &MediaStream) -> &'static str {
    match stream.codec {
        StreamCodec::Auto => "auto",
        StreamCodec::Avc => "avc",
        StreamCodec::Hevc => "hevc",
        StreamCodec::Av1 => "av1",
        StreamCodec::Unknown => "unknown",
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
    use super::{
        AccountCookieCache, AppState, PartHydrationRequest, StartupRecoverySnapshot,
        append_new_tasks, append_source_page, dedupe_tasks_by_id, hydrate_placeholder_part,
        load_account_snapshot, next_page_request, normalize_selected_part_ids,
        prepare_startup_recovery, remap_selected_part_ids, select_startup_data_dir,
        select_task_stream, selected_hydration_requests, should_continue_loading,
        should_expand_initial_source, task_media_refresh_ids,
    };
    use crate::secure_store::SecureStore;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use bdl_core::account::AccountSummary;
    use bdl_core::ids::{GroupId, ItemId, PartId, SourceId};
    use bdl_core::input::ClassifiedInput;
    use bdl_core::model::{
        HeaderPair, MediaKind, MediaStream, NormalizedGroup, NormalizedItem, NormalizedPart,
        NormalizedSourceTree, PageState, SourceKind, SourceSummary, StreamCodec, StreamQuality,
    };
    use bdl_core::naming::DuplicateNamingStrategy;
    use bdl_core::queue::{
        DownloadResource, DownloadResourceIntent, DownloadResourceKind, DownloadTask,
        DownloadTaskMediaSelection, DownloadTaskRefreshInput, DownloadTaskRefreshIntent,
        DuplicateTaskPolicy, ResourceStatus, TaskStatus,
    };
    use bdl_core::resolver::paged::PageRequest;
    use bdl_core::settings::AppSettings;
    use bdl_core::storage::TaskStorage;
    use tokio::sync::Notify;

    #[test]
    fn startup_data_dir_uses_legacy_state_only_when_preferred_state_is_absent() {
        let root = temp_state_dir();
        let preferred = root.join("preferred");
        let legacy = root.join("legacy");
        std::fs::create_dir_all(&legacy).expect("legacy directory should be created");
        std::fs::write(legacy.join("tasks.sqlite"), []).expect("legacy marker should be written");

        assert_eq!(select_startup_data_dir(&preferred, &legacy), legacy);

        std::fs::create_dir_all(&preferred).expect("preferred directory should be created");
        std::fs::write(preferred.join("settings.json"), "{}")
            .expect("preferred marker should be written");
        assert_eq!(select_startup_data_dir(&preferred, &legacy), preferred);

        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn append_new_tasks_skips_existing_and_incoming_duplicate_ids() {
        let mut queue = vec![
            task_with_id("task:video:fixture:part:one"),
            task_with_id("task:video:fixture:part:one"),
        ];

        let removed_duplicates = dedupe_tasks_by_id(&mut queue);
        let inserted = append_new_tasks(
            &mut queue,
            vec![
                task_with_id("task:video:fixture:part:one"),
                task_with_id("task:video:fixture:part:two"),
                task_with_id("task:video:fixture:part:two"),
            ],
        );

        assert!(removed_duplicates);
        assert_eq!(
            queue_ids(&queue),
            ["task:video:fixture:part:one", "task:video:fixture:part:two"]
        );
        assert_eq!(queue_ids(&inserted), ["task:video:fixture:part:two"]);
    }

    #[test]
    fn resumed_task_waits_for_previous_attempt_to_exit() {
        let data_dir = temp_state_dir();
        let mut task = task_with_id("task:refresh");
        task.status = TaskStatus::Downloading;
        let state = test_state_with_tasks(&data_dir, vec![task]);
        let token = state.register_task_cancel_token("task:refresh").unwrap();
        state
            .update_task_status("task:refresh", TaskStatus::Paused)
            .unwrap();
        assert!(token.is_cancelled());
        state
            .update_task_status("task:refresh", TaskStatus::Waiting)
            .unwrap();
        assert!(state.take_next_startable_task().unwrap().is_none());
        assert!(
            state
                .finish_task_attempt("task:refresh", TaskStatus::Failed, &token)
                .unwrap()
                .is_none()
        );
        state.clear_task_cancel_token("task:refresh").unwrap();
        assert_eq!(
            state.take_next_startable_task().unwrap().unwrap().id,
            "task:refresh"
        );
        drop(state);
        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[test]
    fn active_attempt_can_retry_and_mux_failure_is_recorded() {
        let data_dir = temp_state_dir();
        let mut task = task_with_id("task:refresh");
        task.status = TaskStatus::Downloading;
        task.resources = vec![
            resource_with_status(ResourceStatus::Completed),
            resource_with_status(ResourceStatus::Failed),
        ];
        task.resources[1].id = "resource:audio".into();
        let state = test_state_with_tasks(&data_dir, vec![task]);
        let token = state.register_task_cancel_token("task:refresh").unwrap();
        let retried = state
            .finish_task_attempt("task:refresh", TaskStatus::Waiting, &token)
            .unwrap()
            .unwrap();
        assert_eq!(retried.status, TaskStatus::Waiting);
        assert_eq!(retried.resources[0].status, ResourceStatus::Completed);
        assert_eq!(retried.resources[1].status, ResourceStatus::Pending);
        state
            .update_task_status("task:refresh", TaskStatus::Muxing)
            .unwrap();
        token.cancel(); // Pause during muxing is ignored; a real mux failure still counts.
        let failed = state
            .finish_task_attempt("task:refresh", TaskStatus::Failed, &token)
            .unwrap()
            .unwrap();
        assert_eq!(failed.status, TaskStatus::Failed);
        drop(state);
        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[test]
    fn late_refresh_result_preserves_stopped_task() {
        for status in [TaskStatus::Paused, TaskStatus::Cancelled] {
            for next in [TaskStatus::Waiting, TaskStatus::Failed] {
                let data_dir = temp_state_dir();
                let mut task = task_with_id("task:refresh");
                task.status = TaskStatus::Downloading;
                let state = test_state_with_tasks(&data_dir, vec![task]);
                let token = state.register_task_cancel_token("task:refresh").unwrap();
                state.cancel_running_task("task:refresh").unwrap();
                state.update_task_status("task:refresh", status).unwrap();
                assert!(
                    state
                        .finish_task_attempt("task:refresh", next, &token)
                        .unwrap()
                        .is_none()
                );
                assert_eq!(state.task_status("task:refresh").unwrap(), status);
                drop(state);
                let _ = std::fs::remove_dir_all(data_dir);
            }
        }
    }

    #[test]
    fn enqueue_rechecks_newly_completed_output_using_naming_preference() {
        for strategy in [
            DuplicateNamingStrategy::SkipExisting,
            DuplicateNamingStrategy::AppendSuffix,
            DuplicateNamingStrategy::OverwriteExisting,
        ] {
            let data_dir = temp_state_dir();
            let state = test_state_with_tasks(&data_dir, Vec::new());
            let mut task = task_with_id("task:new");
            task.output_path = data_dir.join("same.mp4");
            let path = task.output_path.clone();
            // Planning already finished when an earlier task creates this file.
            std::fs::write(&path, b"existing video").unwrap();
            let result = state
                .enqueue_tasks_with_duplicate_policy(
                    vec![task],
                    DuplicateTaskPolicy::Skip,
                    strategy,
                )
                .unwrap();
            match strategy {
                DuplicateNamingStrategy::SkipExisting => {
                    assert!(result.inserted.is_empty());
                    assert_eq!(result.skipped_existing, 1);
                }
                DuplicateNamingStrategy::AppendSuffix => {
                    assert_ne!(result.inserted[0].output_path, path)
                }
                DuplicateNamingStrategy::OverwriteExisting => {
                    assert_eq!(result.inserted[0].output_path, path)
                }
            }
            assert_eq!(std::fs::read(&path).unwrap(), b"existing video");
            drop(state);
            let _ = std::fs::remove_dir_all(data_dir);
        }
    }

    #[test]
    fn separate_enqueue_calls_reserve_same_named_outputs_and_resources() {
        let data_dir = temp_state_dir();
        let state = test_state_with_tasks(&data_dir, Vec::new());
        let mut inserted = Vec::new();
        for id in ["task:first", "task:second"] {
            let mut task = task_with_id(id);
            task.status = TaskStatus::Waiting;
            task.output_path = data_dir.join("same.mp4");
            let mut resource = resource_with_status(ResourceStatus::Pending);
            resource.id = format!("{id}:resource:video");
            resource.target_path = data_dir.join("same.video.m4s");
            resource.temp_path = data_dir.join("same.video.m4s.bdlpart");
            task.resources.push(resource);
            inserted.extend(
                state
                    .enqueue_tasks_with_duplicate_policy(
                        vec![task],
                        DuplicateTaskPolicy::Skip,
                        DuplicateNamingStrategy::AppendSuffix,
                    )
                    .unwrap()
                    .inserted,
            );
        }
        assert_eq!(inserted.len(), 2);
        assert_ne!(inserted[0].output_path, inserted[1].output_path);
        assert_ne!(
            inserted[0].resources[0].target_path,
            inserted[1].resources[0].target_path
        );
        assert_ne!(
            inserted[0].resources[0].temp_path,
            inserted[1].resources[0].temp_path
        );
        assert_eq!(inserted[1].id, "task:second");
        assert_eq!(inserted[1].resources[0].id, "task:second:resource:video");
        drop(state);
        let persisted = TaskStorage::open(data_dir.join("tasks.sqlite"))
            .unwrap()
            .load_tasks()
            .unwrap();
        assert_eq!(persisted.len(), 2);
        assert_ne!(persisted[0].output_path, persisted[1].output_path);
        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[test]
    fn duplicate_policy_ask_returns_matches_without_partial_insertion() {
        let data_dir = temp_state_dir();
        let existing = task_with_id("task:source:part");
        let state = test_state_with_tasks(&data_dir, vec![existing]);

        let result = state
            .enqueue_tasks_with_duplicate_policy(
                vec![task_with_id("task:source:part")],
                DuplicateTaskPolicy::Ask,
                DuplicateNamingStrategy::SkipExisting,
            )
            .expect("duplicate preflight should succeed");

        assert!(result.requires_confirmation);
        assert!(result.inserted.is_empty());
        assert_eq!(result.duplicates[0].existing_task_id, "task:source:part");
        assert_eq!(state.queue_snapshot().unwrap().len(), 1);
        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[test]
    fn duplicate_policy_create_reserves_path_and_rekeys_task_resources_atomically() {
        let data_dir = temp_state_dir();
        let existing = task_with_id("task:source:part");
        let existing_copy = task_with_id("task:source:part:copy:1");
        let state = test_state_with_tasks(&data_dir, vec![existing, existing_copy]);
        let mut planned = task_with_id("task:source:part");
        planned.resources = vec![DownloadResource {
            id: "task:source:part:resource:video".to_owned(),
            kind: DownloadResourceKind::Video,
            intent: DownloadResourceIntent::Video,
            current_urls: vec!["https://example.invalid/video.m4s".to_owned()],
            headers: Vec::new(),
            target_path: PathBuf::from("downloads/fixture.video.m4s"),
            temp_path: PathBuf::from("downloads/fixture.video.m4s.bdlpart"),
            status: ResourceStatus::Pending,
        }];

        let result = state
            .enqueue_tasks_with_duplicate_policy(
                vec![planned],
                DuplicateTaskPolicy::Create,
                DuplicateNamingStrategy::AppendSuffix,
            )
            .expect("duplicate copy should be created");

        assert_eq!(result.inserted[0].id, "task:source:part:copy:2");
        assert!(result.inserted[0].output_path.ends_with("fixture (1).mp4"));
        assert_eq!(
            result.inserted[0].resources[0].id,
            "task:source:part:copy:2:resource:video"
        );
        assert!(
            result.inserted[0].resources[0]
                .target_path
                .ends_with("fixture (1).video.m4s")
        );
        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[test]
    fn prepare_startup_recovery_pauses_interrupted_tasks_by_default() {
        let mut interrupted = task_with_id("task:interrupted");
        interrupted.status = TaskStatus::Downloading;
        interrupted.resources = vec![resource_with_status(ResourceStatus::Downloading)];
        let mut completed = task_with_id("task:completed");
        completed.status = TaskStatus::Completed;
        let mut queue = vec![interrupted, completed];

        let snapshot = prepare_startup_recovery(&mut queue, false);

        assert_eq!(snapshot.task_ids, ["task:interrupted"]);
        assert!(!snapshot.auto_recovery_enabled);
        assert_eq!(queue[0].status, TaskStatus::Paused);
        assert_eq!(queue[0].resources[0].status, ResourceStatus::Pending);
        assert_eq!(queue[1].status, TaskStatus::Completed);
    }

    #[test]
    fn prepare_startup_recovery_queues_interrupted_tasks_when_auto_enabled() {
        let mut waiting = task_with_id("task:waiting");
        waiting.status = TaskStatus::Waiting;
        let mut paused = task_with_id("task:paused");
        paused.status = TaskStatus::Paused;
        let mut queue = vec![waiting, paused];

        let snapshot = prepare_startup_recovery(&mut queue, true);

        assert_eq!(snapshot.task_ids, ["task:waiting"]);
        assert!(snapshot.auto_recovery_enabled);
        assert_eq!(queue[0].status, TaskStatus::Waiting);
        assert_eq!(queue[1].status, TaskStatus::Paused);
    }

    #[test]
    fn prepare_startup_recovery_preserves_scheduled_waiting_tasks() {
        let mut scheduled = task_with_id("task:scheduled");
        scheduled.status = TaskStatus::Waiting;
        scheduled.scheduled_at = Some(Utc::now() + chrono::Duration::hours(1));
        let mut queue = vec![scheduled];

        let snapshot = prepare_startup_recovery(&mut queue, false);

        assert!(snapshot.task_ids.is_empty());
        assert_eq!(queue[0].status, TaskStatus::Waiting);
        assert!(queue[0].scheduled_at.is_some());
    }

    #[test]
    fn load_account_snapshot_uses_saved_summary_without_secure_store_access() {
        let data_dir = temp_state_dir();
        std::fs::create_dir_all(&data_dir).expect("temp data dir should be created");
        let mut storage =
            TaskStorage::open(data_dir.join("tasks.sqlite")).expect("storage should open");
        let expected = AccountSummary {
            logged_in: true,
            name: Some("BDL user".to_owned()),
            avatar_url: None,
            mid: Some("42".to_owned()),
            vip_label: None,
        };
        storage
            .save_account_summary(&expected)
            .expect("summary should save");

        let account = load_account_snapshot(&storage).expect("load account");

        assert_eq!(account, expected);
        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[test]
    fn parse_client_reuses_session_and_invalidates_after_account_change() {
        let data_dir = temp_state_dir();
        let state = test_state_with_tasks(&data_dir, Vec::new());
        let first = state.shared_resolver_client().unwrap();
        let second = state.shared_resolver_client().unwrap();
        assert!(std::sync::Arc::ptr_eq(&first, &second));
        state
            .account_session_revision
            .fetch_add(1, Ordering::SeqCst);
        let changed = state.shared_resolver_client().unwrap();
        assert!(!std::sync::Arc::ptr_eq(&first, &changed));
        drop(state);
        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[test]
    fn account_session_mutations_advance_the_library_revision() {
        let data_dir = temp_state_dir();
        let state = test_state_with_tasks(&data_dir, Vec::new());
        let initial = state.account_session_revision.load(Ordering::SeqCst);

        state
            .import_cookie("DedeUserID=42; SESSDATA=session; bili_jct=csrf")
            .expect("fixture account should import");
        let imported = state.account_session_revision.load(Ordering::SeqCst);
        state.logout().expect("fixture account should log out");
        let logged_out = state.account_session_revision.load(Ordering::SeqCst);

        assert!(imported > initial);
        assert!(logged_out > imported);
        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[test]
    fn parse_all_without_limit_continues_beyond_the_legacy_hundred_item_cap() {
        assert!(should_continue_loading(true, 100, None));
        assert!(should_continue_loading(true, 149, None));
        assert!(!should_continue_loading(false, 150, None));
        assert!(should_continue_loading(true, 99, Some(100)));
        assert!(!should_continue_loading(true, 100, Some(100)));
    }

    #[test]
    fn workspace_parse_keeps_large_uploader_sources_paged() {
        assert!(!should_expand_initial_source(SourceKind::Uploader, true));
    }

    #[test]
    fn workspace_parse_keeps_container_sources_paged() {
        assert!(!should_expand_initial_source(SourceKind::Favorite, true));
        assert!(!should_expand_initial_source(SourceKind::Collection, true));
        assert!(!should_expand_initial_source(SourceKind::Series, true));
    }

    #[test]
    fn persisted_queue_management_workflows_survive_reload() {
        let data_dir = temp_state_dir();
        let mut waiting = task_with_id("task:waiting");
        waiting.status = TaskStatus::Waiting;
        waiting.resources = vec![resource_with_intent_status(
            "task:waiting",
            DownloadResourceIntent::Video,
            ResourceStatus::Pending,
        )];

        let mut paused = task_with_id("task:paused");
        paused.status = TaskStatus::Paused;
        paused.resources = vec![resource_with_intent_status(
            "task:paused",
            DownloadResourceIntent::Video,
            ResourceStatus::Paused,
        )];

        let mut failed = task_with_id("task:failed");
        failed.status = TaskStatus::Failed;
        failed.resources = vec![
            resource_with_intent_status(
                "task:failed",
                DownloadResourceIntent::Video,
                ResourceStatus::Completed,
            ),
            resource_with_intent_status(
                "task:failed",
                DownloadResourceIntent::Audio,
                ResourceStatus::Failed,
            ),
        ];

        let mut refresh = task_with_id("task:refresh");
        refresh.status = TaskStatus::Failed;
        refresh.resources = vec![
            resource_with_intent_status(
                "task:refresh",
                DownloadResourceIntent::Video,
                ResourceStatus::Failed,
            ),
            resource_with_intent_status(
                "task:refresh",
                DownloadResourceIntent::Audio,
                ResourceStatus::Failed,
            ),
        ];

        let mut completed = task_with_id("task:completed");
        completed.status = TaskStatus::Completed;
        completed.resources = vec![resource_with_intent_status(
            "task:completed",
            DownloadResourceIntent::Video,
            ResourceStatus::Completed,
        )];

        let mut remove = task_with_id("task:remove");
        remove.status = TaskStatus::Paused;

        let mut muxing = task_with_id("task:muxing");
        muxing.status = TaskStatus::Muxing;

        let state = test_state_with_tasks(
            &data_dir,
            vec![waiting, paused, failed, refresh, completed, remove, muxing],
        );

        let paused_task = state
            .update_task_status("task:waiting", TaskStatus::Paused)
            .expect("waiting task should pause");
        assert_eq!(paused_task.status, TaskStatus::Paused);

        let muxing_task = state
            .update_task_status("task:muxing", TaskStatus::Paused)
            .expect("a committed mux should ignore pause");
        assert_eq!(muxing_task.status, TaskStatus::Muxing);

        let completed_task = state
            .update_task_status("task:completed", TaskStatus::Paused)
            .expect("a completed task should ignore pause");
        assert_eq!(completed_task.status, TaskStatus::Completed);

        let resumed_task = state
            .update_task_status("task:paused", TaskStatus::Waiting)
            .expect("paused task should resume to waiting");
        assert_eq!(resumed_task.status, TaskStatus::Waiting);
        assert_eq!(resumed_task.resources[0].status, ResourceStatus::Pending);

        let cancel_token = state
            .register_task_cancel_token("task:waiting")
            .expect("cancel token should register");
        assert!(!cancel_token.is_cancelled());
        assert!(
            state
                .cancel_running_task("task:waiting")
                .expect("running task should cancel")
        );
        assert!(cancel_token.is_cancelled());

        let retried_task = state
            .retry_task("task:failed")
            .expect("failed task should retry");
        assert_eq!(retried_task.status, TaskStatus::Waiting);
        assert_eq!(retried_task.resources[0].status, ResourceStatus::Completed);
        assert_eq!(retried_task.resources[1].status, ResourceStatus::Pending);

        let video = media_stream(MediaKind::Video, "https://cdn.example/new-video.m4s");
        let audio = media_stream(MediaKind::Audio, "https://cdn.example/new-audio.m4s");
        let refreshed_task = state
            .replace_task_media_urls("task:refresh", Some(&video), Some(&audio))
            .expect("refresh should replace media urls");
        assert_eq!(
            refreshed_task.resources[0].current_urls,
            ["https://cdn.example/new-video.m4s"]
        );
        assert_eq!(
            refreshed_task.resources[1].current_urls,
            ["https://cdn.example/new-audio.m4s"]
        );
        let refreshed_retry = state
            .retry_task("task:refresh")
            .expect("refreshed task should retry");
        assert_eq!(refreshed_retry.status, TaskStatus::Waiting);

        assert!(
            state
                .remove_task("task:remove")
                .expect("task should remove")
        );
        for task_id in state
            .queue_snapshot()
            .expect("queue snapshot should load")
            .into_iter()
            .filter(|task| task.status == TaskStatus::Completed)
            .map(|task| task.id)
            .collect::<Vec<_>>()
        {
            assert!(state.remove_task(&task_id).expect("completed task removed"));
        }

        drop(state);

        let reloaded = TaskStorage::open(data_dir.join("tasks.sqlite"))
            .expect("storage should reopen")
            .load_tasks()
            .expect("tasks should reload");
        assert!(reloaded.iter().all(|task| task.id != "task:remove"));
        assert!(
            reloaded
                .iter()
                .all(|task| task.status != TaskStatus::Completed)
        );

        let failed = task_by_id(&reloaded, "task:failed");
        assert_eq!(failed.status, TaskStatus::Waiting);
        assert_eq!(failed.resources[0].status, ResourceStatus::Completed);
        assert_eq!(failed.resources[1].status, ResourceStatus::Pending);

        let refresh = task_by_id(&reloaded, "task:refresh");
        assert_eq!(refresh.status, TaskStatus::Waiting);
        assert_eq!(
            refresh.resources[0].current_urls,
            ["https://cdn.example/new-video.m4s"]
        );
        assert_eq!(
            refresh.resources[1].current_urls,
            ["https://cdn.example/new-audio.m4s"]
        );

        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[test]
    fn queue_skips_future_scheduled_tasks_until_they_are_due() {
        let data_dir = temp_state_dir();
        let now = Utc::now();
        let mut scheduled = task_with_id("task:scheduled");
        scheduled.status = TaskStatus::Waiting;
        scheduled.scheduled_at = Some(now + chrono::Duration::minutes(5));
        let mut immediate = task_with_id("task:immediate");
        immediate.status = TaskStatus::Waiting;
        let state = test_state_with_tasks(&data_dir, vec![scheduled, immediate]);

        let started = state
            .take_next_startable_task_at(now)
            .expect("queue lookup should succeed")
            .expect("the immediate task should start");

        assert_eq!(started.id, "task:immediate");
        let due = state
            .take_next_startable_task_at(now + chrono::Duration::minutes(5))
            .expect("due queue lookup should succeed")
            .expect("scheduled task should fill another worker slot");
        assert_eq!(due.id, "task:scheduled");
        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[test]
    fn pausing_a_scheduled_task_clears_its_schedule() {
        let data_dir = temp_state_dir();
        let mut task = task_with_id("task:scheduled");
        task.status = TaskStatus::Waiting;
        task.scheduled_at = Some(Utc::now() + chrono::Duration::minutes(5));
        let state = test_state_with_tasks(&data_dir, vec![task]);

        let paused = state
            .update_task_status("task:scheduled", TaskStatus::Paused)
            .expect("scheduled task should pause");

        assert_eq!(paused.scheduled_at, None);
        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[test]
    fn clearing_a_schedule_makes_the_task_startable_immediately() {
        let data_dir = temp_state_dir();
        let mut task = task_with_id("task:scheduled");
        task.status = TaskStatus::Waiting;
        task.scheduled_at = Some(Utc::now() + chrono::Duration::minutes(5));
        let state = test_state_with_tasks(&data_dir, vec![task]);

        let updated = state
            .set_task_schedule("task:scheduled", None)
            .expect("schedule should clear");

        assert_eq!(updated.scheduled_at, None);
        assert!(updated.can_start_at(Utc::now()));
        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[test]
    fn paused_task_can_receive_a_persisted_speed_limit() {
        let data_dir = temp_state_dir();
        let mut task = task_with_id("task:limited");
        task.status = TaskStatus::Paused;
        let state = test_state_with_tasks(&data_dir, vec![task]);

        let updated = state
            .set_task_speed_limit("task:limited", Some(2 * 1024 * 1024))
            .expect("paused task should accept a speed limit");

        assert_eq!(updated.speed_limit_bytes_per_second, Some(2 * 1024 * 1024));
        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[test]
    fn downloading_task_rejects_a_speed_limit_change() {
        let data_dir = temp_state_dir();
        let mut task = task_with_id("task:active");
        task.status = TaskStatus::Downloading;
        let state = test_state_with_tasks(&data_dir, vec![task]);

        let error = state
            .set_task_speed_limit("task:active", Some(2 * 1024 * 1024))
            .expect_err("active task should not change speed limit in place");

        assert!(error.to_string().contains("暂停"));
        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[tokio::test]
    async fn settings_update_wakes_a_waiting_worker_and_exposes_the_latest_snapshot() {
        let data_dir = temp_state_dir();
        let state = test_state_with_tasks(&data_dir, Vec::new());
        let mut updated = state.settings().expect("settings should load");
        updated.concurrent_tasks = 3;
        updated.ffmpeg_path = Some("C:/tools/ffmpeg.exe".to_owned());

        state
            .update_settings(updated)
            .expect("settings update should succeed");
        tokio::time::timeout(
            std::time::Duration::from_millis(100),
            state.wait_for_queue_change(),
        )
        .await
        .expect("a sleeping queue worker should be notified");

        let latest = state.settings().expect("latest settings should load");
        assert_eq!(latest.concurrent_tasks, 3);
        assert_eq!(latest.ffmpeg_path.as_deref(), Some("C:/tools/ffmpeg.exe"));
        let _ = std::fs::remove_dir_all(data_dir);
    }

    #[test]
    fn selected_hydration_requests_returns_unique_selected_placeholders() {
        let tree = uploader_tree();
        let placeholder = PartId("part:uploader:1001:BV1xx411c7mD".to_owned());
        let requests = selected_hydration_requests(
            &tree,
            &[
                placeholder.clone(),
                placeholder.clone(),
                PartId("part:missing".to_owned()),
            ],
        )
        .expect("hydration requests should be valid");

        assert_eq!(
            requests,
            vec![PartHydrationRequest {
                part_id: placeholder,
                input: ClassifiedInput::VideoBvid("BV1xx411c7mD".to_owned()),
                target_cid: None,
            }]
        );
    }

    #[test]
    fn selected_hydration_requests_builds_episode_request_for_bangumi() {
        let tree = episode_tree(SourceKind::Bangumi, "bangumi");
        let placeholder = PartId("part:bangumi:123:456:789".to_owned());

        let requests = selected_hydration_requests(&tree, std::slice::from_ref(&placeholder))
            .expect("hydration requests should be valid");

        assert_eq!(
            requests,
            vec![PartHydrationRequest {
                part_id: placeholder,
                input: ClassifiedInput::Bangumi {
                    raw_url: "https://www.bilibili.com/bangumi/play/ep456".to_owned(),
                },
                target_cid: Some(789),
            }]
        );
    }

    #[test]
    fn hydrate_placeholder_part_expands_video_parts_and_preserves_list_metadata() {
        let mut tree = uploader_tree();
        let placeholder = PartId("part:uploader:1001:BV1xx411c7mD".to_owned());

        let hydrated_ids = hydrate_placeholder_part(&mut tree, &placeholder, None, video_tree())
            .expect("placeholder should hydrate");

        assert_eq!(
            hydrated_ids,
            vec![
                PartId("part:BV1xx411c7mD:62131".to_owned()),
                PartId("part:BV1xx411c7mD:62132".to_owned()),
            ]
        );

        let item = &tree.groups[0].items[0];
        assert_eq!(item.id.0, "item:uploader:1001:BV1xx411c7mD");
        assert_eq!(item.owner_name.as_deref(), Some("fixture owner"));
        assert_eq!(
            item.cover_url.as_deref(),
            Some("https://example.invalid/list-cover.jpg")
        );
        assert_eq!(item.parts.len(), 2);
        assert_eq!(item.parts[0].cid, Some(62131));
        assert_eq!(item.parts[1].cid, Some(62132));
    }

    #[test]
    fn hydrated_list_item_accepts_its_original_placeholder_selection() {
        let mut tree = uploader_tree();
        let placeholder = PartId("part:uploader:1001:BV1xx411c7mD".to_owned());
        let hydrated_ids = hydrate_placeholder_part(&mut tree, &placeholder, None, video_tree())
            .expect("placeholder should hydrate");

        let selected = normalize_selected_part_ids(&tree, std::slice::from_ref(&placeholder));

        assert_eq!(selected, hydrated_ids);
        assert!(
            selected
                .iter()
                .all(|part_id| super::find_part(&tree, part_id).is_some()),
            "the planner must only receive part IDs present in the hydrated tree"
        );
    }

    #[test]
    fn hydrate_known_part_keeps_selection_on_matching_cid() {
        let mut tree = uploader_tree();
        let placeholder = PartId("part:uploader:1001:BV1xx411c7mD".to_owned());

        let hydrated_ids =
            hydrate_placeholder_part(&mut tree, &placeholder, Some(62132), video_tree())
                .expect("placeholder should hydrate to requested cid");

        assert_eq!(
            hydrated_ids,
            vec![PartId("part:BV1xx411c7mD:62132".to_owned())]
        );
    }

    #[test]
    fn hydrate_known_part_preserves_streams_for_previously_hydrated_parts() {
        let mut tree = video_tree();
        tree.groups[0].items[0].parts[0].streams = vec![media_stream(
            MediaKind::Video,
            "https://cdn.example/p1-video.m4s",
        )];
        let selected_part = tree.groups[0].items[0].parts[1].id.clone();

        let mut hydrated = video_tree();
        hydrated.groups[0].items[0].parts[1].streams = vec![media_stream(
            MediaKind::Video,
            "https://cdn.example/p2-video.m4s",
        )];

        hydrate_placeholder_part(&mut tree, &selected_part, Some(62132), hydrated)
            .expect("known part should hydrate without replacing its siblings");

        let streams = tree.groups[0].items[0]
            .parts
            .iter()
            .map(|part| part.streams.first().map(|stream| stream.urls[0].as_str()))
            .collect::<Vec<_>>();
        assert_eq!(
            streams,
            vec![
                Some("https://cdn.example/p1-video.m4s"),
                Some("https://cdn.example/p2-video.m4s"),
            ]
        );
    }

    #[test]
    fn remap_selected_part_ids_expands_placeholder_to_hydrated_parts() {
        let placeholder = PartId("part:uploader:1001:BV1xx411c7mD".to_owned());
        let mut selected = vec![PartId("part:other".to_owned()), placeholder.clone()];

        remap_selected_part_ids(
            &mut selected,
            &placeholder,
            &[
                PartId("part:BV1xx411c7mD:62131".to_owned()),
                PartId("part:BV1xx411c7mD:62132".to_owned()),
            ],
        );

        assert_eq!(
            selected,
            vec![
                PartId("part:other".to_owned()),
                PartId("part:BV1xx411c7mD:62131".to_owned()),
                PartId("part:BV1xx411c7mD:62132".to_owned()),
            ]
        );
    }

    #[test]
    fn next_page_request_advances_from_current_page_state() {
        let request = next_page_request(&uploader_tree()).expect("next page should exist");

        assert_eq!(
            request,
            PageRequest {
                page_number: 2,
                page_size: 30,
            }
        );
    }

    #[test]
    fn append_source_page_merges_items_and_updates_cumulative_page_state() {
        let mut tree = uploader_tree();

        append_source_page(&mut tree, next_uploader_tree()).expect("page should append");

        assert_eq!(tree.source.loaded_count, 2);
        assert_eq!(tree.source.total_count, Some(45));
        assert!(tree.source.has_more);
        assert_eq!(tree.groups[0].items.len(), 2);

        let page = tree.groups[0]
            .page
            .as_ref()
            .expect("page state should exist");
        assert_eq!(page.page_number, 2);
        assert_eq!(page.page_size, 30);
        assert_eq!(page.loaded_count, 2);
        assert_eq!(page.total_count, Some(45));
        assert!(page.has_more);
    }

    #[test]
    fn append_source_page_stops_when_api_page_has_no_more() {
        let mut tree = uploader_tree();
        let mut next = next_uploader_tree();
        next.source.has_more = false;

        append_source_page(&mut tree, next).expect("page should append");

        assert!(!tree.source.has_more);
        assert!(
            !tree.groups[0]
                .page
                .as_ref()
                .expect("page state should exist")
                .has_more
        );
    }

    #[test]
    fn task_media_refresh_ids_uses_durable_refresh_intent_before_task_id() {
        let mut task = task_with_id("task:opaque");
        task.refresh_intent = Some(DownloadTaskRefreshIntent {
            input: DownloadTaskRefreshInput::VideoBvid {
                bvid: "BV1refresh11".to_owned(),
            },
            cid: 42,
            page_number: None,
        });

        let ids = task_media_refresh_ids(&task).expect("refresh intent should be used");

        assert_eq!(
            ids.input,
            ClassifiedInput::VideoBvid("BV1refresh11".to_owned())
        );
        assert_eq!(ids.cid, 42);
    }

    #[test]
    fn task_media_refresh_ids_extracts_bvid_and_cid_from_legacy_task_id() {
        let task = task_with_id("task:video:av333290567:part:BV1ZA411g7Sb:346910923");

        let ids = task_media_refresh_ids(&task).expect("task id should contain media ids");

        assert_eq!(
            ids.input,
            ClassifiedInput::VideoBvid("BV1ZA411g7Sb".to_owned())
        );
        assert_eq!(ids.cid, 346910923);
    }

    #[test]
    fn task_media_refresh_ids_extracts_aid_and_cid_from_legacy_task_id() {
        let task = task_with_id("task:video:av333290567:part:av333290567:346910923");

        let ids = task_media_refresh_ids(&task).expect("task id should contain media ids");

        assert_eq!(ids.input, ClassifiedInput::VideoAid(333290567));
        assert_eq!(ids.cid, 346910923);
    }

    #[test]
    fn task_media_refresh_ids_extracts_bangumi_episode_and_cid_from_legacy_task_id() {
        let task = task_with_id("task:bangumi:123:part:bangumi:123:456:789");

        let ids = task_media_refresh_ids(&task).expect("task id should contain media ids");

        assert_eq!(
            ids.input,
            ClassifiedInput::Bangumi {
                raw_url: "https://www.bilibili.com/bangumi/play/ep456".to_owned(),
            }
        );
        assert_eq!(ids.cid, 789);
    }

    #[test]
    fn task_media_refresh_ids_extracts_cheese_episode_and_cid_from_legacy_task_id() {
        let task = task_with_id("task:cheese:123:part:cheese:123:456:789");

        let ids = task_media_refresh_ids(&task).expect("task id should contain media ids");

        assert_eq!(
            ids.input,
            ClassifiedInput::Cheese {
                raw_url: "https://www.bilibili.com/cheese/play/ep456".to_owned(),
            }
        );
        assert_eq!(ids.cid, 789);
    }

    #[test]
    fn select_task_stream_prefers_saved_quality_and_codec() {
        let mut part = video_tree().groups[0].items[0].parts[0].clone();
        part.streams = vec![
            profiled_media_stream(
                MediaKind::Video,
                StreamQuality::Quality(80),
                StreamCodec::Avc,
                2_000_000,
                "https://cdn.example/video-80-avc.m4s",
            ),
            profiled_media_stream(
                MediaKind::Video,
                StreamQuality::Quality(64),
                StreamCodec::Hevc,
                1_800_000,
                "https://cdn.example/video-64-hevc.m4s",
            ),
            profiled_media_stream(
                MediaKind::Video,
                StreamQuality::Quality(64),
                StreamCodec::Avc,
                1_500_000,
                "https://cdn.example/video-64-avc.m4s",
            ),
        ];

        let selected = select_task_stream(&part, MediaKind::Video, "64", Some("hevc"))
            .expect("matching stream should be selected");

        assert_eq!(selected.urls, ["https://cdn.example/video-64-hevc.m4s"]);
    }

    fn task_with_id(id: &str) -> DownloadTask {
        DownloadTask {
            id: id.to_owned(),
            title: "fixture task".to_owned(),
            source_id: "video:fixture".to_owned(),
            status: TaskStatus::Failed,
            resources: Vec::new(),
            output_path: PathBuf::from("downloads/fixture.mp4"),
            refresh_intent: None,
            media_selection: DownloadTaskMediaSelection::default(),
            scheduled_at: None,
            speed_limit_bytes_per_second: None,
        }
    }

    fn resource_with_status(status: ResourceStatus) -> DownloadResource {
        DownloadResource {
            id: "resource:video".to_owned(),
            kind: DownloadResourceKind::Video,
            intent: DownloadResourceIntent::Video,
            current_urls: vec!["https://example.invalid/video.m4s".to_owned()],
            headers: Vec::new(),
            target_path: PathBuf::from("downloads/video.m4s"),
            temp_path: PathBuf::from("downloads/video.m4s.bdlpart"),
            status,
        }
    }

    fn resource_with_intent_status(
        task_id: &str,
        intent: DownloadResourceIntent,
        status: ResourceStatus,
    ) -> DownloadResource {
        let suffix = format!("{intent:?}").to_ascii_lowercase();
        let task_slug = task_id.replace(':', "-");
        DownloadResource {
            id: format!("{task_id}:resource:{suffix}"),
            kind: match intent {
                DownloadResourceIntent::Video => DownloadResourceKind::Video,
                DownloadResourceIntent::Audio => DownloadResourceKind::Audio,
                DownloadResourceIntent::Cover
                | DownloadResourceIntent::Subtitle
                | DownloadResourceIntent::Danmaku
                | DownloadResourceIntent::Nfo => DownloadResourceKind::Asset,
            },
            intent,
            current_urls: vec![format!("https://cdn.example/old-{suffix}.m4s")],
            headers: Vec::new(),
            target_path: PathBuf::from(format!("downloads/{task_slug}-{suffix}.m4s")),
            temp_path: PathBuf::from(format!("downloads/{task_slug}-{suffix}.m4s.bdlpart")),
            status,
        }
    }

    fn media_stream(kind: MediaKind, url: &str) -> MediaStream {
        profiled_media_stream(kind, StreamQuality::Best, StreamCodec::Avc, 1_000_000, url)
    }

    fn profiled_media_stream(
        kind: MediaKind,
        quality: StreamQuality,
        codec: StreamCodec,
        bandwidth: u64,
        url: &str,
    ) -> MediaStream {
        MediaStream {
            id: format!("stream:{kind:?}").to_ascii_lowercase(),
            kind,
            quality,
            codec,
            bandwidth: Some(bandwidth),
            urls: vec![url.to_owned()],
            headers: vec![HeaderPair {
                name: "Referer".to_owned(),
                value: "https://www.bilibili.com".to_owned(),
            }],
            acquired_at: Utc::now(),
        }
    }

    fn test_state_with_tasks(data_dir: &std::path::Path, tasks: Vec<DownloadTask>) -> AppState {
        std::fs::create_dir_all(data_dir).expect("temp data dir should be created");
        let mut storage =
            TaskStorage::open(data_dir.join("tasks.sqlite")).expect("storage should open");
        storage
            .replace_tasks(&tasks)
            .expect("tasks should persist before state is created");

        AppState {
            parse_pacer: Default::default(),
            parse_control: Default::default(),
            resolver_client: Default::default(),
            parse_sources: Mutex::new(HashMap::new()),
            queue: Mutex::new(tasks),
            storage: Mutex::new(storage),
            settings: Mutex::new(AppSettings::default()),
            settings_path: data_dir.join("settings.json"),
            data_dir: data_dir.to_path_buf(),
            account: Mutex::new(AccountSummary::default()),
            account_cookie: Mutex::new(AccountCookieCache::Loaded(None)),
            account_session_revision: AtomicU64::new(0),
            queue_worker_active: AtomicBool::new(false),
            queue_changed: Notify::new(),
            queue_cancellations: Mutex::new(HashMap::new()),
            startup_recovery: Mutex::new(StartupRecoverySnapshot::default()),
            secure_store: SecureStore::in_memory(),
        }
    }

    fn task_by_id<'a>(tasks: &'a [DownloadTask], task_id: &str) -> &'a DownloadTask {
        tasks
            .iter()
            .find(|task| task.id == task_id)
            .expect("task should exist")
    }

    fn queue_ids(tasks: &[DownloadTask]) -> Vec<&str> {
        tasks.iter().map(|task| task.id.as_str()).collect()
    }

    fn temp_state_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();

        std::env::temp_dir().join(format!("bdl-state-{nanos}"))
    }

    fn uploader_tree() -> NormalizedSourceTree {
        NormalizedSourceTree {
            source: SourceSummary {
                id: SourceId("uploader:1001:videos".to_owned()),
                kind: SourceKind::Uploader,
                input: "https://space.bilibili.com/1001/video".to_owned(),
                title: "fixture owner 的投稿".to_owned(),
                loaded_count: 1,
                total_count: Some(1),
                has_more: false,
            },
            groups: vec![NormalizedGroup {
                id: GroupId("group:uploader:1001:videos".to_owned()),
                kind: "uploader_videos".to_owned(),
                title: "fixture owner 的投稿".to_owned(),
                items: vec![NormalizedItem {
                    id: ItemId("item:uploader:1001:BV1xx411c7mD".to_owned()),
                    title: "fixture upload".to_owned(),
                    owner_name: Some("fixture owner".to_owned()),
                    owner_mid: Some(1001),
                    cover_url: Some("https://example.invalid/list-cover.jpg".to_owned()),
                    duration_seconds: Some(62),
                    parts: vec![NormalizedPart {
                        id: PartId("part:uploader:1001:BV1xx411c7mD".to_owned()),
                        title: "fixture upload".to_owned(),
                        aid: Some(170001),
                        bvid: Some("BV1xx411c7mD".to_owned()),
                        cid: None,
                        duration_seconds: Some(62),
                        streams: Vec::new(),
                        assets: Vec::new(),
                    }],
                }],
                page: Some(PageState {
                    page_number: 1,
                    page_size: 30,
                    loaded_count: 1,
                    total_count: Some(45),
                    has_more: true,
                }),
            }],
        }
    }

    fn episode_tree(kind: SourceKind, label: &str) -> NormalizedSourceTree {
        NormalizedSourceTree {
            source: SourceSummary {
                id: SourceId(format!("{label}:123")),
                kind,
                input: format!("https://www.bilibili.com/{label}/play/ss123"),
                title: "fixture source".to_owned(),
                loaded_count: 1,
                total_count: Some(1),
                has_more: false,
            },
            groups: vec![NormalizedGroup {
                id: GroupId(format!("group:{label}:123")),
                kind: label.to_owned(),
                title: "fixture source".to_owned(),
                items: vec![NormalizedItem {
                    id: ItemId(format!("item:{label}:123:456")),
                    title: "fixture episode".to_owned(),
                    owner_name: None,
                    owner_mid: None,
                    cover_url: None,
                    duration_seconds: Some(62),
                    parts: vec![NormalizedPart {
                        id: PartId(format!("part:{label}:123:456:789")),
                        title: "fixture episode".to_owned(),
                        aid: Some(170001),
                        bvid: None,
                        cid: Some(789),
                        duration_seconds: Some(62),
                        streams: Vec::new(),
                        assets: Vec::new(),
                    }],
                }],
                page: None,
            }],
        }
    }

    fn next_uploader_tree() -> NormalizedSourceTree {
        NormalizedSourceTree {
            source: SourceSummary {
                id: SourceId("uploader:1001:videos".to_owned()),
                kind: SourceKind::Uploader,
                input: "https://space.bilibili.com/1001/video".to_owned(),
                title: "fixture owner 的投稿".to_owned(),
                loaded_count: 1,
                total_count: Some(45),
                has_more: true,
            },
            groups: vec![NormalizedGroup {
                id: GroupId("group:uploader:1001:videos".to_owned()),
                kind: "uploader_videos".to_owned(),
                title: "fixture owner 的投稿".to_owned(),
                items: vec![NormalizedItem {
                    id: ItemId("item:uploader:1001:BV1yy411c7mD".to_owned()),
                    title: "fixture upload page 2".to_owned(),
                    owner_name: Some("fixture owner".to_owned()),
                    owner_mid: Some(1001),
                    cover_url: None,
                    duration_seconds: None,
                    parts: vec![NormalizedPart {
                        id: PartId("part:uploader:1001:BV1yy411c7mD".to_owned()),
                        title: "fixture upload page 2".to_owned(),
                        aid: Some(170002),
                        bvid: Some("BV1yy411c7mD".to_owned()),
                        cid: None,
                        duration_seconds: None,
                        streams: Vec::new(),
                        assets: Vec::new(),
                    }],
                }],
                page: Some(PageState {
                    page_number: 2,
                    page_size: 30,
                    loaded_count: 1,
                    total_count: Some(45),
                    has_more: true,
                }),
            }],
        }
    }

    fn video_tree() -> NormalizedSourceTree {
        NormalizedSourceTree {
            source: SourceSummary {
                id: SourceId("video:BV1xx411c7mD".to_owned()),
                kind: SourceKind::Video,
                input: "BV1xx411c7mD".to_owned(),
                title: "fixture upload".to_owned(),
                loaded_count: 1,
                total_count: Some(1),
                has_more: false,
            },
            groups: vec![NormalizedGroup {
                id: GroupId("group:BV1xx411c7mD".to_owned()),
                kind: "video".to_owned(),
                title: "fixture upload".to_owned(),
                items: vec![NormalizedItem {
                    id: ItemId("item:BV1xx411c7mD".to_owned()),
                    title: "fixture upload".to_owned(),
                    owner_name: None,
                    owner_mid: None,
                    cover_url: None,
                    duration_seconds: None,
                    parts: vec![
                        NormalizedPart {
                            id: PartId("part:BV1xx411c7mD:62131".to_owned()),
                            title: "P1".to_owned(),
                            aid: Some(170001),
                            bvid: Some("BV1xx411c7mD".to_owned()),
                            cid: Some(62131),
                            duration_seconds: Some(30),
                            streams: Vec::new(),
                            assets: Vec::new(),
                        },
                        NormalizedPart {
                            id: PartId("part:BV1xx411c7mD:62132".to_owned()),
                            title: "P2".to_owned(),
                            aid: Some(170001),
                            bvid: Some("BV1xx411c7mD".to_owned()),
                            cid: Some(62132),
                            duration_seconds: Some(32),
                            streams: Vec::new(),
                            assets: Vec::new(),
                        },
                    ],
                }],
                page: None,
            }],
        }
    }
}
