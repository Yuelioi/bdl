use crate::BpiError;
use serde::Deserialize;

/// 搜索建议结果
#[derive(Debug, Deserialize)]
pub struct SearchSuggest {
    pub tag: Option<Vec<SearchSuggestItem>>,
}

/// 搜索建议项
#[derive(Debug, Deserialize)]
pub struct SearchSuggestItem {
    pub value: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub item_type: Option<String>,
}

/// 搜索建议 endpoint 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchSuggestParams {
    term: String,
}

impl SearchSuggestParams {
    pub fn new(term: impl Into<String>) -> Result<Self, BpiError> {
        let term = term.into().trim().to_string();
        if term.is_empty() {
            return Err(BpiError::invalid_parameter(
                "term",
                "search suggestion term cannot be blank",
            ));
        }

        Ok(Self { term })
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("term", self.term.clone())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ApiEnvelope, BpiClient, BpiResult,
        probe::{contract::HttpMethod, endpoint_contract::EndpointContract},
    };
    use tracing::info;

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/search/read/suggest/contract.json"
        ))
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/search/read/suggest/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn search_suggest_params_serializes_term_query() -> Result<(), BpiError> {
        let params = SearchSuggestParams::new("rust lang")?;

        assert_eq!(
            params.query_pairs(),
            vec![("term", "rust lang".to_string())]
        );
        Ok(())
    }

    #[test]
    fn search_suggest_params_trims_term() -> Result<(), BpiError> {
        let params = SearchSuggestParams::new("  rust  ")?;

        assert_eq!(params.query_pairs(), vec![("term", "rust".to_string())]);
        Ok(())
    }

    #[test]
    fn search_suggest_params_rejects_blank_term() {
        let err = SearchSuggestParams::new(" \t ").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "term", .. }
        ));
    }

    #[test]
    fn search_suggest_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "search.suggest");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://s.search.bilibili.com/main/suggest"
        );
        assert_eq!(
            contract.request.query.get("term").map(String::as_str),
            Some("rust")
        );
        assert!(!contract.request.auth.requires_wbi());
        for case in &contract.cases {
            assert_eq!(case.response.http_status, Some(200));
            assert_eq!(case.response.api_code, Some(0));
            assert_eq!(case.response.rust_model.as_deref(), Some("SearchSuggest"));
        }
        Ok(())
    }

    #[test]
    fn search_suggest_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<SearchSuggest>::from_slice(include_bytes!(
            "../../tests/contracts/search/read/suggest/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.tag.unwrap_or_default().len(), 1);
        Ok(())
    }

    #[test]
    fn search_suggest_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };

            let payload =
                serde_json::from_value::<ApiEnvelope<SearchSuggest>>(body)?.into_payload()?;
            assert!(payload.tag.is_some());
        }
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_search_suggest() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let params = SearchSuggestParams::new("rust")?;
        let resp = bpi.search().suggest(params).await;

        assert!(resp.is_ok());

        if let Ok(suggests) = resp {
            info!("搜索建议返回: {:?}", suggests);

            if let Some(tags) = suggests.tag {
                assert!(!tags.is_empty());
                info!("获取到搜索建议列表，数量：{}", tags.len());

                if let Some(first_suggest) = tags.first() {
                    info!("第一个建议关键词: {:?}", first_suggest.value);
                    info!("第一个建议显示内容: {:?}", first_suggest.name);
                }
            } else {
                info!("搜索建议列表为空。");
            }
        }

        Ok(())
    }
}
