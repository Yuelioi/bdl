use serde::{Deserialize, Serialize};

/// 默认搜索内容
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DefaultSearchData {
    /// 搜索 seid
    pub seid: String,
    /// 默认搜索 id
    pub id: u64,
    /// 类型，固定 0
    pub r#type: u32,
    /// 显示文字
    pub show_name: String,
    /// 空字段
    pub name: Option<String>,
    /// 跳转类型，1: 视频
    pub goto_type: u32,
    /// 搜索目标 id，视频为稿件 avid
    pub goto_value: String,
    /// 搜索目标跳转 url
    pub url: String,
}

/// 热搜条目
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HotWordItem {
    pub hot_id: u64,
    pub keyword: String,
    pub show_name: String,

    pub heat_score: u64,
    pub word_type: u32,

    // pub icon: Option<String>,
    // pub resource_id: Option<u64>,
    pub live_id: Option<Vec<serde_json::Value>>,
    // pub name_type: Option<String>,
}

/// 热搜返回数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HotWordDataResponse {
    pub code: u32,
    pub list: Vec<HotWordItem>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ApiEnvelope, BpiClient, BpiError, BpiResult,
        probe::{contract::HttpMethod, endpoint_contract::EndpointContract},
    };

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes: &[u8] = match endpoint {
            "default" => include_bytes!("../../tests/contracts/search/read/default/contract.json"),
            "hotwords" => {
                include_bytes!("../../tests/contracts/search/read/hotwords/contract.json")
            }
            _ => {
                return Err(BpiError::invalid_parameter(
                    "endpoint",
                    "unknown search hot contract",
                ));
            }
        };

        EndpointContract::from_slice(bytes)
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/search/read/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn search_default_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("default")?;

        assert_eq!(contract.name, "search.default");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/web-interface/wbi/search/default"
        );
        assert_eq!(
            contract.request.query.get("foo").map(String::as_str),
            Some("bar")
        );
        assert!(contract.request.auth.requires_wbi());
        for case in &contract.cases {
            assert_eq!(case.response.http_status, Some(200));
            assert_eq!(case.response.api_code, Some(0));
            assert_eq!(
                case.response.rust_model.as_deref(),
                Some("DefaultSearchData")
            );
        }
        Ok(())
    }

    #[test]
    fn search_default_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<DefaultSearchData>::from_slice(include_bytes!(
            "../../tests/contracts/search/read/default/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.show_name, "sanitized keyword");
        Ok(())
    }

    #[test]
    fn search_default_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body("default", profile) else {
                continue;
            };

            let payload =
                serde_json::from_value::<ApiEnvelope<DefaultSearchData>>(body)?.into_payload()?;
            assert!(!payload.seid.is_empty());
        }
        Ok(())
    }

    #[test]
    fn search_hotwords_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("hotwords")?;

        assert_eq!(contract.name, "search.hotwords");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://s.search.bilibili.com/main/hotword"
        );
        assert!(!contract.request.auth.requires_wbi());
        for case in &contract.cases {
            assert_eq!(case.response.http_status, Some(200));
            assert_eq!(case.response.api_code, Some(0));
            assert_eq!(
                case.response.rust_model.as_deref(),
                Some("HotWordDataResponse")
            );
        }
        Ok(())
    }

    #[test]
    fn search_hotwords_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = serde_json::from_slice::<HotWordDataResponse>(include_bytes!(
            "../../tests/contracts/search/read/hotwords/responses/success.json"
        ))?;

        assert_eq!(payload.code, 0);
        assert_eq!(payload.list.len(), 1);
        Ok(())
    }

    #[test]
    fn search_hotwords_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body("hotwords", profile) else {
                continue;
            };

            let payload = serde_json::from_value::<HotWordDataResponse>(body)?;
            assert_eq!(payload.code, 0);
        }
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_default_search() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi.search().default().await?;
        tracing::info!("{:#?}", data);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_hotword_list() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi.search().hotwords().await?;
        tracing::info!("{:#?}", data);

        Ok(())
    }
}
