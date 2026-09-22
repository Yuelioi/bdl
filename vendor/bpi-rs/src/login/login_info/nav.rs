//! 导航栏用户信息
//!
//! [查看 API 文档](https://socialsisteryi.github.io/bilibili-API-collect/docs/login/login_info_info.html#导航栏用户信息)
use serde::{Deserialize, Serialize};

#[cfg(test)]
const NAV_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/nav";

// ============ 导航栏用户信息 ============

/// 用户信息数据
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NavData {
    /// 是否已登录 false：未登录 true：已登录
    #[serde(rename = "isLogin")]
    pub is_login: bool,

    /// Wbi 签名实时口令（该字段即使用户未登录也存在）
    pub wbi_img: WbiImg,

    /// 是否验证邮箱地址 0：未验证 1：已验证
    pub email_verified: i32,

    /// 用户头像 url
    pub face: String,

    /// 头像 NFT 类型
    pub face_nft: i32,

    /// 等级信息
    pub level_info: LevelInfo,

    /// 用户 mid
    pub mid: u64,

    /// 是否验证手机号 0：未验证 1：已验证
    pub mobile_verified: i32,

    /// 拥有硬币数
    pub money: f64,

    /// 当前节操值，上限为70
    pub moral: i32,

    /// 认证信息
    pub official: Official,

    /// 认证信息 2
    #[serde(rename = "officialVerify")]
    pub official_verify: OfficialVerify,

    /// 头像框信息
    pub pendant: Pendant,

    /// 未知字段
    pub scores: i32,

    /// 用户昵称
    pub uname: String,

    /// 会员到期时间毫秒时间戳
    #[serde(rename = "vipDueDate")]
    pub vip_due_date: u64,

    /// 大会员状态
    /// - 1：正常
    /// - 2：IP频繁更换，服务被冻结
    /// - 3：大会员账号风险过高，功能锁定
    #[serde(rename = "vipStatus")]
    pub vip_status: i32,

    /// 会员类型 0：无 1：月度大会员 2：年度及以上大会员
    #[serde(rename = "vipType")]
    pub vip_type: i32,

    /// 会员开通状态 0：无 1：有
    pub vip_pay_type: i32,

    /// 未知字段
    pub vip_theme_type: i32,

    /// 会员标签
    pub vip_label: VipLabel,

    /// 是否显示会员图标 0：不显示 1：显示
    pub vip_avatar_subscript: i32,

    /// 会员昵称颜色（颜色码）
    pub vip_nickname_color: String,

    /// 会员信息
    pub vip: Vip,

    /// B币钱包信息
    pub wallet: Wallet,

    /// 是否拥有推广商品 false：无 true：有
    pub has_shop: bool,

    /// 商品推广页面 url
    pub shop_url: String,

    /// 是否硬核会员 0：非硬核会员 1：硬核会员
    pub is_senior_member: i32,

    /// 是否风纪委员 true：风纪委员 false：非风纪委员
    pub is_jury: bool,

    /// 用户名渲染信息
    pub name_render: Option<serde_json::Value>,
}

/// 钱包信息
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub mid: u64,
    pub bcoin_balance: i64,
    pub coupon_balance: i64,
    pub coupon_due_time: i64,
}

/// Wbi 图片信息
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct WbiImg {
    pub img_url: String,
    pub sub_url: String,
}

use crate::models::{LevelInfo, Official, OfficialVerify, Pendant, Vip, VipLabel};

// 测试模块
#[cfg(test)]
mod tests {
    use super::*;
    use tracing::info;

    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../../tests/contracts/login/nav/contract.json"
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
    /// 测试登录
    async fn test_bilibili_uinfo() -> Result<(), BpiError> {
        if !live_login_tests_enabled() {
            return Ok(());
        }

        let bpi = live_client()?;

        let data = bpi.login().nav().await?;

        if data.is_login {
            info!(
                "登录成功！UID={:?} 昵称={:?} ",
                data.mid.as_ref().map(|mid| mid.get()),
                data.uname
            );
        }

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_user_info() -> Result<(), BpiError> {
        if !live_login_tests_enabled() {
            return Ok(());
        }

        let bpi = live_client()?;

        let user_info = bpi.login().nav().await?;

        info!("用户信息：{:?}", user_info);

        Ok(())
    }

    #[test]
    fn legacy_login_info_nav_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "login.nav");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), NAV_ENDPOINT);
        assert!(contract.request.query.is_empty());
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(contract.cases[0].response.api_code, Some(-101));
        assert_eq!(contract.cases[1].response.api_code, Some(0));
        assert_eq!(contract.cases[2].response.api_code, Some(0));
        Ok(())
    }

    #[test]
    fn legacy_login_info_nav_fixtures_parse_promoted_contract_models() -> BpiResult<()> {
        for (bytes, expected_mid) in [
            (
                include_bytes!("../../../tests/contracts/login/nav/responses/normal.success.json")
                    .as_slice(),
                1_000_001,
            ),
            (
                include_bytes!("../../../tests/contracts/login/nav/responses/vip.success.json")
                    .as_slice(),
                1_000_002,
            ),
        ] {
            let payload = ApiEnvelope::<NavData>::from_slice(bytes)?.into_payload()?;
            assert!(payload.is_login);
            assert_eq!(payload.mid, expected_mid);
            assert!(!payload.uname.trim().is_empty());
        }

        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../../tests/contracts/login/nav/responses/anonymous.error.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());
        Ok(())
    }
}
