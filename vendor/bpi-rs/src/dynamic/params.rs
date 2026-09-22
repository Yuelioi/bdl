use crate::ids::{DynamicId, Mid};
use crate::{BpiError, BpiResult};

const DEFAULT_ALL_FEATURES: &str = "itemOpusStyle,listOnlyfans,opusBigCover,onlyfansVote,decorationCard,onlyfansAssetsV2,forwardListHidden,ugcDelete";
const DEFAULT_ALL_WEB_LOCATION: &str = "333.1365";
const DEFAULT_DETAIL_FEATURES: &str = "htmlNewStyle,itemOpusStyle,decorationCard";

/// `/x/polymer/web-dynamic/v1/feed/all` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicAllParams {
    features: String,
    web_location: String,
    host_mid: Option<Mid>,
    offset: Option<String>,
    update_baseline: Option<String>,
}

impl Default for DynamicAllParams {
    fn default() -> Self {
        Self {
            features: DEFAULT_ALL_FEATURES.to_string(),
            web_location: DEFAULT_ALL_WEB_LOCATION.to_string(),
            host_mid: None,
            offset: None,
            update_baseline: None,
        }
    }
}

impl DynamicAllParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_features(mut self, features: impl Into<String>) -> BpiResult<Self> {
        self.features = normalize_non_blank("features", features.into())?;
        Ok(self)
    }

    pub fn with_web_location(mut self, web_location: impl Into<String>) -> BpiResult<Self> {
        self.web_location = normalize_non_blank("web_location", web_location.into())?;
        Ok(self)
    }

    pub fn with_host_mid(mut self, host_mid: Mid) -> Self {
        self.host_mid = Some(host_mid);
        self
    }

    pub fn with_offset(mut self, offset: impl Into<String>) -> BpiResult<Self> {
        self.offset = Some(normalize_non_blank("offset", offset.into())?);
        Ok(self)
    }

    pub fn with_update_baseline(mut self, update_baseline: impl Into<String>) -> BpiResult<Self> {
        self.update_baseline = Some(normalize_non_blank(
            "update_baseline",
            update_baseline.into(),
        )?);
        Ok(self)
    }

    pub fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut query = vec![
            ("features", self.features.clone()),
            ("web_location", self.web_location.clone()),
        ];

        if let Some(host_mid) = self.host_mid {
            query.push(("host_mid", host_mid.to_string()));
        }
        if let Some(offset) = &self.offset {
            query.push(("offset", offset.clone()));
        }
        if let Some(update_baseline) = &self.update_baseline {
            query.push(("update_baseline", update_baseline.clone()));
        }

        query
    }
}

/// `/x/polymer/web-dynamic/v1/feed/all/update` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicCheckNewParams {
    update_baseline: String,
    typ: Option<String>,
}

impl DynamicCheckNewParams {
    pub fn new(update_baseline: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            update_baseline: normalize_non_blank("update_baseline", update_baseline.into())?,
            typ: None,
        })
    }

    pub fn with_type(mut self, typ: impl Into<String>) -> BpiResult<Self> {
        self.typ = Some(normalize_non_blank("type", typ.into())?);
        Ok(self)
    }

    pub fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut query = vec![("update_baseline", self.update_baseline.clone())];
        if let Some(typ) = &self.typ {
            query.push(("type", typ.clone()));
        }

        query
    }
}

/// `/x/polymer/web-dynamic/v1/feed/nav` 的参数。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DynamicNavFeedParams {
    update_baseline: Option<String>,
    offset: Option<String>,
}

impl DynamicNavFeedParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_update_baseline(mut self, update_baseline: impl Into<String>) -> BpiResult<Self> {
        self.update_baseline = Some(normalize_non_blank(
            "update_baseline",
            update_baseline.into(),
        )?);
        Ok(self)
    }

    pub fn with_offset(mut self, offset: impl Into<String>) -> BpiResult<Self> {
        self.offset = Some(normalize_non_blank("offset", offset.into())?);
        Ok(self)
    }

    pub fn query_pairs(&self) -> Vec<(&'static str, String)> {
        optional_cursor_query(self.update_baseline.as_deref(), self.offset.as_deref())
    }
}

