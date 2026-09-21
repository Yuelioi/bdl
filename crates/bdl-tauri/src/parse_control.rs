use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};

use bdl_core::fetcher::FetchCancelToken;
use bdl_core::{BdlError, BdlResult};
use serde::Serialize;
use tokio::time::Instant;

#[derive(Default)]
pub(crate) struct ParseControl(Mutex<HashMap<String, Vec<Arc<Operation>>>>);

struct Operation {
    cancel: FetchCancelToken,
    phase: Mutex<Phase>,
}

#[derive(Clone, Copy)]
pub(crate) enum Phase {
    Queued,
    Waiting(Instant),
    Running,
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
    pub fn begin(&self, source: &str) -> ParseOperation<'_> {
        let operation = Arc::new(Operation {
            cancel: FetchCancelToken::new(),
            phase: Mutex::new(Phase::Queued),
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

    pub fn cancel(&self, source: &str) {
        if let Some(operations) = self.0.lock().unwrap().get(source) {
            for operation in operations {
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

    pub async fn run<T>(&self, work: impl Future<Output = BdlResult<T>>) -> BdlResult<T> {
        tokio::select! {
            biased;
            _ = self.operation.cancel.cancelled() => Err(BdlError::Planning { message: "解析已停止".into() }),
            result = work => result,
        }
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
        let operation = control.begin("a");
        let other = control.begin("b");
        let start = Instant::now();
        let work = operation.run(async {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            Ok(())
        });
        let stop = async {
            tokio::task::yield_now().await;
            control.cancel("a");
        };
        let (result, ()) = tokio::join!(work, stop);
        assert!(result.is_err());
        assert_eq!(start.elapsed(), std::time::Duration::ZERO);
        assert!(other.run(async { Ok(()) }).await.is_ok());
        drop(operation);
        assert!(!control.progress("a").active);
        assert!(control.begin("a").run(async { Ok(()) }).await.is_ok());
    }
}
