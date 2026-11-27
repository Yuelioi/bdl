use std::future::Future;
use std::time::Duration;

use crate::parse_control::Phase;
use bdl_core::settings::ParseRules;
use bdl_core::{BdlError, BdlResult};
use tokio::sync::Mutex;
use tokio::time::Instant;

#[derive(Default)]
pub(crate) struct ParsePacer(Mutex<PaceState>);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum CooldownBehavior {
    Report,
    Wait,
}

#[derive(Default)]
struct PaceState {
    last_page: Option<Instant>,
    blocked_until: Option<Instant>,
}

impl ParsePacer {
    // Retry read-only resolver operations once, never retry a restriction response.
    // Only follow-up pages supply an interval. A first lookup can still start
    // the clock when its result reveals a paginated source.
    pub(crate) async fn run_retry<T, F: Future<Output = BdlResult<T>>>(
        &self,
        page_interval: Option<ParseRules>,
        cooldown: CooldownBehavior,
        mut operation: impl FnMut() -> F,
        is_page: impl Fn(&T) -> bool,
        phase: impl Fn(Phase),
    ) -> BdlResult<T> {
        let first = self
            .run_observed(page_interval, cooldown, operation(), &is_page, &phase)
            .await;
        if first.as_ref().is_err_and(is_transient) {
            let until = Instant::now() + Duration::from_secs(2);
            phase(Phase::Waiting(until));
            tokio::time::sleep_until(until).await;
            self.run_observed(page_interval, cooldown, operation(), is_page, phase)
                .await
        } else {
            first
        }
    }
    // Shared by pagination and stream hydration, separate from media file downloads.
    // A resolver operation can itself make several HTTP calls.
    #[cfg(test)]
    pub(crate) async fn run<T>(
        &self,
        page_interval: Option<ParseRules>,
        operation: impl Future<Output = BdlResult<T>>,
        is_page: impl FnOnce(&T) -> bool,
    ) -> BdlResult<T> {
        self.run_observed(
            page_interval,
            CooldownBehavior::Report,
            operation,
            is_page,
            |_| {},
        )
        .await
    }

    pub(crate) async fn run_observed<T>(
        &self,
        page_interval: Option<ParseRules>,
        cooldown: CooldownBehavior,
        operation: impl Future<Output = BdlResult<T>>,
        is_page: impl FnOnce(&T) -> bool,
        phase: impl Fn(Phase),
    ) -> BdlResult<T> {
        let mut state = loop {
            phase(Phase::Queued);
            let state = self.0.lock().await;
            let now = Instant::now();
            let restricted = state.blocked_until.filter(|until| *until > now);
            if let Some(until) = restricted
                && cooldown == CooldownBehavior::Report
            {
                let remaining = until - now;
                return Err(BdlError::ParseRestricted {
                    seconds: remaining.as_secs() + u64::from(remaining.subsec_nanos() > 0),
                    source: None,
                });
            }
            let next_page = state
                .last_page
                .zip(page_interval)
                .map(|(last, rules)| last + Duration::from_secs(rules.interval_seconds));
            let Some(until) = restricted.max(next_page).filter(|until| *until > now) else {
                break state;
            };
            drop(state);
            phase(Phase::Waiting(until));
            tokio::time::sleep_until(until).await;
        };
        phase(Phase::Running);
        let started = Instant::now();
        // The shared BpiClient bounds each HTTP request. Hydrating a multi-part
        // video can take many healthy requests, so do not cap the whole operation.
        let result = operation.await;
        tracing::debug!(
            elapsed_ms = started.elapsed().as_millis() as u64,
            success = result.is_ok(),
            "resolver operation completed"
        );
        let now = Instant::now();
        match result {
            Ok(value) => {
                if is_page(&value) {
                    state.last_page = Some(now);
                }
                Ok(value)
            }
            Err(error) if is_rate_limit(&error) => {
                let seconds = 60;
                state.blocked_until = Some(now + Duration::from_secs(seconds));
                Err(BdlError::ParseRestricted {
                    seconds,
                    source: Some(Box::new(error)),
                })
            }
            Err(error) => Err(error),
        }
    }
}

fn is_rate_limit(error: &BdlError) -> bool {
    let BdlError::Bpi(message) = error else {
        return false;
    };
    bdl_core::diagnostics::classify_failure(message).is_some_and(|rule| {
        matches!(
            rule.code.as_str(),
            "source_rate_limited" | "source_verification_required"
        )
    })
}

