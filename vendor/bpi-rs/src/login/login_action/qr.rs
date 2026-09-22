use serde::{Deserialize, Serialize};

#[cfg(test)]
use crate::login::LoginQrPollParams;
#[cfg(test)]
use crate::{BpiClient, BpiError};

#[cfg(test)]
const QR_GENERATE_ENDPOINT: &str =
    "https://passport.bilibili.com/x/passport-login/web/qrcode/generate";
#[cfg(test)]
const QR_POLL_ENDPOINT: &str = "https://passport.bilibili.com/x/passport-login/web/qrcode/poll";

/// 生成 QRCode 数据
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct GenerateQrCodeData {
    /// 调用方渲染的 QR 登录 URL。
    pub url: String,
    /// 用于轮询 QR 登录状态的临时 key。
    pub qrcode_key: String,
}

/// 二维码状态数据
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct CheckQrCodeStatusData {
    pub url: String,           // 游戏分站跨域登录 url
    pub refresh_token: String, // 刷新令牌
    pub timestamp: u64,        // 时间戳
    pub code: i32,             // 状态码
    pub message: String,       // 扫码状态信息

    /// 仅在 QR 登录成功后返回的 cookie。
    #[serde(default)]
    pub cookies: Vec<(String, String)>,
}

/// 二维码图片数据
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QrcodeImageData {
    pub qr_image: String, // base64 编码的二维码图片
    pub expires_in: u64,  // 过期时间（秒）
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApiEnvelope;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::probe::flow::ProbeFlow;
    use tokio;

    fn local_qr_probe_body(endpoint: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/login/qr/{endpoint}/anonymous.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    fn local_qr_flow_probe() -> Option<serde_json::Value> {
        let bytes =
            std::fs::read("target/bpi-probe-runs/login/qr/flow/anonymous.response.json").ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    #[test]
    fn qr_generate_contract_matches_endpoint_request() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../../tests/contracts/login/qr/generate/contract.json"
        ))?;

        assert_eq!(contract.name, "login.qr_generate");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), QR_GENERATE_ENDPOINT);
        assert!(contract.request.query.is_empty());
        assert!(!contract.request.auth.requires_cookie());
        assert_eq!(contract.cases[0].response.api_code, Some(0));
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("GenerateQrCodeData")
        );
        Ok(())
    }

    #[test]
    fn qr_poll_contract_matches_endpoint_request() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../../tests/contracts/login/qr/poll/contract.json"
        ))?;

        assert_eq!(contract.name, "login.qr_poll");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), QR_POLL_ENDPOINT);
        assert_eq!(
            contract.request.query.get("qrcode_key").map(String::as_str),
            Some("${qrcode_key}")
        );
        assert!(!contract.request.auth.requires_cookie());
        assert_eq!(contract.cases[0].response.api_code, Some(0));
        Ok(())
    }

    #[test]
    fn qr_flow_contract_covers_generate_and_poll_requests() -> Result<(), BpiError> {
        let flow = ProbeFlow::from_slice(include_bytes!(
            "../../../tests/contracts/login/qr/flow/contract.json"
        ))?;

        assert_eq!(flow.name, "login.qr.flow");
        assert_eq!(flow.steps[0].name, "generate");
        assert_eq!(flow.steps[1].name, "poll");
        assert_eq!(
            flow.steps[0].extract["qrcode_key"],
            "/response/body/data/qrcode_key"
        );
        assert_eq!(
            flow.steps[1].contract["request"]["query"]["qrcode_key"],
            "${qrcode_key}"
        );
        Ok(())
    }

    #[test]
    fn qr_response_fixtures_parse_declared_models() -> Result<(), BpiError> {
        let generate = ApiEnvelope::<GenerateQrCodeData>::from_slice(include_bytes!(
            "../../../tests/contracts/login/qr/generate/responses/anonymous.success.json"
        ))?
        .into_payload()?;
        let poll = ApiEnvelope::<CheckQrCodeStatusData>::from_slice(include_bytes!(
            "../../../tests/contracts/login/qr/poll/responses/waiting.success.json"
        ))?
        .into_payload()?;

        assert!(generate.url.starts_with("https://"));
        assert_eq!(poll.code, 86101);
        Ok(())
    }

    #[test]
    fn qr_generate_model_matches_local_probe_output_when_available() -> Result<(), BpiError> {
        let Some(body) = local_qr_probe_body("generate") else {
            return Ok(());
        };

        let data =
            serde_json::from_value::<ApiEnvelope<GenerateQrCodeData>>(body)?.into_payload()?;

        assert!(data.url.starts_with("https://"));
        assert!(!data.qrcode_key.trim().is_empty());
        Ok(())
    }

    #[test]
    fn qr_poll_model_matches_local_probe_output_when_available() -> Result<(), BpiError> {
        let Some(body) = local_qr_probe_body("poll") else {
            return Ok(());
        };

        let response = serde_json::from_value::<ApiEnvelope<CheckQrCodeStatusData>>(body)?;
        let data = response.into_data()?;

        assert_eq!(data.code, 86101);
        assert!(!data.message.trim().is_empty());
        Ok(())
    }

    #[test]
    fn qr_flow_output_matches_local_probe_when_available() -> Result<(), BpiError> {
        let Some(flow) = local_qr_flow_probe() else {
            return Ok(());
        };

        let steps = flow["steps"]
            .as_array()
            .ok_or_else(|| BpiError::unsupported_response("flow output missing steps"))?;
        let generate = &steps[0]["result"]["response"]["body"];
        let poll = &steps[1]["result"]["response"]["body"];

        assert_eq!(generate["code"], 0);
        assert_eq!(poll["code"], 0);
        assert_eq!(poll["data"]["code"], 86101);
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_send_qrcode() {
        use tokio::time::{Duration, sleep};

        tracing::info!("获取二维码...");

        let bpi = BpiClient::new().expect("client should build");
        match bpi.login().qr_generate().await {
            Ok(data) => {
                tracing::info!("二维码URL: {}", data.url);
                tracing::info!("已获取二维码轮询 key");

                for _ in 1..=3 {
                    // 每次循环延迟 5 秒
                    sleep(Duration::from_secs(20)).await;
                    let params = LoginQrPollParams::new(data.qrcode_key.as_str()).unwrap();
                    let resp = bpi.login().qr_poll(params).await;

                    if resp.is_ok() {
                        tracing::info!("扫码成功{:?}", resp);
                    } else if let Err(error) = resp {
                        tracing::error!("扫码失败: {:?}", error);
                    }
                }
            }
            Err(e) => {
                tracing::info!("二维码请求失败: {}", e);
            }
        }
    }
}
