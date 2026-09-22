use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct QualityDescription {
    /// 画质代码
    pub qn: i32,
    /// 该代码对应的画质名称
    pub desc: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct LiveStreamUrl {
    /// 直播流url
    pub url: String,
    /// 服务器线路序号
    pub order: i32,
    /// 作用尚不明确
    pub stream_type: i32,
    /// 作用尚不明确
    pub p2p_type: i32,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct LiveStreamData {
    /// 当前画质代码qn
    pub current_quality: i32,
    /// 可选画质数参数
    pub accept_quality: Vec<String>,
    /// 当前画质代码quality
    pub current_qn: i32,
    /// 可选画质参数quality
    pub quality_description: Vec<QualityDescription>,
    /// 直播流url组
    pub durl: Vec<LiveStreamUrl>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/live/public-core/stream/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_live_stream() {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .live()
            .stream(14073662, Some("web"), None, Some(10000))
            .await
            .unwrap();
        tracing::info!("{:?}", data);
    }

    #[test]
    fn live_stream_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "live.stream");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.live.bilibili.com/room/v1/Room/playUrl"
        );
        assert_eq!(
            contract.request.query.get("cid").map(String::as_str),
            Some("14073662")
        );
        assert_eq!(
            contract.request.query.get("platform").map(String::as_str),
            Some("web")
        );
        assert_eq!(
            contract.request.query.get("qn").map(String::as_str),
            Some("10000")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("LiveStreamData")
        );
        Ok(())
    }

    #[test]
    fn live_stream_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<LiveStreamData>::from_slice(include_bytes!(
            "../../tests/contracts/live/public-core/stream/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.current_qn, 10000);
        assert_eq!(payload.durl.len(), 1);
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/live/public-core/stream/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn live_stream_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body(profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<LiveStreamData>>(body)?.into_payload()?;
                assert!(!payload.durl.is_empty());
            }
        }
        Ok(())
    }
}
