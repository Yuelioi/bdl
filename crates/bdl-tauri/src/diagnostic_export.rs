//! Sanitization rules applied before diagnostic data leaves local storage.

use bdl_core::account::redact_sensitive;
use bdl_core::queue::{DownloadTask, QueueLogEntry};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct DiagnosticEnvironment {
    pub app_version: String,
    pub os: String,
    pub arch: String,
    pub device: crate::task_execution::DeviceEnvironment,
    pub device_info_available: bool,
    pub segment_count: usize,
    pub concurrent_tasks: usize,
    pub retry_count: usize,
    pub proxy_mode: bdl_core::settings::ProxyMode,
}

pub(crate) async fn collect_environment(
    state: &crate::state::AppState,
) -> bdl_core::BdlResult<DiagnosticEnvironment> {
    let settings = state.settings()?;
    let backend = state.task_execution();
    let device = tauri::async_runtime::spawn_blocking(move || backend.device_environment()).await;
    let (device, device_info_available) = match device {
        Ok(Ok(device)) => (device, true),
        _ => (Default::default(), false),
    };
    Ok(DiagnosticEnvironment {
        app_version: crate::version().to_owned(),
        os: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
        device,
        device_info_available,
        segment_count: settings.segment_count,
        concurrent_tasks: settings.concurrent_tasks,
        retry_count: settings.retry_count,
        proxy_mode: settings.proxy_mode,
    })
}

pub(crate) fn redact_url(raw: &str) -> String {
    let Some(scheme_end) = raw.find("://") else {
        return raw.to_owned();
    };
    let Some(credentials_end) = raw[scheme_end + 3..].find('@') else {
        return raw.to_owned();
    };
    let host_start = scheme_end + 3 + credentials_end + 1;
    format!("{}://<redacted>@{}", &raw[..scheme_end], &raw[host_start..])
}

pub(crate) fn redact_task(mut task: DownloadTask) -> DownloadTask {
    for resource in &mut task.resources {
        resource.current_urls = resource
            .current_urls
            .iter()
            .map(|url| redact_sensitive(url))
            .collect();
        for header in &mut resource.headers {
            if is_sensitive_header(&header.name) {
                header.value = "<redacted>".to_owned();
            } else {
                header.value = redact_sensitive(&header.value);
            }
        }
    }
    task
}

pub(crate) fn redact_log(mut log: QueueLogEntry) -> QueueLogEntry {
    log.message = bdl_core::account::redact_diagnostic(&log.message);
    log
}

fn is_sensitive_header(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "cookie" | "authorization" | "proxy-authorization"
    )
}

#[cfg(test)]
mod tests {
    use super::redact_url;

    #[test]
    fn proxy_credentials_are_removed_without_changing_the_host() {
        assert_eq!(
            redact_url("https://user:secret@example.com/path"),
            "https://<redacted>@example.com/path"
        );
    }
}
