use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};

use bdl_core::fetcher::FetchCancelToken;
use bdl_core::{BdlError, BdlResult};
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

#[derive(Default)]
pub(crate) struct ParseControl(Mutex<HashMap<String, Vec<Arc<Operation>>>>);

struct Operation {
    id: uuid::Uuid,
    action: &'static str,
    started: Instant,
    cancel: FetchCancelToken,
    phase: Mutex<Phase>,
    target: Mutex<String>,
    cancelled: Mutex<Option<String>>,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CancelReason {
    StopRequested,
    WorkspaceCleared,
    SourceClosed,
    LibraryDeactivated,
    LibraryFolderClosed,
    AccountChanged,
}

impl CancelReason {
    fn name(self) -> &'static str {
        match self {
            Self::StopRequested => "stop_requested",
            Self::WorkspaceCleared => "workspace_cleared",
            Self::SourceClosed => "source_closed",
            Self::LibraryDeactivated => "library_deactivated",
            Self::LibraryFolderClosed => "library_folder_closed",
            Self::AccountChanged => "account_changed",
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Phase {
    Queued,
    Waiting(Instant),
    Running,
}

impl Phase {
    fn name(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Waiting(_) => "waiting",
            Self::Running => "running",
        }
    }
}

#[derive(Default, Serialize)]
pub struct ParseProgress {
    pub active: bool,
    pub waiting_seconds: u64,
    pub queued: bool,
}

pub(crate) struct ParseOperation<'a> {
    control: &'a ParseControl,
    source: String,
    operation: Arc<Operation>,
}

impl ParseControl {
    pub fn begin(&self, source: &str, action: &'static str) -> ParseOperation<'_> {
        let operation = Arc::new(Operation {
            id: uuid::Uuid::new_v4(),
            action,
            started: Instant::now(),
            cancel: FetchCancelToken::new(),
            phase: Mutex::new(Phase::Queued),
            target: Mutex::new(String::new()),
            cancelled: Mutex::new(None),
        });
        self.0
            .lock()
            .unwrap()
            .entry(source.to_owned())
            .or_default()
            .push(operation.clone());
        ParseOperation {
            control: self,
            source: source.to_owned(),
            operation,
        }
    }

    pub fn cancel(&self, source: &str, reason: CancelReason) {
        if let Some(operations) = self.0.lock().unwrap().get(source) {
            for operation in operations {
                // Preserve the first signal, including its phase and target.
                operation.cancelled.lock().unwrap().get_or_insert_with(|| format!(
                    "cancel_reason={} phase_at_cancel={} elapsed_at_cancel_ms={} active_same_source={} target_at_cancel=[{}]",
                    reason.name(), operation.phase.lock().unwrap().name(),
                    operation.started.elapsed().as_millis(), operations.len(),
                    operation.target.lock().unwrap(),
                ));
                operation.cancel.cancel();
            }
        }
    }

    pub fn progress(&self, source: &str) -> ParseProgress {
        let operations = self.0.lock().unwrap();
        let Some(operations) = operations.get(source) else {
            return ParseProgress::default();
        };
        let mut progress = ParseProgress {
            active: true,
            ..Default::default()
        };
        for operation in operations {
            match *operation.phase.lock().unwrap() {
                Phase::Waiting(until) => {
                    progress.waiting_seconds = progress.waiting_seconds.max(
                        until
                            .saturating_duration_since(Instant::now())
                            .as_millis()
                            .div_ceil(1000) as u64,
                    )
                }
                Phase::Queued => progress.queued = true,
                Phase::Running => {}
            }
        }
        progress
    }
}

impl ParseOperation<'_> {
    pub fn phase(&self, phase: Phase) {
        *self.operation.phase.lock().unwrap() = phase;
    }

    pub fn target(&self, target: String) {
        *self.operation.target.lock().unwrap() = target;
    }

    fn diagnostic(&self) -> String {
        format!(
            "operation_id={} action={} source_id={} phase={} elapsed_ms={} target=[{}]",
            self.operation.id,
            self.operation.action,
            self.source,
            self.operation.phase.lock().unwrap().name(),
            self.operation.started.elapsed().as_millis(),
            self.operation.target.lock().unwrap(),
        )
    }

    pub async fn run<T>(&self, work: impl Future<Output = BdlResult<T>>) -> BdlResult<T> {
        let result = tokio::select! {
            biased;
            _ = self.operation.cancel.cancelled() => {
                let signal = self.operation.cancelled.lock().unwrap().clone()
                    .expect("parse cancellation must have a recorded signal");
                return Err(BdlError::ParseCancelled {
                    context: format!("[stage=parse_cancelled] {} {signal}", self.diagnostic()),
                });
            },
            result = work => result,
        };
        result.map_err(|error| {
            error.with_context(format!("[stage=parse_resolve] {}", self.diagnostic()))
        })
    }
}

impl Drop for ParseOperation<'_> {
    fn drop(&mut self) {
        let mut sources = self.control.0.lock().unwrap();
        if let Some(operations) = sources.get_mut(&self.source) {
            operations.retain(|operation| !Arc::ptr_eq(operation, &self.operation));
            if operations.is_empty() {
                sources.remove(&self.source);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn stop_interrupts_wait_and_preserves_other_sources() {
        let control = ParseControl::default();
        let operation = control.begin("a", "test");
        let other = control.begin("b", "test");
        let start = Instant::now();
        let work = operation.run(async {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            Ok(())
        });
        let stop = async {
            tokio::task::yield_now().await;
            control.cancel("a", CancelReason::StopRequested);
        };
        let (result, ()) = tokio::join!(work, stop);
        assert!(result.is_err());
        assert_eq!(start.elapsed(), std::time::Duration::ZERO);
        assert!(other.run(async { Ok(()) }).await.is_ok());
        drop(operation);
        assert!(!control.progress("a").active);
        assert!(
            control
                .begin("a", "test")
                .run(async { Ok(()) })
                .await
                .is_ok()
        );
        for name in [
            "stop_requested",
            "workspace_cleared",
            "source_closed",
            "library_deactivated",
            "library_folder_closed",
            "account_changed",
        ] {
            let reason: CancelReason = serde_json::from_value(name.into()).unwrap();
            let operation = control.begin("a", "prepare_selection");
            operation.target("part_id=fixture item=3/20".into());
            control.cancel("a", reason);
            control.cancel("a", CancelReason::SourceClosed);
            let error = operation
                .run(async { Ok(()) })
                .await
                .unwrap_err()
                .to_string();
            assert!(error.contains(&format!("cancel_reason={name} ")), "{error}");
            assert!(error.contains("part_id=fixture item=3/20"), "{error}");
        }
    }
}
