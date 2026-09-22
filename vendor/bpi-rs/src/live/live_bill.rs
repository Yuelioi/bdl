use serde::{Deserialize, Serialize};

// ================= 数据结构 =================

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct GiftTypeItem {
    /// 礼物id
    pub gift_id: i64,
    /// 礼物名称
    pub gift_name: String,
    /// 瓜子数量（电池礼物为金瓜子数量，银瓜子礼物为银瓜子数量）
    #[serde(default)]
    pub price: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/live/gift-read/gift-types/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_gift_types() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi.live().gift_types().await?;

        assert!(data.is_empty());
        Ok(())
    }

    #[test]
    fn live_gift_types_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "live.gift_types");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.live.bilibili.com/gift/v1/master/getGiftTypes"
        );
        assert!(contract.request.query.is_empty());
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(contract.cases[0].response.api_code, Some(-401));
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("Vec<GiftTypeItem>")
        );
        Ok(())
    }

    #[test]
    fn live_gift_types_response_fixtures_parse_declared_models() -> BpiResult<()> {
        let err = ApiEnvelope::<Vec<GiftTypeItem>>::from_slice(include_bytes!(
            "../../tests/contracts/live/gift-read/gift-types/responses/anonymous.requires_login.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());

        let payload = ApiEnvelope::<Vec<GiftTypeItem>>::from_slice(include_bytes!(
            "../../tests/contracts/live/gift-read/gift-types/responses/authenticated.success.json"
        ))?
        .into_payload()?;
        assert!(payload.is_empty());
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/live/gift-read/gift-types/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn live_gift_types_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body(profile) {
                let envelope = serde_json::from_value::<ApiEnvelope<Vec<GiftTypeItem>>>(body)?;
                if profile == "anonymous" {
                    let err = envelope.ensure_success().unwrap_err();
                    assert!(err.requires_login());
                } else {
                    let payload = envelope.into_payload()?;
                    assert!(payload.is_empty());
                }
            }
        }
        Ok(())
    }
}
