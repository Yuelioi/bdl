//! 视频流URL
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/bangumi/videostream_url.md)
use serde::{Deserialize, Serialize};

/// 番剧视频流响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiVideoStreamData {
    #[serde(flatten)]
    pub base: crate::models::VideoStreamData,

    /// 响应码
    pub code: u32,
    /// fnver参数
    pub fnver: u32,
    /// 是否为视频项目
    pub video_project: bool,
    /// 数据类型
    pub r#type: String,
    /// bp参数
    pub bp: u32,
    /// VIP类型
    pub vip_type: Option<u32>,
    /// VIP状态
    pub vip_status: Option<u32>,
    /// 是否为DRM
    pub is_drm: bool,
    /// 是否重编码
    pub no_rexcode: u32,
    /// 记录信息
    pub record_info: Option<BangumiRecordInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiRecordInfo {
    pub record_icon: String,
    pub record: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bangumi::BangumiVideoStreamParams;
    use crate::ids::{Cid, EpisodeId};
    use crate::models::{Fnval, VideoQuality};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    const TEST_EP_ID: u64 = 21265; // epid
    const TEST_CID: u64 = 91549662;

    #[test]
    fn bangumi_stream_uses_current_quality_durl_instead_of_quality_groups() {
        let stream = serde_json::from_value::<BangumiVideoStreamData>(serde_json::json!({
            "quality": 32,
            "accept_quality": [32],
            "accept_format": "flv480",
            "accept_description": ["清晰 480P"],
            "format": "flv480",
            "video_codecid": 7,
            "durl": [{
                "size": 100,
                "ahead": "",
                "length": 1000,
                "vhead": "",
                "backup_url": [],
                "url": "https://example.invalid/current.flv",
                "order": 1
            }],
            "durls": [{
                "quality": 32,
                "durl": [{ "size": 100 }]
            }],
            "dash": null,
            "has_paid": false,
            "support_formats": [],
            "code": 0,
            "fnver": 0,
            "video_project": true,
            "type": "FLV",
            "bp": 0,
            "vip_type": null,
            "vip_status": null,
            "is_drm": false,
            "no_rexcode": 0,
            "record_info": null
        }))
        .expect("non-empty quality groups should not be parsed as current durl entries");

        let durl = stream.base.durl.expect("current quality durl should parse");
        assert_eq!(durl.len(), 1);
        assert_eq!(durl[0].size, 100);
    }

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/bangumi/playurl/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_bangumi_video_stream_url_simple() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .bangumi()
            .video_stream(
                BangumiVideoStreamParams::from_episode_id(EpisodeId::new(TEST_EP_ID)?)
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

        tracing::info!(
            "==========最佳格式==========\n{:#?}",
            data.base.best_format()
        );
        tracing::info!(
            "==========最佳视频==========\n{:#?}",
            data.base.best_video()
        );

        assert!(data.base.timelength.unwrap() > 0);
        assert!(!data.base.accept_format.is_empty());
        assert!(!data.base.accept_quality.is_empty());

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_bangumi_video_stream_url_by_cid() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .bangumi()
            .video_stream(
                BangumiVideoStreamParams::from_cid(Cid::new(TEST_CID)?)
                    .with_quality(VideoQuality::P480)
                    .with_fnval(Fnval::DASH),
            )
            .await?;
        tracing::info!("{:#?}", data);
        Ok(())
    }

    #[test]
    fn bangumi_video_stream_params_rejects_zero_episode_id() {
        let result = EpisodeId::new(0);

        assert!(matches!(
            result.unwrap_err(),
            BpiError::InvalidParameter { field: "ep_id", .. }
        ));
    }

    #[test]
    fn bangumi_playurl_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params = BangumiVideoStreamParams::from_episode_id(EpisodeId::new(TEST_EP_ID)?)
            .with_quality(VideoQuality::P480)
            .with_fnval(Fnval::DASH);

        assert_eq!(contract.name, "bangumi.playurl");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/pgc/player/web/playurl"
        );
        assert_eq!(
            contract.request.query.get("fnver").map(String::as_str),
            Some("0")
        );
        assert_eq!(
            contract.request.query.get("ep_id").map(String::as_str),
            Some("21265")
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
                ("fnver", "0".to_string()),
                ("ep_id", "21265".to_string()),
                ("qn", "32".to_string()),
                ("fnval", "16".to_string()),
            ]
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("BangumiVideoStreamData")
        );
        assert_eq!(
            contract.cases[0].response.fixture_kind.as_deref(),
            Some("sanitized_probe_body")
        );
        Ok(())
    }

    #[test]
    fn bangumi_playurl_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/bangumi/playurl/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!("../../tests/contracts/bangumi/playurl/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/bangumi/playurl/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload =
                ApiEnvelope::<BangumiVideoStreamData>::from_slice(bytes)?.into_payload()?;

            assert_eq!(payload.base.quality, VideoQuality::P480.as_u32());
            assert!(payload.base.supports_dash());
            assert_eq!(
                payload
                    .base
                    .best_video()
                    .map(|track| track.base_url.as_str()),
                Some("https://example.invalid/bilibili/playurl/redacted.m4s")
            );
        }
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/bangumi/playurl/playurl/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn bangumi_playurl_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload = serde_json::from_value::<ApiEnvelope<BangumiVideoStreamData>>(body)?
                .into_payload()?;

            assert_eq!(payload.base.quality, VideoQuality::P480.as_u32());
            assert!(payload.base.supports_dash());
        }
        Ok(())
    }
}
