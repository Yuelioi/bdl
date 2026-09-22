use serde::{Deserialize, Serialize};

// ================= 数据结构 =================

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct RedPocketAward {
    /// 礼物id
    pub gift_id: i64,
    /// 数量
    pub num: i32,
    /// 礼物名称
    pub gift_name: String,
    /// 礼物图片
    pub gift_pic: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct PopularityRedPocket {
    /// 红包id
    pub lot_id: i64,
    /// 红包发送者uid
    pub sender_uid: i64,
    /// 红包发送者昵称
    pub sender_name: String,
    /// 红包发送者头像
    pub sender_face: String,
    /// 参与条件
    pub join_requirement: i32,
    /// 参与红包时自动发送的弹幕内容
    pub danmu: String,
    /// 红包内容
    pub awards: Vec<RedPocketAward>,
    /// 开始时间
    pub start_time: i64,
    /// 结束时间
    pub end_time: i64,
    /// 持续时间
    pub last_time: i64,
    /// 移除时间
    pub remove_time: i64,
    /// 替换时间
    pub replace_time: i64,
    /// 当前时间
    pub current_time: i64,
    /// 红包状态
    pub lot_status: i32,
    /// 红包界面
    pub h5_url: String,
    /// 用户是否已参与
    pub user_status: i32,
    /// 红包配置id
    pub lot_config_id: i64,
    /// 红包总计价格
    pub total_price: i64,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ActivityBoxInfo {
    /// 直播抽奖 endpoint 返回的未文档化活动盒子字段。
    #[serde(default)]
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct LotteryInfoData {
    /// 人气红包信息
    pub popularity_red_pocket: Option<Vec<PopularityRedPocket>>,
    /// 活动盒子信息
    pub activity_box_info: Option<ActivityBoxInfo>,
    /// 未文档化的抽奖分区，例如 anchor、guard、gift、storm 或红包状态。
    #[serde(default)]
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/live/room-interaction-read/lottery-info/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_live_lottery_info() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        bpi.live().lottery_info(23174842).await?;

        // 注意：直播间可能没有红包，所以不做额外断言
        Ok(())
    }

    #[test]
    fn live_lottery_info_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "live.lottery_info");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.live.bilibili.com/xlive/lottery-interface/v1/lottery/getLotteryInfoWeb"
        );
        assert_eq!(
            contract.request.query.get("roomid").map(String::as_str),
            Some("23174842")
        );
        assert!(contract.request.auth.requires_wbi());
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.error.as_deref(),
            Some("wbi_risk_control")
        );
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("LotteryInfoData")
        );
        Ok(())
    }

    #[test]
    fn live_lottery_info_response_fixtures_parse_declared_model() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/live/room-interaction-read/lottery-info/responses/anonymous.error.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert_eq!(err.code(), Some(-352));

        let payload = ApiEnvelope::<LotteryInfoData>::from_slice(include_bytes!(
            "../../tests/contracts/live/room-interaction-read/lottery-info/responses/authenticated.success.json"
        ))?
        .into_payload()?;
        assert!(payload.popularity_red_pocket.is_none());
        assert!(payload.extra.contains_key("activity_box"));
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!(
            "target/bpi-probe-runs/live/room-interaction-read/lottery-info/{profile}.response.json"
        );
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn live_lottery_info_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let envelope = serde_json::from_value::<ApiEnvelope<LotteryInfoData>>(body)?;

            if profile == "anonymous" {
                assert_eq!(envelope.ensure_success().unwrap_err().code(), Some(-352));
            } else {
                let payload = envelope.into_payload()?;
                assert!(payload.extra.contains_key("activity_box"));
            }
        }
        Ok(())
    }
}
