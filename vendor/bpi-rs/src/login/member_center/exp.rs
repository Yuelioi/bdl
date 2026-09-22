//! 查询每日投币获得经验数
//!
//! [文档](https://socialsisteryi.github.io/bilibili-API-collect/docs/login/member_center.html#查询每日投币获得经验数)

#[cfg(test)]
use crate::BpiError;

#[cfg(test)]
const TODAY_COIN_EXP_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/coin/today/exp";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApiEnvelope;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;

    #[test]
    fn member_center_today_coin_exp_matches_login_contract() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../../tests/contracts/login/today-coin-exp/contract.json"
        ))?;

        assert_eq!(contract.name, "login.today_coin_exp");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), TODAY_COIN_EXP_ENDPOINT);
        assert!(contract.request.query.is_empty());
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(contract.cases[0].response.api_code, Some(-101));
        assert_eq!(contract.cases[1].response.api_code, Some(0));
        assert_eq!(contract.cases[2].response.api_code, Some(0));
        Ok(())
    }

    #[test]
    fn member_center_today_coin_exp_fixtures_parse_legacy_u32() -> Result<(), BpiError> {
        for bytes in [
            include_bytes!(
                "../../../tests/contracts/login/today-coin-exp/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../../tests/contracts/login/today-coin-exp/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let value = ApiEnvelope::<u32>::from_slice(bytes)?.into_payload()?;
            assert!(value <= 50);
        }

        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../../tests/contracts/login/today-coin-exp/responses/anonymous.error.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());
        Ok(())
    }
}
