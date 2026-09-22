//! IP 地址归属地查询 API
//!
//! [参考文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/clientinfo/ip.md)

use serde::{Deserialize, Serialize};

// ==========================
// 数据结构
// ==========================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpInfo {
    /// 国家
    pub country: Option<String>,
    /// 省份
    pub province: Option<String>,
    /// 城市
    pub city: Option<String>,
    /// ISP 运营商
    pub isp: Option<String>,
    /// IP 地址
    pub addr: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clientinfo::params::ClientInfoIpParams;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    const TEST_IP: &str = "8.8.8.8";

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_clientinfo_ip() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");

        let params = ClientInfoIpParams::new().with_ip_str(TEST_IP)?;
        let data = bpi.clientinfo().ip(params).await?;
        tracing::info!(
            "IP 地址: {}, 省份: {:?}, 城市: {:?}, ISP: {:?}",
            data.addr.unwrap_or_default(),
            data.province,
            data.city,
            data.isp
        );

        Ok(())
    }

    #[test]
    fn clientinfo_ip_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/clientinfo/ip/contract.json"
        ))?;

        assert_eq!(contract.name, "clientinfo.ip");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.live.bilibili.com/ip_service/v1/ip_service/get_ip_addr"
        );
        assert_eq!(contract.request.query.get("ip"), Some(&TEST_IP.to_string()));
        assert_eq!(contract.cases.len(), 3);
        Ok(())
    }

    #[test]
    fn clientinfo_ip_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!("../../tests/contracts/clientinfo/ip/responses/anonymous.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/clientinfo/ip/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/clientinfo/ip/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<IpInfo>::from_slice(bytes)?.into_payload()?;

            assert_eq!(payload.addr.as_deref(), Some(TEST_IP));
        }
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/clientinfo/ip/ip/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn clientinfo_ip_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };

            let payload = serde_json::from_value::<ApiEnvelope<IpInfo>>(body)?.into_payload()?;

            assert_eq!(payload.addr.as_deref(), Some(TEST_IP));
        }
        Ok(())
    }
}
