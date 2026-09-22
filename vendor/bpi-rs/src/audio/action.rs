// 音频投币&收藏
//
// [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/audio/action.md)
//

use crate::BilibiliRequest;
use crate::audio::AudioClient;
use crate::audio::params::{AudioCoinParams, AudioCollectionToFavParams, AudioCollectionToParams};
use crate::response::BpiResult;
use serde::{Deserialize, Serialize};

const COLLECTION_TO_FAV_ENDPOINT: &str =
    "https://api.bilibili.com/medialist/gateway/coll/resource/deal";
const COLLECTION_TO_ENDPOINT: &str =
    "https://www.bilibili.com/audio/music-service-c/web/collections/songs-coll";
const COIN_ENDPOINT: &str = "https://www.bilibili.com/audio/music-service-c/web/coin/add";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptData {
    /// 是否为未关注用户收藏
    prompt: bool,
}

impl<'a> AudioClient<'a> {
    /// 将音频歌曲添加到收藏夹，或从收藏夹中移除。
    pub async fn favorite(&self, params: AudioCollectionToFavParams) -> BpiResult<PromptData> {
        let csrf = self.client.csrf()?;

        self.client
            .post(COLLECTION_TO_FAV_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_payload("audio.favorite")
            .await
    }

    /// 将音频歌曲添加到合集。
    pub async fn collect(&self, params: AudioCollectionToParams) -> BpiResult<bool> {
        let csrf = self.client.csrf()?;

        self.client
            .post(COLLECTION_TO_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_payload("audio.collect")
            .await
    }

    /// 给音频歌曲投币并返回标准 payload 结果。
    pub async fn coin(&self, params: AudioCoinParams) -> BpiResult<String> {
        let csrf = self.client.csrf()?;

        self.client
            .post(COIN_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_payload("audio.coin")
            .await
    }
}

#[cfg(test)]
mod tests {

    use crate::audio::params::AudioSongParams;
    use crate::ids::AudioId;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiResult};

    // https://www.bilibili.com/audio/au13598

    const TEST_SID: u64 = 13603;

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "collection-status" => {
                include_bytes!("../../tests/contracts/audio/collection-status/contract.json")
                    .as_slice()
            }
            "coin-count" => {
                include_bytes!("../../tests/contracts/audio/coin-count/contract.json").as_slice()
            }
            _ => unreachable!("unknown audio action contract"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn audio_collection_status_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("collection-status")?;
        let params = AudioSongParams::new(AudioId::new(TEST_SID)?);

        assert_eq!(contract.name, "audio.collection_status");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://www.bilibili.com/audio/music-service-c/web/collections/songs-coll"
        );
        assert_eq!(
            contract.request.query.get("sid").map(String::as_str),
            Some("13603")
        );
        assert_eq!(params.query_pairs(), vec![("sid", "13603".to_string())]);
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.error.as_deref(),
            Some("requires_login")
        );
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("bool")
        );
        Ok(())
    }

    #[test]
    fn audio_collection_status_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/audio/collection-status/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/audio/collection-status/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let _payload = ApiEnvelope::<bool>::from_slice(bytes)?.into_payload()?;
        }
        Ok(())
    }

    #[test]
    fn audio_collection_status_anonymous_fixture_records_login_error() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/audio/collection-status/responses/anonymous.error.json"
        ))?
        .ensure_success()
        .unwrap_err();

        assert_eq!(err.code(), Some(4_511_003));
        Ok(())
    }

    #[test]
    fn audio_coin_count_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("coin-count")?;
        let params = AudioSongParams::new(AudioId::new(TEST_SID)?);

        assert_eq!(contract.name, "audio.coin_count");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://www.bilibili.com/audio/music-service-c/web/coin/audio"
        );
        assert_eq!(
            contract.request.query.get("sid").map(String::as_str),
            Some("13603")
        );
        assert_eq!(params.query_pairs(), vec![("sid", "13603".to_string())]);
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.error.as_deref(),
            Some("requires_login")
        );
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("i32")
        );
        Ok(())
    }

    #[test]
    fn audio_coin_count_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!("../../tests/contracts/audio/coin-count/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/audio/coin-count/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<i32>::from_slice(bytes)?.into_payload()?;

            assert!((0..=2).contains(&payload));
        }
        Ok(())
    }

    #[test]
    fn audio_coin_count_anonymous_fixture_records_login_error() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/audio/coin-count/responses/anonymous.error.json"
        ))?
        .ensure_success()
        .unwrap_err();

        assert_eq!(err.code(), Some(4_511_003));
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/audio/public-read/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn audio_action_read_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["normal", "vip"] {
            if let Some(body) = local_probe_body("collection-status", profile) {
                let _payload = serde_json::from_value::<ApiEnvelope<bool>>(body)?.into_payload()?;
            }

            if let Some(body) = local_probe_body("coin-count", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<i32>>(body)?.into_payload()?;

                assert!((0..=2).contains(&payload));
            }
        }
        Ok(())
    }
}
