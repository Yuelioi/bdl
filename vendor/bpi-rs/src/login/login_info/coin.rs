//! 获取硬币数
//!
//! [查看 API 文档](https://socialsisteryi.github.io/bilibili-API-collect/docs/login/login_info_info.html#获取硬币数)

use serde::{Deserialize, Serialize};

#[cfg(test)]
const COIN_ENDPOINT: &str = "https://account.bilibili.com/site/getCoin";

/// 获取硬币数 - 响应结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinInfo {
    /// 当前硬币数
    pub money: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../../tests/contracts/login/coin/contract.json"
        ))
    }

    fn live_login_tests_enabled() -> bool {
        std::env::var("BPI_LIVE_TEST").ok().as_deref() == Some("1")
    }

    fn live_client() -> Result<BpiClient, BpiError> {
        match std::env::var("BPI_COOKIE") {
            Ok(cookie) if !cookie.trim().is_empty() => BpiClient::builder().cookie(cookie).build(),
            _ => BpiClient::new(),
        }
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_coin() -> Result<(), BpiError> {
        if !live_login_tests_enabled() {
            return Ok(());
        }

        let bpi = live_client()?;

        match bpi.login().coin().await {
            Ok(data) => {
                tracing::info!("获取硬币数成功: {:?}", data.money);
            }
            Err(err) => {
                return Err(err);
            }
        }

        Ok(())
    }

    #[test]
    fn legacy_login_info_coin_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "login.coin");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), COIN_ENDPOINT);
        assert!(contract.request.query.is_empty());
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(contract.cases[0].response.api_code, Some(-101));
        assert_eq!(contract.cases[1].response.api_code, Some(0));
        assert_eq!(contract.cases[2].response.api_code, Some(0));
        Ok(())
    }

    #[test]
    fn legacy_login_info_coin_fixtures_parse_promoted_contract_models() -> BpiResult<()> {
        for bytes in [
            include_bytes!("../../../tests/contracts/login/coin/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../../tests/contracts/login/coin/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<CoinInfo>::from_slice(bytes)?.into_payload()?;
            assert!(payload.money >= 0.0);
        }

        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../../tests/contracts/login/coin/responses/anonymous.error.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());
        Ok(())
    }
}
