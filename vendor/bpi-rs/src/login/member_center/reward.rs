//! 查询每日奖励状态
//!
//! [文档](https://socialsisteryi.github.io/bilibili-API-collect/docs/login/member_center.html#查询每日奖励状态)

#[cfg(test)]
use crate::BpiError;
use crate::login::LoginDailyReward;

#[cfg(test)]
const DAILY_REWARD_ENDPOINT: &str = "https://api.bilibili.com/x/member/web/exp/reward";

/// 旧版会员中心每日奖励类型。
pub type DailyReward = LoginDailyReward;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApiEnvelope;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;

    #[test]
    fn member_center_daily_reward_matches_login_contract() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../../tests/contracts/login/daily-reward/contract.json"
        ))?;

        assert_eq!(contract.name, "login.daily_reward");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), DAILY_REWARD_ENDPOINT);
        assert!(contract.request.query.is_empty());
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(contract.cases[0].response.http_status, Some(412));
        assert_eq!(
            contract.cases[0].response.error.as_deref(),
            Some("risk_control")
        );
        assert!(contract.cases[0].response.fixture.is_none());
        assert_eq!(contract.cases[1].response.api_code, Some(0));
        assert_eq!(contract.cases[2].response.http_status, Some(412));
        assert_eq!(
            contract.cases[2].response.error.as_deref(),
            Some("risk_control")
        );
        assert!(contract.cases[2].response.fixture.is_none());
        Ok(())
    }

    #[test]
    fn member_center_daily_reward_fixture_parses_legacy_alias() -> Result<(), BpiError> {
        let reward = ApiEnvelope::<DailyReward>::from_slice(include_bytes!(
            "../../../tests/contracts/login/daily-reward/responses/normal.success.json"
        ))?
        .into_payload()?;

        assert!(reward.coins <= 50);
        Ok(())
    }
}
