//! B站用户关注分组相关接口
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/user)
use serde::{Deserialize, Serialize};

/// 关注分组
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FollowTag {
    pub tagid: i64,          // 分组 id (-10: 特别关注, 0: 默认分组)
    pub name: String,        // 分组名称
    pub count: i64,          // 分组成员数
    pub tip: Option<String>, // 提示信息
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use tracing::info;

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../../tests/contracts/user/relation-read/follow-tags/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_user_follow_tags_cookie() -> Result<(), BpiError> {
        if std::env::var_os("BPI_LIVE_TEST").is_none() || std::env::var_os("BPI_COOKIE").is_none() {
            return Ok(());
        }

        let bpi = BpiClient::new().expect("client should build");
        let data = bpi.user().follow_tags().await?;

        info!("关注分组列表: {:?}", data);
        Ok(())
    }

    #[test]
    fn legacy_user_follow_tags_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "user.follow_tags");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/relation/tags"
        );
        assert!(contract.request.query.is_empty());
        assert_eq!(
            contract.cases[0].response.error.as_deref(),
            Some("requires_login")
        );
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("Vec<UserFollowTag>")
        );
        Ok(())
    }

    #[test]
    fn legacy_user_follow_tags_fixtures_parse_promoted_contract_models() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../../tests/contracts/user/relation-read/follow-tags/responses/anonymous.error.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());

        let tags = ApiEnvelope::<Vec<FollowTag>>::from_slice(include_bytes!(
            "../../../tests/contracts/user/relation-read/follow-tags/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(tags.len(), 2);
        assert_eq!(tags[0].tagid, -10);
        Ok(())
    }
}
