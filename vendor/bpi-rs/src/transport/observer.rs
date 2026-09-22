//! Optional task-local HTTP observation; absent unless the caller opts in.
use crate::BpiResult;
use reqwest::header::HeaderMap;
use std::{future::Future, pin::Pin, sync::Arc};

pub trait RequestPermit: Send {
    fn observe(&mut self, status: u16, headers: &HeaderMap, body: &[u8]) -> BpiResult<()>;
}

pub trait RequestObserver: Send + Sync {
    fn retry_transient_get(&self) -> bool {
        false
    }
    fn before(
        &self,
    ) -> Pin<Box<dyn Future<Output = BpiResult<Box<dyn RequestPermit>>> + Send + '_>>;
}

pub(crate) fn retry_transient_get() -> bool {
    OBSERVER
        .try_with(|observer| observer.retry_transient_get())
        .unwrap_or(false)
}

tokio::task_local! { static OBSERVER: Arc<dyn RequestObserver>; }

pub async fn scope<T>(observer: Arc<dyn RequestObserver>, operation: impl Future<Output = T>) -> T {
    OBSERVER.scope(observer, operation).await
}

pub(crate) async fn before() -> BpiResult<Option<Box<dyn RequestPermit>>> {
    match OBSERVER.try_with(Arc::clone) {
        Ok(observer) => observer.before().await.map(Some),
        Err(_) => Ok(None),
    }
}
