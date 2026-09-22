//! 活动列表
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/activity/list.md)

use crate::{BpiError, BpiResult};
use serde::{Deserialize, Serialize};

const DEFAULT_PLATFORM_FILTER: &str = "1,3";
const DEFAULT_MOLD: u32 = 0;
const DEFAULT_HTTP_MODE: u32 = 3;
const DEFAULT_PAGE: u32 = 1;
const DEFAULT_PAGE_SIZE: u32 = 15;

/// 活动列表数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityListData {
    /// 活动列表
    pub list: Vec<ActivityItem>,
    /// 当前页码
    pub num: i32,
    /// 每页条数
    pub size: i32,
    /// 总条数
    pub total: i32,
}

/// 活动项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    /// 活动 ID
    pub id: i32,
    /// 固定值 1
    pub state: i32,
    /// 开始时间 UNIX 秒级时间戳
    pub stime: i64,
    /// 结束时间 UNIX 秒级时间戳
    pub etime: i64,
    /// 创建时间? UNIX 秒级时间戳, 可能为 0
    pub ctime: i64,
    /// 修改时间? UNIX 秒级时间戳, 可能为 0
    pub mtime: i64,
    /// 活动名称
    pub name: String,
    /// 活动链接
    pub h5_url: String,
    /// 活动封面
    pub h5_cover: String,
    /// 页面名称
    pub page_name: String,
    /// 活动平台类型? 即 URL 中 `plat` 参数
    pub plat: i32,
    /// 活动描述
    pub desc: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityListParams {
    plat: String,
    mold: u32,
    http: u32,
    pn: u32,
    ps: u32,
}

impl Default for ActivityListParams {
    fn default() -> Self {
        Self {
            plat: DEFAULT_PLATFORM_FILTER.to_string(),
            mold: DEFAULT_MOLD,
            http: DEFAULT_HTTP_MODE,
            pn: DEFAULT_PAGE,
            ps: DEFAULT_PAGE_SIZE,
        }
    }
}

impl ActivityListParams {
    /// 使用 Bilibili Web 默认值创建活动列表参数。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置平台过滤器，例如 `1,3`。
    pub fn platform_filter(mut self, plat: impl Into<String>) -> BpiResult<Self> {
        let plat = plat.into();
        validate_non_blank("plat", &plat)?;
        self.plat = plat;
        Ok(self)
    }

    /// 设置 API mold 标记。默认值为 `0`。
    pub fn mold(mut self, mold: u32) -> Self {
        self.mold = mold;
        self
    }

    /// 设置 API HTTP mode 标记。默认值为 `3`。
    pub fn http_mode(mut self, http: u32) -> Self {
        self.http = http;
        self
    }

    /// 设置页码。
    pub fn page(mut self, page: u32) -> BpiResult<Self> {
        self.pn = validate_positive("pn", page)?;
        Ok(self)
    }

    /// 设置每页数量。
    pub fn page_size(mut self, page_size: u32) -> BpiResult<Self> {
        self.ps = validate_positive("ps", page_size)?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("plat", self.plat.clone()),
            ("mold", self.mold.to_string()),
            ("http", self.http.to_string()),
            ("pn", self.pn.to_string()),
            ("ps", self.ps.to_string()),
        ]
    }
}

fn validate_non_blank(field: &'static str, value: &str) -> BpiResult<()> {
    if value.trim().is_empty() {
        return Err(BpiError::invalid_parameter(field, "value cannot be blank"));
    }

    Ok(())
}

fn validate_positive(field: &'static str, value: u32) -> BpiResult<u32> {
    if value == 0 {
        return Err(BpiError::invalid_parameter(field, "value must be non-zero"));
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/activity/list/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_activity_list() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");

        // 测试获取活动列表
        let params = ActivityListParams::new().page_size(4)?;
        let data = bpi.activity().list(params).await?;
        tracing::info!("{:#?}", data);

        assert!(!data.list.is_empty());
        assert_eq!(data.num, 1);
        assert_eq!(data.size, 4);
        assert!(data.total > 0);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_activity_list_simple() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");

        // 测试简化版本获取活动列表
        let data = bpi.activity().list_default().await?;
        tracing::info!("{:#?}", data);

        assert!(!data.list.is_empty());
        assert_eq!(data.num, 1);
        assert_eq!(data.size, 15);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_activity_item_fields() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");

        let params = ActivityListParams::new().page_size(1)?;
        let data = bpi.activity().list(params).await?;
        tracing::info!("{:#?}", data);

        if let Some(activity) = data.list.first() {
            assert!(activity.id > 0);
            assert_eq!(activity.state, 1);
            assert!(!activity.name.is_empty());
            assert!(!activity.page_name.is_empty());
        }

        Ok(())
    }

    #[test]
    fn activity_list_params_serializes_defaults() {
        let params = ActivityListParams::new();

        assert_eq!(
            params.query_pairs(),
            vec![
                ("plat", "1,3".to_string()),
                ("mold", "0".to_string()),
                ("http", "3".to_string()),
                ("pn", "1".to_string()),
                ("ps", "15".to_string()),
            ]
        );
    }

    #[test]
    fn activity_list_params_serializes_custom_values() -> Result<(), BpiError> {
        let params = ActivityListParams::new()
            .platform_filter("1")?
            .mold(2)
            .http_mode(4)
            .page(3)?
            .page_size(30)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("plat", "1".to_string()),
                ("mold", "2".to_string()),
                ("http", "4".to_string()),
                ("pn", "3".to_string()),
                ("ps", "30".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn activity_list_params_rejects_blank_platform_filter() {
        let err = ActivityListParams::new().platform_filter("  ").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "plat", .. }
        ));
    }

    #[test]
    fn activity_list_params_rejects_zero_page() {
        let err = ActivityListParams::new().page(0).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "pn", .. }
        ));
    }

    #[test]
    fn activity_list_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "activity.list");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/activity/page/list"
        );
        assert_eq!(
            contract.request.query.get("plat").map(String::as_str),
            Some(DEFAULT_PLATFORM_FILTER)
        );
        assert_eq!(
            contract.request.query.get("ps").map(String::as_str),
            Some("1")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("ActivityListData")
        );
        Ok(())
    }

    #[test]
    fn activity_list_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!("../../tests/contracts/activity/list/responses/anonymous.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/activity/list/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/activity/list/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<ActivityListData>::from_slice(bytes)?.into_payload()?;

            assert_eq!(payload.num, 1);
            assert_eq!(payload.size, 1);
            assert_eq!(payload.list.len(), 1);
        }
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/activity/public/list/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn activity_list_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<ActivityListData>>(body)?.into_payload()?;

            assert_eq!(payload.num, 1);
            assert_eq!(payload.size, 1);
            assert_eq!(payload.list.len(), 1);
        }
        Ok(())
    }
}
