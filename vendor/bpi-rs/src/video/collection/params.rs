use crate::ids::{Mid, SeasonId};
use crate::{BpiError, BpiResult};

/// 视频合集稿件排序方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionArchiveSort {
    /// 最早的在前。
    Asc,
    /// 最新的在前。
    Desc,
}

impl CollectionArchiveSort {
    fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

/// `/x/polymer/web-space/seasons_archives_list` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoCollectionSeasonsArchivesParams {
    mid: Mid,
    season_id: SeasonId,
    sort_reverse: Option<bool>,
    page_num: u64,
    page_size: u64,
}

impl VideoCollectionSeasonsArchivesParams {
    /// 为指定用户的视频合集创建参数。
    pub fn new(mid: Mid, season_id: SeasonId) -> Self {
        Self {
            mid,
            season_id,
            sort_reverse: None,
            page_num: 1,
            page_size: 20,
        }
    }

    /// 设置稿件列表是否按倒序返回。
    pub fn with_sort_reverse(mut self, sort_reverse: bool) -> Self {
        self.sort_reverse = Some(sort_reverse);
        self
    }

    /// 设置页码。
    pub fn with_page_num(mut self, page_num: u64) -> BpiResult<Self> {
        self.page_num = validate_positive_u64("page_num", page_num)?;
        Ok(self)
    }

    /// 设置每页数量。
    pub fn with_page_size(mut self, page_size: u64) -> BpiResult<Self> {
        self.page_size = validate_positive_u64("page_size", page_size)?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = vec![
            ("mid", self.mid.to_string()),
            ("season_id", self.season_id.to_string()),
            ("page_num", self.page_num.to_string()),
            ("page_size", self.page_size.to_string()),
        ];

        if let Some(sort_reverse) = self.sort_reverse {
            pairs.push(("sort_reverse", sort_reverse.to_string()));
        }

        pairs
    }
}

/// `/x/polymer/web-space/home/seasons_series` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoCollectionHomeSeasonsSeriesParams {
    mid: Mid,
    page_num: u64,
    page_size: u64,
}

impl VideoCollectionHomeSeasonsSeriesParams {
    /// 为主页合集/系列接口创建参数。
    pub fn new(mid: Mid) -> Self {
        Self {
            mid,
            page_num: 1,
            page_size: 10,
        }
    }

    /// 设置页码。
    pub fn with_page_num(mut self, page_num: u64) -> BpiResult<Self> {
        self.page_num = validate_positive_u64("page_num", page_num)?;
        Ok(self)
    }

    /// 设置每页数量。
    pub fn with_page_size(mut self, page_size: u64) -> BpiResult<Self> {
        self.page_size = validate_positive_u64("page_size", page_size)?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("mid", self.mid.to_string()),
            ("page_num", self.page_num.to_string()),
            ("page_size", self.page_size.to_string()),
        ]
    }
}

/// `/x/polymer/web-space/seasons_series_list` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoCollectionSeasonsSeriesParams {
    mid: Mid,
    page_num: Option<u64>,
    page_size: Option<u64>,
}

impl VideoCollectionSeasonsSeriesParams {
    /// 为合集和系列列表接口创建参数。
    pub fn new(mid: Mid) -> Self {
        Self {
            mid,
            page_num: None,
            page_size: None,
        }
    }

    /// 设置页码。
    pub fn with_page_num(mut self, page_num: u64) -> BpiResult<Self> {
        self.page_num = Some(validate_positive_u64("page_num", page_num)?);
        Ok(self)
    }

    /// 设置每页数量。
    pub fn with_page_size(mut self, page_size: u64) -> BpiResult<Self> {
        self.page_size = Some(validate_positive_u64("page_size", page_size)?);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = vec![("mid", self.mid.to_string())];

        if let Some(page_num) = self.page_num {
            pairs.push(("page_num", page_num.to_string()));
        }
        if let Some(page_size) = self.page_size {
            pairs.push(("page_size", page_size.to_string()));
        }

        pairs
    }
}

/// `/x/series/series` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoCollectionSeriesInfoParams {
    series_id: u64,
}

impl VideoCollectionSeriesInfoParams {
    /// 为指定视频系列创建参数。
    pub fn new(series_id: u64) -> BpiResult<Self> {
        Ok(Self {
            series_id: validate_positive_u64("series_id", series_id)?,
        })
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("series_id", self.series_id.to_string())]
    }
}

