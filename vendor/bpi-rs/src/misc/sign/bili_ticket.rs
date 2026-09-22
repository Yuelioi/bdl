//! 用于生成 bili_ticket
//!
//! bili_ticket 位于请求头 Cookie 中, 非必需, 但存在可降低风控概率
//! 是 JWT 令牌，有效时长为 259200 秒，即 3 天。
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/misc/sign/bili_ticket.md)

use serde::{Deserialize, Serialize};

#[cfg(test)]
const BILI_TICKET_ENDPOINT: &str =
    "https://api.bilibili.com/bapis/bilibili.api.ticket.v1.Ticket/GenWebTicket";

/// bili_ticket 响应数据
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TicketData {
    /// bili_ticket JWT 令牌
    pub ticket: String,
    /// 创建时间 UNIX 秒级时间戳
    pub created_at: i64,
    /// 有效时长 259200 秒 (3 天)
    pub ttl: i32,
    /// 空对象
    pub context: serde_json::Value,
    /// WBI 相关信息
    pub nav: NavData,
}

/// WBI 导航数据
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NavData {
    /// img_key 值
    pub img: String,
    /// sub_key 值
    pub sub: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApiEnvelope;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::sign::bili_ticket::{hexsign, ticket_request_params};
    use crate::{BpiClient, BpiError};

    fn local_bili_ticket_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/misc/sign/bili-ticket/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn hmac_sha256_returns_hex_digest() -> Result<(), BpiError> {
        let result = hexsign("XgwSnGZ1p", 1_234_567_890)?;

        assert_eq!(result.len(), 64);
        assert!(result.chars().all(|c| c.is_ascii_hexdigit()));
        Ok(())
    }

    #[test]
    fn bili_ticket_contract_matches_endpoint_request() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../../tests/contracts/misc/sign/bili-ticket/contract.json"
        ))?;

        assert_eq!(contract.name, "misc.bili_ticket");
        assert_eq!(contract.request.method, HttpMethod::Post);
        assert_eq!(contract.request.url.as_str(), BILI_TICKET_ENDPOINT);
        assert_eq!(
            contract.request.query.get("key_id").map(String::as_str),
            Some("ec02")
        );
        assert_eq!(
            contract
                .request
                .query
                .get("context[ts]")
                .map(String::as_str),
            Some("${unix_ts}")
        );
        assert_eq!(
            contract.request.query.get("hexsign").map(String::as_str),
            Some("${bili_ticket_hexsign}")
        );
        assert_eq!(
            contract.request.query.get("csrf").map(String::as_str),
            Some("${csrf}")
        );
        assert_eq!(contract.cases.len(), 3);
        Ok(())
    }

    #[test]
    fn bili_ticket_contract_covers_guest_and_authenticated_profiles() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../../tests/contracts/misc/sign/bili-ticket/contract.json"
        ))?;

        let anonymous = &contract.cases[0];
        assert_eq!(anonymous.profile.as_deref(), Some("anonymous"));
        assert!(!anonymous.auth.requires_cookie());
        assert_eq!(anonymous.response.api_code, Some(0));

        for case in &contract.cases[1..] {
            assert!(matches!(case.name.as_str(), "normal" | "vip"));
            assert!(case.auth.requires_cookie());
            assert!(case.auth.requires_csrf());
            assert_eq!(case.response.rust_model.as_deref(), Some("TicketData"));
        }
        Ok(())
    }

    #[test]
    fn bili_ticket_response_fixture_parses_declared_model() -> Result<(), BpiError> {
        let data = ApiEnvelope::<TicketData>::from_slice(include_bytes!(
            "../../../tests/contracts/misc/sign/bili-ticket/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(data.ttl, 259_200);
        assert_eq!(data.ticket.split('.').count(), 3);
        assert!(data.nav.img.starts_with("https://"));
        assert!(data.nav.sub.starts_with("https://"));
        Ok(())
    }

    #[test]
    fn bili_ticket_model_matches_local_probe_outputs_when_available() -> Result<(), BpiError> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_bili_ticket_probe_body(profile) else {
                continue;
            };

            let data = serde_json::from_value::<ApiEnvelope<TicketData>>(body)?.into_payload()?;

            assert_eq!(data.ttl, 259_200);
            assert_eq!(data.ticket.split('.').count(), 3);
            assert!(!data.nav.img.trim().is_empty());
            assert!(!data.nav.sub.trim().is_empty());
        }
        Ok(())
    }

    #[test]
    fn bili_ticket_client_can_build_guest_request_params() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        assert!(client.get_account().is_none());
        assert!(client.csrf().is_err());
        let params = ticket_request_params(1_234_567_890, "")?;

        assert_eq!(params[3], ("csrf".to_string(), "".to_string()));
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_generate_bili_ticket() -> Result<(), BpiError> {
        let Some(bpi) = live_client_or_skip()? else {
            return Ok(());
        };

        match bpi.misc().bili_ticket().await {
            Ok(data) => {
                tracing::info!("Ticket: {}", data.ticket);
                tracing::info!("创建时间: {}", data.created_at);
                tracing::info!(
                    "有效时长: {} 秒 ({:.1} 天)",
                    data.ttl,
                    (data.ttl as f64) / 86400.0
                );
                tracing::info!("WBI img: {}", data.nav.img);
                tracing::info!("WBI sub: {}", data.nav.sub);

                // 验证 ticket 是 JWT 格式
                assert!(data.ticket.contains('.'));
                assert!(data.ttl > 250000); // 大约 3 天
            }

            Err(err) => {
                panic!("生成 bili_ticket 失败: {}", err);
            }
        }

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_bili_ticket_string() -> Result<(), BpiError> {
        let Some(bpi) = live_client_or_skip()? else {
            return Ok(());
        };

        match bpi.misc().bili_ticket_string().await {
            Ok(ticket) => {
                tracing::info!("获取到的 bili_ticket: {}", ticket);

                // 验证 ticket 格式
                assert!(!ticket.is_empty());
                assert!(ticket.contains('.'));

                // JWT 应该有 3 部分（header.payload.signature）
                let parts: Vec<&str> = ticket.split('.').collect();
                assert_eq!(parts.len(), 3);
            }
            Err(err) => {
                panic!("获取 bili_ticket 字符串失败: {}", err);
            }
        }

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_with_csrf() -> Result<(), BpiError> {
        let Some(bpi) = live_client_or_skip()? else {
            return Ok(());
        };

        // 测试带 CSRF 的情况
        match bpi.misc().bili_ticket().await {
            Ok(data) => {
                tracing::info!("带 CSRF 的 bili_ticket 生成成功: {}", data.ticket);
            }
            Err(err) => {
                tracing::info!("带 CSRF 测试失败（预期可能失败）: {}", err);
                // 这里不 panic，因为没有真实的 CSRF token 可能会失败
            }
        }

        Ok(())
    }

    fn live_client_or_skip() -> Result<Option<BpiClient>, BpiError> {
        if std::env::var("BPI_LIVE_TEST").ok().as_deref() != Some("1") {
            return Ok(None);
        }

        let Some(cookie) = std::env::var("BPI_COOKIE")
            .ok()
            .filter(|value| !value.is_empty())
        else {
            return Ok(None);
        };

        BpiClient::builder().cookie(cookie).build().map(Some)
    }
}
