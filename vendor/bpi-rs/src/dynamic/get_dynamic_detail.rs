use serde::{Deserialize, Deserializer, Serialize, de};

use crate::models::{LevelInfo, Official, Pendant, Vip};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct DynamicCardData {
    pub card: DynamicCard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicCard {
    pub desc: Desc,
    pub card: String,
    pub extend_json: String,
    pub display: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Desc {
    pub uid: i64,
    #[serde(rename = "type")]
    pub type_field: i64,
    pub rid: i64,
    pub acl: i64,
    pub view: i64,
    pub repost: i64,
    pub comment: i64,
    pub like: i64,
    pub is_liked: i64,
    pub dynamic_id: i64,
    pub timestamp: i64,
    pub pre_dy_id: i64,
    pub orig_dy_id: i64,
    pub orig_type: i64,
    pub user_profile: UserProfile,
    pub spec_type: i64,
    pub uid_type: i64,
    pub stype: i64,
    pub r_type: i64,
    pub inner_id: i64,
    pub status: i64,
    pub dynamic_id_str: String,
    pub pre_dy_id_str: String,
    pub orig_dy_id_str: String,
    pub rid_str: String,
    pub bvid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub info: Info,
    pub card: Card,
    pub vip: Vip,
    pub pendant: Pendant,
    pub rank: String,
    pub sign: String,
    pub level_info: LevelInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Info {
    pub uid: i64,
    pub uname: String,
    pub face: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub official_verify: OfficialVerify,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficialVerify {
    #[serde(rename = "type")]
    pub type_field: i64,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct RecentUpData {
    /// 直播用户（暂不明确，可能为 null）
    pub live_users: Option<serde_json::Value>,
    /// 我的信息
    pub my_info: Option<MyInfo>,
    /// 最近更新的 UP 主列表
    pub up_list: Vec<UpUser>,
}

/// 我的信息对象
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct MyInfo {
    /// 个人动态数
    #[serde(deserialize_with = "deserialize_i32_from_string_or_number")]
    pub dyns: i32,
    /// 头像地址
    pub face: String,
    /// 粉丝数
    pub follower: String,
    /// 我的关注数
    #[serde(deserialize_with = "deserialize_i32_from_string_or_number")]
    pub following: i32,
    /// 等级信息
    pub level_info: LevelInfo,
    /// 用户 mid
    #[serde(deserialize_with = "deserialize_i64_from_string_or_number")]
    pub mid: i64,
    /// 用户昵称
    pub name: String,
    /// 认证信息
    #[serde(rename = "official")]
    pub official: Official,
    /// 个人空间背景图
    pub space_bg: String,
    /// 会员信息
    pub vip: Vip,
}

/// 最近更新的 UP 主
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct UpUser {
    /// 头像
    pub face: String,
    /// 是否有更新
    pub has_update: bool,
    /// 作用不明
    pub is_reserve_recall: bool,
    /// 用户 mid
    #[serde(deserialize_with = "deserialize_i64_from_string_or_number")]
    pub mid: i64,
    /// 用户昵称
    pub uname: String,
}

fn deserialize_i32_from_string_or_number<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let value = parse_i64_from_string_or_number(value)?;
    i32::try_from(value).map_err(|_| de::Error::custom("value must fit in i32"))
}

fn deserialize_i64_from_string_or_number<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    parse_i64_from_string_or_number(serde_json::Value::deserialize(deserializer)?)
}

fn parse_i64_from_string_or_number<E>(value: serde_json::Value) -> Result<i64, E>
where
    E: de::Error,
{
    match value {
        serde_json::Value::Number(number) => number
            .as_i64()
            .ok_or_else(|| E::custom("value must be an integer")),
        serde_json::Value::String(text) => text
            .parse::<i64>()
            .map_err(|_| E::custom("value must be a numeric string")),
        _ => Err(E::custom("value must be a string or number")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiResult};

    fn recent_up_contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/dynamic/content/recent-up/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_dynamic_recent_up_list() {
        let bpi = BpiClient::new().expect("client should build");
        let resp = bpi.dynamic().recent_up().await;
        assert!(resp.is_ok());
        if let Ok(data) = resp {
            tracing::info!("{:#?}", data.up_list.len());
        }
    }

    #[test]
    fn dynamic_recent_up_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = recent_up_contract()?;

        assert_eq!(contract.name, "dynamic.recent_up");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/polymer/web-dynamic/v1/portal"
        );
        assert!(contract.request.query.is_empty());
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.error.as_deref(),
            Some("requires_login")
        );
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("RecentUpData")
        );
        Ok(())
    }

    #[test]
    fn dynamic_recent_up_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/dynamic/content/recent-up/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/dynamic/content/recent-up/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<RecentUpData>::from_slice(bytes)?.into_payload()?;
            let my_info = payload
                .my_info
                .expect("sanitized fixture should include my_info");
            assert_eq!(my_info.dyns, 0);
            assert_eq!(my_info.following, 0);
            assert_eq!(my_info.mid, 1);
        }
        Ok(())
    }

    #[test]
    fn dynamic_recent_up_anonymous_fixture_records_login_error() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/dynamic/content/recent-up/responses/anonymous.requires_login.json"
        ))?
        .ensure_success()
        .unwrap_err();

        assert_eq!(err.code(), Some(-101));
        Ok(())
    }

    fn recent_up_local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!(
            "target/bpi-probe-runs/dynamic/content-readonly/recent-up/{profile}.response.json"
        );
        local_probe_response_body(&path)
    }

    fn local_probe_response_body(path: &str) -> Option<serde_json::Value> {
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn dynamic_recent_up_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["normal", "vip"] {
            let Some(body) = recent_up_local_probe_body(profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<RecentUpData>>(body)?.into_payload()?;
            assert!(payload.my_info.is_some());
        }

        if let Some(body) = recent_up_local_probe_body("anonymous") {
            let err = serde_json::from_value::<ApiEnvelope<serde_json::Value>>(body)?
                .ensure_success()
                .unwrap_err();
            assert_eq!(err.code(), Some(-101));
        }
        Ok(())
    }
}
