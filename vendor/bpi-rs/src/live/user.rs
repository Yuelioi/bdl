use serde::{Deserialize, Serialize};

// ================= 数据结构 =================

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct PageInfo {
    /// 页码总长度
    pub total_page: i32,
    /// 当前返回的页码
    pub cur_page: i32,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct FansMedalItem {
    /// 可否删除
    #[serde(rename = "can_deleted")]
    pub can_delete: bool,
    /// 日经验上限（原力值）
    pub day_limit: i32,
    /// 大航海等级
    pub guard_level: i32,
    /// 加成状态
    pub guard_medal_title: String,
    /// 当前已得亲密度
    pub intimacy: i32,
    /// 是否点亮
    pub is_lighted: i32,
    /// 勋章等级
    pub level: i32,
    /// 勋章名
    pub medal_name: String,
    /// 勋章边框颜色信息
    pub medal_color_border: i32,
    /// 勋章起始颜色
    pub medal_color_start: i32,
    /// 勋章结束颜色
    pub medal_color_end: i32,
    /// 粉丝勋章id
    pub medal_id: i64,
    /// 升级所需经验
    pub next_intimacy: i32,
    /// 本日亲密度
    pub today_feed: i32,
    /// 直播间房间号
    pub roomid: i64,
    /// 状态
    pub status: i32,
    /// up主mid
    pub target_id: i64,
    /// up主用户名
    pub target_name: String,
    /// up主用户名
    pub uname: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct MyMedalsData {
    /// 勋章数量
    pub count: i32,
    /// 粉丝勋章信息本体
    pub items: Vec<FansMedalItem>,
    /// 页码信息
    pub page_info: PageInfo,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/live/account-private-read/my-medals/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_my_medals() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi.live().my_medals(1, 10).await?;

        tracing::info!("{:?}", data);
        Ok(())
    }

    #[test]
    fn live_my_medals_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params = [
            ("page", 1_i32.to_string()),
            ("page_size", 10_i32.to_string()),
        ];

        assert_eq!(contract.name, "live.my_medals");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.live.bilibili.com/xlive/app-ucenter/v1/user/GetMyMedals"
        );
        assert_eq!(
            contract.request.query.get("page").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            contract.request.query.get("page_size").map(String::as_str),
            Some("10")
        );
        assert_eq!(
            params,
            [("page", "1".to_string()), ("page_size", "10".to_string())]
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.error.as_deref(),
            Some("requires_login")
        );
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("MyMedalsData")
        );
        Ok(())
    }

    #[test]
    fn live_my_medals_response_fixtures_parse_declared_model() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/live/account-private-read/my-medals/responses/anonymous.requires_login.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());

        let normal = ApiEnvelope::<MyMedalsData>::from_slice(include_bytes!(
            "../../tests/contracts/live/account-private-read/my-medals/responses/normal.empty.success.json"
        ))?
        .into_payload()?;
        assert!(normal.items.is_empty());

        let vip = ApiEnvelope::<MyMedalsData>::from_slice(include_bytes!(
            "../../tests/contracts/live/account-private-read/my-medals/responses/vip.sample.success.json"
        ))?
        .into_payload()?;
        assert_eq!(vip.items.len(), 1);
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!(
            "target/bpi-probe-runs/live/account-private-read/my-medals/{profile}.response.json"
        );
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn live_my_medals_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let envelope = serde_json::from_value::<ApiEnvelope<MyMedalsData>>(body)?;

            if profile == "anonymous" {
                assert!(envelope.ensure_success().unwrap_err().requires_login());
            } else {
                let payload = envelope.into_payload()?;
                assert!(payload.count >= 0);
            }
        }
        Ok(())
    }
}
