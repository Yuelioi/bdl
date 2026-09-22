use std::str::FromStr;

use crate::{BpiError, BpiResult};

/// `/x/web-interface/popular` 的参数。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VideoPopularListParams {
    page: Option<u32>,
    page_size: Option<u32>,
}

impl VideoPopularListParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.page = Some(validate_positive("pn", page)?);
        Ok(self)
    }

    pub fn with_page_size(mut self, page_size: u32) -> BpiResult<Self> {
        self.page_size = Some(validate_positive("ps", page_size)?);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();

        if let Some(page) = self.page {
            pairs.push(("pn", page.to_string()));
        }
        if let Some(page_size) = self.page_size {
            pairs.push(("ps", page_size.to_string()));
        }

        pairs
    }
}

/// `/x/web-interface/popular/series/one` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PopularSeriesOneParams {
    number: u32,
}

impl PopularSeriesOneParams {
    pub fn new(number: u32) -> BpiResult<Self> {
        Ok(Self {
            number: validate_positive("number", number)?,
        })
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("number", self.number.to_string())]
    }
}

/// `/x/web-interface/ranking/v2` 接受的排行类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoRankingType {
    All,
    Rookie,
    Origin,
}

impl VideoRankingType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Rookie => "rookie",
            Self::Origin => "origin",
        }
    }
}

impl FromStr for VideoRankingType {
    type Err = BpiError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "all" => Ok(Self::All),
            "rookie" => Ok(Self::Rookie),
            "origin" => Ok(Self::Origin),
            _ => Err(BpiError::invalid_parameter(
                "type",
                "supported ranking types are all, rookie, and origin",
            )),
        }
    }
}

/// `/x/web-interface/ranking/v2` 的参数。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct VideoRankingListParams {
    rid: Option<u32>,
    ranking_type: Option<VideoRankingType>,
}

impl VideoRankingListParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_rid(mut self, rid: u32) -> BpiResult<Self> {
        self.rid = Some(validate_positive("rid", rid)?);
        Ok(self)
    }

    pub fn with_type(mut self, ranking_type: VideoRankingType) -> Self {
        self.ranking_type = Some(ranking_type);
        self
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();

        if let Some(rid) = self.rid {
            pairs.push(("rid", rid.to_string()));
        }
        if let Some(ranking_type) = self.ranking_type {
            pairs.push(("type", ranking_type.as_str().to_string()));
        }

        pairs
    }
}

/// `/x/web-interface/dynamic/region` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoRegionDynamicParams {
    rid: u32,
    page: Option<u32>,
    page_size: Option<u32>,
}

impl VideoRegionDynamicParams {
    pub fn new(rid: u32) -> BpiResult<Self> {
        Ok(Self {
            rid: validate_positive("rid", rid)?,
            page: None,
            page_size: None,
        })
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.page = Some(validate_positive("pn", page)?);
        Ok(self)
    }

    pub fn with_page_size(mut self, page_size: u32) -> BpiResult<Self> {
        self.page_size = Some(validate_positive("ps", page_size)?);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = vec![("rid", self.rid.to_string())];
        append_optional_pagination(&mut pairs, self.page, self.page_size);
        pairs
    }
}

/// `/x/web-interface/dynamic/tag` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoRegionTagDynamicParams {
    rid: u32,
    tag_id: u64,
    page: Option<u32>,
    page_size: Option<u32>,
}

impl VideoRegionTagDynamicParams {
    pub fn new(rid: u32, tag_id: u64) -> BpiResult<Self> {
        Ok(Self {
            rid: validate_positive("rid", rid)?,
            tag_id: validate_positive_u64("tag_id", tag_id)?,
            page: None,
            page_size: None,
        })
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.page = Some(validate_positive("pn", page)?);
        Ok(self)
    }

    pub fn with_page_size(mut self, page_size: u32) -> BpiResult<Self> {
        self.page_size = Some(validate_positive("ps", page_size)?);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = vec![
            ("rid", self.rid.to_string()),
            ("tag_id", self.tag_id.to_string()),
        ];
        append_optional_pagination(&mut pairs, self.page, self.page_size);
        pairs
    }
}

/// `/x/web-interface/newlist` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoRegionNewListParams {
    rid: u32,
    page: Option<u32>,
    page_size: Option<u32>,
    typ: Option<u32>,
}

impl VideoRegionNewListParams {
    pub fn new(rid: u32) -> BpiResult<Self> {
        Ok(Self {
            rid: validate_positive("rid", rid)?,
            page: None,
            page_size: None,
            typ: None,
        })
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.page = Some(validate_positive("pn", page)?);
        Ok(self)
    }

    pub fn with_page_size(mut self, page_size: u32) -> BpiResult<Self> {
        self.page_size = Some(validate_positive("ps", page_size)?);
        Ok(self)
    }

    pub fn with_type(mut self, typ: u32) -> BpiResult<Self> {
        self.typ = Some(validate_positive("type", typ)?);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = vec![("rid", self.rid.to_string())];
        append_optional_pagination(&mut pairs, self.page, self.page_size);
        if let Some(typ) = self.typ {
            pairs.push(("type", typ.to_string()));
        }
        pairs
    }
}

/// `/x/web-interface/newlist_rank` 接受的排序方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoNewListRankOrder {
    Click,
    Scores,
    Pubdate,
}

impl VideoNewListRankOrder {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Click => "click",
            Self::Scores => "scores",
            Self::Pubdate => "pubdate",
        }
    }
}

impl FromStr for VideoNewListRankOrder {
    type Err = BpiError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "click" => Ok(Self::Click),
            "scores" => Ok(Self::Scores),
            "pubdate" => Ok(Self::Pubdate),
            _ => Err(BpiError::invalid_parameter(
                "order",
                "supported newlist rank orders are click, scores, and pubdate",
            )),
        }
    }
}

