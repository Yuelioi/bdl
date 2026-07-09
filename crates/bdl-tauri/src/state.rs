use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use bdl_core::ids::SourceId;
use bdl_core::input::classify_input;
use bdl_core::model::NormalizedSourceTree;
use bdl_core::queue::{DownloadTask, TaskStatus};
use bdl_core::resolver::video::VideoResolver;
use bdl_core::resolver::{ResolveOptions, Resolver};
use bdl_core::settings::AppSettings;
use bdl_core::storage::TaskStorage;
use bdl_core::{BdlError, BdlResult};
use serde::{Deserialize, Serialize};

pub struct AppState {
    resolver: VideoResolver,
    parse_sources: Mutex<HashMap<SourceId, NormalizedSourceTree>>,
    queue: Mutex<Vec<DownloadTask>>,
    storage: Mutex<TaskStorage>,
    settings: Mutex<SettingsSnapshot>,
    account: Mutex<AccountSnapshot>,
}

impl AppState {
    pub fn new() -> BdlResult<Self> {
        let storage = TaskStorage::open(default_storage_path()?)?;
        let queue = storage.load_tasks()?;

        Ok(Self {
            resolver: VideoResolver::new()?,
            parse_sources: Mutex::new(HashMap::new()),
            queue: Mutex::new(queue),
            storage: Mutex::new(storage),
            settings: Mutex::new(SettingsSnapshot::default()),
            account: Mutex::new(AccountSnapshot::default()),
        })
    }

    pub async fn parse_source(
        &self,
        input: &str,
        fetch_streams: bool,
    ) -> BdlResult<NormalizedSourceTree> {
        let classified = classify_input(input)?;
        let tree = self
            .resolver
            .resolve(classified, ResolveOptions { fetch_streams })
            .await?;
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

    fn persist_queue(&self, queue: &[DownloadTask]) -> BdlResult<()> {
        self.storage
            .lock()
            .map_err(|_| state_poisoned("storage"))?
            .replace_tasks(queue)
    }
}

pub type SettingsSnapshot = AppSettings;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AccountSnapshot {
    pub logged_in: bool,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
}

fn state_poisoned(name: &'static str) -> BdlError {
    BdlError::Planning {
        message: format!("state lock `{name}` is poisoned"),
    }
}

fn default_storage_path() -> BdlResult<PathBuf> {
    Ok(std::env::current_dir()?.join(".bdl").join("tasks.sqlite"))
}
