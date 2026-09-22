//! B站用户粉丝勋章相关接口
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/user)
use serde::{Deserialize, Serialize};

/// 粉丝勋章响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MedalWallData {
    pub list: Vec<MedalWallItem>,
    pub count: u32,
    pub close_space_medal: u32,
    pub only_show_wearing: u32,
    pub name: String,
    pub icon: String,
    pub uid: u64,
    pub level: u32,
}

/// 勋章项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MedalWallItem {
    pub medal_info: MedalInfo,
    pub target_name: String,
    pub target_icon: String,
    pub link: String,
    pub live_status: u32,
    pub offical: Option<u32>, // 部分用户可能没有认证
    pub uinfo_medal: Option<UinfoMedal>,
}

/// 勋章信息（主播相关）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MedalInfo {
    pub target_id: u64,
    pub level: u32,
    pub medal_name: String,
    pub medal_color_start: u32,
    pub medal_color_end: u32,
    pub medal_color_border: u32,
    pub guard_level: u32,
    pub wearing_status: u32,
    pub medal_id: u64,
    pub intimacy: u64,
    pub next_intimacy: u64,
    pub today_feed: u64,
    pub day_limit: u64,
    pub guard_icon: Option<String>,
    pub honor_icon: Option<String>,
}

/// 用户勋章信息（佩戴者视角）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UinfoMedal {
    pub name: String,
    pub level: u32,
    pub color_start: u32,
    pub color_end: u32,
    pub color_border: u32,
    pub color: u32,
    pub id: u64,
    pub typ: u32,
    pub is_light: u32,
    pub ruid: u64,
    pub guard_level: u32,
    pub score: u64,
    pub guard_icon: Option<String>,
    pub honor_icon: Option<String>,
    pub v2_medal_color_start: Option<String>,
    pub v2_medal_color_end: Option<String>,
    pub v2_medal_color_border: Option<String>,
    pub v2_medal_color_text: Option<String>,
    pub v2_medal_color_level: Option<String>,
    pub user_receive_count: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::Mid;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::user::params::UserMedalWallParams;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use tracing::info;

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_user_medal_wall() -> Result<(), BpiError> {
        if std::env::var_os("BPI_LIVE_TEST").is_none() {
            return Ok(());
        }

        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .user()
            .medal_wall(UserMedalWallParams::new(Mid::new(2)?))
            .await?;
        info!("粉丝勋章墙: {:?}", data);

        Ok(())
    }

    fn medal_wall_contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/medal-wall/contract.json"
        ))
    }

    #[test]
    fn legacy_user_medal_wall_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = medal_wall_contract()?;

        assert_eq!(contract.name, "user.medal_wall");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.live.bilibili.com/xlive/web-ucenter/user/MedalWall"
        );
        assert_eq!(
            contract.request.query.get("target_id").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            contract.cases[0].response.error.as_deref(),
            Some("requires_login")
        );
        Ok(())
    }

    #[test]
    fn legacy_user_medal_wall_fixture_parses_promoted_contract_model() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/medal-wall/responses/anonymous.error.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());

        let wall = ApiEnvelope::<MedalWallData>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/medal-wall/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(wall.uid, 2);
        Ok(())
    }
}
