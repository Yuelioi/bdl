//! 歌单&音频收藏夹详细信息
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/audio/music_list.md)
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioCollectionsListData {
    #[serde(rename = "curPage")]
    pub cur_page: i32,

    #[serde(rename = "pageCount")]
    pub page_count: i32,

    #[serde(rename = "totalSize")]
    pub total_size: i32,

    #[serde(rename = "pageSize")]
    pub page_size: i32,

    pub data: Vec<AudioCollection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioCollection {
    pub id: i64,
    pub uid: i64,
    pub uname: String,
    pub title: String,
    pub r#type: i32,
    pub published: i32,
    pub cover: String,
    pub ctime: i64,
    pub song: i32,
    pub desc: String,
    pub sids: Vec<i64>,
    #[serde(rename = "menuId")]
    pub menu_id: i64,
    pub statistic: AudioCollectionStatistic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioCollectionStatistic {
    pub sid: i64,
    pub play: i64,
    pub collect: i64,
    pub comment: Option<i64>,
    pub share: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioHotMenuData {
    #[serde(rename = "curPage")]
    pub cur_page: i32,

    #[serde(rename = "pageCount")]
    pub page_count: i32,

    #[serde(rename = "totalSize")]
    pub total_size: i32,

    #[serde(rename = "pageSize")]
    pub page_size: i32,

    pub data: Vec<AudioHotMenu>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioHotMenu {
    #[serde(rename = "menuId")]
    pub menu_id: i64,

    pub uid: i64,

    pub uname: String,

    pub title: String,

    pub cover: String,

    pub intro: String,

    pub r#type: i32,

    pub off: i32,

    pub ctime: i64,

    pub curtime: i64,

    pub statistic: AudioHotMenuStatistic,

    pub snum: i32,

    pub attr: i32,

    #[serde(rename = "isDefault")]
    pub is_default: i32,

    #[serde(rename = "collectionId")]
    pub collection_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioHotMenuStatistic {
    pub sid: i64,
    pub play: i64,
    pub collect: i64,
    pub comment: i64,
    pub share: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioRankMenuData {
    #[serde(rename = "curPage")]
    pub cur_page: i32,

    #[serde(rename = "pageCount")]
    pub page_count: i32,

    #[serde(rename = "totalSize")]
    pub total_size: i32,

    #[serde(rename = "pageSize")]
    pub page_size: i32,

    pub data: Vec<AudioRankMenu>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioRankMenu {
    #[serde(rename = "menuId")]
    pub menu_id: i64,

    pub uid: i64,

    pub uname: String,

    pub title: String,

    pub cover: String,

    pub intro: String,

    pub r#type: i32,

    pub off: i32,

    pub ctime: i64,

    pub curtime: i64,

    pub statistic: AudioRankMenuStatistic,

    pub snum: i32,

    pub attr: i32,

    #[serde(rename = "isDefault")]
    pub is_default: i32,

    #[serde(rename = "collectionId")]
    pub collection_id: i64,

    pub audios: Vec<AudioRankItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioRankMenuStatistic {
    pub sid: i64,
    pub play: i64,
    pub collect: i64,
    pub comment: i64,
    pub share: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioRankItem {
    pub id: i64,
    pub title: String,
    pub duration: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::params::{AudioCollectionInfoParams, AudioPageParams};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiResult};

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "collections-list" => {
                include_bytes!("../../tests/contracts/audio/collections-list/contract.json")
                    .as_slice()
            }
            "collection-info" => {
                include_bytes!("../../tests/contracts/audio/collection-info/contract.json")
                    .as_slice()
            }
            "hot-menu" => {
                include_bytes!("../../tests/contracts/audio/hot-menu/contract.json").as_slice()
            }
            "rank-menu" => {
                include_bytes!("../../tests/contracts/audio/rank-menu/contract.json").as_slice()
            }
            _ => unreachable!("unknown audio music list contract"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_audio_collections_list() {
        let bpi = BpiClient::new().expect("client should build");
        let result = bpi
            .audio()
            .collections_list(AudioPageParams::new(1, 2).expect("valid page params"))
            .await;
        if let Ok(data) = result {
            assert!(data.cur_page >= 1);
            assert!(data.page_size > 0);
        }
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_audio_collection_info() {
        let bpi = BpiClient::new().expect("client should build");
        let result = bpi
            .audio()
            .collection_info(AudioCollectionInfoParams::new(15967839).expect("valid collection id"))
            .await;
        assert!(result.is_ok());
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_audio_hot_menu() {
        let bpi = BpiClient::new().expect("client should build");
        let result = bpi
            .audio()
            .hot_menu(AudioPageParams::new(1, 3).expect("valid page params"))
            .await;
        assert!(result.is_ok());
        let data = result.unwrap();

        assert!(data.cur_page >= 1);
        assert!(data.page_size > 0);
        assert!(!data.data.is_empty());
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_audio_rank_menu() {
        let bpi = BpiClient::new().expect("client should build");
        let result = bpi
            .audio()
            .rank_menu(AudioPageParams::new(1, 6).expect("valid page params"))
            .await;
        assert!(result.is_ok());
        let data = result.unwrap();

        assert!(data.cur_page >= 1);
        assert!(data.page_size > 0);
        assert!(!data.data.is_empty());
        // 检查榜单中的音频信息
        for menu in &data.data {
            assert!(!menu.audios.is_empty());
            for audio in &menu.audios {
                assert!(audio.id > 0);
                assert!(!audio.title.is_empty());
                assert!(audio.duration > 0);
            }
        }
    }

    #[test]
    fn audio_collections_list_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("collections-list")?;
        let params = AudioPageParams::new(1, 2)?;

        assert_eq!(contract.name, "audio.collections_list");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://www.bilibili.com/audio/music-service-c/web/collections/list"
        );
        assert_eq!(
            contract.request.query.get("pn").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            contract.request.query.get("ps").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            params.query_pairs(),
            vec![("pn", "1".to_string()), ("ps", "2".to_string())]
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.error.as_deref(),
            Some("requires_login")
        );
        Ok(())
    }

    #[test]
    fn audio_collections_list_response_fixtures_parse_declared_model() -> BpiResult<()> {
        let anonymous = ApiEnvelope::<AudioCollectionsListData>::from_slice(include_bytes!(
            "../../tests/contracts/audio/collections-list/responses/anonymous.error.json"
        ))?;
        assert_eq!(anonymous.code, 4_511_003);

        for bytes in [
            include_bytes!(
                "../../tests/contracts/audio/collections-list/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/audio/collections-list/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload =
                ApiEnvelope::<AudioCollectionsListData>::from_slice(bytes)?.into_payload()?;

            assert!(payload.total_size >= 0);
        }
        Ok(())
    }

    #[test]
    fn audio_collection_info_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("collection-info")?;
        let params = AudioCollectionInfoParams::new(15_967_839)?;

        assert_eq!(contract.name, "audio.collection_info");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://www.bilibili.com/audio/music-service-c/web/collections/info"
        );
        assert_eq!(
            contract.request.query.get("sid").map(String::as_str),
            Some("15967839")
        );
        assert_eq!(params.query_pairs(), vec![("sid", "15967839".to_string())]);
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("Option<AudioCollection>")
        );
        Ok(())
    }

    #[test]
    fn audio_collection_info_response_fixtures_record_optional_success_payload() -> BpiResult<()> {
        let anonymous = ApiEnvelope::<AudioCollection>::from_slice(include_bytes!(
            "../../tests/contracts/audio/collection-info/responses/anonymous.error.json"
        ))?;
        assert_eq!(anonymous.code, 4_511_003);

        for bytes in [
            include_bytes!(
                "../../tests/contracts/audio/collection-info/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/audio/collection-info/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload =
                ApiEnvelope::<AudioCollection>::from_slice(bytes)?.into_optional_payload()?;

            assert!(payload.is_none());
        }
        Ok(())
    }

    #[test]
    fn audio_hot_menu_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("hot-menu")?;

        assert_eq!(contract.name, "audio.hot_menu");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://www.bilibili.com/audio/music-service-c/web/menu/hit"
        );
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("AudioHotMenuData")
        );
        Ok(())
    }

    #[test]
    fn audio_hot_menu_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<AudioHotMenuData>::from_slice(include_bytes!(
            "../../tests/contracts/audio/hot-menu/responses/success.json"
        ))?
        .into_payload()?;

        assert!(!payload.data.is_empty());
        Ok(())
    }

    #[test]
    fn audio_rank_menu_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("rank-menu")?;

        assert_eq!(contract.name, "audio.rank_menu");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://www.bilibili.com/audio/music-service-c/web/menu/rank"
        );
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("AudioRankMenuData")
        );
        Ok(())
    }

    #[test]
    fn audio_rank_menu_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<AudioRankMenuData>::from_slice(include_bytes!(
            "../../tests/contracts/audio/rank-menu/responses/success.json"
        ))?
        .into_payload()?;

        assert!(!payload.data.is_empty());
        assert!(!payload.data[0].audios.is_empty());
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
    fn audio_music_list_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body("collections-list", profile) {
                let envelope =
                    serde_json::from_value::<ApiEnvelope<AudioCollectionsListData>>(body)?;
                if profile == "anonymous" {
                    assert_eq!(envelope.code, 4_511_003);
                } else {
                    let payload = envelope.into_payload()?;
                    assert!(payload.total_size >= 0);
                }
            }

            if let Some(body) = local_probe_body("collection-info", profile) {
                let envelope = serde_json::from_value::<ApiEnvelope<AudioCollection>>(body)?;
                if profile == "anonymous" {
                    assert_eq!(envelope.code, 4_511_003);
                } else {
                    let _payload = envelope.into_optional_payload()?;
                }
            }

            if let Some(body) = local_probe_body("hot-menu", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<AudioHotMenuData>>(body)?
                    .into_payload()?;

                assert!(!payload.data.is_empty());
            }

            if let Some(body) = local_probe_body("rank-menu", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<AudioRankMenuData>>(body)?
                    .into_payload()?;

                assert!(!payload.data.is_empty());
            }
        }
        Ok(())
    }
}
