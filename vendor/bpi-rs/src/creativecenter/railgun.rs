//! 电磁力等级 API
//!
//! [参考文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/railgun.md)

use serde::{Deserialize, Serialize};

/// 电磁力等级信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectromagneticInfo {
    /// 当前用户 mid
    pub mid: u64,
    /// 电磁力等级
    pub level: u32,
    /// 电磁力分数
    pub score: u32,
    /// 信用分
    pub credit: u32,
    /// 状态 (文档不明，返回固定 2)
    pub state: i32,
    /// 更新时间戳。API 对无数据账号返回 0。
    pub update_date: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/creativecenter/railgun-read/electromagnetic-info/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_electromagnetic_info() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");

        let data = bpi.creativecenter().electromagnetic_info().await?;

        tracing::info!(
            "mid={}, level={}, score={}, credit={}, state={}",
            data.mid,
            data.level,
            data.score,
            data.credit,
            data.state
        );

        Ok(())
    }

    #[test]
    fn creativecenter_railgun_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "creativecenter.railgun.electromagnetic_info");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/studio/up-rating/v3/rating/info"
        );
        assert!(contract.request.query.is_empty());
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(contract.cases[0].response.api_code, Some(-101));
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("ElectromagneticInfo")
        );
        Ok(())
    }

    #[test]
    fn creativecenter_railgun_response_fixtures_parse_declared_model() -> BpiResult<()> {
        let anonymous = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/creativecenter/railgun-read/electromagnetic-info/responses/anonymous.requires_login.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(anonymous.requires_login());

        for bytes in [
            include_bytes!(
                "../../tests/contracts/creativecenter/railgun-read/electromagnetic-info/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/creativecenter/railgun-read/electromagnetic-info/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<ElectromagneticInfo>::from_slice(bytes)?.into_payload()?;
            assert_eq!(payload.mid, 0);
            assert_eq!(payload.update_date, 0);
        }
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!(
            "target/bpi-probe-runs/creativecenter/railgun-read/electromagnetic-info/{profile}.response.json"
        );
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn creativecenter_railgun_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let envelope = serde_json::from_value::<ApiEnvelope<ElectromagneticInfo>>(body)?;
            if profile == "anonymous" {
                assert!(envelope.ensure_success().unwrap_err().requires_login());
            } else {
                let payload = envelope.into_payload()?;
                assert_eq!(payload.state, 0);
            }
        }
        Ok(())
    }
}
