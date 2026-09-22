use crate::ids::Aid;
use crate::{BpiError, BpiResult};

/// `/x2/creative/web/archives/sp` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpArchivesListParams {
    page: u32,
    page_size: Option<u32>,
}

impl UpArchivesListParams {
    pub fn new(page: u32) -> BpiResult<Self> {
        Ok(Self {
            page: validate_non_zero_u32("pn", page)?,
            page_size: None,
        })
    }

    pub fn with_page_size(mut self, page_size: u32) -> BpiResult<Self> {
        self.page_size = Some(validate_non_zero_u32("ps", page_size)?);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut query = vec![("pn", self.page.to_string())];

        if let Some(page_size) = self.page_size {
            query.push(("ps", page_size.to_string()));
        }

        query
    }
}

/// `/x/web/archive/videos` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpArchiveVideosParams {
    aid: Aid,
}

impl UpArchiveVideosParams {
    pub fn new(aid: Aid) -> Self {
        Self { aid }
    }

    pub(crate) fn query_pairs(&self) -> [(&'static str, String); 1] {
        [("aid", self.aid.to_string())]
    }
}

/// `/x/web/data/archive_diagnose/compare` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UpArchiveCompareParams {
    timestamp: Option<u64>,
    size: Option<u32>,
}

impl UpArchiveCompareParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timestamp(mut self, timestamp: u64) -> BpiResult<Self> {
        self.timestamp = Some(validate_non_zero_u64("t", timestamp)?);
        Ok(self)
    }

    pub fn with_size(mut self, size: u32) -> BpiResult<Self> {
        self.size = Some(validate_non_zero_u32("size", size)?);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut query = Vec::new();

        if let Some(timestamp) = self.timestamp {
            query.push(("t", timestamp.to_string()));
        }

        if let Some(size) = self.size {
            query.push(("size", size.to_string()));
        }

        query
    }
}

/// `/x/web/data/pandect` 视频趋势查询的指标。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpVideoTrendMetric {
    Play,
    Danmaku,
    Reply,
    Share,
    Coin,
    Favorite,
    Charge,
    Like,
}

impl UpVideoTrendMetric {
    const fn code(self) -> u8 {
        match self {
            Self::Play => 1,
            Self::Danmaku => 2,
            Self::Reply => 3,
            Self::Share => 4,
            Self::Coin => 5,
            Self::Favorite => 6,
            Self::Charge => 7,
            Self::Like => 8,
        }
    }
}

/// `/x/web/data/pandect` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpVideoTrendParams {
    metric: UpVideoTrendMetric,
}

impl UpVideoTrendParams {
    pub fn new(metric: UpVideoTrendMetric) -> Self {
        Self { metric }
    }

    pub(crate) fn query_pairs(&self) -> [(&'static str, String); 1] {
        [("type", self.metric.code().to_string())]
    }
}

/// `/x/web/data/article/thirty` 专栏趋势查询的指标。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpArticleTrendMetric {
    Read,
    Reply,
    Share,
    Coin,
    Favorite,
    Like,
}

impl UpArticleTrendMetric {
    const fn code(self) -> u8 {
        match self {
            Self::Read => 1,
            Self::Reply => 2,
            Self::Share => 3,
            Self::Coin => 4,
            Self::Favorite => 5,
            Self::Like => 6,
        }
    }
}

/// `/x/web/data/article/thirty` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpArticleTrendParams {
    metric: UpArticleTrendMetric,
}

impl UpArticleTrendParams {
    pub fn new(metric: UpArticleTrendMetric) -> Self {
        Self { metric }
    }

    pub(crate) fn query_pairs(&self) -> [(&'static str, String); 1] {
        [("type", self.metric.code().to_string())]
    }
}

fn validate_non_zero_u32(field: &'static str, value: u32) -> BpiResult<u32> {
    if value == 0 {
        return Err(BpiError::invalid_parameter(field, "value must be non-zero"));
    }

    Ok(value)
}

fn validate_non_zero_u64(field: &'static str, value: u64) -> BpiResult<u64> {
    if value == 0 {
        return Err(BpiError::invalid_parameter(field, "value must be non-zero"));
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn up_archives_list_params_serializes_required_page() -> BpiResult<()> {
        let params = UpArchivesListParams::new(1)?;

        assert_eq!(params.query_pairs(), vec![("pn", "1".to_string())]);
        Ok(())
    }

    #[test]
    fn up_archives_list_params_serializes_optional_page_size() -> BpiResult<()> {
        let params = UpArchivesListParams::new(2)?.with_page_size(20)?;

        assert_eq!(
            params.query_pairs(),
            vec![("pn", "2".to_string()), ("ps", "20".to_string())]
        );
        Ok(())
    }

    #[test]
    fn up_archives_list_params_rejects_zero_page() {
        let err = UpArchivesListParams::new(0).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "pn", .. }
        ));
    }

    #[test]
    fn up_archives_list_params_rejects_zero_page_size() -> BpiResult<()> {
        let err = UpArchivesListParams::new(1)?.with_page_size(0).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "ps", .. }
        ));
        Ok(())
    }

    #[test]
    fn up_archive_videos_params_serializes_aid_query() -> BpiResult<()> {
        let params = UpArchiveVideosParams::new(Aid::new(113602455409683)?);

        assert_eq!(
            params.query_pairs(),
            [("aid", "113602455409683".to_string())]
        );
        Ok(())
    }

    #[test]
    fn up_archive_compare_params_serializes_empty_defaults() {
        let params = UpArchiveCompareParams::new();

        assert!(params.query_pairs().is_empty());
    }

    #[test]
    fn up_archive_compare_params_serializes_optional_filters() -> BpiResult<()> {
        let params = UpArchiveCompareParams::new()
            .with_timestamp(1_720_000_000)?
            .with_size(3)?;

        assert_eq!(
            params.query_pairs(),
            vec![("t", "1720000000".to_string()), ("size", "3".to_string())]
        );
        Ok(())
    }

    #[test]
    fn up_archive_compare_params_rejects_zero_size() {
        let err = UpArchiveCompareParams::new().with_size(0).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "size", .. }
        ));
    }

    #[test]
    fn up_video_trend_params_serializes_metric_code() {
        let params = UpVideoTrendParams::new(UpVideoTrendMetric::Play);

        assert_eq!(params.query_pairs(), [("type", "1".to_string())]);
    }

    #[test]
    fn up_video_trend_metric_maps_like_code() {
        let params = UpVideoTrendParams::new(UpVideoTrendMetric::Like);

        assert_eq!(params.query_pairs(), [("type", "8".to_string())]);
    }

    #[test]
    fn up_article_trend_params_serializes_metric_code() {
        let params = UpArticleTrendParams::new(UpArticleTrendMetric::Read);

        assert_eq!(params.query_pairs(), [("type", "1".to_string())]);
    }

    #[test]
    fn up_article_trend_metric_maps_like_code() {
        let params = UpArticleTrendParams::new(UpArticleTrendMetric::Like);

        assert_eq!(params.query_pairs(), [("type", "6".to_string())]);
    }
}
