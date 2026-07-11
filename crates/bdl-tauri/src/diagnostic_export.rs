//! Sanitization rules applied before diagnostic data leaves local storage.

use bdl_core::account::redact_sensitive;
use bdl_core::queue::{DownloadTask, QueueLogEntry};

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
    log.message = redact_sensitive(&log.message);
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
