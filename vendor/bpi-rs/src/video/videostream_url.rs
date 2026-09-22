//! 视频流地址相关接口 (web端)
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/video)
use serde::{Deserialize, Deserializer, Serialize};

pub(crate) const PLAY_URL_ENDPOINT: &str = "https://api.bilibili.com/x/player/wbi/playurl";

// --- 视频流URL相关数据结构体 ---

/// DASH 流信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DashInfo {
    pub video: Vec<DashStream>,
    pub audio: Vec<DashStream>,
    #[serde(rename = "dolby")]
    pub dolby: Option<DashDolby>,
    pub flac: Option<DashFlac>,
    pub duration: u64,
}

/// DASH 流中的 Dolby 音频信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DashDolby {
    pub r#type: u8,
    pub audio: Option<Vec<DashStream>>,
}

/// DASH 流中的 FLAC 音频信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DashFlac {
    #[serde(default)]
    pub display: Option<bool>,
    /// Hi-Res 音轨；上游在无可用音轨时返回 `null`
    #[serde(default)]
    pub audio: Option<DashStream>,
}

/// 单个 DASH 流信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DashStream {
    pub id: u64,
    #[serde(rename = "baseUrl")]
    pub base_url: String,

    #[serde(
        rename = "backupUrl",
        default,
        deserialize_with = "deserialize_vec_or_default"
    )]
    pub backup_url: Vec<String>,
    pub bandwidth: u64,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub codecs: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    #[serde(rename = "frameRate")]
    pub frame_rate: Option<String>,
    pub sar: Option<String>,
    pub start_with_sap: Option<u8>,
    pub segment_base: Option<serde_json::Value>,
    pub md5: Option<String>,
    pub size: Option<u64>,
    pub db_type: Option<u8>,
    pub r#type: Option<String>,
    pub stream_name: Option<String>,
    pub orientation: Option<u8>,
}

/// FLV/MP4 视频分段流信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DurlInfo {
    pub order: u32,
    pub length: u64,
    pub size: u64,
    pub ahead: String,
    pub vhead: String,
    pub url: String,
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub backup_url: Vec<String>,
}

fn deserialize_vec_or_default<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}

/// 支持的格式详细信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SupportFormat {
    pub quality: u64,
    pub format: String,
    pub new_description: String,
    pub display_desc: String,
    pub superscript: String,
    pub codecs: Option<Vec<String>>,
}

/// 视频流URL响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayUrlResponseData {
    pub from: String,
    pub result: String,
    pub message: String,
    pub quality: u64,
    pub format: String,
    pub timelength: u64,
    pub accept_format: String,
    pub accept_description: Vec<String>,
    pub accept_quality: Vec<u64>,
    pub video_codecid: u8,
    pub seek_param: String,
    pub seek_type: String,
    pub durl: Option<Vec<DurlInfo>>,
    pub dash: Option<DashInfo>,
    pub support_formats: Vec<SupportFormat>,
    pub high_format: Option<serde_json::Value>,
    pub last_play_time: i64,
    pub last_play_cid: i64,
}

