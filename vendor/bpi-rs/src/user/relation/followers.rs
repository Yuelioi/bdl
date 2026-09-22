//! B站用户粉丝列表相关接口
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/user)
use crate::models::Vip;
use serde::{Deserialize, Serialize};
// --- 响应数据结构体 ---

/// 用户认证信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OfficialVerify {
    /// 用户认证类型，-1: 无, 0: UP 主认证, 1: 机构认证
    #[serde(rename = "type")]
    pub verify_type: i8,
    /// 用户认证信息，无则为空
    pub desc: String,
}

/// 关系列表对象
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelationListItem {
    /// 用户 mid
    pub mid: u64,
    /// 对方对于自己的关系属性，0: 未关注, 1: 悄悄关注, 2: 已关注, 6: 已互粉, 128: 已拉黑
    pub attribute: u8,
    /// 对方关注目标用户时间，秒级时间戳
    pub mtime: Option<u64>,
    /// 目标用户将对方分组到的 id
    pub tag: Option<Vec<u64>>,
    /// 目标用户特别关注对方标识，0: 否, 1: 是
    pub special: u8,
    pub contract_info: Option<serde_json::Value>,
    /// 用户昵称
    pub uname: String,
    /// 用户头像 url
    pub face: String,
    /// 用户签名
    pub sign: String,
    /// 是否为 NFT 头像
    pub face_nft: u8,
    /// 认证信息
    pub official_verify: OfficialVerify,
    /// 会员信息
    pub vip: Vip,
}

/// 用户粉丝明细响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FansListResponseData {
    /// 明细列表
    pub list: Vec<RelationListItem>,
    /// 偏移量供下次请求使用
    pub offset: String,
    pub re_version: u32,
    /// 粉丝总数
    pub total: u64,
}

// --- 测试模块 ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::Mid;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::user::params::UserFollowersParams;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use tracing::info;

    const TEST_VMID: u64 = 1000001;

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../../tests/contracts/user/relation-read/followers/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_user_followers() -> Result<(), BpiError> {
        if std::env::var_os("BPI_LIVE_TEST").is_none() {
            return Ok(());
        }

        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .user()
            .followers(
                UserFollowersParams::new(Mid::new(TEST_VMID)?)
                    .with_page_size(50)
                    .with_page(1),
            )
            .await?;

        info!("用户粉丝明细: {:?}", data);
        assert!(!data.list.is_empty());
        assert_eq!(data.list.len(), 50);
        assert!(!data.offset.is_empty());
        assert!(data.total > 0);

        Ok(())
    }

    #[test]
    fn legacy_user_followers_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "user.followers");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/relation/fans"
        );
        assert_eq!(
            contract.request.query.get("vmid").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            contract.request.query.get("ps").map(String::as_str),
            Some("20")
        );
        assert_eq!(
            contract.request.query.get("pn").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            contract.cases[0].response.error.as_deref(),
            Some("wbi_risk_control")
        );
        Ok(())
    }

    #[test]
    fn legacy_user_followers_fixtures_parse_promoted_contract_models() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../../tests/contracts/user/relation-read/followers/responses/anonymous.error.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert_eq!(err.code(), Some(-352));

        let followers = ApiEnvelope::<FansListResponseData>::from_slice(include_bytes!(
            "../../../tests/contracts/user/relation-read/followers/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(followers.list.len(), 1);
        assert_eq!(followers.total, 1);
        Ok(())
    }
}
