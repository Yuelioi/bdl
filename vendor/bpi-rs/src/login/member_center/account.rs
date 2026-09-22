//! 获取我的信息
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/login/member_center.md)

#[cfg(test)]
use crate::BpiError;
use crate::login::LoginAccountInfo;

#[cfg(test)]
const ACCOUNT_INFO_ENDPOINT: &str = "https://api.bilibili.com/x/member/web/account";

/// 旧版会员中心账号信息类型。
pub type AccountInfo = LoginAccountInfo;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApiEnvelope;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;

    #[test]
    fn member_center_account_info_matches_login_account_contract() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../../tests/contracts/login/account-info/contract.json"
        ))?;

        assert_eq!(contract.name, "login.account_info");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), ACCOUNT_INFO_ENDPOINT);
        assert!(contract.request.query.is_empty());
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(contract.cases[0].response.api_code, Some(-101));
        assert_eq!(contract.cases[1].response.api_code, Some(0));
        assert_eq!(contract.cases[2].response.api_code, Some(0));
        Ok(())
    }

    fn assert_account_info_success_fixture(bytes: &[u8]) -> Result<(), BpiError> {
        let data = ApiEnvelope::<AccountInfo>::from_slice(bytes)?.into_payload()?;

        assert!(data.mid.get() > 0);
        Ok(())
    }

    #[test]
    fn member_center_account_info_fixtures_parse_legacy_alias() -> Result<(), BpiError> {
        for bytes in [
            include_bytes!(
                "../../../tests/contracts/login/account-info/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../../tests/contracts/login/account-info/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            assert_account_info_success_fixture(bytes)?;
        }

        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../../tests/contracts/login/account-info/responses/anonymous.error.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());
        Ok(())
    }
}
