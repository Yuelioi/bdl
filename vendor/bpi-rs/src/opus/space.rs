//! 空间图文
//!
//! [空间图文](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/opus/space.md#空间图文)

use serde::{Deserialize, Serialize};

/// 空间图文封面信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpaceCover {
    /// 封面高度
    pub height: u32,
    /// 图片 URL
    pub url: String,
    /// 封面宽度
    pub width: u32,
}

/// 空间图文统计信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpaceStat {
    /// 点赞数（字符串）
    pub like: String,
    /// 浏览数（字符串，仅自己可见）
    pub view: Option<String>,
}

/// 空间图文单条信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpaceItem {
    /// 文本内容
    pub content: String,
    /// 封面信息，可选
    pub cover: Option<SpaceCover>,
    /// 跳转 URL
    pub jump_url: String,
    /// opus id
    pub opus_id: String,
    /// 统计信息
    pub stat: SpaceStat,
}

/// 空间图文响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpaceData {
    /// 是否还有更多
    pub has_more: bool,
    /// 图文列表
    pub items: Vec<SpaceItem>,
    /// 下一页 offset
    pub offset: String,
    /// 更新数
    pub update_num: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::Mid;
    use crate::opus::{OpusSpaceFeedKind, OpusSpaceFeedParams};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use tracing::info;

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/opus/space-read/space-feed/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_opus_space_feed() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let params = OpusSpaceFeedParams::new(Mid::new(1000001)?)
            .with_page(1)
            .with_kind(OpusSpaceFeedKind::All);
        let resp = bpi.opus().space_feed(params).await;
        assert!(resp.is_ok());
        if let Ok(r) = resp {
            info!("空间图文返回: {:?}", r);
        }
        Ok(())
    }

    #[test]
    fn opus_space_feed_params_serializes_default_query() -> Result<(), BpiError> {
        let params = OpusSpaceFeedParams::new(Mid::new(1000001)?);

        assert_eq!(
            params.query_pairs(),
            [
                ("host_mid", "1000001".to_string()),
                ("page", "0".to_string()),
                ("type", "all".to_string()),
                ("web_location", "333.1387".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn opus_space_feed_params_serializes_optional_query() -> Result<(), BpiError> {
        let params = OpusSpaceFeedParams::new(Mid::new(1000001)?)
            .with_page(2)
            .with_offset("offset-token")?
            .with_kind(OpusSpaceFeedKind::Article);

        assert_eq!(
            params.query_pairs(),
            [
                ("host_mid", "1000001".to_string()),
                ("page", "2".to_string()),
                ("offset", "offset-token".to_string()),
                ("type", "article".to_string()),
                ("web_location", "333.1387".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn opus_space_feed_params_rejects_blank_offset() -> Result<(), BpiError> {
        let err = OpusSpaceFeedParams::new(Mid::new(1000001)?)
            .with_offset("   ")
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "offset",
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn opus_space_feed_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "opus.space_feed");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/polymer/web-dynamic/v1/opus/feed/space"
        );
        assert_eq!(
            contract.request.query.get("host_mid").map(String::as_str),
            Some("1000001")
        );
        assert_eq!(
            contract.request.query.get("page").map(String::as_str),
            Some("0")
        );
        assert_eq!(
            contract.request.query.get("type").map(String::as_str),
            Some("all")
        );
        assert_eq!(
            contract
                .request
                .query
                .get("web_location")
                .map(String::as_str),
            Some("333.1387")
        );
        assert_eq!(contract.cases.len(), 3);
        for case in &contract.cases {
            assert_eq!(case.response.api_code, Some(0));
            assert_eq!(case.response.rust_model.as_deref(), Some("SpaceData"));
        }
        Ok(())
    }

    #[test]
    fn opus_space_feed_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<SpaceData>::from_slice(include_bytes!(
            "../../tests/contracts/opus/space-read/space-feed/responses/success.json"
        ))?
        .into_payload()?;

        assert!(payload.has_more);
        assert_eq!(payload.items.len(), 2);
        assert!(payload.items[0].cover.is_some());
        assert!(payload.items[1].cover.is_none());
        assert_eq!(payload.items[1].stat.view.as_deref(), Some("0"));
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/opus/space-read/space-feed/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn opus_space_feed_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload = serde_json::from_value::<ApiEnvelope<SpaceData>>(body)?.into_payload()?;

            assert!(!payload.items.is_empty());
            assert!(!payload.offset.is_empty());
        }
        Ok(())
    }
}
