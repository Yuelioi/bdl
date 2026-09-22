//! 课程视频流 URL API
//!
//! [参考文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/cheese/videostream_url.md)

use std::collections::HashMap;

use crate::models::{DashStreams, SupportFormat};
use serde::{Deserialize, Serialize};

/// 课程视频流数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseVideoStreamData {
    #[serde(flatten)]
    pub base: crate::models::VideoStreamData,

    /// 定位参数
    pub seek_param: String,
    /// 是否为视频项目
    pub video_project: bool,
    /// 数据类型
    #[serde(rename = "type")]
    pub data_type: String,
    /// 结果状态
    pub result: String,
    /// 定位类型
    pub seek_type: String,
    /// 来源
    pub from: String,
    /// 是否重编码
    pub no_rexcode: i32,
    /// 响应消息
    pub message: String,
    /// 分片视频信息
    pub fragment_videos: Option<Vec<FragmentVideo>>,
    /// 状态码
    pub status: i32,
}

/// 分片视频
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentVideo {
    pub fragment_info: FragmentInfo,
    pub playable_status: bool,
    pub video_info: VideoInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FragmentInfo {
    pub fragment_type: String,
    pub index: i64,
    pub aid: i64,
    pub fragment_position: String,
    pub cid: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoInfo {
    pub no_rexcode: i64,
    pub fnval: i64,
    pub video_project: bool,
    pub expire_time: i64,
    pub backup_url: Vec<Option<serde_json::Value>>,
    pub fnver: i64,
    pub support_formats: Vec<String>,
    pub support_description: Vec<String>,
    #[serde(rename = "type")]
    pub video_info_type: String,
    pub url: String,
    pub quality: i64,
    pub timelength: i64,
    pub volume: CourseVolume,
    pub accept_formats: Vec<SupportFormat>,
    pub support_quality: Vec<i64>,
    pub file_info: HashMap<String, FileInfo>,
    pub dash: DashStreams,
    pub video_codecid: i64,
    pub cid: i64,
}

/// 音量信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseVolume {
    pub measured_i: f64,
    pub target_i: f64,
    pub target_offset: f64,
    pub measured_lra: f64,
    pub target_tp: f64,
    pub measured_tp: f64,
    pub measured_threshold: f64,
    pub multi_scene_args: MultiSceneArgs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSceneArgs {
    pub normal_target_i: String,
    pub undersized_target_i: String,
    pub high_dynamic_target_i: String,
}

/// 文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub infos: Vec<FileInfoEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfoEntry {
    pub ahead: String,
    pub vhead: String,
    pub filesize: i64,
    pub order: i64,
    pub timelength: i64,
}

// ==========================
// 测试
// ==========================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cheese::CheeseVideoStreamParams;
    use crate::ids::{Aid, Cid, EpisodeId};
    use crate::models::{Fnval, VideoQuality};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    const TEST_AVID: u64 = 997984154;
    const TEST_EP_ID: u64 = 163956;
    const TEST_CID: u64 = 1183682680;

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/cheese/playurl/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_cheese_playurl() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");

        let data = bpi
            .cheese()
            .video_stream(
                CheeseVideoStreamParams::new(
                    Aid::new(TEST_AVID)?,
                    EpisodeId::new(TEST_EP_ID)?,
                    Cid::new(TEST_CID)?,
                )
                .with_quality(VideoQuality::P8K)
                .with_fnval(
                    Fnval::DASH
                        | Fnval::FOURK
                        | Fnval::EIGHTK
                        | Fnval::HDR
                        | Fnval::DOLBY_AUDIO
                        | Fnval::DOLBY_VISION
                        | Fnval::AV1,
                ),
            )
            .await?;

        tracing::info!("{:#?}", data);

        Ok(())
    }

    #[test]
    fn cheese_video_stream_params_serializes_playback_flags() -> Result<(), BpiError> {
        let params = CheeseVideoStreamParams::new(
            Aid::new(TEST_AVID)?,
            EpisodeId::new(TEST_EP_ID)?,
            Cid::new(TEST_CID)?,
        )
        .with_quality(VideoQuality::P8K)
        .with_fnval(Fnval::DASH | Fnval::FOURK);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("avid", TEST_AVID.to_string()),
                ("ep_id", TEST_EP_ID.to_string()),
                ("cid", TEST_CID.to_string()),
                ("fnver", "0".to_string()),
                ("fourk", "1".to_string()),
                ("qn", VideoQuality::P8K.as_u32().to_string()),
                ("fnval", (Fnval::DASH | Fnval::FOURK).bits().to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn cheese_playurl_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params = CheeseVideoStreamParams::new(
            Aid::new(TEST_AVID)?,
            EpisodeId::new(TEST_EP_ID)?,
            Cid::new(TEST_CID)?,
        )
        .with_quality(VideoQuality::P480)
        .with_fnval(Fnval::DASH);

        assert_eq!(contract.name, "cheese.playurl");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/pugv/player/web/playurl"
        );
        assert_eq!(
            contract.request.query.get("avid").map(String::as_str),
            Some("997984154")
        );
        assert_eq!(
            contract.request.query.get("ep_id").map(String::as_str),
            Some("163956")
        );
        assert_eq!(
            contract.request.query.get("cid").map(String::as_str),
            Some("1183682680")
        );
        assert_eq!(
            contract.request.query.get("fnver").map(String::as_str),
            Some("0")
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
            params.query_pairs(),
            vec![
                ("avid", TEST_AVID.to_string()),
                ("ep_id", TEST_EP_ID.to_string()),
                ("cid", TEST_CID.to_string()),
                ("fnver", "0".to_string()),
                ("qn", "32".to_string()),
                ("fnval", "16".to_string()),
            ]
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("CourseVideoStreamData")
        );
        assert_eq!(
            contract.cases[0].response.fixture_kind.as_deref(),
            Some("sanitized_probe_body")
        );
        Ok(())
    }

    #[test]
    fn cheese_playurl_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!("../../tests/contracts/cheese/playurl/responses/anonymous.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/cheese/playurl/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/cheese/playurl/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload =
                ApiEnvelope::<CourseVideoStreamData>::from_slice(bytes)?.into_payload()?;

            assert_eq!(payload.base.quality, VideoQuality::P480.as_u32());
            assert!(!payload.base.has_paid);
            assert!(payload.base.supports_dash());
            assert_eq!(
                payload
                    .base
                    .best_video()
                    .map(|track| track.base_url.as_str()),
                Some("https://example.invalid/bilibili/playurl/redacted.m4s")
            );
            assert!(
                payload
                    .fragment_videos
                    .as_ref()
                    .is_some_and(|fragments| !fragments.is_empty())
            );
        }
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/cheese/read/playurl/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn cheese_playurl_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload = serde_json::from_value::<ApiEnvelope<CourseVideoStreamData>>(body)?
                .into_payload()?;

            assert_eq!(payload.base.quality, VideoQuality::P480.as_u32());
            assert!(!payload.base.has_paid);
            assert!(payload.base.supports_dash());
        }
        Ok(())
    }
}
