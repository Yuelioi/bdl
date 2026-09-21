pub mod bangumi;
pub mod cheese;
pub mod collection;
pub mod favorite;
pub mod paged;
pub mod uploader;
pub mod video;

use async_trait::async_trait;
use chrono::{FixedOffset, TimeZone};

use crate::BdlResult;
use crate::input::ClassifiedInput;
use crate::model::NormalizedSourceTree;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResolveOptions {
    pub fetch_streams: bool,
}

#[async_trait]
pub trait Resolver {
    async fn resolve(
        &self,
        input: ClassifiedInput,
        options: ResolveOptions,
    ) -> BdlResult<NormalizedSourceTree>;
}

pub(crate) fn bilibili_publish_date(timestamp: u64) -> Option<String> {
    if timestamp == 0 {
        return None;
    }
    let timestamp = i64::try_from(timestamp).ok()?;
    let china_standard_time = FixedOffset::east_opt(8 * 60 * 60)?;
    china_standard_time
        .timestamp_opt(timestamp, 0)
        .single()
        .map(|value| value.date_naive().to_string())
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::bilibili_publish_date;

    #[test]
    fn publication_date_uses_bilibili_china_date_at_utc_boundary() {
        let timestamp = Utc
            .with_ymd_and_hms(2026, 1, 1, 16, 30, 0)
            .single()
            .expect("fixture timestamp")
            .timestamp() as u64;

        assert_eq!(
            bilibili_publish_date(timestamp).as_deref(),
            Some("2026-01-02")
        );
        assert_eq!(bilibili_publish_date(0), None);
    }
}
