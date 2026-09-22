use serde::{Deserialize, Serialize};

#[cfg(test)]
use crate::{BpiClient, BpiError};

#[cfg(test)]
const CAPTCHA_GENERATE_ENDPOINT: &str = "https://passport.bilibili.com/x/passport-login/captcha";
#[cfg(test)]
const CAPTCHA_SOURCE: &str = "main_web";

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeetestData {
    #[serde(rename = "type")]
    pub type_field: String,
    pub token: String,
    pub geetest: Geetest,
    pub tencent: Tencent,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Geetest {
    pub challenge: String,
    pub gt: String,
}

#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tencent {
    pub appid: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenerateCaptcha {
    pub token: String,
    pub gt: String,
    pub challenge: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApiEnvelope;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;

    fn local_captcha_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/login/captcha/generate/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn captcha_generate_contract_matches_endpoint_request() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../../tests/contracts/login/captcha/generate/contract.json"
        ))?;

        assert_eq!(contract.name, "login.captcha_generate");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), CAPTCHA_GENERATE_ENDPOINT);
        assert_eq!(
            contract.request.query.get("source").map(String::as_str),
            Some(CAPTCHA_SOURCE)
        );
        assert!(!contract.request.auth.requires_cookie());
        assert_eq!(contract.cases.len(), 3);
        assert!(contract.cases.iter().all(|case| {
            case.response.api_code == Some(0)
                && case.response.rust_model.as_deref() == Some("GeetestData")
        }));
        Ok(())
    }

    #[test]
    fn captcha_generate_contract_covers_all_profiles() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../../tests/contracts/login/captcha/generate/contract.json"
        ))?;

        let anonymous = &contract.cases[0];
        let normal = &contract.cases[1];
        let vip = &contract.cases[2];

        assert_eq!(anonymous.profile.as_deref(), Some("anonymous"));
        assert!(!anonymous.auth.requires_cookie());
        assert_eq!(normal.profile.as_deref(), Some("normal"));
        assert!(normal.auth.requires_cookie());
        assert_eq!(vip.profile.as_deref(), Some("vip"));
        assert!(vip.auth.requires_cookie());
        Ok(())
    }

    #[test]
    fn captcha_response_fixture_parses_declared_model() -> Result<(), BpiError> {
        let data = ApiEnvelope::<GeetestData>::from_slice(include_bytes!(
            "../../../tests/contracts/login/captcha/generate/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(data.type_field, "geetest");
        assert!(!data.token.trim().is_empty());
        assert!(!data.geetest.gt.trim().is_empty());
        assert!(!data.geetest.challenge.trim().is_empty());
        Ok(())
    }

    #[test]
    fn generate_captcha_projection_uses_geetest_payload() -> Result<(), BpiError> {
        let data = ApiEnvelope::<GeetestData>::from_slice(include_bytes!(
            "../../../tests/contracts/login/captcha/generate/responses/success.json"
        ))?
        .into_payload()?;

        let captcha = GenerateCaptcha {
            token: data.token,
            gt: data.geetest.gt,
            challenge: data.geetest.challenge,
        };

        assert!(captcha.token.starts_with("sanitized-"));
        assert!(captcha.gt.starts_with("sanitized-"));
        assert!(captcha.challenge.starts_with("sanitized-"));
        Ok(())
    }

    #[test]
    fn captcha_model_matches_local_probe_outputs_when_available() -> Result<(), BpiError> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_captcha_probe_body(profile) else {
                continue;
            };

            let data = serde_json::from_value::<ApiEnvelope<GeetestData>>(body)?.into_payload()?;

            assert_eq!(data.type_field, "geetest");
            assert_eq!(data.token.len(), 32);
            assert_eq!(data.geetest.gt.len(), 32);
            assert_eq!(data.geetest.challenge.len(), 32);
        }
        Ok(())
    }
}

#[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
#[tokio::test]
async fn test_generate_captcha() {
    let bpi = BpiClient::new().expect("client should build");
    match bpi.login().generate_captcha().await {
        Ok(captcha) => {
            tracing::info!("验证码请求成功！");
            tracing::info!("Token: {}", captcha.token);
            tracing::info!("GT: {}", captcha.gt);
            tracing::info!("Challenge: {}", captcha.challenge);
        }
        Err(e) => {
            tracing::info!("验证码请求失败: {}", e);
        }
    }
}
