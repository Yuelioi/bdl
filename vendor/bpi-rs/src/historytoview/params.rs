use std::str::FromStr;

use crate::{BpiError, BpiResult};

/// `/x/web-interface/history/cursor` 接受的业务分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryBusiness {
    Archive,
    Pgc,
    Live,
    ArticleList,
    Article,
}

impl HistoryBusiness {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Archive => "archive",
            Self::Pgc => "pgc",
            Self::Live => "live",
            Self::ArticleList => "article-list",
            Self::Article => "article",
        }
    }
}

impl FromStr for HistoryBusiness {
    type Err = BpiError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "archive" => Ok(Self::Archive),
            "pgc" => Ok(Self::Pgc),
            "live" => Ok(Self::Live),
            "article-list" => Ok(Self::ArticleList),
            "article" => Ok(Self::Article),
            _ => Err(BpiError::invalid_parameter(
                "business",
                "supported history business values are archive, pgc, live, article-list, and article",
            )),
        }
    }
}

/// `/x/web-interface/history/cursor` 接受的 tab 过滤器。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryListType {
    All,
    Archive,
    Live,
    Article,
}

impl HistoryListType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Archive => "archive",
            Self::Live => "live",
            Self::Article => "article",
        }
    }
}

impl FromStr for HistoryListType {
    type Err = BpiError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim() {
            "all" => Ok(Self::All),
            "archive" => Ok(Self::Archive),
            "live" => Ok(Self::Live),
            "article" => Ok(Self::Article),
            _ => Err(BpiError::invalid_parameter(
                "type",
                "supported history list types are all, archive, live, and article",
            )),
        }
    }
}

/// `/x/web-interface/history/cursor` 的参数。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HistoryListParams {
    max: Option<u64>,
    business: Option<String>,
    view_at: Option<u64>,
    typ: Option<String>,
    page_size: Option<u32>,
}

impl HistoryListParams {
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置上一游标返回的分页目标 id。
    pub fn with_max(mut self, max: u64) -> Self {
        self.max = Some(max);
        self
    }

    /// 设置已知的 Bilibili 业务分类。
    pub fn with_business(mut self, business: HistoryBusiness) -> Self {
        self.business = Some(business.as_str().to_string());
        self
    }

    /// 为前向兼容调用方设置原始业务分类。
    pub fn with_raw_business(mut self, business: impl Into<String>) -> BpiResult<Self> {
        let business = normalize_non_blank("business", business.into())?;
        self.business = Some(business);
        Ok(self)
    }

    /// 设置上一游标返回的分页时间戳。
    pub fn with_view_at(mut self, view_at: u64) -> Self {
        self.view_at = Some(view_at);
        self
    }

    /// 设置已知 tab 过滤器。
    pub fn with_type(mut self, typ: HistoryListType) -> Self {
        self.typ = Some(typ.as_str().to_string());
        self
    }

    /// 为前向兼容调用方设置原始 tab 过滤器。
    pub fn with_raw_type(mut self, typ: impl Into<String>) -> BpiResult<Self> {
        let typ = normalize_non_blank("type", typ.into())?;
        self.typ = Some(typ);
        Ok(self)
    }

    /// 设置请求的每页数量。
    pub fn with_page_size(mut self, page_size: u32) -> BpiResult<Self> {
        if page_size == 0 {
            return Err(BpiError::invalid_parameter(
                "ps",
                "page size must be non-zero",
            ));
        }

        self.page_size = Some(page_size);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();

        if let Some(max) = self.max {
            pairs.push(("max", max.to_string()));
        }
        if let Some(business) = &self.business {
            pairs.push(("business", business.to_string()));
        }
        if let Some(view_at) = self.view_at {
            pairs.push(("view_at", view_at.to_string()));
        }
        if let Some(typ) = &self.typ {
            pairs.push(("type", typ.to_string()));
        }
        if let Some(page_size) = self.page_size {
            pairs.push(("ps", page_size.to_string()));
        }

        pairs
    }
}

/// `/x/v2/history/delete` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryDeleteParams {
    kid: String,
}

impl HistoryDeleteParams {
    pub fn new(kid: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            kid: normalize_non_blank("kid", kid.into())?,
        })
    }

    pub(crate) fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![("kid", self.kid.clone()), ("csrf", csrf.to_string())]
    }
}

/// `/x/v2/history/shadow/set` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HistoryShadowSetParams {
    switch: bool,
}

impl HistoryShadowSetParams {
    pub fn new(switch: bool) -> Self {
        Self { switch }
    }

    pub(crate) fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("switch", self.switch.to_string()),
            ("csrf", csrf.to_string()),
        ]
    }
}

/// `/x/v2/history/toview/add` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToViewAddParams {
    aid: Option<u64>,
    bvid: Option<String>,
}

impl ToViewAddParams {
    pub fn new(aid: Option<u64>, bvid: Option<String>) -> BpiResult<Self> {
        let aid = match aid {
            Some(0) => {
                return Err(BpiError::invalid_parameter("aid", "id must be non-zero"));
            }
            Some(aid) => Some(aid),
            None => None,
        };
        let bvid = match bvid {
            Some(bvid) => Some(normalize_non_blank("bvid", bvid)?),
            None => None,
        };

        if aid.is_none() && bvid.is_none() {
            return Err(BpiError::invalid_parameter(
                "video_id",
                "aid or bvid is required",
            ));
        }

        Ok(Self { aid, bvid })
    }