/// `/dynamic_svr/v1/dynamic_svr/w_live_users` 的参数。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DynamicLiveUsersParams {
    size: Option<u32>,
}

impl DynamicLiveUsersParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_size(mut self, size: u32) -> BpiResult<Self> {
        if size == 0 {
            return Err(BpiError::invalid_parameter(
                "size",
                "value must be non-zero",
            ));
        }

        self.size = Some(size);
        Ok(self)
    }

    pub fn query_pairs(&self) -> Vec<(&'static str, String)> {
        self.size
            .map(|size| vec![("size", size.to_string())])
            .unwrap_or_default()
    }
}

/// `/dynamic_svr/v1/dynamic_svr/w_dyn_uplist` 的参数。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DynamicUpUsersParams {
    teenagers_mode: bool,
}

impl DynamicUpUsersParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_teenagers_mode(mut self, teenagers_mode: bool) -> Self {
        self.teenagers_mode = teenagers_mode;
        self
    }

    pub fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("teenagers_mode", u8::from(self.teenagers_mode).to_string())]
    }
}

/// `/x/polymer/web-dynamic/v1/detail` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicDetailParams {
    id: DynamicId,
    features: String,
}

impl DynamicDetailParams {
    pub fn new(id: DynamicId) -> Self {
        Self {
            id,
            features: DEFAULT_DETAIL_FEATURES.to_string(),
        }
    }

    pub fn with_features(mut self, features: impl Into<String>) -> BpiResult<Self> {
        self.features = normalize_non_blank("features", features.into())?;
        Ok(self)
    }

    pub fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("id", self.id.to_string()),
            ("features", self.features.clone()),
        ]
    }
}

/// `/x/polymer/web-dynamic/v1/detail/reaction` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicReactionsParams {
    id: DynamicId,
    offset: Option<String>,
}

impl DynamicReactionsParams {
    pub fn new(id: DynamicId) -> Self {
        Self { id, offset: None }
    }

    pub fn with_offset(mut self, offset: impl Into<String>) -> BpiResult<Self> {
        self.offset = Some(normalize_non_blank("offset", offset.into())?);
        Ok(self)
    }

    pub fn query_pairs(&self) -> Vec<(&'static str, String)> {
        dynamic_offset_query(&self.id, self.offset.as_deref())
    }
}

/// `/lottery_svr/v1/lottery_svr/lottery_notice` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicLotteryNoticeParams {
    business_id: DynamicId,
}

impl DynamicLotteryNoticeParams {
    pub fn new(business_id: DynamicId) -> Self {
        Self { business_id }
    }

    pub fn query_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("business_id", self.business_id.to_string()),
            ("business_type", "1".to_string()),
            ("csrf", csrf.to_string()),
        ]
    }
}

/// `/x/polymer/web-dynamic/v1/detail/forward` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicForwardsParams {
    id: DynamicId,
    offset: Option<String>,
}

impl DynamicForwardsParams {
    pub fn new(id: DynamicId) -> Self {
        Self { id, offset: None }
    }

    pub fn with_offset(mut self, offset: impl Into<String>) -> BpiResult<Self> {
        self.offset = Some(normalize_non_blank("offset", offset.into())?);
        Ok(self)
    }

    pub fn query_pairs(&self) -> Vec<(&'static str, String)> {
        dynamic_offset_query(&self.id, self.offset.as_deref())
    }
}

/// `/x/polymer/web-dynamic/v1/detail/pic` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicPicsParams {
    id: DynamicId,
}

impl DynamicPicsParams {
    pub fn new(id: DynamicId) -> Self {
        Self { id }
    }

    pub fn query_pairs(&self) -> [(&'static str, String); 1] {
        [("id", self.id.to_string())]
    }
}

/// `/x/polymer/web-dynamic/v1/detail/forward/item` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicForwardItemParams {
    id: DynamicId,
}

impl DynamicForwardItemParams {
    pub fn new(id: DynamicId) -> Self {
        Self { id }
    }

