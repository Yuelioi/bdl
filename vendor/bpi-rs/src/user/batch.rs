//! B站用户批量信息相关接口
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/user)
use serde::{Deserialize, Serialize};

/// UID 查询返回的单个条目
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NameUidItem {
    /// 用户名
    pub name: String,
    /// 用户 mid
    pub uid: String,
}

/// 批量用户名查 UID 的数据本体
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NameToUidData {
    pub uid_list: Vec<NameUidItem>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::user::params::UserNameToUidParams;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use tracing::info;

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_user_name_to_uid() -> Result<(), BpiError> {
        if std::env::var_os("BPI_LIVE_TEST").is_none() {
            return Ok(());
        }

        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .user()
            .name_to_uid(UserNameToUidParams::new(["LexBurner", "某科学"])?)
            .await?;

        info!("用户名查 UID 返回: {:?}", data);
        assert!(!data.uid_list.is_empty());

        Ok(())
    }

    fn name_to_uid_contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/name-to-uid/contract.json"
        ))
    }

    #[test]
    fn legacy_user_name_to_uid_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = name_to_uid_contract()?;

        assert_eq!(contract.name, "user.name_to_uid");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/polymer/web-dynamic/v1/name-to-uid"
        );
        assert_eq!(
            contract.request.query.get("names").map(String::as_str),
            Some("LexBurner,某科学")
        );
        assert_eq!(
            contract.cases[0].response.error.as_deref(),
            Some("requires_login")
        );
        Ok(())
    }

    #[test]
    fn legacy_user_name_to_uid_fixture_parses_promoted_contract_model() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/name-to-uid/responses/anonymous.error.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());

        let data = ApiEnvelope::<NameToUidData>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/name-to-uid/responses/success.json"
        ))?
        .into_payload()?;
        assert!(!data.uid_list.is_empty());
        Ok(())
    }
}
