//! 查询大会员状态
//!
//! [文档](https://socialsisteryi.github.io/bilibili-API-collect/docs/login/member_center.html#查询大会员状态)

#[cfg(test)]
use crate::BpiError;
use crate::login::LoginVipInfo;

#[cfg(test)]
const VIP_INFO_ENDPOINT: &str = "https://api.bilibili.com/x/vip/web/user/info";

/// 旧版会员中心 VIP 信息类型。
pub type VipInfo = LoginVipInfo;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApiEnvelope;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;

    #[test]
    fn member_center_vip_info_matches_login_contract() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../../tests/contracts/login/vip-info/contract.json"
        ))?;

        assert_eq!(contract.name, "login.vip_info");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), VIP_INFO_ENDPOINT);
        assert!(contract.request.query.is_empty());
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(contract.cases[0].response.api_code, Some(-101));
        assert_eq!(contract.cases[1].response.api_code, Some(0));
        assert_eq!(contract.cases[2].response.api_code, Some(0));
        Ok(())
    }

    #[test]
    fn member_center_vip_info_fixtures_parse_legacy_alias() -> Result<(), BpiError> {
        let normal = ApiEnvelope::<VipInfo>::from_slice(include_bytes!(
            "../../../tests/contracts/login/vip-info/responses/normal.success.json"
        ))?
        .into_payload()?;
        let vip = ApiEnvelope::<VipInfo>::from_slice(include_bytes!(
            "../../../tests/contracts/login/vip-info/responses/vip.success.json"
        ))?
        .into_payload()?;

        assert!(!normal.is_active());
        assert!(vip.is_active());

        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../../tests/contracts/login/vip-info/responses/anonymous.error.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());
        Ok(())
    }
}
