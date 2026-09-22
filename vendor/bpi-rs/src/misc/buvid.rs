//! 获取 buvid3 (Web端)
//!
//! [文档](https://api.bilibili.com/x/web-frontend/getbuvid)

use serde::{Deserialize, Serialize};

/// 获取 buvid3 - 响应数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Buvid3Data {
    /// buvid3，需要手动存放至 Cookie 中
    pub buvid: String,
}

/// 获取 buvid3/4 - 响应数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuvidData {
    /// buvid3，需要手动存放至 Cookie 中
    #[serde(rename = "b_3")]
    pub buvid3: String,

    /// buvid4，需要手动存放至 Cookie 中
    #[serde(rename = "b_4")]
    pub buvid4: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiResult};

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "buvid3" => {
                include_bytes!("../../tests/contracts/misc/buvid3/contract.json").as_slice()
            }
            "buvid" => include_bytes!("../../tests/contracts/misc/buvid/contract.json").as_slice(),
            _ => unreachable!("unknown misc buvid contract endpoint"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_buvid3() {
        let bpi = BpiClient::new().expect("client should build");

        match bpi.misc().buvid3().await {
            Ok(data) => {
                tracing::info!("获取 buvid3 成功: {}", data.buvid);
            }
            Err(err) => {
                panic!("请求出错: {}", err);
            }
        }
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_buvid() {
        let bpi = BpiClient::new().expect("client should build");

        match bpi.misc().buvid().await {
            Ok(data) => {
                tracing::info!("获取 buvid3 成功: {}", data.buvid3);
                tracing::info!("获取 buvid4 成功: {}", data.buvid4);
            }
            Err(err) => {
                panic!("请求出错: {}", err);
            }
        }
    }

    #[test]
    fn misc_buvid3_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("buvid3")?;

        assert_eq!(contract.name, "misc.buvid3");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/web-frontend/getbuvid"
        );
        assert!(contract.request.query.is_empty());
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(contract.cases[0].response.api_code, Some(0));
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("Buvid3Data")
        );
        Ok(())
    }

    #[test]
    fn misc_buvid_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("buvid")?;

        assert_eq!(contract.name, "misc.buvid");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/frontend/finger/spi"
        );
        assert!(contract.request.query.is_empty());
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(contract.cases[2].response.api_code, Some(0));
        assert_eq!(
            contract.cases[2].response.rust_model.as_deref(),
            Some("BuvidData")
        );
        Ok(())
    }

    #[test]
    fn misc_buvid3_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!("../../tests/contracts/misc/buvid3/responses/anonymous.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/misc/buvid3/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/misc/buvid3/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<Buvid3Data>::from_slice(bytes)?.into_payload()?;

            assert_eq!(payload.buvid, "BUVID3_SANITIZED");
        }
        Ok(())
    }

    #[test]
    fn misc_buvid_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!("../../tests/contracts/misc/buvid/responses/anonymous.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/misc/buvid/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/misc/buvid/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<BuvidData>::from_slice(bytes)?.into_payload()?;

            assert_eq!(payload.buvid3, "BUVID3_SANITIZED");
            assert_eq!(payload.buvid4, "BUVID4_SANITIZED");
        }
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/misc/buvid/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn misc_buvid_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body("buvid3", profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<Buvid3Data>>(body)?.into_payload()?;

            assert!(!payload.buvid.is_empty());
        }

        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body("buvid", profile) else {
                continue;
            };
            let payload = serde_json::from_value::<ApiEnvelope<BuvidData>>(body)?.into_payload()?;

            assert!(!payload.buvid3.is_empty());
            assert!(!payload.buvid4.is_empty());
        }
        Ok(())
    }
}
