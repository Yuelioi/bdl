use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use bdl_core::account::{AccountSummary, ImportedCookie, verify_cookie_session};
use bdl_core::fetcher::FetchCancelToken;
use bdl_core::ids::{PartId, SourceId};
use bdl_core::input::{ClassifiedInput, classify_input};
use bdl_core::model::{
    MediaKind, MediaStream, NormalizedItem, NormalizedPart, NormalizedSourceTree, PageState,
    SourceKind, StreamCodec, StreamQuality,
};
use bdl_core::queue::{
    DownloadResourceIntent, DownloadTask, DownloadTaskRefreshInput, DownloadTaskRefreshIntent,
    QueueLogEntry, ResourceStatus, TaskStatus,
};
use bdl_core::resolver::bangumi::BangumiResolver;
use bdl_core::resolver::cheese::CheeseResolver;
use bdl_core::resolver::collection::{
    CollectionInputIds, CollectionResolver, SeriesInputIds, SeriesResolver,
};
use bdl_core::resolver::favorite::FavoriteResolver;
use bdl_core::resolver::paged::PageRequest;
use bdl_core::resolver::uploader::UploaderResolver;
use bdl_core::resolver::video::VideoResolver;
use bdl_core::resolver::{ResolveOptions, Resolver};
use bdl_core::settings::AppSettings;
use bdl_core::storage::TaskStorage;
use bdl_core::{BdlError, BdlResult};
use serde::Serialize;

use crate::secure_store::SecureStore;

const DEFAULT_PARSE_ALL_LIMIT: usize = 100;
const MAX_PARSE_ALL_LIMIT: usize = 100;

