//! 音频状态数
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/audio/status_number.md)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStatusNumberData {
    pub sid: i64,
    pub play: i64,
    pub collect: i64,
    pub comment: i64,
    pub share: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::params::AudioSongParams;
    use crate::ids::AudioId;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    const TEST_SID: u64 = 13603;

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/audio/status-number/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_audio_status_number() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .audio()
            .status_number(AudioSongParams::new(AudioId::new(TEST_SID)?))
            .await?;
        tracing::info!("{:#?}", data);

        assert_eq!(data.sid, TEST_SID as i64);
        assert!(data.play >= 0);
        assert!(data.collect >= 0);
        assert!(data.comment >= 0);
        assert!(data.share >= 0);

        Ok(())
    }

    #[test]
    fn audio_status_number_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params = AudioSongParams::new(AudioId::new(TEST_SID)?);

        assert_eq!(contract.name, "audio.status_number");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://www.bilibili.com/audio/music-service-c/web/stat/song"
        );
        assert_eq!(
            contract.request.query.get("sid").map(String::as_str),
            Some("13603")
        );
        assert_eq!(params.query_pairs(), vec![("sid", "13603".to_string())]);
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("AudioStatusNumberData")
        );
        Ok(())
    }

    #[test]
    fn audio_status_number_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/audio/status-number/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/audio/status-number/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!("../../tests/contracts/audio/status-number/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload =
                ApiEnvelope::<AudioStatusNumberData>::from_slice(bytes)?.into_payload()?;

            assert_eq!(payload.sid, TEST_SID as i64);
        }
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!(
            "target/bpi-probe-runs/audio/public-read/status-number/{profile}.response.json"
        );
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn audio_status_number_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload = serde_json::from_value::<ApiEnvelope<AudioStatusNumberData>>(body)?
                .into_payload()?;

            assert_eq!(payload.sid, TEST_SID as i64);
        }
        Ok(())
    }
}
