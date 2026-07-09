use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::Mutex;

use bdl_core::account::{AccountSummary, ImportedCookie};
use bdl_core::ids::{PartId, SourceId};
use bdl_core::input::{ClassifiedInput, classify_input};
use bdl_core::model::{
    NormalizedItem, NormalizedPart, NormalizedSourceTree, PageState, SourceKind,
};
use bdl_core::queue::{DownloadTask, TaskStatus};
use bdl_core::resolver::paged::PageRequest;
use bdl_core::resolver::uploader::UploaderResolver;
use bdl_core::resolver::video::VideoResolver;
use bdl_core::resolver::{ResolveOptions, Resolver};
use bdl_core::settings::AppSettings;
use bdl_core::storage::TaskStorage;
use bdl_core::{BdlError, BdlResult};

use crate::secure_store::SecureStore;

const DEFAULT_PARSE_ALL_LIMIT: usize = 100;
const MAX_PARSE_ALL_LIMIT: usize = 100;

pub struct AppState {
    parse_sources: Mutex<HashMap<SourceId, NormalizedSourceTree>>,
    queue: Mutex<Vec<DownloadTask>>,
    storage: Mutex<TaskStorage>,
    settings: Mutex<SettingsSnapshot>,
    account: Mutex<AccountSnapshot>,
    account_cookie: Mutex<Option<String>>,
    secure_store: SecureStore,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedSelection {
    pub tree: NormalizedSourceTree,
    pub part_ids: Vec<PartId>,
    pub tree_updated: bool,
}

impl AppState {
    pub fn new() -> BdlResult<Self> {
        let storage = TaskStorage::open(default_storage_path()?)?;
        let queue = storage.load_tasks()?;
        let secure_store = SecureStore::new(default_account_cookie_path()?);
        let persisted_cookie = secure_store.load_cookie()?;
        let (account, account_cookie) = load_account_snapshot(&secure_store, persisted_cookie)?;

        Ok(Self {
            parse_sources: Mutex::new(HashMap::new()),
            queue: Mutex::new(queue),
            storage: Mutex::new(storage),
            settings: Mutex::new(SettingsSnapshot::default()),
            account: Mutex::new(account),
            account_cookie: Mutex::new(account_cookie),
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
            SourceKind::Uploader => {
                self.uploader_resolver()?
                    .resolve(classified, options)
                    .await?
            }
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

        let resolver = self.video_resolver()?;
        let mut part_ids = selected_part_ids.to_vec();

        for request in requests {
            let hydrated = resolver
                .resolve(
                    ClassifiedInput::VideoBvid(request.bvid),
                    ResolveOptions {
                        fetch_streams: true,
                    },
                )
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

    pub fn enqueue_tasks(&self, tasks: Vec<DownloadTask>) -> BdlResult<()> {
        let mut queue = self.queue.lock().map_err(|_| state_poisoned("queue"))?;
        queue.extend(tasks);
        self.persist_queue(&queue)?;
        Ok(())
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
        let task = queue[task_index].clone();
        self.persist_queue(&queue)?;
        Ok(task)
    }

    pub fn remove_task(&self, task_id: &str) -> BdlResult<bool> {
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

    pub fn settings(&self) -> BdlResult<SettingsSnapshot> {
        Ok(self
            .settings
            .lock()
            .map_err(|_| state_poisoned("settings"))?
            .clone())
    }

    pub fn update_settings(&self, settings: SettingsSnapshot) -> BdlResult<SettingsSnapshot> {
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
        *self
            .account_cookie
            .lock()
            .map_err(|_| state_poisoned("account_cookie"))? =
            Some(imported_cookie.as_header().to_owned());
        *self.account.lock().map_err(|_| state_poisoned("account"))? = account.clone();

        Ok(account)
    }

    pub fn logout(&self) -> BdlResult<AccountSnapshot> {
        self.secure_store.clear_cookie()?;
        *self
            .account_cookie
            .lock()
            .map_err(|_| state_poisoned("account_cookie"))? = None;
        *self.account.lock().map_err(|_| state_poisoned("account"))? = AccountSnapshot::default();

        self.account()
    }

    pub fn verify_account(&self) -> BdlResult<AccountSnapshot> {
        let Some(raw_cookie) = self
            .account_cookie
            .lock()
            .map_err(|_| state_poisoned("account_cookie"))?
            .clone()
        else {
            return self.account();
        };

        let imported_cookie = ImportedCookie::parse(raw_cookie)?;
        let account = AccountSummary::from_imported_cookie(&imported_cookie);
        *self.account.lock().map_err(|_| state_poisoned("account"))? = account.clone();
        Ok(account)
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
            SourceKind::Uploader => {
                let mid = uploader_mid(tree)?;
                let request = next_page_request(tree)?;
                let next_page = self.uploader_resolver()?.resolve_page(mid, request).await?;
                append_source_page(tree, next_page)
            }
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PartHydrationRequest {
    part_id: PartId,
    bvid: String,
    target_cid: Option<u64>,
}

pub type SettingsSnapshot = AppSettings;
pub type AccountSnapshot = AccountSummary;

fn state_poisoned(name: &'static str) -> BdlError {
    BdlError::Planning {
        message: format!("state lock `{name}` is poisoned"),
    }
}

fn default_storage_path() -> BdlResult<PathBuf> {
    Ok(std::env::current_dir()?.join(".bdl").join("tasks.sqlite"))
}

fn default_account_cookie_path() -> BdlResult<PathBuf> {
    Ok(std::env::current_dir()?.join(".bdl").join("account.cookie"))
}

fn load_account_snapshot(
    secure_store: &SecureStore,
    persisted_cookie: Option<String>,
) -> BdlResult<(AccountSnapshot, Option<String>)> {
    let Some(raw_cookie) = persisted_cookie else {
        return Ok((AccountSnapshot::default(), None));
    };

    match ImportedCookie::parse(&raw_cookie) {
        Ok(imported_cookie) => Ok((
            AccountSummary::from_imported_cookie(&imported_cookie),
            Some(raw_cookie),
        )),
        Err(_) => {
            secure_store.clear_cookie()?;
            Ok((AccountSnapshot::default(), None))
        }
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
    let has_more = total_count
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

        let bvid = part.bvid.clone().ok_or_else(|| BdlError::Planning {
            message: format!("选中的分 P `{}` 缺少 BV ID，无法补齐下载流。", part_id.0),
        })?;

        requests.push(PartHydrationRequest {
            part_id: part_id.clone(),
            bvid,
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

fn hydrate_placeholder_part(
    tree: &mut NormalizedSourceTree,
    placeholder_id: &PartId,
    target_cid: Option<u64>,
    hydrated: NormalizedSourceTree,
) -> BdlResult<Vec<PartId>> {
    let mut hydrated_item = first_hydrated_item(hydrated)?;
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

fn first_hydrated_item(hydrated: NormalizedSourceTree) -> BdlResult<NormalizedItem> {
    hydrated
        .groups
        .into_iter()
        .flat_map(|group| group.items)
        .next()
        .ok_or_else(|| BdlError::Planning {
            message: "视频解析结果为空，无法创建下载任务。".to_owned(),
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

#[cfg(test)]
mod tests {
    use super::{
        PartHydrationRequest, append_source_page, hydrate_placeholder_part, next_page_request,
        remap_selected_part_ids, selected_hydration_requests,
    };
    use bdl_core::ids::{GroupId, ItemId, PartId, SourceId};
    use bdl_core::model::{
        NormalizedGroup, NormalizedItem, NormalizedPart, NormalizedSourceTree, PageState,
        SourceKind, SourceSummary,
    };
    use bdl_core::resolver::paged::PageRequest;

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
                bvid: "BV1xx411c7mD".to_owned(),
                target_cid: None,
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
