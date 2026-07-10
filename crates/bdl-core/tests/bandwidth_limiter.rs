use std::sync::Arc;
use std::time::Duration;

use bdl_core::fetcher::BandwidthLimiter;
use tokio::time::Instant;

#[tokio::test(start_paused = true)]
async fn shared_limiter_applies_one_aggregate_budget_to_concurrent_consumers() {
    let limiter = Arc::new(BandwidthLimiter::new(Some(1024)));
    limiter.acquire(1024).await;
    let started = Instant::now();

    tokio::join!(limiter.acquire(512), limiter.acquire(512));

    assert_eq!(started.elapsed(), Duration::from_secs(1));
}

#[tokio::test(start_paused = true)]
async fn unlimited_limiter_does_not_delay_consumers() {
    let limiter = BandwidthLimiter::new(None);
    let started = Instant::now();

    limiter.acquire(1024 * 1024).await;

    assert_eq!(started.elapsed(), Duration::ZERO);
}