    pub fn query_pairs(&self) -> [(&'static str, String); 1] {
        [("id", self.id.to_string())]
    }
}

fn dynamic_offset_query(id: &DynamicId, offset: Option<&str>) -> Vec<(&'static str, String)> {
    let mut query = vec![("id", id.to_string())];
    if let Some(offset) = offset {
        query.push(("offset", offset.to_string()));
    }
    query
}

fn optional_cursor_query(
    update_baseline: Option<&str>,
    offset: Option<&str>,
) -> Vec<(&'static str, String)> {
    let mut query = Vec::new();
    if let Some(update_baseline) = update_baseline {
        query.push(("update_baseline", update_baseline.to_string()));
    }
    if let Some(offset) = offset {
        query.push(("offset", offset.to_string()));
    }

    query
}

fn normalize_non_blank(field: &'static str, value: String) -> BpiResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(BpiError::invalid_parameter(field, "value cannot be blank"));
    }

    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_params_serializes_defaults() {
        let params = DynamicAllParams::new();

        assert_eq!(
            params.query_pairs(),
            vec![
                (
                    "features",
                    "itemOpusStyle,listOnlyfans,opusBigCover,onlyfansVote,decorationCard,onlyfansAssetsV2,forwardListHidden,ugcDelete".to_string(),
                ),
                ("web_location", "333.1365".to_string()),
            ]
        );
    }

    #[test]
    fn all_params_serializes_optional_filters() -> BpiResult<()> {
        let params = DynamicAllParams::new()
            .with_host_mid(Mid::new(12345)?)
            .with_offset("offset-token")?
            .with_update_baseline("baseline-token")?;

        assert_eq!(
            params.query_pairs(),
            vec![
                (
                    "features",
                    "itemOpusStyle,listOnlyfans,opusBigCover,onlyfansVote,decorationCard,onlyfansAssetsV2,forwardListHidden,ugcDelete".to_string(),
                ),
                ("web_location", "333.1365".to_string()),
                ("host_mid", "12345".to_string()),
                ("offset", "offset-token".to_string()),
                ("update_baseline", "baseline-token".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn all_params_rejects_blank_offset() {
        let err = DynamicAllParams::new().with_offset("   ").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "offset",
                ..
            }
        ));
    }

    #[test]
    fn check_new_params_serializes_required_baseline() -> BpiResult<()> {
        let params = DynamicCheckNewParams::new("baseline-token")?;

        assert_eq!(
            params.query_pairs(),
            vec![("update_baseline", "baseline-token".to_string())]
        );
        Ok(())
    }

    #[test]
    fn check_new_params_serializes_type_filter() -> BpiResult<()> {
        let params = DynamicCheckNewParams::new("baseline-token")?.with_type("video")?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("update_baseline", "baseline-token".to_string()),
                ("type", "video".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn nav_feed_params_serializes_optional_cursor() -> BpiResult<()> {
        let params = DynamicNavFeedParams::new()
            .with_update_baseline("baseline-token")?
            .with_offset("offset-token")?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("update_baseline", "baseline-token".to_string()),
                ("offset", "offset-token".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn live_users_params_serializes_size() -> BpiResult<()> {
        let params = DynamicLiveUsersParams::new().with_size(20)?;

        assert_eq!(params.query_pairs(), vec![("size", "20".to_string())]);
        Ok(())
    }

    #[test]
    fn live_users_params_rejects_zero_size() {
        let err = DynamicLiveUsersParams::new().with_size(0).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "size", .. }
        ));
    }

    #[test]
    fn up_users_params_serializes_default_teenagers_mode() {
        let params = DynamicUpUsersParams::new();

        assert_eq!(
            params.query_pairs(),
            vec![("teenagers_mode", "0".to_string())]
        );
    }

    #[test]
    fn up_users_params_serializes_enabled_teenagers_mode() {
        let params = DynamicUpUsersParams::new().with_teenagers_mode(true);

        assert_eq!(
            params.query_pairs(),
            vec![("teenagers_mode", "1".to_string())]
        );
    }
}
