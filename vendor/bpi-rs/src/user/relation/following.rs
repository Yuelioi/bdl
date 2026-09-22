//! B站用户关注列表相关接口
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/user)
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

/// 大会员标签
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct VipLabel {
    #[serde(default)]
    pub path: String,
}

/// 会员信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VipInfo {
    /// 会员类型，0: 无, 1: 月度大会员, 2: 年度以上大会员
    #[serde(rename = "vipType")]
    #[serde(default)]
    pub vip_type: u8,
    /// 会员到期时间，毫秒级时间戳
    #[serde(rename = "vipDueDate")]
    #[serde(default)]
    pub vip_due_date: u64,
    #[serde(rename = "dueRemark")]
    #[serde(default)]
    pub due_remark: String,
    #[serde(rename = "accessStatus")]
    #[serde(default)]
    pub access_status: u8,
    /// 大会员状态，0: 无, 1: 有
    #[serde(rename = "vipStatus")]
    #[serde(default)]
    pub vip_status: u8,
    #[serde(rename = "vipStatusWarn")]
    #[serde(default)]
    pub vip_status_warn: String,
    #[serde(rename = "themeType")]
    #[serde(default)]
    pub theme_type: u8,
    #[serde(default)]
    pub label: VipLabel,
}

/// 关系列表对象
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelationListItem {
    /// 用户 mid
    pub mid: u64,
    /// 对方对于自己的关系属性，0: 未关注, 1: 悄悄关注, 2: 已关注, 6: 已互粉, 128: 已拉黑
    pub attribute: u8,
    /// 对方关注目标用户时间，秒级时间戳
    pub mtime: u64,
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
    pub vip: VipInfo,
    #[serde(rename = "name_render")]
    pub name_render: Option<serde_json::Value>,
    #[serde(rename = "nft_icon")]
    pub nft_icon: Option<String>,
    /// 推荐该用户的原因
    pub rec_reason: Option<String>,
    #[serde(rename = "track_id")]
    pub track_id: Option<String>,
    #[serde(rename = "follow_time")]
    pub follow_time: Option<String>,
}

/// 用户关注明细响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FollowingListResponseData {
    /// 明细列表
    pub list: Vec<RelationListItem>,
    pub re_version: u32,
    /// 关注总数
    pub total: u64,
}

// --- 测试模块 ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::Mid;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::user::params::UserFollowingsParams;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use tracing::info;

    const TEST_VMID: u64 = 293793435;

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../../tests/contracts/user/relation-read/followings/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_user_followings() -> Result<(), BpiError> {
        if std::env::var_os("BPI_LIVE_TEST").is_none() {
            return Ok(());
        }

        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .user()
            .followings(
                UserFollowingsParams::new(Mid::new(TEST_VMID)?)
                    .with_page_size(50)
                    .with_page(1),
            )
            .await?;

        info!("用户关注明细: {:?}", data);
        assert!(!data.list.is_empty());
        assert_eq!(data.list.len(), 50);

        Ok(())
    }

    #[test]
    fn legacy_user_followings_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "user.followings");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/relation/followings"
        );
        assert_eq!(
            contract.request.query.get("vmid").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            contract.request.query.get("order_type").map(String::as_str),
            Some("attention")
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
            Some("requires_login")
        );
        Ok(())
    }

    #[test]
    fn legacy_user_followings_fixtures_parse_promoted_contract_models() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../../tests/contracts/user/relation-read/followings/responses/anonymous.error.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());

        let followings = ApiEnvelope::<FollowingListResponseData>::from_slice(include_bytes!(
            "../../../tests/contracts/user/relation-read/followings/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(followings.list.len(), 1);
        assert_eq!(followings.total, 1);
        Ok(())
    }
}
