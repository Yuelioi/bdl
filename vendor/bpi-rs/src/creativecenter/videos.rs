//! 创作中心视频管理 API
//!
//! [参考文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/videos.md)

use serde::{Deserialize, Serialize};

/// 稿件统计信息
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ArchiveStat {
    pub aid: i64,
    pub view: i64,
    pub danmaku: i64,
    pub reply: i64,
    pub favorite: i64,
    pub coin: i64,
    pub share: i64,
    pub now_rank: i64,
    pub his_rank: i64,
    pub like: i64,
    pub dislike: i64,
    pub vt: i64,
    pub vv: i64,
}

/// 稿件基本信息
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Archive {
    pub aid: i64,
    pub bvid: String,
    pub title: String,
    pub cover: String,
    pub duration: i64,
    pub desc: String,
}

/// 稿件列表项
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ArcAudit {
    #[serde(rename = "Archive")]
    pub archive: Option<Archive>,
    #[serde(rename = "Videos")]
    pub videos: Option<serde_json::Value>,
    pub stat: ArchiveStat,
    pub state_panel: i64,
    pub parent_tname: Option<String>,
    pub typename: Option<String>,
    pub open_appeal: i64,
    pub activity: Option<serde_json::Value>,
    pub season_add_state: i64,
}

/// 分页信息
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct PageInfo {
    pub pn: i64,
    pub ps: i64,
    pub count: i64,
}

/// 稿件列表数据
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct SpArchivesData {
    pub arc_audits: Vec<ArcAudit>,
    pub page: PageInfo,
    pub play_type: i64,
}

/// 分P 视频信息
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct VideoPart {
    /// 分P cid
    pub cid: i64,
    /// 分P 序号
    pub index: i64,
    /// 分P 标题
    pub title: String,
    /// 视频时长（秒）
    pub duration: i64,
}

/// 稿件信息
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ArchiveInfo {
    /// av号
    pub aid: i64,
    /// bvid
    pub bvid: String,
    /// 标题
    pub title: String,
}

/// 视频基础信息数据
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ArchiveVideosData {
    /// 稿件信息
    pub archive: ArchiveInfo,
    /// 分P 视频列表
    pub videos: Vec<VideoPart>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::creativecenter::{UpArchiveVideosParams, UpArchivesListParams};
    use crate::ids::Aid;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError};
    use std::collections::BTreeMap;
    use tracing::info;

    const TEST_AID: u64 = 113602455409683;

    fn contract(name: &str) -> Result<EndpointContract, BpiError> {
        let bytes = match name {
            "archives-list" => include_bytes!(
                "../../tests/contracts/creativecenter/videos/archives-list/contract.json"
            )
            .as_slice(),
            "archive-videos" => include_bytes!(
                "../../tests/contracts/creativecenter/videos/archive-videos/contract.json"
            )
            .as_slice(),
            _ => unreachable!("unknown creativecenter videos contract"),
        };
        EndpointContract::from_slice(bytes)
    }

    fn query_map<I>(params: I) -> BTreeMap<String, String>
    where
        I: IntoIterator<Item = (&'static str, String)>,
    {
        params
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect()
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_archives_list() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let params = UpArchivesListParams::new(1)?.with_page_size(10)?;
        let data = bpi.creativecenter().archives_list(params).await?;
        info!("稿件列表: {:?}", data);
        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_archive_videos() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let params = UpArchiveVideosParams::new(Aid::new(TEST_AID)?);
        let data = bpi.creativecenter().archive_videos(params).await?;
        info!("视频基础信息: {:?}", data);
        Ok(())
    }

    #[test]
    fn creativecenter_videos_contracts_match_endpoint_requests() -> Result<(), BpiError> {
        let archives_list = contract("archives-list")?;
        let archives_list_params = UpArchivesListParams::new(1)?.with_page_size(10)?;
        assert_eq!(archives_list.name, "creativecenter.videos.archives_list");
        assert_eq!(archives_list.request.method, HttpMethod::Get);
        assert_eq!(
            archives_list.request.url.as_str(),
            "https://member.bilibili.com/x2/creative/web/archives/sp"
        );
        assert_eq!(
            query_map(archives_list_params.query_pairs()),
            archives_list.request.query
        );

        let archive_videos = contract("archive-videos")?;
        let archive_videos_params = UpArchiveVideosParams::new(Aid::new(TEST_AID)?);
        assert_eq!(archive_videos.name, "creativecenter.videos.archive_videos");
        assert_eq!(
            archive_videos.request.url.as_str(),
            "https://member.bilibili.com/x/web/archive/videos"
        );
        assert_eq!(
            query_map(archive_videos_params.query_pairs()),
            archive_videos.request.query
        );
        Ok(())
    }

    #[test]
    fn creativecenter_videos_response_fixtures_parse_declared_models() -> Result<(), BpiError> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/creativecenter/videos/archives-list/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/creativecenter/videos/archives-list/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<SpArchivesData>::from_slice(bytes)?.into_payload()?;
            assert_eq!(payload.arc_audits.len(), 1);
        }

        let payload = ApiEnvelope::<ArchiveVideosData>::from_slice(include_bytes!(
            "../../tests/contracts/creativecenter/videos/archive-videos/responses/vip.success.json"
        ))?
        .into_payload()?;
        assert_eq!(payload.videos.len(), 1);
        Ok(())
    }

    #[test]
    fn creativecenter_videos_error_fixtures_preserve_observed_api_errors() -> Result<(), BpiError> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/creativecenter/videos/archives-list/responses/anonymous.requires_login.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/creativecenter/videos/archive-videos/responses/anonymous.requires_login.json"
            )
            .as_slice(),
        ] {
            let err = ApiEnvelope::<serde_json::Value>::from_slice(bytes)
                .and_then(ApiEnvelope::ensure_success)
                .unwrap_err();
            assert!(err.requires_login());
        }

        let permission = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/creativecenter/videos/archive-videos/responses/normal.permission_denied.json"
        ))
        .and_then(ApiEnvelope::ensure_success)
        .unwrap_err();
        assert_eq!(permission.code(), Some(-403));
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path = format!(
            "target/bpi-probe-runs/creativecenter/videos-read/{endpoint}/{profile}.response.json"
        );
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn creativecenter_videos_models_match_local_probe_outputs_when_available()
    -> Result<(), BpiError> {
        for profile in ["normal", "vip"] {
            let Some(body) = local_probe_body("archives-list", profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<SpArchivesData>>(body)?.into_payload()?;
            assert!(!payload.arc_audits.is_empty());
        }

        if let Some(body) = local_probe_body("archive-videos", "vip") {
            let payload =
                serde_json::from_value::<ApiEnvelope<ArchiveVideosData>>(body)?.into_payload()?;
            assert_eq!(payload.archive.aid, TEST_AID as i64);
        }
        Ok(())
    }
}
