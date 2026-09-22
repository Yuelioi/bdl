use serde::{Deserialize, Serialize};

use crate::dynamic::serde_utils::deserialize_u64_from_string_or_number;

// --- 导航栏动态 API 结构体 ---

/// 导航栏动态列表项的 UP 主信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicNavAuthor {
    /// UP 主头像 URL
    pub face: String,
    /// UP 主 mid (UID)
    #[serde(deserialize_with = "deserialize_u64_from_string_or_number")]
    pub mid: u64,
    /// UP 主昵称
    pub name: String,
}

/// 导航栏动态列表项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicNavItem {
    /// UP 主信息
    pub author: DynamicNavAuthor,
    /// 封面 URL
    pub cover: String,
    /// 动态 ID 字符串
    pub id_str: String,
    /// 发布时间（文字表述的相对时间）
    pub pub_time: String,
    /// 关联 ID，视频即 aid
    #[serde(deserialize_with = "deserialize_u64_from_string_or_number")]
    pub rid: u64,
    /// 标题
    pub title: String,
    /// 动态类型，8 表示视频
    #[serde(rename = "type")]
    pub type_num: u8,
    /// 是否可见
    pub visible: bool,
}

/// 导航栏动态列表响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicNavData {
    /// 是否有更多数据
    pub has_more: bool,
    /// 动态数据数组
    pub items: Vec<DynamicNavItem>,
    /// 偏移量，用于翻页
    pub offset: String,
    /// 更新基线，用于获取新动态
    pub update_baseline: String,
    /// 本次获取到的新动态条数
    #[serde(deserialize_with = "deserialize_u64_from_string_or_number")]
    pub update_num: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::params::DynamicNavFeedParams;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use std::collections::BTreeMap;
    use tracing::info;

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/dynamic/feed/nav/contract.json"
        ))
    }

    fn query_map(query: Vec<(&'static str, String)>) -> BTreeMap<String, String> {
        query
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect()
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_dynamic_nav_feed() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi.dynamic().nav_feed(DynamicNavFeedParams::new()).await?;

        info!("获取到 {} 条动态", data.items.len());
        info!("第一条动态ID: {}", data.items[0].id_str);

        assert!(!data.items.is_empty());

        Ok(())
    }

    #[test]
    fn dynamic_nav_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "dynamic.feed_nav");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/polymer/web-dynamic/v1/feed/nav"
        );
        assert_eq!(
            contract.request.query,
            query_map(DynamicNavFeedParams::new().query_pairs())
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.error.as_deref(),
            Some("requires_login")
        );
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("DynamicNavData")
        );
        Ok(())
    }

    #[test]
    fn dynamic_nav_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!("../../tests/contracts/dynamic/feed/nav/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/dynamic/feed/nav/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<DynamicNavData>::from_slice(bytes)?.into_payload()?;
            assert_eq!(payload.items.len(), 1);
            assert_eq!(payload.items[0].author.mid, 1);
            assert_eq!(payload.update_num, 0);
        }
        Ok(())
    }

    #[test]
    fn dynamic_nav_anonymous_fixture_records_login_error() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/dynamic/feed/nav/responses/anonymous.requires_login.json"
        ))?
        .ensure_success()
        .unwrap_err();

        assert_eq!(err.code(), Some(-101));
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/dynamic/feed-readonly/nav-feed/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn dynamic_nav_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<DynamicNavData>>(body)?.into_payload()?;
            assert!(!payload.items.is_empty());
        }

        if let Some(body) = local_probe_body("anonymous") {
            let err = serde_json::from_value::<ApiEnvelope<serde_json::Value>>(body)?
                .ensure_success()
                .unwrap_err();
            assert_eq!(err.code(), Some(-101));
        }
        Ok(())
    }
}