/// `/x/series/archives` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoCollectionSeriesArchivesParams {
    mid: Mid,
    series_id: u64,
    only_normal: Option<bool>,
    sort: Option<CollectionArchiveSort>,
    page_num: Option<u64>,
    page_size: Option<u64>,
}

impl VideoCollectionSeriesArchivesParams {
    /// 为指定系列内的稿件创建参数。
    pub fn new(mid: Mid, series_id: u64) -> BpiResult<Self> {
        Ok(Self {
            mid,
            series_id: validate_positive_u64("series_id", series_id)?,
            only_normal: None,
            sort: None,
            page_num: None,
            page_size: None,
        })
    }

    /// 控制是否只返回普通稿件。
    pub fn with_only_normal(mut self, only_normal: bool) -> Self {
        self.only_normal = Some(only_normal);
        self
    }

    /// 设置稿件排序方式。
    pub fn with_sort(mut self, sort: CollectionArchiveSort) -> Self {
        self.sort = Some(sort);
        self
    }

    /// 设置页码。
    pub fn with_page_num(mut self, page_num: u64) -> BpiResult<Self> {
        self.page_num = Some(validate_positive_u64("pn", page_num)?);
        Ok(self)
    }

    /// 设置每页数量。
    pub fn with_page_size(mut self, page_size: u64) -> BpiResult<Self> {
        self.page_size = Some(validate_positive_u64("ps", page_size)?);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = vec![
            ("mid", self.mid.to_string()),
            ("series_id", self.series_id.to_string()),
        ];

        if let Some(only_normal) = self.only_normal {
            pairs.push(("only_normal", only_normal.to_string()));
        }
        if let Some(sort) = self.sort {
            pairs.push(("sort", sort.as_str().to_string()));
        }
        if let Some(page_num) = self.page_num {
            pairs.push(("pn", page_num.to_string()));
        }
        if let Some(page_size) = self.page_size {
            pairs.push(("ps", page_size.to_string()));
        }

        pairs
    }
}

fn validate_positive_u64(field: &'static str, value: u64) -> BpiResult<u64> {
    if value == 0 {
        return Err(BpiError::invalid_parameter(field, "value must be non-zero"));
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seasons_archives_params_serializes_defaults() -> BpiResult<()> {
        let params =
            VideoCollectionSeasonsArchivesParams::new(Mid::new(1000001)?, SeasonId::new(4294056)?);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("mid", "1000001".to_string()),
                ("season_id", "4294056".to_string()),
                ("page_num", "1".to_string()),
                ("page_size", "20".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn seasons_archives_params_serializes_sort_and_pagination() -> BpiResult<()> {
        let params =
            VideoCollectionSeasonsArchivesParams::new(Mid::new(1000001)?, SeasonId::new(4294056)?)
                .with_sort_reverse(false)
                .with_page_num(2)?
                .with_page_size(30)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("mid", "1000001".to_string()),
                ("season_id", "4294056".to_string()),
                ("page_num", "2".to_string()),
                ("page_size", "30".to_string()),
                ("sort_reverse", "false".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn home_seasons_series_params_serializes_defaults() -> BpiResult<()> {
        let params = VideoCollectionHomeSeasonsSeriesParams::new(Mid::new(1000001)?);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("mid", "1000001".to_string()),
                ("page_num", "1".to_string()),
                ("page_size", "10".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn seasons_series_params_serializes_optional_pagination() -> BpiResult<()> {
        let params = VideoCollectionSeasonsSeriesParams::new(Mid::new(1000001)?)
            .with_page_num(1)?
            .with_page_size(5)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("mid", "1000001".to_string()),
                ("page_num", "1".to_string()),
                ("page_size", "5".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn series_info_params_rejects_zero_series_id() {
        let err = VideoCollectionSeriesInfoParams::new(0).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "series_id",
                ..
            }
        ));
    }

    #[test]
    fn series_archives_params_serializes_optional_filters() -> BpiResult<()> {
        let params = VideoCollectionSeriesArchivesParams::new(Mid::new(1000001)?, 250285)?
            .with_sort(CollectionArchiveSort::Asc)
            .with_page_num(1)?
            .with_page_size(10)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("mid", "1000001".to_string()),
                ("series_id", "250285".to_string()),
                ("sort", "asc".to_string()),
                ("pn", "1".to_string()),
                ("ps", "10".to_string())
            ]
        );
        Ok(())
    }
}