fn is_transient(error: &BdlError) -> bool {
    if is_rate_limit(error) {
        return false;
    }
    match error {
        BdlError::BpiTransport { source, .. } => {
            source.is_timeout()
                || bdl_core::account::redact_diagnostic(&error.to_string())
                    .to_lowercase()
                    .contains("connection reset")
                || error
                    .to_string()
                    .to_lowercase()
                    .contains("connection closed")
        }
        BdlError::Bpi(message) => {
            let message = message.to_ascii_lowercase();
            [
                "timed out",
                "timeout",
                "connection reset",
                "connection closed",
            ]
            .iter()
            .any(|pattern| message.contains(pattern))
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn elapsed_cooldown_does_not_emit_waiting_or_wait_again() {
        let pacer = ParsePacer::default();
        let rules = ParseRules {
            interval_seconds: 3,
        };
        pacer
            .run(Some(rules), async { Ok(()) }, |_| true)
            .await
            .unwrap();
        tokio::time::advance(Duration::from_secs(30)).await;
        let resumed = Instant::now();
        pacer
            .run_observed(
                Some(rules),
                CooldownBehavior::Report,
                async { Ok(()) },
                |_| true,
                |phase| {
                    assert!(!matches!(phase, Phase::Waiting(_)));
                },
            )
            .await
            .unwrap();
        assert_eq!(resumed.elapsed(), Duration::ZERO);
    }

    #[tokio::test(start_paused = true)]
    async fn cancel_cooldown_releases_shared_pool_without_consuming_a_page() {
        let pacer = ParsePacer::default();
        let control = crate::parse_control::ParseControl::default();
        let rules = ParseRules {
            interval_seconds: 10,
        };
        pacer
            .run(Some(rules), async { Ok(()) }, |_| true)
            .await
            .unwrap();
        let operation = control.begin("source", "test");
        let work = operation.run(pacer.run_observed(
            Some(rules),
            CooldownBehavior::Report,
            async {
                panic!("cancelled waiting request must not reach the server");
                #[allow(unreachable_code)]
                Ok(())
            },
            |_| true,
            |phase| operation.phase(phase),
        ));
        let stop = async {
            tokio::task::yield_now().await;
            assert_eq!(control.progress("source").waiting_seconds, 10);
            control.cancel("source", crate::parse_control::CancelReason::StopRequested);
        };
        let start = Instant::now();
        let (result, ()) = tokio::join!(work, stop);
        assert!(result.is_err());
        assert_eq!(start.elapsed(), Duration::ZERO);
        // Stream hydration may proceed while a subsequent page still owes the wait.
        pacer.run(None, async { Ok(()) }, |_| false).await.unwrap();
        assert_eq!(start.elapsed(), Duration::ZERO);
        pacer
            .run(Some(rules), async { Ok(()) }, |_| true)
            .await
            .unwrap();
        assert_eq!(start.elapsed(), Duration::from_secs(10));
    }

    #[tokio::test(start_paused = true)]
    async fn transient_failure_retries_once_but_restrictions_never_retry() {
        assert!(!is_rate_limit(&BdlError::Bpi(
            "[stage=bilibili_api origin=source.rs:429:12] missing response data".into()
        )));
        use std::sync::atomic::{AtomicUsize, Ordering};
        for (message, expected) in [("connection reset", 2), ("HTTP 429", 1), ("code=-352", 1)] {
            let pacer = ParsePacer::default();
            let attempts = AtomicUsize::new(0);
            let result: BdlResult<()> = pacer
                .run_retry(
                    None,
                    CooldownBehavior::Report,
                    || {
                        attempts.fetch_add(1, Ordering::SeqCst);
                        async { Err(BdlError::Bpi(message.into())) }
                    },
                    |_| true,
                    |_| {},
                )
                .await;
            assert!(result.is_err());
            assert_eq!(attempts.load(Ordering::SeqCst), expected);
        }
    }

    #[tokio::test(start_paused = true)]
    async fn healthy_multi_part_hydration_can_exceed_one_minute() {
        let pacer = ParsePacer::default();
        let result = pacer
            .run(
                None,
                async {
                    for _ in 0..31 {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                    Ok(31)
                },
                |_| false,
            )
            .await;
        assert_eq!(result.unwrap(), 31);
    }

    #[tokio::test(start_paused = true)]
    async fn request_timeout_releases_pool() {
        let pacer = ParsePacer::default();
        let result: BdlResult<()> = pacer
            .run(
                None,
                async {
                    tokio::time::sleep(Duration::from_secs(20)).await;
                    Err(BdlError::Bpi("request timeout".into()))
                },
                |_| false,
            )
            .await;
        assert!(result.unwrap_err().to_string().contains("timeout"));
        assert!(pacer.run(None, async { Ok(()) }, |_| false).await.is_ok());
    }

    #[tokio::test(start_paused = true)]
    async fn each_native_page_waits_once_regardless_of_item_count() {
        let pacer = ParsePacer::default();
        let rules = ParseRules {
            interval_seconds: 2,
        };
        let start = Instant::now();
        pacer.run(None, async { Ok(30) }, |_| true).await.unwrap();
        assert_eq!(start.elapsed(), Duration::ZERO);
        pacer
            .run(Some(rules), async { Ok(30) }, |_| true)
            .await
            .unwrap();
        assert_eq!(start.elapsed(), Duration::from_secs(2));
        tokio::time::advance(Duration::from_secs(1)).await;
        pacer.run(None, async { Ok(()) }, |_| false).await.unwrap();
        pacer
            .run(Some(rules), async { Ok(1) }, |_| true)
            .await
            .unwrap();
        assert_eq!(start.elapsed(), Duration::from_secs(4));
    }

    #[tokio::test(start_paused = true)]
    async fn rate_limit_stops_waiting_operations_without_retry() {
        let pacer = ParsePacer::default();
        let rules = ParseRules::default();
        let failed: BdlResult<()> = pacer
            .run(
                Some(rules),
                async { Err(BdlError::Bpi("HTTP 412".into())) },
                |_| true,
            )
            .await;
        let failure = crate::commands::CommandError::from(failed.unwrap_err());
        assert_eq!(failure.code, "source_verification_required");
        assert!(failure.message.contains("60 秒"));
        assert!(failure.detail.unwrap().contains("HTTP 412"));
        let blocked = pacer
            .run(
                Some(rules),
                async {
                    panic!("blocked work must not run");
                    #[allow(unreachable_code)]
                    Ok(())
                },
                |_| true,
            )
            .await;
        let blocked = crate::commands::CommandError::from(blocked.unwrap_err());
        assert_eq!(blocked.code, "source_access_restricted");
        assert!(blocked.message.contains("60 秒"));
        let waiting_since = Instant::now();
        let result = pacer
            .run_observed(
                Some(rules),
                CooldownBehavior::Wait,
                async { Ok(waiting_since.elapsed()) },
                |_| false,
                |_| {},
            )
            .await;
        assert_eq!(result.unwrap(), Duration::from_secs(60));
    }
}