    pub(crate) fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        let mut pairs = vec![("csrf", csrf.to_string())];
        if let Some(aid) = self.aid {
            pairs.push(("aid", aid.to_string()));
        }
        if let Some(bvid) = &self.bvid {
            pairs.push(("bvid", bvid.clone()));
        }
        pairs
    }
}

/// `/x/v2/history/toview/del` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToViewDeleteParams {
    aid: Option<u64>,
    viewed: Option<bool>,
}

impl ToViewDeleteParams {
    pub fn new(aid: Option<u64>, viewed: Option<bool>) -> BpiResult<Self> {
        let aid = match aid {
            Some(0) => {
                return Err(BpiError::invalid_parameter("aid", "id must be non-zero"));
            }
            Some(aid) => Some(aid),
            None => None,
        };

        if aid.is_none() && viewed.is_none() {
            return Err(BpiError::invalid_parameter(
                "selector",
                "aid or viewed is required",
            ));
        }

        Ok(Self { aid, viewed })
    }

    pub(crate) fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        let mut pairs = vec![("csrf", csrf.to_string())];
        if let Some(aid) = self.aid {
            pairs.push(("aid", aid.to_string()));
        }
        if let Some(viewed) = self.viewed {
            pairs.push(("viewed", viewed.to_string()));
        }
        pairs
    }
}

fn normalize_non_blank(field: &'static str, value: String) -> BpiResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BpiError::invalid_parameter(field, "value cannot be blank"));
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_business_parses_supported_values() -> BpiResult<()> {
        assert_eq!(
            "archive".parse::<HistoryBusiness>()?,
            HistoryBusiness::Archive
        );
        assert_eq!("pgc".parse::<HistoryBusiness>()?, HistoryBusiness::Pgc);
        assert_eq!("live".parse::<HistoryBusiness>()?, HistoryBusiness::Live);
        assert_eq!(
            "article-list".parse::<HistoryBusiness>()?,
            HistoryBusiness::ArticleList
        );
        assert_eq!(
            "article".parse::<HistoryBusiness>()?,
            HistoryBusiness::Article
        );
        Ok(())
    }

    #[test]
    fn history_business_rejects_unknown_value() {
        let err = HistoryBusiness::from_str("unknown").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "business",
                ..
            }
        ));
    }

    #[test]
    fn history_list_type_parses_supported_values() -> BpiResult<()> {
        assert_eq!("all".parse::<HistoryListType>()?, HistoryListType::All);
        assert_eq!(
            "archive".parse::<HistoryListType>()?,
            HistoryListType::Archive
        );
        assert_eq!("live".parse::<HistoryListType>()?, HistoryListType::Live);
        assert_eq!(
            "article".parse::<HistoryListType>()?,
            HistoryListType::Article
        );
        Ok(())
    }

    #[test]
    fn history_list_params_serializes_empty_defaults() {
        let params = HistoryListParams::new();

        assert!(params.query_pairs().is_empty());
    }

    #[test]
    fn history_list_params_serializes_optional_filters() -> BpiResult<()> {
        let params = HistoryListParams::new()
            .with_max(1001)
            .with_business(HistoryBusiness::Archive)
            .with_view_at(1_700_000_000)
            .with_type(HistoryListType::All)
            .with_page_size(20)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("max", "1001".to_string()),
                ("business", "archive".to_string()),
                ("view_at", "1700000000".to_string()),
                ("type", "all".to_string()),
                ("ps", "20".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn history_list_params_trims_raw_filters() -> BpiResult<()> {
        let params = HistoryListParams::new()
            .with_raw_business(" custom-business ")?
            .with_raw_type(" custom-type ")?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("business", "custom-business".to_string()),
                ("type", "custom-type".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn history_list_params_rejects_blank_raw_business() {
        let err = HistoryListParams::new()
            .with_raw_business("  ")
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "business",
                ..
            }
        ));
    }

    #[test]
    fn history_list_params_rejects_zero_page_size() {
        let err = HistoryListParams::new().with_page_size(0).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "ps", .. }
        ));
    }

    #[test]
    fn history_delete_params_rejects_blank_kid() {
        let err = HistoryDeleteParams::new(" ").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "kid", .. }
        ));
    }

    #[test]
    fn toview_add_params_requires_video_id() {
        let err = ToViewAddParams::new(None, None).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "video_id",
                ..
            }
        ));
    }

    #[test]
    fn toview_delete_params_requires_delete_selector() {
        let err = ToViewDeleteParams::new(None, None).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "selector",
                ..
            }
        ));
    }

    #[test]
    fn history_shadow_set_params_serializes_bool() {
        let params = HistoryShadowSetParams::new(true);

        assert_eq!(
            params.form_pairs("csrf-token"),
            vec![
                ("switch", "true".to_string()),
                ("csrf", "csrf-token".to_string()),
            ]
        );
    }
}
