use std::future::Future;
use std::time::Duration;

use crate::parse_control::Phase;
use bdl_core::settings::ParseRules;
use bdl_core::{BdlError, BdlResult};
use tokio::sync::Mutex;
use tokio::time::Instant;

#[derive(Default)]
pub(crate) struct ParsePacer(Mutex<PaceState>);

#[derive(Default)]
struct PaceState {
    last_completed: Option<Instant>,
    next: Option<Instant>,
    blocked_until: Option<Instant>,
    pages: usize,
    pages_since_rest: usize,
}

impl ParsePacer {
    // Retry read-only resolver operations once, never retry a restriction response.
    pub(crate) async fn run_retry<T, F: Future<Output = BdlResult<T>>>(
        &self,
        rules: ParseRules,
        mut operation: impl FnMut() -> F,
        page_count: impl Fn(&T) -> usize,
        phase: impl Fn(Phase),
    ) -> BdlResult<T> {
        let first = self
            .run_observed(rules, operation(), &page_count, &phase)
            .await;
        if first.as_ref().is_err_and(is_transient) {
            let until = Instant::now() + Duration::from_secs(2);
            phase(Phase::Waiting(until));
            tokio::time::sleep_until(until).await;
            self.run_observed(rules, operation(), page_count, phase)
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
        rules: ParseRules,
        operation: impl Future<Output = BdlResult<T>>,
        page_count: impl FnOnce(&T) -> usize,
    ) -> BdlResult<T> {
        self.run_observed(rules, operation, page_count, |_| {})
            .await
    }

    pub(crate) async fn run_observed<T>(
        &self,
        rules: ParseRules,
        operation: impl Future<Output = BdlResult<T>>,
        page_count: impl FnOnce(&T) -> usize,
        phase: impl Fn(Phase),
    ) -> BdlResult<T> {
        phase(Phase::Queued);
        let mut state = self.0.lock().await;
        if let Some(until) = state.blocked_until.filter(|until| *until > Instant::now()) {
            return Err(BdlError::Planning {
                message: format!(
                    "解析已因限流暂停，请约 {} 秒后手动重试。",
                    (until - Instant::now()).as_secs().saturating_add(1)
                ),
            });
        }
        // Time spent idle is already a rest, including between manual batches.
        if rules.rest_seconds > 0
            && state
                .last_completed
                .is_some_and(|last| last.elapsed() >= Duration::from_secs(rules.rest_seconds))
        {
            state.pages = 0;
            state.pages_since_rest = 0;
        }
        if let Some(next) = state.next.filter(|next| *next > Instant::now()) {
            phase(Phase::Waiting(next));
            tokio::time::sleep_until(next).await;
        }
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
        state.last_completed = Some(now);
        state.next = None;
        match result {
            Ok(value) => {
                let pages = page_count(&value);
                state.pages = state.pages.saturating_add(pages);
                state.pages_since_rest = state.pages_since_rest.saturating_add(pages);
                if state.pages_since_rest >= 5 {
                    state.pages = 0;
                    state.pages_since_rest = 0;
                    state.next = Some(now + Duration::from_secs(rules.rest_seconds));
                } else if state.pages >= rules.pages_per_round {
                    state.pages = 0;
                    state.next = Some(now + Duration::from_secs(rules.interval_seconds));
                }
                Ok(value)
            }
            Err(error) if is_rate_limit(&error) => {
                state.blocked_until = Some(now + Duration::from_secs(60));
                Err(BdlError::Planning {
                    message: format!(
                        "源站限制了请求，已停止解析，至少暂停60秒。已解析结果保留，请稍后手动重试；等待结束不代表限制已解除。{error}"
                    ),
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
    async fn idle_time_resets_partial_batch_without_adding_another_rest() {
        let pacer = ParsePacer::default();
        let rules = ParseRules {
            pages_per_round: 5,
            interval_seconds: 1,
            rest_seconds: 3,
        };
        for _ in 0..4 {
            pacer.run(rules, async { Ok(()) }, |_| 1).await.unwrap();
        }
        tokio::time::advance(Duration::from_secs(30)).await;
        let resumed = Instant::now();
        for _ in 0..5 {
            pacer.run(rules, async { Ok(()) }, |_| 1).await.unwrap();
        }
        assert_eq!(resumed.elapsed(), Duration::ZERO);
        pacer.run(rules, async { Ok(()) }, |_| 1).await.unwrap();
        assert_eq!(resumed.elapsed(), Duration::from_secs(3));
    }

    #[tokio::test(start_paused = true)]
    async fn elapsed_cooldown_does_not_emit_waiting_or_wait_again() {
        let pacer = ParsePacer::default();
        let rules = ParseRules {
            pages_per_round: 1,
            interval_seconds: 3,
            rest_seconds: 3,
        };
        pacer.run(rules, async { Ok(()) }, |_| 1).await.unwrap();
        tokio::time::advance(Duration::from_secs(30)).await;
        let resumed = Instant::now();
        pacer
            .run_observed(
                rules,
                async { Ok(()) },
                |_| 1,
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
            pages_per_round: 1,
            interval_seconds: 10,
            ..Default::default()
        };
        pacer.run(rules, async { Ok(()) }, |_| 1).await.unwrap();
        let operation = control.begin("source", "test");
        let work = operation.run(pacer.run_observed(
            rules,
            async {
                panic!("cancelled waiting request must not reach the server");
                #[allow(unreachable_code)]
                Ok(())
            },
            |_| 1,
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
        // A stream operation joins the same gate and still observes its remaining wait.
        pacer.run(rules, async { Ok(()) }, |_| 0).await.unwrap();
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
                    ParseRules::default(),
                    || {
                        attempts.fetch_add(1, Ordering::SeqCst);
                        async { Err(BdlError::Bpi(message.into())) }
                    },
                    |_| 1,
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
                ParseRules::default(),
                async {
                    for _ in 0..31 {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                    Ok(31)
                },
                |_| 0,
            )
            .await;
        assert_eq!(result.unwrap(), 31);
    }

    #[tokio::test(start_paused = true)]
    async fn request_timeout_releases_pool() {
        let pacer = ParsePacer::default();
        let result: BdlResult<()> = pacer
            .run(
                ParseRules::default(),
                async {
                    tokio::time::sleep(Duration::from_secs(20)).await;
                    Err(BdlError::Bpi("request timeout".into()))
                },
                |_| 0,
            )
            .await;
        assert!(result.unwrap_err().to_string().contains("timeout"));
        assert!(
            pacer
                .run(ParseRules::default(), async { Ok(()) }, |_| 0)
                .await
                .is_ok()
        );
    }

    #[tokio::test(start_paused = true)]
    async fn intervals_and_batch_pause_are_shared() {
        let pacer = ParsePacer::default();
        let rules = ParseRules {
            interval_seconds: 2,
            rest_seconds: 2,
            pages_per_round: 2,
        };
        let start = Instant::now();
        pacer.run(rules, async { Ok(20) }, |_| 1).await.unwrap();
        pacer.run(rules, async { Ok(20) }, |_| 1).await.unwrap();
        assert_eq!(start.elapsed(), Duration::ZERO);
        pacer.run(rules, async { Ok(1) }, |n| *n).await.unwrap();
        assert_eq!(start.elapsed(), Duration::from_secs(2));
    }

    #[tokio::test(start_paused = true)]
    async fn two_hundred_items_have_predictable_non_stacked_waits() {
        for (pages_per_round, expected_seconds) in [(3, 5), (5, 3), (1, 11)] {
            let pacer = ParsePacer::default();
            let rules = ParseRules {
                pages_per_round,
                ..Default::default()
            };
            let start = Instant::now();
            for _ in 0..10 {
                pacer.run(rules, async { Ok(()) }, |_| 1).await.unwrap();
            }
            assert_eq!(start.elapsed(), Duration::from_secs(expected_seconds));
        }
    }

    #[tokio::test(start_paused = true)]
    async fn rate_limit_stops_waiting_operations_without_retry() {
        let pacer = ParsePacer::default();
        let rules = ParseRules::default();
        let failed: BdlResult<()> = pacer
            .run(
                rules,
                async { Err(BdlError::Bpi("HTTP 412".into())) },
                |_| 1,
            )
            .await;
        assert!(failed.unwrap_err().to_string().contains("暂停60秒"));
        let blocked = pacer
            .run(
                rules,
                async {
                    panic!("blocked work must not run");
                    #[allow(unreachable_code)]
                    Ok(())
                },
                |_| 1,
            )
            .await;
        assert!(blocked.is_err());
        tokio::time::advance(Duration::from_secs(60)).await;
        assert!(pacer.run(rules, async { Ok(()) }, |_| 1).await.is_ok());
    }
}