pub struct AppState {
    parse_sources: Mutex<HashMap<SourceId, NormalizedSourceTree>>,
    queue: Mutex<Vec<DownloadTask>>,
    storage: Mutex<TaskStorage>,
    settings: Mutex<SettingsSnapshot>,
    settings_path: PathBuf,
    data_dir: PathBuf,
    account: Mutex<AccountSnapshot>,
    account_cookie: Mutex<Option<String>>,
    queue_worker_active: AtomicBool,
    queue_cancellations: Mutex<HashMap<String, FetchCancelToken>>,
    startup_recovery: Mutex<StartupRecoverySnapshot>,
    secure_store: SecureStore,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedSelection {
    pub tree: NormalizedSourceTree,
    pub part_ids: Vec<PartId>,
    pub tree_updated: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct StartupRecoverySnapshot {
    pub task_ids: Vec<String>,
    pub auto_recovery_enabled: bool,
}

impl AppState {
    pub fn new() -> BdlResult<Self> {
        let settings_path = default_settings_path()?;
        let settings = load_settings(&settings_path)?;
        let data_dir = settings
            .data_dir
            .as_deref()
            .map(PathBuf::from)
            .unwrap_or(default_data_dir()?);
        let mut storage = TaskStorage::open(data_dir.join("tasks.sqlite"))?;
        let mut queue = storage.load_tasks()?;
        let startup_recovery = prepare_startup_recovery(&mut queue, settings.startup_auto_recovery);
        if !startup_recovery.task_ids.is_empty() {
            storage.replace_tasks(&queue)?;
        }
        let secure_store = SecureStore::new();
        let (account, account_cookie) =
            load_account_snapshot(&mut storage, &secure_store, &data_dir)?;

        Ok(Self {
            parse_sources: Mutex::new(HashMap::new()),
            queue: Mutex::new(queue),
            storage: Mutex::new(storage),
            settings: Mutex::new(settings),
            settings_path,
            data_dir,
            account: Mutex::new(account),
            account_cookie: Mutex::new(account_cookie),
            queue_worker_active: AtomicBool::new(false),
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
        let classified = classify_input(input)?;
        let options = ResolveOptions { fetch_streams };
        let tree = match classified.source_kind() {
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
            SourceKind::Series => self.series_resolver()?.resolve(classified, options).await?,
            SourceKind::Bangumi => {
                self.bangumi_resolver()?
                    .resolve(classified, options)
                    .await?
            }
            SourceKind::Cheese => self.cheese_resolver()?.resolve(classified, options).await?,
            _ => self.video_resolver()?.resolve(classified, options).await?,
        };
        self.parse_sources
            .lock()
            .map_err(|_| state_poisoned("parse_sources"))?
            .insert(tree.source.id.clone(), tree.clone());
        Ok(tree)
    }

    pub fn close_source(&self, source_id: &SourceId) -> BdlResult<bool> {
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
        self.parse_source(&current.source.input, true).await
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
        let limit = limit
            .unwrap_or(DEFAULT_PARSE_ALL_LIMIT)
            .clamp(1, MAX_PARSE_ALL_LIMIT);

        while tree.source.has_more && tree.source.loaded_count < limit {
            let loaded_before = tree.source.loaded_count;
            self.append_next_page(&mut tree).await?;
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
        let requests = selected_hydration_requests(&tree, selected_part_ids)?;
        if requests.is_empty() {
            return Ok(PreparedSelection {
                tree,
                part_ids: selected_part_ids.to_vec(),
                tree_updated: false,
            });
        }

        let mut part_ids = selected_part_ids.to_vec();

        for request in requests {
            let hydrated = self
                .resolve_media_input_with_streams(request.input.clone())
                .await?;
            let hydrated_part_ids = hydrate_placeholder_part(
                &mut tree,
                &request.part_id,
                request.target_cid,
                hydrated,
            )?;
            remap_selected_part_ids(&mut part_ids, &request.part_id, &hydrated_part_ids);
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

    pub fn enqueue_tasks(&self, tasks: Vec<DownloadTask>) -> BdlResult<Vec<DownloadTask>> {
        let mut queue = self.queue.lock().map_err(|_| state_poisoned("queue"))?;
        let removed_existing_duplicates = dedupe_tasks_by_id(&mut queue);
        let inserted = append_new_tasks(&mut queue, tasks);
        if removed_existing_duplicates || !inserted.is_empty() {
            self.persist_queue(&queue)?;
        }
        Ok(inserted)
    }

    pub fn try_start_queue_worker(&self) -> bool {
        !self.queue_worker_active.swap(true, Ordering::SeqCst)
    }

    pub fn finish_queue_worker(&self) {
        self.queue_worker_active.store(false, Ordering::SeqCst);
    }

    pub fn take_next_startable_task(&self) -> BdlResult<Option<DownloadTask>> {
        let mut queue = self.queue.lock().map_err(|_| state_poisoned("queue"))?;
        let Some(task_index) = queue.iter().position(|task| task.status.can_start()) else {
            return Ok(None);
        };

        queue[task_index].status = TaskStatus::Downloading;
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
            .any(|task| task.status.can_start()))
    }

    pub fn update_task_status(&self, task_id: &str, status: TaskStatus) -> BdlResult<DownloadTask> {
        let mut queue = self.queue.lock().map_err(|_| state_poisoned("queue"))?;
        let task_index = queue
            .iter()
            .position(|task| task.id == task_id)
            .ok_or_else(|| BdlError::Planning {
                message: format!("任务 `{task_id}` 不存在。"),
            })?;

        queue[task_index].status = status;
        if status.can_start() {
            reset_interrupted_resources(&mut queue[task_index].resources);
        }
        let task = queue[task_index].clone();
        self.persist_queue(&queue)?;
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

        let should_redownload_all = queue[task_index].status == TaskStatus::Completed;
        queue[task_index].status = TaskStatus::Waiting;
        for resource in &mut queue[task_index].resources {
            if should_redownload_all || resource.status != ResourceStatus::Completed {
                resource.status = ResourceStatus::Pending;
            }
        }

        let task = queue[task_index].clone();
        self.persist_queue(&queue)?;
        Ok(task)
    }

    pub async fn refresh_task_media_urls(&self, task_id: &str) -> BdlResult<DownloadTask> {
        let task = self.task_snapshot(task_id)?;
        let refresh_ids = task_media_refresh_ids(&task)?;
        let refreshed = self
            .resolve_media_input_with_streams(refresh_ids.input)
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
            Some(imported_cookie.as_header().to_owned());
        *self.account.lock().map_err(|_| state_poisoned("account"))? = account.clone();

        Ok(account)
    }

    pub fn logout(&self) -> BdlResult<AccountSnapshot> {
        self.clear_account_state()
    }

    pub async fn verify_account(&self) -> BdlResult<AccountSnapshot> {
        let Some(raw_cookie) = self
            .account_cookie
            .lock()
            .map_err(|_| state_poisoned("account_cookie"))?
            .clone()
        else {
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

    fn clear_account_state(&self) -> BdlResult<AccountSnapshot> {
        self.secure_store.clear_cookie()?;
        self.storage
            .lock()
            .map_err(|_| state_poisoned("storage"))?
            .clear_account_summary()?;
        *self
            .account_cookie
            .lock()
            .map_err(|_| state_poisoned("account_cookie"))? = None;
        *self.account.lock().map_err(|_| state_poisoned("account"))? = AccountSnapshot::default();

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
    ) -> BdlResult<NormalizedSourceTree> {
        let options = ResolveOptions {
            fetch_streams: true,
        };

        match input.source_kind() {
            SourceKind::Video => self.video_resolver()?.resolve(input, options).await,
            SourceKind::Bangumi => self.bangumi_resolver()?.resolve(input, options).await,
            SourceKind::Cheese => self.cheese_resolver()?.resolve(input, options).await,
            kind => Err(BdlError::UnsupportedSource {
                kind: source_kind_name(kind).to_owned(),
            }),
        }
    }

    fn video_resolver(&self) -> BdlResult<VideoResolver> {
        match self
            .account_cookie
            .lock()
            .map_err(|_| state_poisoned("account_cookie"))?
            .clone()
        {
            Some(cookie) => VideoResolver::from_cookie(&cookie),
            None => VideoResolver::new(),
        }
    }

    fn uploader_resolver(&self) -> BdlResult<UploaderResolver> {
        match self
            .account_cookie
            .lock()
            .map_err(|_| state_poisoned("account_cookie"))?
            .clone()
        {
            Some(cookie) => UploaderResolver::from_cookie(&cookie),
            None => UploaderResolver::new(),
        }
    }

    fn favorite_resolver(&self) -> BdlResult<FavoriteResolver> {
        match self
            .account_cookie
            .lock()
            .map_err(|_| state_poisoned("account_cookie"))?
            .clone()
        {
            Some(cookie) => FavoriteResolver::from_cookie(&cookie),
            None => FavoriteResolver::new(),
        }
    }

    fn collection_resolver(&self) -> BdlResult<CollectionResolver> {
        match self
            .account_cookie
            .lock()
            .map_err(|_| state_poisoned("account_cookie"))?
            .clone()
        {
            Some(cookie) => CollectionResolver::from_cookie(&cookie),
            None => CollectionResolver::new(),
        }
    }

    fn series_resolver(&self) -> BdlResult<SeriesResolver> {
        match self
            .account_cookie
            .lock()
            .map_err(|_| state_poisoned("account_cookie"))?
            .clone()
        {
            Some(cookie) => SeriesResolver::from_cookie(&cookie),
            None => SeriesResolver::new(),
        }
    }

    fn bangumi_resolver(&self) -> BdlResult<BangumiResolver> {
        match self
            .account_cookie
            .lock()
            .map_err(|_| state_poisoned("account_cookie"))?
            .clone()
        {
            Some(cookie) => BangumiResolver::from_cookie(&cookie),
            None => BangumiResolver::new(),
        }
    }

    fn cheese_resolver(&self) -> BdlResult<CheeseResolver> {
        match self
            .account_cookie
            .lock()
            .map_err(|_| state_poisoned("account_cookie"))?
            .clone()
        {
            Some(cookie) => CheeseResolver::from_cookie(&cookie),
            None => CheeseResolver::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PartHydrationRequest {
    part_id: PartId,
    input: ClassifiedInput,
    target_cid: Option<u64>,
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

fn default_data_dir() -> BdlResult<PathBuf> {
    Ok(std::env::current_dir()?.join(".bdl"))
}

fn default_settings_path() -> BdlResult<PathBuf> {
    Ok(std::env::current_dir()?.join(".bdl").join("settings.json"))
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

fn load_account_snapshot(
    storage: &mut TaskStorage,
    secure_store: &SecureStore,
    data_dir: &std::path::Path,
) -> BdlResult<(AccountSnapshot, Option<String>)> {
    let legacy_cookie_path = data_dir.join("account.cookie");
    let persisted_cookie = match secure_store.load_cookie()? {
        Some(cookie) => {
            SecureStore::clear_legacy_cookie_file(&legacy_cookie_path)?;
            Some(cookie)
        }
        None => secure_store.migrate_legacy_cookie_file(&legacy_cookie_path)?,
    };

    let Some(raw_cookie) = persisted_cookie else {
        storage.clear_account_summary()?;
        return Ok((AccountSnapshot::default(), None));
    };

    match ImportedCookie::parse(&raw_cookie) {
        Ok(imported_cookie) => {
            let account = stored_or_imported_account(storage, &imported_cookie)?;
            storage.save_account_summary(&account)?;
            Ok((account, Some(raw_cookie)))
        }
        Err(_) => {
            secure_store.clear_cookie()?;
            storage.clear_account_summary()?;
            Ok((AccountSnapshot::default(), None))
        }
    }
}

fn stored_or_imported_account(
    storage: &TaskStorage,
    imported_cookie: &ImportedCookie,
) -> BdlResult<AccountSnapshot> {
    let mut account = storage
        .load_account_summary()?
        .filter(|account| account.logged_in)
        .unwrap_or_else(|| AccountSummary::from_imported_cookie(imported_cookie));

    account.logged_in = true;
    if account.mid.is_none() {
        account.mid = imported_cookie.dede_user_id().map(str::to_owned);
    }

    Ok(account)
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

    if let Some(target_quality) = parse_quality_label(quality_label) {
        if let Some(exact) = candidates
            .iter()
            .copied()
            .filter(|stream| stream_quality_rank(stream.quality) == target_quality)
            .max_by_key(|stream| stream_selection_rank(stream))
        {
            return Some(exact);
        }
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

fn uploader_mid(tree: &NormalizedSourceTree) -> BdlResult<u64> {
    match classify_input(&tree.source.input)? {
        ClassifiedInput::Uploader { mid } => Ok(mid),
        other => Err(BdlError::UnsupportedSource {
            kind: source_kind_name(other.source_kind()).to_owned(),
        }),
    }
}

fn favorite_media_id(tree: &NormalizedSourceTree) -> BdlResult<u64> {
    tree.source
        .id
        .0
        .strip_prefix("favorite:")
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| BdlError::Planning {
            message: format!("来源 `{}` 缺少收藏夹 ID。", tree.source.id.0),
        })
}

fn collection_ids(tree: &NormalizedSourceTree) -> BdlResult<CollectionInputIds> {
    let (mid, season_id) = parse_two_part_source_id(&tree.source.id, "collection")?;
    Ok(CollectionInputIds { mid, season_id })
}

fn series_ids(tree: &NormalizedSourceTree) -> BdlResult<SeriesInputIds> {
    let (mid, series_id) = parse_two_part_source_id(&tree.source.id, "series")?;
    Ok(SeriesInputIds {
        mid: Some(mid),
        series_id,
    })
}

fn cheese_season_id(tree: &NormalizedSourceTree) -> BdlResult<u64> {
    tree.source
        .id
        .0
        .strip_prefix("cheese:")
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| BdlError::Planning {
            message: format!("来源 `{}` 缺少课程 season ID。", tree.source.id.0),
        })
}

fn parse_two_part_source_id(source_id: &SourceId, prefix: &str) -> BdlResult<(u64, u64)> {
    let mut parts = source_id
        .0
        .strip_prefix(prefix)
        .and_then(|value| value.strip_prefix(':'))
        .into_iter()
        .flat_map(|value| value.split(':'));
    let first = parts
        .next()
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| BdlError::Planning {
            message: format!("来源 `{}` 缺少 {prefix} mid。", source_id.0),
        })?;
    let second = parts
        .next()
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| BdlError::Planning {
            message: format!("来源 `{}` 缺少 {prefix} ID。", source_id.0),
        })?;

    Ok((first, second))
}

fn next_page_request(tree: &NormalizedSourceTree) -> BdlResult<PageRequest> {
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

fn append_source_page(
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
    let page_state = PageState {
        page_number: next_page_state.page_number,
        page_size: next_page_state.page_size,
        loaded_count: group.items.len(),
        total_count,
        has_more,
    };

    group.page = Some(page_state);
    existing.source.loaded_count = existing.groups.iter().map(|group| group.items.len()).sum();
    existing.source.total_count = total_count;
    existing.source.has_more = has_more;

    Ok(())
}

fn selected_hydration_requests(
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

        if !part_needs_hydration(part) {
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

fn find_part<'a>(tree: &'a NormalizedSourceTree, part_id: &PartId) -> Option<&'a NormalizedPart> {
    tree.groups
        .iter()
        .flat_map(|group| &group.items)
        .flat_map(|item| &item.parts)
        .find(|part| &part.id == part_id)
}

fn part_needs_hydration(part: &NormalizedPart) -> bool {
    part.cid.is_none() || part.streams.is_empty()
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
        _ => {
            let bvid = part.bvid.clone().ok_or_else(|| BdlError::Planning {
                message: format!("选中的分 P `{}` 缺少 BV ID，无法补齐下载流。", part_id.0),
            })?;
            Ok(ClassifiedInput::VideoBvid(bvid))
        }
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
    let _season_id = parse_positive_u64(parts.next()).ok_or_else(|| BdlError::Planning {
        message: format!("选中的分 P `{}` 缺少 season ID。", part_id.0),
    })?;

    parse_positive_u64(parts.next()).ok_or_else(|| BdlError::Planning {
        message: format!("选中的分 P `{}` 缺少 ep ID。", part_id.0),
    })
}

fn parse_positive_u64(value: Option<&str>) -> Option<u64> {
    value?.parse::<u64>().ok().filter(|value| *value > 0)
}

fn hydrate_placeholder_part(
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

        let existing_item = &group.items[item_index];
        hydrated_item.id = existing_item.id.clone();
        merge_missing_item_metadata(&mut hydrated_item, existing_item);
        group.items[item_index] = hydrated_item;
        return Ok(hydrated_part_ids);
    }

    Err(BdlError::Planning {
        message: format!(
            "选中的分 P `{}` 未加载，请重新解析后再试。",
            placeholder_id.0
        ),
    })
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

    Ok(hydrated_parts
        .iter()
        .map(|part| part.id.clone())
        .collect::<Vec<_>>())
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

fn remap_selected_part_ids(
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

fn dedupe_tasks_by_id(queue: &mut Vec<DownloadTask>) -> bool {
    let original_len = queue.len();
    let mut seen = HashSet::new();
    queue.retain(|task| seen.insert(task.id.clone()));
    queue.len() != original_len
}

fn append_new_tasks(queue: &mut Vec<DownloadTask>, tasks: Vec<DownloadTask>) -> Vec<DownloadTask> {
    let mut known_ids = queue
        .iter()
        .map(|task| task.id.clone())
        .collect::<HashSet<_>>();
    let mut inserted = Vec::new();

    for task in tasks {
        if known_ids.insert(task.id.clone()) {
            inserted.push(task.clone());
            queue.push(task);
        }
    }

    inserted
}

fn prepare_startup_recovery(
    queue: &mut [DownloadTask],
    auto_recovery_enabled: bool,
) -> StartupRecoverySnapshot {
    let mut task_ids = Vec::new();
    for task in queue {
        if !needs_startup_recovery(task.status) {
            continue;
        }

        task_ids.push(task.id.clone());
        task.status = if auto_recovery_enabled {
            TaskStatus::Waiting
        } else {
            TaskStatus::Paused
        };
        reset_interrupted_resources(&mut task.resources);
    }

    StartupRecoverySnapshot {
        task_ids,
        auto_recovery_enabled,
    }
}

fn needs_startup_recovery(status: TaskStatus) -> bool {
    matches!(
        status,
        TaskStatus::Waiting | TaskStatus::Parsing | TaskStatus::Downloading | TaskStatus::Muxing
    )
}

fn reset_interrupted_resources(resources: &mut [bdl_core::queue::DownloadResource]) {
    for resource in resources {
        if matches!(
            resource.status,
            ResourceStatus::Downloading | ResourceStatus::Paused
        ) {
            resource.status = ResourceStatus::Pending;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AppState, PartHydrationRequest, StartupRecoverySnapshot, append_new_tasks,
        append_source_page, dedupe_tasks_by_id, hydrate_placeholder_part, load_account_snapshot,
        next_page_request, prepare_startup_recovery, remap_selected_part_ids, select_task_stream,
        selected_hydration_requests, task_media_refresh_ids,
    };
    use crate::secure_store::SecureStore;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::sync::atomic::AtomicBool;
    use std::time::{SystemTime, UNIX_EPOCH};

    use bdl_core::account::AccountSummary;
    use bdl_core::ids::{GroupId, ItemId, PartId, SourceId};
    use bdl_core::input::ClassifiedInput;
    use bdl_core::model::{
        HeaderPair, MediaKind, MediaStream, NormalizedGroup, NormalizedItem, NormalizedPart,
        NormalizedSourceTree, PageState, SourceKind, SourceSummary, StreamCodec, StreamQuality,
    };
    use bdl_core::queue::{
        DownloadResource, DownloadResourceIntent, DownloadResourceKind, DownloadTask,
        DownloadTaskMediaSelection, DownloadTaskRefreshInput, DownloadTaskRefreshIntent,
        ResourceStatus, TaskStatus,
    };
    use bdl_core::resolver::paged::PageRequest;
    use bdl_core::settings::AppSettings;
    use bdl_core::storage::TaskStorage;

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
    fn load_account_snapshot_migrates_legacy_cookie_and_persists_summary() {
        let data_dir = temp_state_dir();
        std::fs::create_dir_all(&data_dir).expect("temp data dir should be created");
        let legacy_cookie_path = data_dir.join("account.cookie");
        std::fs::write(
            &legacy_cookie_path,
            "DedeUserID=42; SESSDATA=session; bili_jct=csrf\n",
        )
        .expect("legacy cookie should be written");
        let secure_store = SecureStore::in_memory();
        let mut storage =
            TaskStorage::open(data_dir.join("tasks.sqlite")).expect("storage should open");

        let (account, cookie) =
            load_account_snapshot(&mut storage, &secure_store, &data_dir).expect("load account");

        assert!(account.logged_in);
        assert_eq!(account.mid.as_deref(), Some("42"));
        assert_eq!(
            cookie.as_deref(),
            Some("DedeUserID=42; SESSDATA=session; bili_jct=csrf")
        );
        assert_eq!(
            secure_store
                .load_cookie()
                .expect("cookie should load")
                .as_deref(),
            Some("DedeUserID=42; SESSDATA=session; bili_jct=csrf")
        );
        assert_eq!(
            storage
                .load_account_summary()
                .expect("summary should load")
                .as_ref()
                .and_then(|summary| summary.mid.as_deref()),
            Some("42")
        );
        assert!(!legacy_cookie_path.exists());

        let _ = std::fs::remove_dir_all(data_dir);
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

        let state = test_state_with_tasks(
            &data_dir,
            vec![waiting, paused, failed, refresh, completed, remove],
        );

        let paused_task = state
            .update_task_status("task:waiting", TaskStatus::Paused)
            .expect("waiting task should pause");
        assert_eq!(paused_task.status, TaskStatus::Paused);

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
            parse_sources: Mutex::new(HashMap::new()),
            queue: Mutex::new(tasks),
            storage: Mutex::new(storage),
            settings: Mutex::new(AppSettings::default()),
            settings_path: data_dir.join("settings.json"),
            data_dir: data_dir.to_path_buf(),
            account: Mutex::new(AccountSummary::default()),
            account_cookie: Mutex::new(None),
            queue_worker_active: AtomicBool::new(false),
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
                    cover_url: Some("https://example.invalid/list-cover.jpg".to_owned()),
                    duration_seconds: Some(62),
                    parts: vec![NormalizedPart {
                        id: PartId("part:uploader:1001:BV1xx411c7mD".to_owned()),
                        title: "fixture upload".to_owned(),
                        aid: Some(170001),
                        bvid: Some("BV1xx411c7mD".to_owned()),
                        cid: None,
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
                    cover_url: None,
                    duration_seconds: Some(62),
                    parts: vec![NormalizedPart {
                        id: PartId(format!("part:{label}:123:456:789")),
                        title: "fixture episode".to_owned(),
                        aid: Some(170001),
                        bvid: None,
                        cid: Some(789),
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
                    cover_url: None,
                    duration_seconds: None,
                    parts: vec![NormalizedPart {
                        id: PartId("part:uploader:1001:BV1yy411c7mD".to_owned()),
                        title: "fixture upload page 2".to_owned(),
                        aid: Some(170002),
                        bvid: Some("BV1yy411c7mD".to_owned()),
                        cid: None,
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
                    cover_url: None,
                    duration_seconds: None,
                    parts: vec![
                        NormalizedPart {
                            id: PartId("part:BV1xx411c7mD:62131".to_owned()),
                            title: "P1".to_owned(),
                            aid: Some(170001),
                            bvid: Some("BV1xx411c7mD".to_owned()),
                            cid: Some(62131),
                            streams: Vec::new(),
                            assets: Vec::new(),
                        },
                        NormalizedPart {
                            id: PartId("part:BV1xx411c7mD:62132".to_owned()),
                            title: "P2".to_owned(),
                            aid: Some(170001),
                            bvid: Some("BV1xx411c7mD".to_owned()),
                            cid: Some(62132),
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
