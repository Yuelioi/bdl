use std::collections::HashMap;

use bdl_core::{BdlError, BdlResult};
use bdl_tauri::task_execution::{
    NotificationPermissionState, TaskExecutionBackend, TaskExecutionBackendImpl,
};
use serde::Serialize;
use tauri::{
    Manager, Runtime,
    plugin::{Builder, PermissionState as TauriPermissionState, PluginHandle, TauriPlugin},
};

const PLUGIN_IDENTIFIER: &str = "com.yueli.bdl.execution";
const NOTIFICATION_PERMISSION_ALIAS: &str = "notifications";

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("bdl-mobile-execution")
        .setup(|app, api| {
            let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "TaskExecutionPlugin")?;
            app.manage(TaskExecutionBackend::from_impl(
                AndroidTaskExecutionBackend { handle },
            ));
            Ok(())
        })
        .build()
}

struct AndroidTaskExecutionBackend<R: Runtime> {
    handle: PluginHandle<R>,
}

#[derive(Serialize)]
struct ExecutionPayload {
    active: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ScheduleWakeupPayload {
    scheduled_at_unix_millis: Vec<i64>,
}

#[derive(Serialize)]
struct PermissionRequestPayload<'a> {
    permissions: [&'a str; 1],
}

fn map_permission_state(state: TauriPermissionState) -> NotificationPermissionState {
    match state {
        TauriPermissionState::Granted => NotificationPermissionState::Granted,
        TauriPermissionState::Denied => NotificationPermissionState::Denied,
        TauriPermissionState::Prompt => NotificationPermissionState::Prompt,
        TauriPermissionState::PromptWithRationale => {
            NotificationPermissionState::PromptWithRationale
        }
    }
}

impl<R: Runtime> TaskExecutionBackendImpl for AndroidTaskExecutionBackend<R> {
    fn set_active(&self, active: bool) -> BdlResult<()> {
        self.handle
            .run_mobile_plugin::<serde_json::Value>("setActive", ExecutionPayload { active })
            .map(|_| ())
            .map_err(|error| BdlError::Platform {
                message: format!("Android 后台任务状态切换失败：{error}"),
            })
    }

    fn sync_scheduled_wakeups(&self, scheduled_at_unix_millis: &[i64]) -> BdlResult<()> {
        self.handle
            .run_mobile_plugin::<serde_json::Value>(
                "syncScheduledWakeups",
                ScheduleWakeupPayload {
                    scheduled_at_unix_millis: scheduled_at_unix_millis.to_vec(),
                },
            )
            .map(|_| ())
            .map_err(|error| BdlError::Platform {
                message: format!("Android 定时任务唤醒同步失败：{error}"),
            })
    }

    fn notification_permission_state(&self) -> BdlResult<NotificationPermissionState> {
        let states = self
            .handle
            .run_mobile_plugin::<HashMap<String, TauriPermissionState>>("checkPermissions", ())
            .map_err(|error| BdlError::Platform {
                message: format!("Android 通知权限状态读取失败：{error}"),
            })?;
        states
            .get(NOTIFICATION_PERMISSION_ALIAS)
            .copied()
            .map(map_permission_state)
            .ok_or_else(|| BdlError::Platform {
                message: "Android 通知权限状态缺少 notifications 项。".to_owned(),
            })
    }

    fn request_notification_permission(&self) -> BdlResult<NotificationPermissionState> {
        let states = self
            .handle
            .run_mobile_plugin::<HashMap<String, TauriPermissionState>>(
                "requestPermissions",
                PermissionRequestPayload {
                    permissions: [NOTIFICATION_PERMISSION_ALIAS],
                },
            )
            .map_err(|error| BdlError::Platform {
                message: format!("Android 通知权限请求失败：{error}"),
            })?;
        states
            .get(NOTIFICATION_PERMISSION_ALIAS)
            .copied()
            .map(map_permission_state)
            .ok_or_else(|| BdlError::Platform {
                message: "Android 通知权限请求结果缺少 notifications 项。".to_owned(),
            })
    }
}
