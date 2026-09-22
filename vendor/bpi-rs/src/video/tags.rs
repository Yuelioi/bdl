//! 视频 TAG 相关接口
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/video)
use serde::{Deserialize, Serialize};

pub(crate) const TAGS_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/view/detail/tag";

// --- 响应数据结构体 ---

/// 视频 TAG 信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VideoTag {
    /// tag ID, 当 tag_type 不为 bgm 时有效
    pub tag_id: Option<u64>,
    /// TAG 名称
    pub tag_name: String,
    /// 背景音乐 ID, 当 tag_type 为 bgm 时有效
    pub music_id: Option<String>,
    /// TAG 类型, old_channel: 普通标签, topic: 话题, bgm: 背景音乐
    pub tag_type: String,
    /// 跳转 url, 当 tag_type 为 topic 或 bgm 时有效
    pub jump_url: Option<String>,
}

// --- 测试模块 ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{Aid, Cid};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::video::params::VideoTagsParams;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use tracing::info;

    const TEST_AID: u64 = 89772773;
    const TEST_BVID: &str = "BV1M741177Kg";
    const TEST_CID: u64 = 153322313;

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_tags_by_aid() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let params = VideoTagsParams::from_aid(Aid::new(TEST_AID)?).cid(Cid::new(TEST_CID)?);
        let data = bpi.video().tags(params).await?;

        info!("视频 TAG 列表: {:?}", data);

        assert!(!data.is_empty());

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_tags_by_bvid() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .video()
            .tags(VideoTagsParams::from_bvid(TEST_BVID.parse()?))
            .await?;

        info!("视频 TAG 列表: {:?}", data);

        assert!(!data.is_empty());

        Ok(())
    }

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/video/player-read/tags/contract.json"
        ))
    }

    #[test]
    fn video_tags_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params = VideoTagsParams::from_bvid("BV1xx411c7mD".parse()?).cid(Cid::new(62131)?);

        assert_eq!(contract.name, "video.tags");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), TAGS_ENDPOINT);
        assert_eq!(
            contract.request.query.get("bvid").map(String::as_str),
            Some("BV1xx411c7mD")
        );
        assert_eq!(
            contract.request.query.get("cid").map(String::as_str),
            Some("62131")
        );
        assert_eq!(
            params.query_pairs(),
            vec![
                ("bvid", "BV1xx411c7mD".to_string()),
                ("cid", "62131".to_string())
            ]
        );
        assert_eq!(contract.cases.len(), 3);
        Ok(())
    }

    #[test]
    fn video_tags_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<Vec<VideoTag>>::from_slice(include_bytes!(
            "../../tests/contracts/video/player-read/tags/responses/success.json"
        ))?
        .into_payload()?;

        assert!(payload.is_empty());
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/video/player-read/tags/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn video_tags_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let _payload =
                serde_json::from_value::<ApiEnvelope<Vec<VideoTag>>>(body)?.into_payload()?;
        }
        Ok(())
    }
}
