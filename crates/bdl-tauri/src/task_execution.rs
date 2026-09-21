use std::sync::Arc;

use bdl_core::BdlResult;
use serde::Serialize;

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
    fn set_active(&self, active: bool) -> BdlResult<()>;
    fn sync_scheduled_wakeups(&self, scheduled_at_unix_millis: &[i64]) -> BdlResult<()>;
    fn notification_permission_state(&self) -> BdlResult<NotificationPermissionState>;
    fn request_notification_permission(&self) -> BdlResult<NotificationPermissionState>;
}

#[derive(Clone)]
pub struct TaskExecutionBackend {
    inner: Arc<dyn TaskExecutionBackendImpl>,
}

impl TaskExecutionBackend {
    pub fn from_impl<T>(backend: T) -> Self
    where
        T: TaskExecutionBackendImpl + 'static,
    {
        Self {
            inner: Arc::new(backend),
        }
    }

    pub fn noop() -> Self {
        Self::from_impl(NoopTaskExecutionBackend)
    }

    pub fn set_active(&self, active: bool) -> BdlResult<()> {
        self.inner.set_active(active)
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
