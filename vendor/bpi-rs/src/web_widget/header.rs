//! B站首页头图相关接口
//!
//! [查看 API 文档](https://socialsisteryi.github.io/bilibili-API-collect/docs/web_widget/header.html)
use serde::{Deserialize, Serialize};

use crate::BpiError;

/// B站首页头图数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HeaderData {
    /// 空
    pub name: String,
    /// 静态头图 URL
    pub pic: String,
    /// Bilibili 标志 URL
    pub litpic: String,
    /// 空
    pub url: String,
    /// 是否分层, 1: 是
    pub is_split_layer: u32,
    /// 分层信息，一个套在字符串里的 JSON 对象
    pub split_layer: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub split_layer_obj: Option<SplitLayer>,
}

impl HeaderData {
    pub fn parse_split_layer(&mut self) -> Result<(), BpiError> {
        let result = serde_json::from_str(&self.split_layer);
        match result {
            Ok(r) => {
                self.split_layer_obj = Some(r);
                Ok(())
            }
            Err(e) => Err(BpiError::parse(format!("解析split_layer失败: {:?}", e))),
        }
    }
}

/// 分层信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SplitLayer {
    /// 版本号
    pub version: String,
    /// 层信息
    pub layers: Vec<Layer>,
}
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Layer {
    pub resources: Vec<Resource>,
    pub scale: Scale,
    pub rotate: Rotate,
    pub translate: Translate,
    pub blur: Blur,
    pub opacity: Opacity,
    pub id: i64,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub src: String,
    pub id: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scale {
    pub initial: Option<f64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Rotate {
    pub offset: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Translate {
    pub offset: Option<Vec<i64>>,
    pub initial: Option<Vec<i64>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Blur {
    pub initial: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Opacity {
    pub wrap: String,
    pub initial: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::web_widget::params::WebWidgetHeaderPageParams;
    use crate::{ApiEnvelope, BpiClient, BpiResult};
    use tracing::info;

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/web_widget/header-page/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_header_page() {
        let bpi = BpiClient::new().expect("client should build");
        let resp = bpi
            .web_widget()
            .header_page(WebWidgetHeaderPageParams::new())
            .await;
        info!("响应: {:?}", resp);
        assert!(resp.is_ok());
    }

    #[test]
    fn web_widget_header_page_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "web_widget.header_page");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/web-show/page/header"
        );
        assert_eq!(
            contract
                .request
                .query
                .get("resource_id")
                .map(String::as_str),
            Some("142")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("HeaderData")
        );
        Ok(())
    }

    #[test]
    fn web_widget_header_page_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/web_widget/header-page/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/web_widget/header-page/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/web_widget/header-page/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let mut payload = ApiEnvelope::<HeaderData>::from_slice(bytes)?.into_payload()?;

            assert!(payload.split_layer_obj.is_none());
            payload.parse_split_layer()?;
            assert!(payload.split_layer_obj.is_some());
        }
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/web_widget/public/header-page/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn web_widget_header_page_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let mut payload =
                serde_json::from_value::<ApiEnvelope<HeaderData>>(body)?.into_payload()?;

            payload.parse_split_layer()?;
            assert!(payload.split_layer_obj.is_some());
        }
        Ok(())
    }
}