// --- 测试模块 ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{Aid, Cid};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::video::params::VideoPlayUrlParams;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use tracing::info;

    const TEST_AID: u64 = 113898824998659;
    const TEST_CID: u64 = 28104724389;

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/video/playurl/play-url/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_playurl_mp4_by_aid() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let params = VideoPlayUrlParams::from_aid(Aid::new(TEST_AID)?, Cid::new(TEST_CID)?)
            .quality(64)
            .format_flags(1);
        let data = bpi.video().play_url(params).await?;

        info!("MP4 视频流信息: {:?}", data);
        assert!(data.durl.is_some());
        assert_eq!(data.quality, 64);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_playurl_4k() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let params = VideoPlayUrlParams::from_aid(Aid::new(TEST_AID)?, Cid::new(TEST_CID)?)
            .quality(120)
            .format_flags(16 | 128)
            .format_version(0)
            .fourk(true);
        let data = bpi.video().play_url(params).await?;

        info!("4K 视频流信息: {:?}", data);
        assert!(data.dash.is_some());
        assert_eq!(data.quality, 120);

        Ok(())
    }

    #[test]
    fn video_play_url_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params = VideoPlayUrlParams::from_bvid("BV1xx411c7mD".parse()?, Cid::new(62131)?)
            .quality(32)
            .format_flags(16)
            .format_version(0);

        assert_eq!(contract.name, "video.play_url");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), PLAY_URL_ENDPOINT);
        assert!(contract.request.auth.requires_wbi());
        assert_eq!(
            contract.request.query.get("cid").map(String::as_str),
            Some("62131")
        );
        assert_eq!(
            contract.request.query.get("bvid").map(String::as_str),
            Some("BV1xx411c7mD")
        );
        assert_eq!(
            contract.request.query.get("qn").map(String::as_str),
            Some("32")
        );
        assert_eq!(
            contract.request.query.get("fnval").map(String::as_str),
            Some("16")
        );
        assert_eq!(
            contract.request.query.get("fnver").map(String::as_str),
            Some("0")
        );
        assert_eq!(
            contract.request.query.get("platform").map(String::as_str),
            Some("pc")
        );
        assert_eq!(
            params.query_pairs(),
            vec![
                ("cid", "62131".to_string()),
                ("bvid", "BV1xx411c7mD".to_string()),
                ("qn", "32".to_string()),
                ("fnval", "16".to_string()),
                ("fnver", "0".to_string()),
                ("platform", "pc".to_string()),
            ]
        );
        assert_eq!(contract.cases.len(), 3);
        assert!(
            contract
                .cases
                .iter()
                .all(|case| case.response.api_code == Some(0))
        );
        assert!(
            contract
                .cases
                .iter()
                .all(|case| case.response.rust_model.as_deref() == Some("PlayUrlResponseData"))
        );
        Ok(())
    }

    #[test]
    fn video_play_url_response_fixtures_parse_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<PlayUrlResponseData>::from_slice(include_bytes!(
            "../../tests/contracts/video/playurl/play-url/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.quality, 32);
        assert_eq!(payload.format, "flv480");
        assert_eq!(
            payload
                .dash
                .as_ref()
                .and_then(|dash| dash.video.first())
                .map(|track| track.base_url.as_str()),
            Some("https://example.invalid/bilibili/playurl/redacted.m4s")
        );
        Ok(())
    }

    #[test]
    fn video_play_url_accepts_negative_resume_sentinels() -> BpiResult<()> {
        let mut response: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../tests/contracts/video/playurl/play-url/responses/success.json"
        ))?;
        response["data"]["last_play_time"] = serde_json::json!(-1000);
        response["data"]["last_play_cid"] = serde_json::json!(-1000);

        let payload =
            serde_json::from_value::<ApiEnvelope<PlayUrlResponseData>>(response)?.into_payload()?;

        assert_eq!(
            (payload.last_play_time, payload.last_play_cid),
            (-1000, -1000)
        );
        Ok(())
    }

    #[test]
    fn dash_info_accepts_nullable_and_single_stream_flac_audio() {
        let base = serde_json::json!({
            "video": [],
            "audio": [],
            "duration": 192,
            "dolby": { "type": 1, "audio": null },
        });
        let stream = serde_json::json!({
            "id": 30251,
            "baseUrl": "https://example.invalid/flac.m4s",
            "backupUrl": [],
            "bandwidth": 1000,
            "mimeType": "audio/mp4",
            "codecs": "fLaC",
        });

        let mut null_response = base.clone();
        null_response["flac"] = serde_json::json!({
            "display": true,
            "audio": null,
        });
        let null_flac = serde_json::from_value::<DashInfo>(null_response)
            .expect("flac.audio=null should not prevent DASH parsing")
            .flac
            .expect("the FLAC descriptor should remain available");
        assert_eq!(null_flac.display, Some(true));
        assert!(null_flac.audio.is_none());

        let mut stream_response = base;
        stream_response["flac"] = serde_json::json!({
            "display": true,
            "audio": stream,
        });
        let stream_flac = serde_json::from_value::<DashInfo>(stream_response)
            .expect("a single flac.audio stream should parse")
            .flac
            .expect("the FLAC descriptor should remain available");
        assert_eq!(stream_flac.display, Some(true));
        assert_eq!(stream_flac.audio.map(|audio| audio.id), Some(30251));
    }

    #[test]
    fn play_url_models_accept_null_backup_urls() {
        let dash = serde_json::from_value::<DashInfo>(serde_json::json!({
            "video": [{
                "id": 64,
                "baseUrl": "https://example.invalid/video.m4s",
                "backupUrl": null,
                "bandwidth": 1000,
                "mimeType": "video/mp4",
                "codecs": "avc1.640028"
            }],
            "audio": [],
            "dolby": null,
            "flac": null,
            "duration": 60
        }))
        .expect("dash backupUrl=null should mean no backup URLs");
        assert!(dash.video[0].backup_url.is_empty());

        let durl = serde_json::from_value::<DurlInfo>(serde_json::json!({
            "order": 1,
            "length": 60_000,
            "size": 1_000_000,
            "ahead": "",
            "vhead": "",
            "url": "https://example.invalid/video.mp4",
            "backup_url": null
        }))
        .expect("durl backup_url=null should mean no backup URLs");
        assert!(durl.backup_url.is_empty());
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/video/playurl/play-url/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn video_play_url_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<PlayUrlResponseData>>(body)?.into_payload()?;

            assert_eq!(payload.quality, 32);
            assert!(payload.dash.is_some());
        }
        Ok(())
    }
}
