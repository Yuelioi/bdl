use serde::{Deserialize, Serialize};

use crate::dynamic::serde_utils::deserialize_u64_from_string_or_number;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DynamicAllData {
    pub has_more: bool,
    pub items: Vec<DynamicItem>,
    pub offset: String,
    pub update_baseline: String,
    #[serde(deserialize_with = "deserialize_u64_from_string_or_number")]
    pub update_num: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DynamicItem {
    pub basic: Basic,
    pub id_str: String,
    pub modules: serde_json::Value,
    #[serde(rename = "type")]
    pub type_field: String,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Basic {
    pub comment_id_str: String,
    pub comment_type: i64,
    pub like_icon: serde_json::Value,
    pub rid_str: String,
    pub is_only_fans: Option<bool>,
    pub jump_url: Option<String>,
}

/// 检测新动态响应数据
#[derive(Debug, Clone, Deserialize)]
pub struct DynamicUpdateData {
    /// 新动态的数量
    pub update_num: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::params::{DynamicAllParams, DynamicCheckNewParams};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use std::collections::BTreeMap;
    use tracing::info;

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "all" => {
                include_bytes!("../../tests/contracts/dynamic/feed/all/contract.json").as_slice()
            }
            "check-new" => {
                include_bytes!("../../tests/contracts/dynamic/feed/check-new/contract.json")
                    .as_slice()
            }
            _ => unreachable!("unknown dynamic feed endpoint"),
        };

        EndpointContract::from_slice(bytes)
    }

    fn query_map(query: Vec<(&'static str, String)>) -> BTreeMap<String, String> {
        query
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect()
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_dynamic_get_all() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi.dynamic().all(DynamicAllParams::new()).await?;

        info!("成功获取 {} 条动态", data.items.len());

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_dynamic_check_new() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let update_baseline = "0";
        let data = bpi
            .dynamic()
            .check_new(DynamicCheckNewParams::new(update_baseline)?)
            .await?;

        info!("成功检测到 {} 条新动态", data.update_num);

        Ok(())
    }

    #[test]
    fn dynamic_feed_contracts_match_endpoint_requests() -> BpiResult<()> {
        let all = contract("all")?;
        assert_eq!(all.name, "dynamic.feed_all");
        assert_eq!(all.request.method, HttpMethod::Get);
        assert_eq!(
            all.request.url.as_str(),
            "https://api.bilibili.com/x/polymer/web-dynamic/v1/feed/all"
        );
        assert_eq!(
            all.request.query,
            query_map(DynamicAllParams::new().query_pairs())
        );
        assert_eq!(all.cases.len(), 3);
        assert_eq!(
            all.cases[0].response.error.as_deref(),
            Some("requires_login")
        );

        let check_new = contract("check-new")?;
        assert_eq!(check_new.name, "dynamic.feed_all_update");
        assert_eq!(check_new.request.method, HttpMethod::Get);
        assert_eq!(
            check_new.request.url.as_str(),
            "https://api.bilibili.com/x/polymer/web-dynamic/v1/feed/all/update"
        );
        assert_eq!(
            check_new.request.query,
            query_map(DynamicCheckNewParams::new("0")?.query_pairs())
        );
        assert_eq!(check_new.cases.len(), 3);
        assert_eq!(
            check_new.cases[0].response.error.as_deref(),
            Some("requires_login")
        );
        Ok(())
    }

    #[test]
    fn dynamic_feed_response_fixtures_parse_declared_models() -> BpiResult<()> {
        for bytes in [
            include_bytes!("../../tests/contracts/dynamic/feed/all/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/dynamic/feed/all/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<DynamicAllData>::from_slice(bytes)?.into_payload()?;
            assert_eq!(payload.items.len(), 1);
        }

        for bytes in [
            include_bytes!(
                "../../tests/contracts/dynamic/feed/check-new/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/dynamic/feed/check-new/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<DynamicUpdateData>::from_slice(bytes)?.into_payload()?;
            assert_eq!(payload.update_num, 0);
        }
        Ok(())
    }

    #[test]
    fn dynamic_feed_anonymous_fixtures_record_login_errors() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/dynamic/feed/all/responses/anonymous.requires_login.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/dynamic/feed/check-new/responses/anonymous.requires_login.json"
            )
            .as_slice(),
        ] {
            let err = ApiEnvelope::<serde_json::Value>::from_slice(bytes)?
                .ensure_success()
                .unwrap_err();
            assert_eq!(err.code(), Some(-101));
        }
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path = format!(
            "target/bpi-probe-runs/dynamic/feed-readonly/{endpoint}/{profile}.response.json"
        );
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn dynamic_feed_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["normal", "vip"] {
            if let Some(body) = local_probe_body("all", profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<DynamicAllData>>(body)?.into_payload()?;
                assert!(!payload.items.is_empty());
            }

            if let Some(body) = local_probe_body("check-new", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<DynamicUpdateData>>(body)?
                    .into_payload()?;
                let _ = payload.update_num;
            }
        }

        for endpoint in ["all", "check-new"] {
            if let Some(body) = local_probe_body(endpoint, "anonymous") {
                let err = serde_json::from_value::<ApiEnvelope<serde_json::Value>>(body)?
                    .ensure_success()
                    .unwrap_err();
                assert_eq!(err.code(), Some(-101));
            }
        }
        Ok(())
    }
}
