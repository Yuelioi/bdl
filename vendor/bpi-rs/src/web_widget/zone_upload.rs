use serde::Deserialize;
use std::collections::HashMap;

/// 分区当日投稿稿件数信息
/// 使用 `HashMap<u64, u64>` 存储，键为分区 ID，值为当日投稿数。
#[derive(Debug, Clone, Deserialize)]
pub struct OnlineRegionCount(pub HashMap<String, u64>);

/// 分区当日投稿数数据
#[derive(Debug, Clone, Deserialize)]
pub struct OnlineData {
    pub region_count: OnlineRegionCount,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiResult};
    use tracing::info;

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/web_widget/online/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_online() {
        let bpi = BpiClient::new().expect("client should build");
        let resp = bpi.web_widget().online().await;
        info!("响应: {:?}", resp);
        assert!(resp.is_ok());

        if let Ok(data) = resp {
            for (region_id, count) in data.region_count.0 {
                info!("分区ID: {}, 投稿数: {}", region_id, count);
            }
        }
    }

    #[test]
    fn web_widget_online_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "web_widget.online");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/web-interface/online"
        );
        assert!(contract.request.query.is_empty());
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("OnlineData")
        );
        Ok(())
    }

    #[test]
    fn web_widget_online_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/web_widget/online/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!("../../tests/contracts/web_widget/online/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/web_widget/online/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<OnlineData>::from_slice(bytes)?.into_payload()?;

            assert!(!payload.region_count.0.is_empty());
        }
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/web_widget/public/online/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn web_widget_online_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<OnlineData>>(body)?.into_payload()?;

            assert!(!payload.region_count.0.is_empty());
        }
        Ok(())
    }
}
