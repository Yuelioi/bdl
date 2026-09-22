use serde::{Deserialize, Serialize};

/// 用户钱包数据
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserWallet {
    /// 用户 mid
    pub mid: i64,
    /// 总计 B 币
    pub total_bp: f64,
    /// 默认 B 币
    pub default_bp: f64,
    /// iOS B 币
    pub ios_bp: f64,
    /// 优惠券余额
    pub coupon_balance: f64,
    /// 可用 B 币
    pub available_bp: f64,
    /// 不可用 B 币
    pub unavailable_bp: f64,
    /// 不可用原因
    pub unavailable_reason: String,
    /// 提示信息
    pub tip: String,
    /// 需要显示类余额, 1
    pub need_show_class_balance: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApiEnvelope;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::wallet::params::WalletInfoParams;
    use crate::{BpiClient, BpiError};
    use tracing::info;

    const WALLET_INFO_ENDPOINT: &str = "https://pay.bilibili.com/paywallet/wallet/getUserWallet";

    fn local_wallet_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/wallet/read/info/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn wallet_info_contract_matches_endpoint_request() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/wallet/read/info/contract.json"
        ))?;

        assert_eq!(contract.name, "wallet.info");
        assert_eq!(contract.request.method, HttpMethod::Post);
        assert_eq!(contract.request.url.as_str(), WALLET_INFO_ENDPOINT);
        assert!(contract.request.query.is_empty());
        assert_eq!(contract.cases.len(), 2);

        let body = contract
            .request
            .body
            .as_ref()
            .ok_or_else(|| BpiError::unsupported_response("missing wallet contract body"))?;
        assert_eq!(body["csrf"], "${csrf}");
        assert_eq!(body["platformType"], 3);
        assert_eq!(body["timestamp"], 1_700_000_000_000_i64);
        assert_eq!(body["traceId"], 1_700_000_000_000_i64);
        assert_eq!(body["version"], "1.0");
        Ok(())
    }

    #[test]
    fn wallet_info_contract_covers_authenticated_profiles() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/wallet/read/info/contract.json"
        ))?;

        for case in &contract.cases {
            assert!(matches!(case.name.as_str(), "normal" | "vip"));
            assert!(case.auth.requires_cookie());
            assert!(case.auth.requires_csrf());
            assert_eq!(case.response.api_code, Some(0));
            assert_eq!(case.response.rust_model.as_deref(), Some("UserWallet"));
            assert_eq!(
                case.response.fixture.as_deref(),
                Some("responses/authenticated.success.json")
            );
        }
        Ok(())
    }

    #[test]
    fn wallet_response_fixture_parses_declared_model() -> Result<(), BpiError> {
        let wallet = ApiEnvelope::<UserWallet>::from_slice(include_bytes!(
            "../../tests/contracts/wallet/read/info/responses/authenticated.success.json"
        ))?
        .into_payload()?;

        assert_eq!(wallet.mid, 1_000_001);
        assert_eq!(wallet.need_show_class_balance, 1);
        Ok(())
    }

    #[test]
    fn wallet_model_matches_local_probe_outputs_when_available() -> Result<(), BpiError> {
        for profile in ["normal", "vip"] {
            let Some(body) = local_wallet_probe_body(profile) else {
                continue;
            };

            let wallet = serde_json::from_value::<ApiEnvelope<UserWallet>>(body)?.into_payload()?;

            assert!(wallet.mid > 0);
            assert_eq!(wallet.need_show_class_balance, 1);
        }
        Ok(())
    }

    #[test]
    fn wallet_anonymous_local_probe_preserves_login_error_when_available() -> Result<(), BpiError> {
        let Some(body) = local_wallet_probe_body("anonymous") else {
            return Ok(());
        };

        let err = serde_json::from_value::<ApiEnvelope<serde_json::Value>>(body)?
            .ensure_success()
            .unwrap_err();

        assert!(err.requires_login());
        assert_eq!(err.code(), Some(800501007));
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_user_wallet() {
        let bpi = BpiClient::new().expect("client should build");
        let resp = bpi.wallet().info(WalletInfoParams::new()).await;
        info!("响应: {:?}", resp);
        assert!(resp.is_ok());
        if let Ok(wallet) = resp {
            info!("用户mid: {}", wallet.mid);
        }
    }
}
