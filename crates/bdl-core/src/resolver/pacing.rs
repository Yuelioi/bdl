//! Process-local guard for resolver operations, not a count of individual HTTP requests.
use crate::{BdlError, BdlResult};
use std::{future::Future, time::Duration};
use tokio::{sync::Mutex, time::Instant};

#[derive(Debug, Clone, Copy)]
pub struct ResolvePolicy {
    pub interval: Duration,
    pub max_operations: u32,
}

impl Default for ResolvePolicy {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(500),
            max_operations: 50,
        }
    }
}

pub(crate) struct ResolveGate {
    policy: ResolvePolicy,
    state: Mutex<State>,
}

#[derive(Default)]
struct State {
    operations: u32,
    next: Option<Instant>,
    stopped: bool,
}

impl ResolveGate {
    pub fn new(policy: ResolvePolicy) -> Self {
        Self {
            policy,
            state: Mutex::new(State::default()),
        }
    }

    pub async fn run<T>(&self, operation: impl Future<Output = BdlResult<T>>) -> BdlResult<T> {
        let mut state = self.state.lock().await;
        if state.stopped {
            return Err(error(
                "source restriction: this session is stopped; retry manually later",
            ));
        }
        if state.operations >= self.policy.max_operations {
            return Err(error(
                "resolver operation budget exhausted; progress is retained when --state is set; resume manually later",
            ));
        }
        if let Some(next) = state.next {
            tokio::time::sleep_until(next).await;
        }
        state.operations += 1;
        // Reserve the next slot before polling: cancellation also consumes a slot.
        let wait = self.policy.interval;
        state.next = Some(Instant::now() + wait);
        let result = operation.await;
        state.next = Some(Instant::now() + wait);
        if result
            .as_ref()
            .is_err_and(|err| is_source_restriction(&err.to_string()))
        {
            state.stopped = true;
            return Err(error(
                "source restriction: server rejected or requested verification; stopped without retry; wait before manually resuming (waiting does not guarantee access)",
            ));
        }
        result
    }
}

pub fn is_source_restriction(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower
        .split(|c: char| !c.is_ascii_digit() && c != '-')
        .any(|code| {
            matches!(
                code,
                "-352" | "-412" | "-509" | "-799" | "403" | "412" | "429"
            )
        })
        || [
            "source restriction",
            "captcha",
            "验证",
            "风控",
            "too many requests",
        ]
        .iter()
        .any(|text| lower.contains(text))
}

fn error(message: &str) -> BdlError {
    BdlError::Planning {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[tokio::test(start_paused = true)]
    async fn shared_budget_and_spacing_are_enforced_before_polling() {
        let gate = ResolveGate::new(ResolvePolicy {
            interval: Duration::from_secs(2),
            max_operations: 2,
        });
        let start = Instant::now();
        gate.run(async { Ok(()) }).await.unwrap();
        gate.run(async { Ok(()) }).await.unwrap();
        assert!(start.elapsed() >= Duration::from_secs(2));
        let called = AtomicUsize::new(0);
        assert!(
            gate.run(async {
                called.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
            .await
            .is_err()
        );
        assert_eq!(called.load(Ordering::SeqCst), 0);
    }
    #[tokio::test(start_paused = true)]
    async fn restriction_stops_following_operations_without_retry() {
        for message in [
            "HTTP 429",
            "code=-352",
            "HTTP 412",
            "captcha required",
            "HTTP 403",
        ] {
            let gate = ResolveGate::new(ResolvePolicy::default());
            assert!(
                gate.run(async { Err::<(), _>(BdlError::Bpi(message.into())) })
                    .await
                    .is_err()
            );
            let called = AtomicUsize::new(0);
            assert!(
                gate.run(async {
                    called.fetch_add(1, Ordering::SeqCst);
                    Ok(())
                })
                .await
                .is_err()
            );
            assert_eq!(called.load(Ordering::SeqCst), 0);
        }
        assert!(!is_source_restriction("connection timeout"));
    }
}
