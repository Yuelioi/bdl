//! 视频 AI 总结相关接口
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/video)
use serde::{Deserialize, Serialize};

pub(crate) const AI_SUMMARY_ENDPOINT: &str =
    "https://api.bilibili.com/x/web-interface/view/conclusion/get";

// --- 响应数据结构体 ---

/// AI 总结提纲分段要点
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AiSummaryPartOutline {
    /// 要点起始时间，单位为秒
    pub timestamp: u64,
    /// 小结内容
    pub content: String,
}

/// AI 总结提纲
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AiSummaryOutline {
    /// 分段标题
    pub title: String,
    /// 分段要点
    pub part_outline: Vec<AiSummaryPartOutline>,
    /// 分段起始时间，单位为秒
    pub timestamp: u64,
}

/// AI 总结摘要内容
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AiSummaryModelResult {
    /// 数据类型, 0: 没有摘要, 1: 仅有摘要总结, 2: 有摘要及提纲
    pub result_type: u8,
    /// 视频摘要
    pub summary: String,
    /// 分段提纲
    pub outline: Option<Vec<AiSummaryOutline>>,
}

/// 视频 AI 总结响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AiSummaryResponseData {
    /// 返回值, -1: 不支持 AI 摘要, 0: 有摘要, 1: 无摘要
    pub code: i8,
    /// 摘要内容
    pub model_result: Option<AiSummaryModelResult>,
    /// 摘要 id
    pub stid: Option<String>,
    pub status: Option<u8>,
    /// 点赞数
    pub like_num: u64,
    /// 点踩数
    pub dislike_num: u64,
}

// --- 测试模块 ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{Aid, Cid};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::video::params::VideoAiSummaryParams;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use tracing::info;

    const TEST_AID: u64 = 10001;

    const TEST_CID: u64 = 16546;
    const TEST_UP_MID: u64 = 34893;

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_ai_summary_by_aid() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let params =
            VideoAiSummaryParams::from_aid(Aid::new(TEST_AID)?, Cid::new(TEST_CID)?, TEST_UP_MID)?;
        let data = bpi.video().ai_summary(params).await?;

        info!("视频 AI 总结: {:?}", data);

        if data.code == 0 {
            assert!(data.model_result.is_some());
        }

        Ok(())
    }

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/video/player-read/ai-summary/contract.json"
        ))
    }

    #[test]
    fn video_ai_summary_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params =
            VideoAiSummaryParams::from_bvid("BV1xx411c7mD".parse()?, Cid::new(62131)?, 928123)?;

        assert_eq!(contract.name, "video.ai_summary");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), AI_SUMMARY_ENDPOINT);
        assert!(contract.request.auth.requires_wbi());
        assert_eq!(
            contract.request.query.get("bvid").map(String::as_str),
            Some("BV1xx411c7mD")
        );
        assert_eq!(
            contract.request.query.get("up_mid").map(String::as_str),
            Some("928123")
        );
        assert_eq!(
            params.query_pairs(),
            vec![
                ("cid", "62131".to_string()),
                ("up_mid", "928123".to_string()),
                ("bvid", "BV1xx411c7mD".to_string())
            ]
        );
        assert_eq!(
            contract.cases[0].response.error.as_deref(),
            Some("requires_login")
        );
        Ok(())
    }

    #[test]
    fn video_ai_summary_response_fixtures_parse_declared_model() -> BpiResult<()> {
        let anonymous = ApiEnvelope::<AiSummaryResponseData>::from_slice(include_bytes!(
            "../../tests/contracts/video/player-read/ai-summary/responses/anonymous.error.json"
        ))?;
        assert_eq!(anonymous.code, -101);

        let payload = ApiEnvelope::<AiSummaryResponseData>::from_slice(include_bytes!(
            "../../tests/contracts/video/player-read/ai-summary/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.code, -1);
        assert!(payload.model_result.is_some());
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/video/player-read/ai-summary/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn video_ai_summary_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let envelope = serde_json::from_value::<ApiEnvelope<AiSummaryResponseData>>(body)?;
            if profile == "anonymous" {
                assert_eq!(envelope.code, -101);
            } else {
                let payload = envelope.into_payload()?;
                assert!(payload.model_result.is_some());
            }
        }
        Ok(())
    }
}
