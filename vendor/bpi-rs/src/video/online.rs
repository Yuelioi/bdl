//! 视频在线人数相关接口
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/video)
use serde::{Deserialize, Serialize};

pub(crate) const ONLINE_TOTAL_ENDPOINT: &str = "https://api.bilibili.com/x/player/online/total";

// --- 响应数据结构体 ---

/// 在线人数数据控制
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OnlineTotalShowSwitch {
    /// 展示所有终端总计人数
    pub total: bool,
    /// 展示web端实时在线人数
    pub count: bool,
}

/// 视频在线人数响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OnlineTotalResponseData {
    /// 所有终端总计人数
    pub total: String,
    /// web端实时在线人数
    pub count: String,
    /// 数据显示控制
    pub show_switch: OnlineTotalShowSwitch,
}

// --- 测试模块 ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{Aid, Cid};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::video::params::VideoOnlineTotalParams;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use tracing::info;

    // 假设这是一个已知的视频
    const TEST_AID: u64 = 759949922;
    const TEST_CID: u64 = 392402545;
    const TEST_BVID: &str = "BV1y64y1q757";

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_online_total_by_aid() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let params = VideoOnlineTotalParams::from_aid(Aid::new(TEST_AID)?, Cid::new(TEST_CID)?);
        let data = bpi.video().online_total(params).await?;

        info!("视频在线人数: {:?}", data);
        assert!(!data.count.is_empty());
        assert!(!data.total.is_empty());

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_online_total_by_bvid() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let params = VideoOnlineTotalParams::from_bvid(TEST_BVID.parse()?, Cid::new(TEST_CID)?);
        let data = bpi.video().online_total(params).await?;

        info!("视频在线人数: {:?}", data);

        assert!(!data.count.is_empty());
        assert!(!data.total.is_empty());

        Ok(())
    }

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/video/player-read/online-total/contract.json"
        ))
    }

    #[test]
    fn video_online_total_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params = VideoOnlineTotalParams::from_bvid("BV1xx411c7mD".parse()?, Cid::new(62131)?);

        assert_eq!(contract.name, "video.online_total");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), ONLINE_TOTAL_ENDPOINT);
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
                ("cid", "62131".to_string()),
                ("bvid", "BV1xx411c7mD".to_string())
            ]
        );
        assert_eq!(contract.cases.len(), 3);
        Ok(())
    }

    #[test]
    fn video_online_total_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<OnlineTotalResponseData>::from_slice(include_bytes!(
            "../../tests/contracts/video/player-read/online-total/responses/success.json"
        ))?
        .into_payload()?;

        assert!(!payload.count.is_empty());
        assert!(!payload.total.is_empty());
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/video/player-read/online-total/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn video_online_total_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload = serde_json::from_value::<ApiEnvelope<OnlineTotalResponseData>>(body)?
                .into_payload()?;

            assert!(!payload.total.is_empty());
        }
        Ok(())
    }
}
