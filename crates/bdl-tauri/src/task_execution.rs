use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use bdl_core::BdlResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceEnvironment {
    pub os_version: Option<String>,
    pub android_sdk: Option<u32>,
    pub webview_version: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationPermissionState {
    Granted,
    Denied,
    Prompt,
    PromptWithRationale,
    Unsupported,
}

pub trait TaskExecutionBackendImpl: Send + Sync {
    fn device_environment(&self) -> BdlResult<DeviceEnvironment> {
        Ok(DeviceEnvironment::default())
    }
    fn set_active(&self, active: bool) -> BdlResult<()>;
    fn sync_scheduled_wakeups(&self, scheduled_at_unix_millis: &[i64]) -> BdlResult<()>;
    fn notification_permission_state(&self) -> BdlResult<NotificationPermissionState>;
    fn request_notification_permission(&self) -> BdlResult<NotificationPermissionState>;
}

#[derive(Clone)]
pub struct TaskExecutionBackend {
    inner: Arc<dyn TaskExecutionBackendImpl>,
    activity: Arc<Mutex<Activity>>,
}

#[derive(Default)]
struct Activity {
    downloads: bool,
    preparations: HashSet<String>,
}

impl TaskExecutionBackend {
    pub fn device_environment(&self) -> BdlResult<DeviceEnvironment> {
        self.inner.device_environment()
    }
    pub fn from_impl<T>(backend: T) -> Self
    where
        T: TaskExecutionBackendImpl + 'static,
    {
        Self {
            inner: Arc::new(backend),
            activity: Default::default(),
        }
    }

    pub fn noop() -> Self {
        Self::from_impl(NoopTaskExecutionBackend)
    }

    pub fn set_active(&self, active: bool) -> BdlResult<()> {
        let mut activity = self.activity.lock().unwrap();
        let effective = active || !activity.preparations.is_empty();
        self.inner.set_active(effective)?;
        activity.downloads = active;
        Ok(())
    }

    pub(crate) fn set_preparation_active(&self, id: &str, active: bool) -> BdlResult<()> {
        let mut activity = self.activity.lock().unwrap();
        let mut preparations = activity.preparations.clone();
        if active {
            preparations.insert(id.to_owned());
        } else {
            preparations.remove(id);
        }
        self.inner
            .set_active(activity.downloads || !preparations.is_empty())?;
        activity.preparations = preparations;
        Ok(())
    }

    pub fn sync_scheduled_wakeups(&self, scheduled_at_unix_millis: &[i64]) -> BdlResult<()> {
        self.inner.sync_scheduled_wakeups(scheduled_at_unix_millis)
    }

    pub fn notification_permission_state(&self) -> BdlResult<NotificationPermissionState> {
        self.inner.notification_permission_state()
    }

    pub fn request_notification_permission(&self) -> BdlResult<NotificationPermissionState> {
        self.inner.request_notification_permission()
    }
}

struct NoopTaskExecutionBackend;

impl TaskExecutionBackendImpl for NoopTaskExecutionBackend {
    fn set_active(&self, _active: bool) -> BdlResult<()> {
        Ok(())
    }

    fn sync_scheduled_wakeups(&self, _scheduled_at_unix_millis: &[i64]) -> BdlResult<()> {
        Ok(())
    }

    fn notification_permission_state(&self) -> BdlResult<NotificationPermissionState> {
        Ok(NotificationPermissionState::Unsupported)
    }

    fn request_notification_permission(&self) -> BdlResult<NotificationPermissionState> {
        Ok(NotificationPermissionState::Unsupported)
    }
}
