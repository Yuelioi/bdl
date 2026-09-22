//! 用于生成 b23.tv 短链
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/misc/b23tv.md)

use serde::{Deserialize, Serialize};

#[cfg(test)]
const B23_SHORT_LINK_ENDPOINT: &str = "https://api.biliapi.net/x/share/click";

/// 生成 b23.tv 短链 - 响应数据
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortLinkData {
    /// 原始返回内容（标题 + 短链）
    pub content: String,

    /// 恒为 0
    pub count: i32,

    /// 纯短链 URL
    #[serde(skip_serializing, skip_deserializing)]
    pub link: String,
    /// 标题
    #[serde(skip_serializing, skip_deserializing)]
    pub title: String,
}

impl ShortLinkData {
    pub fn extract(&mut self) {
        if let Some(pos) = self.content.find("https://b23.tv/") {
            self.link = self.content[pos..].to_string().trim().to_string();
            self.title = self.content[..pos].trim().to_string();
        } else {
            self.link = String::new();
            self.title = self.content.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApiEnvelope;
    use crate::BpiError;
    use crate::ids::Aid;
    use crate::misc::MiscB23ShortLinkParams;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;

    fn local_b23_short_link_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/misc/b23tv/short-link/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn misc_b23_short_link_contract_matches_endpoint_request() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/misc/b23tv/short-link/contract.json"
        ))?;

        assert_eq!(contract.name, "misc.b23tv.short_link");
        assert_eq!(contract.request.method, HttpMethod::Post);
        assert_eq!(contract.request.url.as_str(), B23_SHORT_LINK_ENDPOINT);
        assert!(contract.request.query.is_empty());

        let form = contract
            .request
            .form
            .as_ref()
            .ok_or_else(|| BpiError::unsupported_response("missing b23tv contract form"))?;
        assert_eq!(form.get("platform").map(String::as_str), Some("unix"));
        assert_eq!(form.get("share_channel").map(String::as_str), Some("COPY"));
        assert_eq!(
            form.get("share_id").map(String::as_str),
            Some("main.ugc-video-detail.0.0.pv")
        );
        assert_eq!(form.get("share_mode").map(String::as_str), Some("4"));
        assert_eq!(form.get("oid").map(String::as_str), Some("10001"));
        assert_eq!(form.get("buvid").map(String::as_str), Some("qwq"));
        assert_eq!(form.get("build").map(String::as_str), Some("6114514"));
        assert_eq!(contract.cases.len(), 3);
        Ok(())
    }

    #[test]
    fn misc_b23_short_link_contract_covers_profiles() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/misc/b23tv/short-link/contract.json"
        ))?;

        let anonymous = &contract.cases[0];
        assert_eq!(anonymous.profile.as_deref(), Some("anonymous"));
        assert!(!anonymous.auth.requires_cookie());

        for case in &contract.cases[1..] {
            assert!(matches!(case.name.as_str(), "normal" | "vip"));
            assert!(case.auth.requires_cookie());
            assert_eq!(case.response.api_code, Some(0));
            assert_eq!(case.response.rust_model.as_deref(), Some("ShortLinkData"));
        }
        Ok(())
    }

    #[test]
    fn misc_b23_short_link_response_fixture_parses_declared_model() -> Result<(), BpiError> {
        let mut data = ApiEnvelope::<ShortLinkData>::from_slice(include_bytes!(
            "../../tests/contracts/misc/b23tv/short-link/responses/success.json"
        ))?
        .into_payload()?;

        data.extract();

        assert_eq!(data.count, 0);
        assert_eq!(data.title, "sanitized-title");
        assert_eq!(data.link, "https://b23.tv/sanitized");
        Ok(())
    }

    #[test]
    fn misc_b23_short_link_model_matches_local_probe_outputs_when_available() -> Result<(), BpiError>
    {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_b23_short_link_probe_body(profile) else {
                continue;
            };

            let mut data =
                serde_json::from_value::<ApiEnvelope<ShortLinkData>>(body)?.into_payload()?;
            data.extract();

            assert_eq!(data.count, 0);
            assert!(data.link.starts_with("https://b23.tv/"));
            assert!(!data.title.trim().is_empty());
        }
        Ok(())
    }

    #[test]
    fn misc_b23_short_link_params_serializes_default_form() -> Result<(), BpiError> {
        let params = MiscB23ShortLinkParams::new(Aid::new(10001)?);

        assert_eq!(
            params.form_pairs(),
            [
                ("platform", "unix".to_string()),
                ("share_channel", "COPY".to_string()),
                ("share_id", "main.ugc-video-detail.0.0.pv".to_string()),
                ("share_mode", "4".to_string()),
                ("oid", "10001".to_string()),
                ("buvid", "qwq".to_string()),
                ("build", "6114514".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn misc_b23_short_link_params_serializes_custom_form() -> Result<(), BpiError> {
        let params = MiscB23ShortLinkParams::new(Aid::new(10001)?)
            .with_platform("web")?
            .with_share_channel("WEIXIN")?
            .with_share_id("custom.share.id")?
            .with_share_mode(5)
            .with_buvid("custom-buvid")?
            .with_build(123456);

        assert_eq!(
            params.form_pairs(),
            [
                ("platform", "web".to_string()),
                ("share_channel", "WEIXIN".to_string()),
                ("share_id", "custom.share.id".to_string()),
                ("share_mode", "5".to_string()),
                ("oid", "10001".to_string()),
                ("buvid", "custom-buvid".to_string()),
                ("build", "123456".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn misc_b23_short_link_params_rejects_blank_buvid() -> Result<(), BpiError> {
        let err = MiscB23ShortLinkParams::new(Aid::new(10001)?)
            .with_buvid("   ")
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "buvid", .. }
        ));
        Ok(())
    }
}
