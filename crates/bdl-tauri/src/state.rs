use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use bdl_core::account::{AccountSummary, ImportedCookie};
use bdl_core::ids::SourceId;
use bdl_core::input::classify_input;
use bdl_core::model::{NormalizedSourceTree, SourceKind};
use bdl_core::queue::{DownloadTask, TaskStatus};
use bdl_core::resolver::uploader::UploaderResolver;
use bdl_core::resolver::video::VideoResolver;
use bdl_core::resolver::{ResolveOptions, Resolver};
use bdl_core::settings::AppSettings;
use bdl_core::storage::TaskStorage;
use bdl_core::{BdlError, BdlResult};

use crate::secure_store::SecureStore;

pub struct AppState {
    parse_sources: Mutex<HashMap<SourceId, NormalizedSourceTree>>,
    queue: Mutex<Vec<DownloadTask>>,
    storage: Mutex<TaskStorage>,
    settings: Mutex<SettingsSnapshot>,
    account: Mutex<AccountSnapshot>,
    account_cookie: Mutex<Option<String>>,
    secure_store: SecureStore,
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
