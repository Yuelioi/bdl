use serde::{Deserialize, Serialize};

#[cfg(test)]
use crate::BpiClient;
#[cfg(test)]
use crate::BpiError;
use crate::ids::Mid;
#[cfg(test)]
use crate::login::params::{LoginLogParams, LoginNoticeParams};

// --- API 结构体 ---

/// 查询指定登录记录的响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginNoticeData {
    pub mid: Mid,
    pub device_name: String,
    pub login_type: String,
    pub login_time: String,
    pub location: String,
    pub ip: String,
}

/// 最近一周登录情况的响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginLogData {
    pub count: u32,
    pub list: Vec<LoginLogEntry>,
}

/// 登录日志列表中的单条记录
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoginLogEntry {
    pub ip: String,
    pub time: u64,
    pub time_at: String,
    pub status: bool,
    #[serde(rename = "type")]
    pub login_type: u8,
    #[serde(rename = "geo")]
    pub location: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApiEnvelope;
    use crate::ids::Mid;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;

    fn contract(endpoint: &str) -> Result<EndpointContract, BpiError> {
        let bytes = match endpoint {
            "login-notice" => {
                include_bytes!("../../tests/contracts/login/notice/login-notice/contract.json")
                    .as_slice()
            }
            "login-log" => {
                include_bytes!("../../tests/contracts/login/notice/login-log/contract.json")
                    .as_slice()
            }
            _ => unreachable!("unknown login notice contract endpoint"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_login_notice() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let mid = 1000001;

        let data = bpi
            .login()
            .notice(LoginNoticeParams::new(Mid::new(mid)?))
            .await?;

        println!("指定登录记录:");
        println!("  设备名: {}", data.device_name);
        println!("  登录方式: {}", data.login_type);
        println!("  登录时间: {}", data.login_time);
        println!("  登录位置: {}", data.location);
        println!("  IP: {}", data.ip);

        assert_eq!(data.mid.get(), mid);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_login_log() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");

        let data = bpi.login().log(LoginLogParams::new()).await?;

        println!("最近一周登录记录 (共 {} 条):", data.count);
        for entry in data.list {
            println!("  时间: {} ({})", entry.time_at, entry.time);
            println!("    IP: {}", entry.ip);
            println!("    位置: {}", entry.location);
            println!("    登录成功: {}", entry.status);
            println!("    登录类型: {}", entry.login_type);
        }

        Ok(())
    }

    #[test]
    fn login_notice_contract_matches_endpoint_request() -> Result<(), BpiError> {
        let contract = contract("login-notice")?;

        assert_eq!(contract.name, "login.notice");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/safecenter/login_notice"
        );
        assert_eq!(
            contract.request.query.get("mid").map(String::as_str),
            Some("1000001")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(contract.cases[0].response.api_code, Some(-101));
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("LoginNoticeData")
        );
        Ok(())
    }

    #[test]
    fn login_log_contract_matches_endpoint_request() -> Result<(), BpiError> {
        let contract = contract("login-log")?;

        assert_eq!(contract.name, "login.log");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/member/web/login/log"
        );
        assert_eq!(
            contract.request.query.get("jsonp").map(String::as_str),
            Some("jsonp")
        );
        assert_eq!(
            contract
                .request
                .query
                .get("web_location")
                .map(String::as_str),
            Some("333.33")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(contract.cases[0].response.api_code, Some(-101));
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("LoginLogData")
        );
        Ok(())
    }

    #[test]
    fn login_notice_response_fixtures_parse_declared_models() -> Result<(), BpiError> {
        let normal = ApiEnvelope::<LoginNoticeData>::from_slice(include_bytes!(
            "../../tests/contracts/login/notice/login-notice/responses/normal.success.json"
        ))?
        .into_payload()?;
        let vip = ApiEnvelope::<LoginNoticeData>::from_slice(include_bytes!(
            "../../tests/contracts/login/notice/login-notice/responses/vip.success.json"
        ))?
        .into_payload()?;
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/login/notice/login-notice/responses/anonymous.error.json"
        ))?
        .ensure_success()
        .unwrap_err();

        assert_eq!(normal.mid.get(), 1000001);
        assert_eq!(vip.mid.get(), 1000002);
        assert!(err.requires_login());
        Ok(())
    }

    #[test]
    fn login_log_response_fixtures_parse_declared_models() -> Result<(), BpiError> {
        let normal = ApiEnvelope::<LoginLogData>::from_slice(include_bytes!(
            "../../tests/contracts/login/notice/login-log/responses/normal.success.json"
        ))?
        .into_payload()?;
        let vip = ApiEnvelope::<LoginLogData>::from_slice(include_bytes!(
            "../../tests/contracts/login/notice/login-log/responses/vip.success.json"
        ))?
        .into_payload()?;
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/login/notice/login-log/responses/anonymous.error.json"
        ))?
        .ensure_success()
        .unwrap_err();

        assert_eq!(normal.count, 1);
        assert_eq!(vip.list[0].ip, "203.0.113.20");
        assert!(err.requires_login());
        Ok(())
    }
}
