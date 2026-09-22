use serde::{Deserialize, Serialize};

/// 直播的已关注者列表项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiveUser {
    /// 直播者头像 URL
    pub face: String,
    /// 直播链接
    pub link: String,
    /// 直播标题
    pub title: String,
    /// 直播者 ID
    pub uid: u64,
    /// 直播者昵称
    pub uname: String,
}

/// 正在直播的已关注者响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiveUsersData {
    /// 直播者数量
    pub count: u64,
    /// 作用尚不明确
    pub group: String,
    /// 直播者列表
    pub items: Vec<LiveUser>,
}

/// 发布新动态的已关注者列表项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynUpUser {
    pub user_profile: UserProfile,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserProfile {
    pub info: UserInfo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserInfo {
    pub uid: u64,
    pub uname: String,
    pub face: String,
}

/// 发布新动态的已关注者响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynUpUsersData {
    /// 作用尚不明确
    pub button_statement: String,
    /// 更新者列表
    pub items: Vec<DynUpUser>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::params::{DynamicLiveUsersParams, DynamicUpUsersParams};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use std::collections::BTreeMap;
    use tracing::info;

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "live-users" => {
                include_bytes!("../../tests/contracts/dynamic/content/live-users/contract.json")
                    .as_slice()
            }
            "up-users" => {
                include_bytes!("../../tests/contracts/dynamic/content/up-users/contract.json")
                    .as_slice()
            }
            _ => unreachable!("unknown dynamic content endpoint"),
        };

        EndpointContract::from_slice(bytes)
    }

    fn query_map(query: Vec<(&'static str, String)>) -> BTreeMap<String, String> {
        query
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect()
    }

    // 您需要在 `Cargo.toml` 中添加 `dotenvy` 和 `tracing` 依赖，并在 `main.rs` 或测试入口处初始化日志
    // 例如: tracing_subscriber::fmt::init();

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_live_users() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .dynamic()
            .live_users(DynamicLiveUsersParams::new().with_size(1)?)
            .await?;

        info!("直播中的关注者数量: {}", data.count);
        info!("第一位直播中的关注者: {:?}", data.items.first());

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_dyn_up_users() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi.dynamic().up_users(DynamicUpUsersParams::new()).await?;

        info!("发布新动态的关注者列表: {:?}", data.items);
        assert!(!data.items.is_empty());

        Ok(())
    }

    #[test]
    fn dynamic_content_contracts_match_endpoint_requests() -> BpiResult<()> {
        let live_users = contract("live-users")?;
        assert_eq!(live_users.name, "dynamic.live_users");
        assert_eq!(live_users.request.method, HttpMethod::Get);
        assert_eq!(
            live_users.request.url.as_str(),
            "https://api.vc.bilibili.com/dynamic_svr/v1/dynamic_svr/w_live_users"
        );
        assert_eq!(
            live_users.request.query,
            query_map(DynamicLiveUsersParams::new().with_size(1)?.query_pairs())
        );
        assert_eq!(live_users.cases.len(), 3);
        assert_eq!(
            live_users.cases[0].response.error.as_deref(),
            Some("requires_login")
        );

        let up_users = contract("up-users")?;
        assert_eq!(up_users.name, "dynamic.up_users");
        assert_eq!(up_users.request.method, HttpMethod::Get);
        assert_eq!(
            up_users.request.url.as_str(),
            "https://api.vc.bilibili.com/dynamic_svr/v1/dynamic_svr/w_dyn_uplist"
        );
        assert_eq!(
            up_users.request.query,
            query_map(DynamicUpUsersParams::new().query_pairs())
        );
        assert_eq!(up_users.cases.len(), 3);
        assert_eq!(
            up_users.cases[0].response.error.as_deref(),
            Some("requires_login")
        );
        Ok(())
    }

    #[test]
    fn dynamic_content_response_fixtures_parse_declared_models() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/dynamic/content/live-users/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/dynamic/content/live-users/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<LiveUsersData>::from_slice(bytes)?.into_payload()?;
            assert_eq!(payload.group, "default");
        }

        for bytes in [
            include_bytes!(
                "../../tests/contracts/dynamic/content/up-users/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/dynamic/content/up-users/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let _ = ApiEnvelope::<DynUpUsersData>::from_slice(bytes)?.into_payload()?;
        }
        Ok(())
    }

    #[test]
    fn dynamic_content_anonymous_fixtures_record_login_errors() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/dynamic/content/live-users/responses/anonymous.requires_login.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/dynamic/content/up-users/responses/anonymous.requires_login.json"
            )
            .as_slice(),
        ] {
            let err = ApiEnvelope::<serde_json::Value>::from_slice(bytes)?
                .ensure_success()
                .unwrap_err();
            assert_eq!(err.code(), Some(4100000));
        }
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path = format!(
            "target/bpi-probe-runs/dynamic/content-readonly/{endpoint}/{profile}.response.json"
        );
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn dynamic_content_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["normal", "vip"] {
            if let Some(body) = local_probe_body("live-users", profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<LiveUsersData>>(body)?.into_payload()?;
                assert_eq!(payload.group, "default");
            }

            if let Some(body) = local_probe_body("up-users", profile) {
                let _ =
                    serde_json::from_value::<ApiEnvelope<DynUpUsersData>>(body)?.into_payload()?;
            }
        }

        for endpoint in ["live-users", "up-users"] {
            if let Some(body) = local_probe_body(endpoint, "anonymous") {
                let err = serde_json::from_value::<ApiEnvelope<serde_json::Value>>(body)?
                    .ensure_success()
                    .unwrap_err();
                assert_eq!(err.code(), Some(4100000));
            }
        }
        Ok(())
    }
}
