//! Cross-process guard for the CLI's actual resolver HTTP requests.
use bpi_rs::{
    BpiError, BpiResult,
    transport::observer::{RequestObserver, RequestPermit},
};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const WINDOW_MS: u64 = 3_600_000;
const WINDOW_REQUESTS: u32 = 7200;

#[derive(Clone)]
pub struct HttpGuard {
    path: PathBuf,
    interval: Duration,
    max_requests: u32,
    used: Arc<AtomicU32>,
}

#[derive(Default, Serialize, Deserialize)]
struct Shared {
    window_started_ms: u64,
    requests: u32,
    next_ms: u64,
    blocked_until_ms: u64,
}

impl HttpGuard {
    pub async fn record_restriction(&self) -> BpiResult<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(io_error)?;
        }
        loop {
            let lock = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(self.path.with_extension("lock"))
                .map_err(io_error)?;
            match lock.try_lock_exclusive() {
                Ok(()) => {
                    let mut state = read_state(&self.path)?;
                    state.blocked_until_ms =
                        state.blocked_until_ms.max(now_ms().saturating_add(300_000));
                    return write_state(&self.path, &state);
                }
                Err(error)
                    if error.raw_os_error() == fs2::lock_contended_error().raw_os_error() =>
                {
                    tokio::time::sleep(Duration::from_millis(100)).await
                }
                Err(error) => return Err(io_error(error)),
            }
        }
    }
    pub fn new(path: PathBuf, interval: Duration, max_requests: u32) -> Self {
        Self {
            path,
            interval,
            max_requests,
            used: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn user_state_path() -> Result<PathBuf, std::io::Error> {
        #[cfg(windows)]
        let root = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
        #[cfg(not(windows))]
        let root = std::env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state"))
            });
        root.map(|root| root.join("bdl/cli-http.json"))
            .ok_or_else(|| std::io::Error::other("cannot locate per-user HTTP state directory"))
    }

    async fn acquire(&self) -> BpiResult<Box<dyn RequestPermit>> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(io_error)?;
        }
        loop {
            let lock = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(self.path.with_extension("lock"))
                .map_err(io_error)?;
            match lock.try_lock_exclusive() {
                Ok(()) => {}
                Err(error)
                    if error.raw_os_error() == fs2::lock_contended_error().raw_os_error() =>
                {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    continue;
                }
                Err(error) => return Err(io_error(error)),
            }
            let mut state = read_state(&self.path)?;
            let now = now_ms();
            if state.blocked_until_ms > now {
                return Err(BpiError::network(format!(
                    "source restriction: shared cooldown active for {} seconds; retry manually later",
                    (state.blocked_until_ms - now).div_ceil(1000)
                )));
            }
            if now.saturating_sub(state.window_started_ms) >= WINDOW_MS {
                state.window_started_ms = now;
                state.requests = 0;
            }
            if state.requests >= WINDOW_REQUESTS {
                return Err(BpiError::network(
                    "shared HTTP budget exhausted (7200 attempts per hour); resume manually later",
                ));
            }
            if self.used.load(Ordering::SeqCst) >= self.max_requests {
                return Err(BpiError::network(
                    "HTTP request budget exhausted for this invocation; resume manually later",
                ));
            }
            if state.next_ms > now {
                let delay = state.next_ms - now;
                drop(lock);
                tokio::time::sleep(Duration::from_millis(delay)).await;
                continue;
            }
            state.requests += 1;
            self.used.fetch_add(1, Ordering::SeqCst);
            let wait_ms = self.interval.as_millis() as u64;
            // Retain a timeout-sized reservation if the process dies mid-request.
            state.next_ms = now.saturating_add(30_000).saturating_add(wait_ms);
            write_state(&self.path, &state)?;
            return Ok(Box::new(Permit {
                _lock: lock,
                path: self.path.clone(),
                state,
                next_ms: now.saturating_add(wait_ms),
            }));
        }
    }
}

impl RequestObserver for HttpGuard {
    fn retry_transient_get(&self) -> bool {
        true
    }
    fn before(
        &self,
    ) -> Pin<Box<dyn Future<Output = BpiResult<Box<dyn RequestPermit>>> + Send + '_>> {
        Box::pin(self.acquire())
    }
}

struct Permit {
    _lock: File,
    path: PathBuf,
    state: Shared,
    next_ms: u64,
}
impl RequestPermit for Permit {
    fn observe(
        &mut self,
        status: u16,
        headers: &reqwest::header::HeaderMap,
        body: &[u8],
    ) -> BpiResult<()> {
        let json = serde_json::from_slice::<serde_json::Value>(body).ok();
        let code = json
            .as_ref()
            .and_then(|value| value.get("code"))
            .and_then(serde_json::Value::as_i64);
        let message = json
            .as_ref()
            .and_then(|value| value.get("message").or_else(|| value.get("msg")))
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        let html_challenge = status == 200
            && headers
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.contains("text/html"));
        let restricted = matches!(status, 403 | 412 | 429)
            || code.is_some_and(|code| matches!(code, -352 | -412 | -509 | -799))
            || super::pacing::is_source_restriction(message)
            || html_challenge;
        if restricted {
            let retry_seconds = headers
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|value| value.to_str().ok())
                .and_then(|value| {
                    value.parse::<u64>().ok().or_else(|| {
                        chrono::DateTime::parse_from_rfc2822(value)
                            .ok()
                            .map(|date| {
                                (date.timestamp_millis().max(0) as u64)
                                    .saturating_sub(now_ms())
                                    .div_ceil(1000)
                            })
                    })
                })
                .unwrap_or(0);
            self.state.blocked_until_ms = self
                .state
                .blocked_until_ms
                .max(now_ms().saturating_add(retry_seconds.max(300).saturating_mul(1000)));
            write_state(&self.path, &self.state)?;
            return Err(BpiError::network(
                "source restriction: HTTP/API verification or rate limit response; shared cooldown recorded, no retry",
            ));
        }
        Ok(())
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        self.state.next_ms = self.next_ms;
        if let Err(error) = write_state(&self.path, &self.state) {
            tracing::error!("cannot persist HTTP completion spacing: {error}");
        }
        // The operating system releases the lock even on process termination.
    }
}

fn read_state(path: &Path) -> BpiResult<Shared> {
    match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes).map_err(|_| {
            BpiError::network("shared HTTP state is invalid; refusing to reset its budget")
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Shared {
            window_started_ms: now_ms(),
            ..Shared::default()
        }),
        Err(error) => Err(io_error(error)),
    }
}
fn write_state(path: &Path, state: &Shared) -> BpiResult<()> {
    use std::io::Write;
    let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)
        .map_err(io_error)?;
    file.write_all(
        &serde_json::to_vec(state).map_err(|_| BpiError::network("cannot encode HTTP state"))?,
    )
    .map_err(io_error)?;
    file.sync_all().map_err(io_error)?;
    drop(file);
    std::fs::rename(temporary, path).map_err(io_error)
}
fn io_error(error: std::io::Error) -> BpiError {
    BpiError::network(format!("HTTP guard storage/lock failure: {error}"))
}
fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
