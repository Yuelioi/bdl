//! 音频流URL
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/audio/musicstream_url.md)
use serde::{Deserialize, Serialize};

/// 音质参数定义
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioQuality {
    /// 流畅 128K
    Smooth = 0,
    /// 标准 192K
    Standard = 1,
    /// 高品质 320K
    HighQuality = 2,
    /// 无损 FLAC （大会员）
    Lossless = 3,
}

impl AudioQuality {
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStreamUrlWebData {
    pub sid: u64,
    pub r#type: u32,
    pub info: String,
    pub timeout: u64,
    pub size: u64,
    pub cdns: Vec<String>,
    pub qualities: Option<serde_json::Value>,
    pub title: String,
    pub cover: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStreamUrlData {
    pub sid: u64,
    pub r#type: u32,
    pub info: String,
    pub timeout: u64,
    pub size: u64,
    pub cdns: Vec<String>,
    pub qualities: Vec<AudioQualityInfo>,
    pub title: String,
    pub cover: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioQualityInfo {
    pub r#type: u32,
    pub desc: String,
    pub size: u64,
    pub bps: String,
    pub tag: String,
    pub require: u32,
    pub requiredesc: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::params::{AudioStreamUrlParams, AudioStreamUrlWebParams};
    use crate::ids::AudioId;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiResult};

    const TEST_SID: u64 = 13603;
    const TEST_STREAM_SID: u64 = 15664;

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "stream-url-web" => {
                include_bytes!("../../tests/contracts/audio/stream-url-web/contract.json")
                    .as_slice()
            }
            "stream-url" => {
                include_bytes!("../../tests/contracts/audio/stream-url/contract.json").as_slice()
            }
            _ => unreachable!("unknown audio stream contract"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_audio_stream_url_web() {
        let bpi = BpiClient::new().expect("client should build");
        let result = bpi
            .audio()
            .stream_url_web(AudioStreamUrlWebParams::new(
                AudioId::new(TEST_SID).expect("valid audio id"),
            ))
            .await;
        assert!(result.is_ok());
        let data = result.unwrap();

        assert_eq!(data.sid, TEST_SID);
        assert!(data.timeout > 0);
        assert!(!data.cdns.is_empty());
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_audio_stream_url() {
        let bpi = BpiClient::new().expect("client should build");
        let result = bpi
            .audio()
            .stream_url(AudioStreamUrlParams::new(
                AudioId::new(15664).expect("valid audio id"),
                AudioQuality::HighQuality,
            ))
            .await;
        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.sid, 15664);
        assert!(data.timeout > 0);
        assert!(!data.cdns.is_empty());
        assert!(!data.qualities.is_empty());
    }

    #[test]
    fn audio_stream_url_web_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("stream-url-web")?;
        let params = AudioStreamUrlWebParams::new(AudioId::new(TEST_SID)?);

        assert_eq!(contract.name, "audio.stream_url_web");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://www.bilibili.com/audio/music-service-c/web/url"
        );
        assert_eq!(
            contract.request.query.get("sid").map(String::as_str),
            Some("13603")
        );
        assert_eq!(
            params.query_pairs(),
            vec![
                ("sid", "13603".to_string()),
                ("quality", "2".to_string()),
                ("privilege", "2".to_string()),
            ]
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("AudioStreamUrlWebData")
        );
        Ok(())
    }

    #[test]
    fn audio_stream_url_web_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<AudioStreamUrlWebData>::from_slice(include_bytes!(
            "../../tests/contracts/audio/stream-url-web/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.sid, TEST_SID);
        assert_eq!(
            payload.cdns,
            vec!["https://example.invalid/audio/stream-url-web.m4a"]
        );
        Ok(())
    }

    #[test]
    fn audio_stream_url_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("stream-url")?;
        let params =
            AudioStreamUrlParams::new(AudioId::new(TEST_STREAM_SID)?, AudioQuality::HighQuality);

        assert_eq!(contract.name, "audio.stream_url");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/audio/music-service-c/url"
        );
        assert_eq!(
            contract.request.query.get("songid").map(String::as_str),
            Some("15664")
        );
        assert_eq!(
            params.query_pairs(),
            vec![
                ("songid", "15664".to_string()),
                ("quality", "2".to_string()),
                ("privilege", "2".to_string()),
                ("mid", "2".to_string()),
                ("platform", "android".to_string()),
            ]
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("AudioStreamUrlData")
        );
        Ok(())
    }

    #[test]
    fn audio_stream_url_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<AudioStreamUrlData>::from_slice(include_bytes!(
            "../../tests/contracts/audio/stream-url/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.sid, TEST_STREAM_SID);
        assert!(!payload.qualities.is_empty());
        assert_eq!(
            payload.cdns,
            vec!["https://example.invalid/audio/stream-url.m4a"]
        );
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/audio/extra-read/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn audio_stream_url_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body("stream-url-web", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<AudioStreamUrlWebData>>(body)?
                    .into_payload()?;

                assert_eq!(payload.sid, TEST_SID);
            }

            if let Some(body) = local_probe_body("stream-url", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<AudioStreamUrlData>>(body)?
                    .into_payload()?;

                assert_eq!(payload.sid, TEST_STREAM_SID);
                assert!(!payload.qualities.is_empty());
            }
        }
        Ok(())
    }
}
