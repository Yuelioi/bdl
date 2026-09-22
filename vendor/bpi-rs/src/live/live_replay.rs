use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct LiveInfo {
    /// 直播标题
    pub title: String,
    /// 直播封面
    pub cover: String,
    /// 直播时间
    pub live_time: i64,
    /// 直播类型
    pub live_type: i32,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct VideoInfo {
    /// 回放状态
    pub replay_status: i32,
    /// 直播回放合成结束时间
    pub estimated_time: String,
    /// 直播时长（秒）
    pub duration: i32,
    /// 下载链接片段
    pub download_url: Option<String>,
    /// 快速检查警告代码
    pub alert_code: Option<i32>,
    /// 快速检查警告信息
    pub alert_message: Option<String>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct AlarmInfo {
    /// 回放合成警报代码
    pub code: i32,
    /// 回放合成错误信息
    pub message: String,
    /// 当前时间戳
    pub cur_time: i64,
    /// 是否禁止发布
    pub is_ban_publish: bool,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ReplayInfo {
    /// 直播回放id
    pub replay_id: i64,
    /// 直播信息
    pub live_info: LiveInfo,
    /// 回放视频信息
    pub video_info: VideoInfo,
    /// 警报信息
    pub alarm_info: AlarmInfo,
    /// 直播间id
    pub room_id: i64,
    /// 标记直播场次的key
    pub live_key: String,
    /// 直播开始秒时间戳
    pub start_time: i64,
    /// 直播结束秒时间戳
    pub end_time: i64,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Pagination {
    /// 请求的页码
    pub page: i32,
    /// 内容数量
    pub page_size: i32,
    /// 总计内容数量
    pub total: Option<i32>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ReplayListData {
    /// 回放信息列表
    pub replay_info: Option<Vec<ReplayInfo>>,
    /// 分页信息
    pub pagination: Pagination,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/live/account-private-read/replay-list/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_live_replay_list() {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi.live().replay_list(Some(1), Some(2)).await.unwrap();
        tracing::info!("{:?}", data);
    }

    #[test]
    fn live_replay_list_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params = [
            ("page", 1_i32.to_string()),
            ("page_size", 2_i32.to_string()),
        ];

        assert_eq!(contract.name, "live.replay_list");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.live.bilibili.com/xlive/app-blink/v1/anchorVideo/AnchorGetReplayList"
        );
        assert_eq!(
            contract.request.query.get("page").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            contract.request.query.get("page_size").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            params,
            [("page", "1".to_string()), ("page_size", "2".to_string())]
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.error.as_deref(),
            Some("requires_login")
        );
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("ReplayListData")
        );
        Ok(())
    }

    #[test]
    fn live_replay_list_response_fixtures_parse_declared_model() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/live/account-private-read/replay-list/responses/anonymous.requires_login.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());

        let payload = ApiEnvelope::<ReplayListData>::from_slice(include_bytes!(
            "../../tests/contracts/live/account-private-read/replay-list/responses/authenticated.empty.success.json"
        ))?
        .into_payload()?;
        assert!(payload.replay_info.is_none());
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!(
            "target/bpi-probe-runs/live/account-private-read/replay-list/{profile}.response.json"
        );
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn live_replay_list_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let envelope = serde_json::from_value::<ApiEnvelope<ReplayListData>>(body)?;

            if profile == "anonymous" {
                assert!(envelope.ensure_success().unwrap_err().requires_login());
            } else {
                let payload = envelope.into_payload()?;
                assert_eq!(payload.pagination.page, 1);
            }
        }
        Ok(())
    }
}