/// `/x/web-interface/newlist_rank` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoRegionNewListRankParams {
    cate_id: u32,
    order: Option<VideoNewListRankOrder>,
    page: Option<u32>,
    page_size: u32,
    time_from: String,
    time_to: String,
}

impl VideoRegionNewListRankParams {
    pub fn new(
        cate_id: u32,
        page_size: u32,
        time_from: impl Into<String>,
        time_to: impl Into<String>,
    ) -> BpiResult<Self> {
        let time_from = time_from.into();
        let time_to = time_to.into();
        validate_non_blank("time_from", &time_from)?;
        validate_non_blank("time_to", &time_to)?;

        Ok(Self {
            cate_id: validate_positive("cate_id", cate_id)?,
            order: None,
            page: None,
            page_size: validate_positive("pagesize", page_size)?,
            time_from,
            time_to,
        })
    }

    pub fn with_order(mut self, order: VideoNewListRankOrder) -> Self {
        self.order = Some(order);
        self
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.page = Some(validate_positive("page", page)?);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = vec![
            ("search_type", "video".to_string()),
            ("view_type", "hot_rank".to_string()),
            ("cate_id", self.cate_id.to_string()),
            ("pagesize", self.page_size.to_string()),
            ("time_from", self.time_from.clone()),
            ("time_to", self.time_to.clone()),
        ];

        if let Some(order) = self.order {
            pairs.push(("order", order.as_str().to_string()));
        }
        if let Some(page) = self.page {
            pairs.push(("page", page.to_string()));
        }

        pairs
    }
}

fn append_optional_pagination(
    pairs: &mut Vec<(&'static str, String)>,
    page: Option<u32>,
    page_size: Option<u32>,
) {
    if let Some(page) = page {
        pairs.push(("pn", page.to_string()));
    }
    if let Some(page_size) = page_size {
        pairs.push(("ps", page_size.to_string()));
    }
}

fn validate_positive(field: &'static str, value: u32) -> BpiResult<u32> {
    if value == 0 {
        return Err(BpiError::invalid_parameter(field, "value must be non-zero"));
    }

    Ok(value)
}

fn validate_positive_u64(field: &'static str, value: u64) -> BpiResult<u64> {
    if value == 0 {
        return Err(BpiError::invalid_parameter(field, "value must be non-zero"));
    }

    Ok(value)
}

fn validate_non_blank(field: &'static str, value: &str) -> BpiResult<()> {
    if value.trim().is_empty() {
        return Err(BpiError::invalid_parameter(field, "value cannot be blank"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn popular_list_params_serializes_empty_defaults() {
        let params = VideoPopularListParams::new();

        assert!(params.query_pairs().is_empty());
    }

    #[test]
    fn popular_list_params_serializes_pagination() -> BpiResult<()> {
        let params = VideoPopularListParams::new()
            .with_page(1)?
            .with_page_size(2)?;

        assert_eq!(
            params.query_pairs(),
            vec![("pn", "1".to_string()), ("ps", "2".to_string())]
        );
        Ok(())
    }

    #[test]
    fn popular_series_one_params_rejects_zero_number() {
        let err = PopularSeriesOneParams::new(0).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "number",
                ..
            }
        ));
    }

    #[test]
    fn ranking_type_parses_supported_values() -> BpiResult<()> {
        assert_eq!("all".parse::<VideoRankingType>()?, VideoRankingType::All);
        assert_eq!(
            "rookie".parse::<VideoRankingType>()?,
            VideoRankingType::Rookie
        );
        assert_eq!(
            "origin".parse::<VideoRankingType>()?,
            VideoRankingType::Origin
        );
        Ok(())
    }

    #[test]
    fn ranking_list_params_serializes_filters() -> BpiResult<()> {
        let params = VideoRankingListParams::new()
            .with_rid(21)?
            .with_type(VideoRankingType::Rookie);

        assert_eq!(
            params.query_pairs(),
            vec![("rid", "21".to_string()), ("type", "rookie".to_string())]
        );
        Ok(())
    }

    #[test]
    fn region_dynamic_params_serializes_pagination() -> BpiResult<()> {
        let params = VideoRegionDynamicParams::new(21)?
            .with_page(1)?
            .with_page_size(2)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("rid", "21".to_string()),
                ("pn", "1".to_string()),
                ("ps", "2".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn region_tag_dynamic_params_serializes_required_values() -> BpiResult<()> {
        let params = VideoRegionTagDynamicParams::new(136, 10026108)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("rid", "136".to_string()),
                ("tag_id", "10026108".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn region_newlist_params_serializes_type_filter() -> BpiResult<()> {
        let params = VideoRegionNewListParams::new(231)?.with_type(1)?;

        assert_eq!(
            params.query_pairs(),
            vec![("rid", "231".to_string()), ("type", "1".to_string())]
        );
        Ok(())
    }

    #[test]
    fn newlist_rank_params_serializes_required_and_optional_values() -> BpiResult<()> {
        let params = VideoRegionNewListRankParams::new(231, 2, "20260701", "20260703")?
            .with_order(VideoNewListRankOrder::Click)
            .with_page(1)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("search_type", "video".to_string()),
                ("view_type", "hot_rank".to_string()),
                ("cate_id", "231".to_string()),
                ("pagesize", "2".to_string()),
                ("time_from", "20260701".to_string()),
                ("time_to", "20260703".to_string()),
                ("order", "click".to_string()),
                ("page", "1".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn newlist_rank_params_rejects_blank_time() {
        let err = VideoRegionNewListRankParams::new(231, 2, " ", "20260703").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "time_from",
                ..
            }
        ));
    }
}
