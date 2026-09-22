use serde::de;
use serde::{Deserialize, Deserializer, Serialize};

use crate::ids::Mid;

/// `/x/web-interface/nav` 返回的登录和导航状态。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginNav {
    /// 当前会话是否已登录。
    #[serde(rename = "isLogin")]
    pub is_login: bool,
    /// 已登录用户 ID。游客响应返回 `0`，对外暴露为 `None`。
    #[serde(default, deserialize_with = "deserialize_optional_mid")]
    pub mid: Option<Mid>,
    /// 已登录显示名称。游客空值对外暴露为 `None`。
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub uname: Option<String>,
    /// 已登录头像 URL。游客空值对外暴露为 `None`。
    #[serde(default, deserialize_with = "deserialize_optional_string")]
    pub face: Option<String>,
    /// WBI 图片 key。Bilibili 对游客会话也会返回这些字段。
    pub wbi_img: LoginWbiImg,
}

/// 登录 nav 响应中嵌入的 WBI 图片 key URL。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginWbiImg {
    /// 包含 img key 文件名的 URL。
    pub img_url: String,
    /// 包含 sub key 文件名的 URL。
    pub sub_url: String,
}

/// `/x/web-interface/nav/stat` 返回的已认证用户社交计数。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginStats {
    /// 关注用户数。
    pub following: u64,
    /// 粉丝数。
    pub follower: u64,
    /// 已发布动态数。
    pub dynamic_count: u64,
}

/// 当前已认证账号的硬币余额。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LoginCoinBalance {
    /// 当前硬币余额。
    pub money: f64,
}

/// 今日通过投币操作获得的经验。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct LoginTodayCoinExp {
    /// 今日获得的经验。
    pub value: u32,
}

/// `/x/member/web/exp/reward` 返回的每日奖励完成状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginDailyReward {
    /// 每日登录奖励是否完成。
    pub login: bool,
    /// 每日观看奖励是否完成。
    pub watch: bool,
    /// 每日投币操作获得的经验。
    pub coins: u32,
    /// 每日分享奖励是否完成。
    pub share: bool,
    /// 邮箱绑定奖励是否完成。
    pub email: bool,
    /// 手机绑定奖励是否完成。
    pub tel: bool,
    /// 安全问题奖励是否完成。
    pub safe_question: bool,
    /// 实名认证奖励是否完成。
    pub identify_card: bool,
}

/// `/x/member/web/account` 返回的已认证账号资料。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginAccountInfo {
    /// 当前用户 ID。
    pub mid: Mid,
    /// 当前用户显示名称。
    pub uname: String,
    /// 登录用户名，可能不同于显示名称。
    pub userid: String,
    /// 当前个人签名。
    pub sign: String,
    /// Bilibili 返回的生日字符串，通常为 `YYYY-MM-DD`。
    pub birthday: String,
    /// Bilibili 返回的性别标签。
    pub sex: String,
    /// 账号是否尚未设置自定义昵称。
    pub nick_free: bool,
    /// Bilibili 返回的会员 rank 字符串。
    pub rank: String,
}

/// `/x/vip/web/user/info` 返回的已认证账号 VIP 状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoginVipInfo {
    /// 当前用户 ID。
    pub mid: Mid,
    /// Bilibili 返回的 VIP 类型。
    pub vip_type: u8,
    /// Bilibili 返回的 VIP 状态。
    pub vip_status: u8,
    /// VIP 到期时间戳，单位毫秒。
    pub vip_due_date: u64,
    /// Bilibili 返回的 VIP 支付类型。
    pub vip_pay_type: u8,
    /// Bilibili 返回的 VIP 主题类型。
    pub theme_type: u8,
}

impl LoginVipInfo {
    /// 返回账号当前是否拥有有效 VIP 状态。
    pub fn is_active(self) -> bool {
        self.vip_status == 1 && self.vip_due_date > 0
    }
}

fn deserialize_optional_mid<'de, D>(deserializer: D) -> Result<Option<Mid>, D::Error>
where
    D: Deserializer<'de>,
{
    match Option::<u64>::deserialize(deserializer)? {
        Some(0) | None => Ok(None),
        Some(value) => Mid::new(value).map(Some).map_err(de::Error::custom),
    }
}

fn deserialize_optional_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(Option::<String>::deserialize(deserializer)?
        .and_then(|value| (!value.trim().is_empty()).then_some(value)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;

    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiError};

    const READ_INFO_ENDPOINTS: &[&str] = &["account-info", "coin", "nav", "stat", "today-coin-exp"];

    fn local_vip_info_probe_body(name: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/login/vip-info/vip-info/{name}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    fn local_read_info_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/login/read-info/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    fn read_info_payload<T>(endpoint: &str, profile: &str) -> Result<Option<T>, BpiError>
    where
        T: DeserializeOwned,
    {
        let Some(body) = local_read_info_probe_body(endpoint, profile) else {
            return Ok(None);
        };

        serde_json::from_value::<ApiEnvelope<T>>(body)?
            .into_payload()
            .map(Some)
    }

    fn fixture_bytes(
        endpoint: &str,
        case: &crate::probe::endpoint_contract::EndpointCase,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let fixture = case
            .response
            .fixture
            .as_deref()
            .ok_or_else(|| BpiError::unsupported_response("contract case missing fixture"))?;
        let path = format!("tests/contracts/login/{endpoint}/{fixture}");

        Ok(std::fs::read(path)?)
    }

    fn assert_fixture_matches_model(
        model: &str,
        bytes: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        match model {
            "LoginAccountInfo" => {
                let payload = ApiEnvelope::<LoginAccountInfo>::from_slice(bytes)?.into_payload()?;
                assert!(payload.mid.get() > 0);
            }
            "LoginCoinBalance" => {
                let payload = ApiEnvelope::<LoginCoinBalance>::from_slice(bytes)?.into_payload()?;
                assert!(payload.money >= 0.0);
            }
            "LoginNav" => {
                let payload = ApiEnvelope::<LoginNav>::from_slice(bytes)?.into_payload()?;
                assert!(payload.is_login);
            }
            "LoginStats" => {
                let payload = ApiEnvelope::<LoginStats>::from_slice(bytes)?.into_payload()?;
                assert!(payload.following > 0);
            }
            "LoginTodayCoinExp" => {
                let payload =
                    ApiEnvelope::<LoginTodayCoinExp>::from_slice(bytes)?.into_payload()?;
                assert!(payload.value <= 50);
            }
            "LoginDailyReward" => {
                let payload = ApiEnvelope::<LoginDailyReward>::from_slice(bytes)?.into_payload()?;
                assert!(payload.coins <= 50);
            }
            "LoginVipInfo" => {
                let payload = ApiEnvelope::<LoginVipInfo>::from_slice(bytes)?.into_payload()?;
                assert!(payload.mid.get() > 0);
            }
            _ => {
                return Err(Box::new(BpiError::unsupported_response(format!(
                    "unknown login response model {model}"
                ))));
            }
        }

        Ok(())
    }

    fn assert_login_contract_fixtures_parse(
        endpoint: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let contract = EndpointContract::from_slice(&std::fs::read(format!(
            "tests/contracts/login/{endpoint}/contract.json"
        ))?)?;

        for case in &contract.cases {
            if case.response.fixture.is_none() {
                assert!(
                    case.response.error.is_some() || case.response.http_status.is_some(),
                    "contract case without fixture must document observed error/status"
                );
                continue;
            }

            let bytes = fixture_bytes(endpoint, case)?;
            if let Some(model) = &case.response.rust_model {
                assert_fixture_matches_model(model, &bytes)?;
            } else if case.response.error.as_deref() == Some("requires_login") {
                let err = ApiEnvelope::<serde_json::Value>::from_slice(&bytes)?
                    .ensure_success()
                    .unwrap_err();
                assert!(err.requires_login());
            }
        }

        Ok(())
    }

    #[test]
    fn login_vip_info_matches_local_probe_outputs_when_available() -> Result<(), BpiError> {
        let Some(normal_body) = local_vip_info_probe_body("normal") else {
            return Ok(());
        };
        let Some(active_body) = local_vip_info_probe_body("vip") else {
            return Ok(());
        };

        let normal: LoginVipInfo =
            serde_json::from_value::<ApiEnvelope<LoginVipInfo>>(normal_body)?.into_payload()?;
        let active: LoginVipInfo =
            serde_json::from_value::<ApiEnvelope<LoginVipInfo>>(active_body)?.into_payload()?;

        assert!(!normal.is_active());
        assert!(active.is_active());
        Ok(())
    }

    #[test]
    fn login_vip_info_anonymous_probe_returns_login_required_when_available() -> Result<(), BpiError>
    {
        let Some(body) = local_vip_info_probe_body("anonymous") else {
            return Ok(());
        };

        let err = serde_json::from_value::<ApiEnvelope<LoginVipInfo>>(body)?
            .ensure_success()
            .unwrap_err();

        assert!(err.requires_login());
        Ok(())
    }

    #[test]
    fn login_read_info_models_match_local_probe_outputs_when_available() -> Result<(), BpiError> {
        for profile in ["normal", "vip"] {
            if let Some(nav) = read_info_payload::<LoginNav>("nav", profile)? {
                assert!(nav.is_login);
                assert!(nav.mid.is_some());
            }

            let _ = read_info_payload::<LoginStats>("stat", profile)?;

            if let Some(coin) = read_info_payload::<LoginCoinBalance>("coin", profile)? {
                assert!(coin.money >= 0.0);
            }

            if let Some(exp) = read_info_payload::<LoginTodayCoinExp>("today-coin-exp", profile)? {
                assert!(exp.value <= 50);
            }

            if let Some(account) = read_info_payload::<LoginAccountInfo>("account-info", profile)? {
                assert!(account.mid.get() > 0);
                assert!(!account.uname.trim().is_empty());
            }
        }
        Ok(())
    }

    #[test]
    fn login_read_info_anonymous_probes_return_login_required_when_available()
    -> Result<(), BpiError> {
        for endpoint in READ_INFO_ENDPOINTS {
            let Some(body) = local_read_info_probe_body(endpoint, "anonymous") else {
                continue;
            };

            let err = serde_json::from_value::<ApiEnvelope<serde_json::Value>>(body)?
                .ensure_success()
                .unwrap_err();

            assert!(err.requires_login());
        }
        Ok(())
    }

    #[test]
    fn login_contract_response_fixtures_parse_declared_models()
    -> Result<(), Box<dyn std::error::Error>> {
        assert_login_contract_fixtures_parse("vip-info")?;
        assert_login_contract_fixtures_parse("daily-reward")?;
        for endpoint in READ_INFO_ENDPOINTS {
            assert_login_contract_fixtures_parse(endpoint)?;
        }
        Ok(())
    }
}
