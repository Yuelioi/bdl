use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct FollowUpLiveItem {
    /// 房间号
    pub roomid: i64,
    /// 主播uid
    pub uid: i64,
    /// 主播名
    pub uname: String,
    /// 直播标题
    pub title: String,
    /// 主播头像
    pub face: String,
    /// 是否正在直播
    pub live_status: i32,
    /// 主播上一次直播结束的时间戳
    pub record_live_time: i64,
    /// 频道的名称
    pub area_name_v2: String,
    /// 房间公告
    pub room_news: String,
    /// 作用尚不明确，当主播正在直播时，为在线人数(可能)
    pub text_small: String,
    /// 房间封面图片的URL
    pub room_cover: String,
    /// 父分区id
    pub parent_area_id: i32,
    /// 分区id
    pub area_id: i32,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct FollowUpLiveData {
    /// 标题
    pub title: String,

    /// 每页的数据数量
    #[serde(rename = "pageSize")]
    pub page_size: i32,

    /// 分页数量
    #[serde(rename = "totalPage")]
    pub total_page: i32,

    /// UP直播情况列表
    pub list: Vec<FollowUpLiveItem>,

    /// 曾直播过的UP数量
    pub count: i32,

    /// 未直播过的UP数量
    pub never_lived_count: i32,

    /// 正在直播的UP数量
    pub live_count: i32,

    /// 作用尚不明确
    pub never_lived_faces: Vec<String>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct LiveRoom {
    /// 直播间标题
    pub title: String,
    /// 直播间真实id
    pub room_id: i64,
    /// 目标用户mid
    pub uid: i64,
    /// 观看人数
    pub online: i32,
    /// 已经直播的时长（单位为秒）
    pub live_time: i64,
    /// 开播状态
    pub live_status: i32,
    /// 直播间短id
    pub short_id: i32,
    /// 分区id
    pub area: i32,
    /// 分区名称
    pub area_name: String,
    /// 二级分区id
    pub area_v2_id: i32,
    /// 二级分区名
    pub area_v2_name: String,
    /// 二级父分区名
    pub area_v2_parent_name: String,
    /// 二级父分区id
    pub area_v2_parent_id: i32,
    /// 用户名
    pub uname: String,
    /// 用户头像图片链接
    pub face: String,
    /// 标签名
    pub tag_name: String,
    /// 标签列表
    pub tags: String,
    /// 直播间封面图片链接
    pub cover_from_user: String,
    /// 关键帧图片链接
    pub keyframe: String,
    /// 未知
    pub lock_till: String,
    /// 未知
    pub hidden_till: String,
    /// 广播类型
    pub broadcast_type: i32,
    /// 直播间是否加密
    pub is_encrypt: bool,
    /// 直播间链接
    pub link: String,
    /// 用户昵称
    pub nickname: String,
    /// 直播间名称
    pub roomname: String,
    /// 直播间真实id
    pub roomid: i64,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct LiveWebListData {
    /// 正在直播的房间列表
    pub rooms: Vec<LiveRoom>,
    /// 正在直播的房间列表
    pub list: Vec<LiveRoom>,
    /// 关注列表中正在直播的人数
    pub count: i32,
    /// 关注列表中未开播的人数
    pub not_living_num: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiResult};

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "follow-up-list" => include_bytes!(
                "../../tests/contracts/live/account-private-read/follow-up-list/contract.json"
            )
            .as_slice(),
            "follow-up-web-list" => include_bytes!(
                "../../tests/contracts/live/account-private-read/follow-up-web-list/contract.json"
            )
            .as_slice(),
            _ => unreachable!("unknown live follow-up contract endpoint"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_follow_up_live_list() {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .live()
            .follow_up_list(Some(1), Some(2), Some(1), Some(true))
            .await
            .unwrap();
        tracing::info!("{:?}", data);
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_follow_up_live_web_list() {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi.live().follow_up_web_list(Some(false)).await.unwrap();
        tracing::info!("{:?}", data);
    }

    #[test]
    fn live_follow_up_list_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("follow-up-list")?;
        let params = [
            ("page", 1_i32.to_string()),
            ("page_size", 2_i32.to_string()),
            ("ignoreRecord", 1_i32.to_string()),
            ("hit_ab", true.to_string()),
        ];

        assert_eq!(contract.name, "live.follow_up_list");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.live.bilibili.com/xlive/web-ucenter/user/following"
        );
        assert_eq!(
            contract
                .request
                .query
                .get("ignoreRecord")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            contract.request.query.get("hit_ab").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            params,
            [
                ("page", "1".to_string()),
                ("page_size", "2".to_string()),
                ("ignoreRecord", "1".to_string()),
                ("hit_ab", "true".to_string())
            ]
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("FollowUpLiveData")
        );
        Ok(())
    }

    #[test]
    fn live_follow_up_web_list_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("follow-up-web-list")?;
        let params = [("hit_ab", false.to_string())];

        assert_eq!(contract.name, "live.follow_up_web_list");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.live.bilibili.com/xlive/web-ucenter/v1/xfetter/GetWebList"
        );
        assert_eq!(
            contract.request.query.get("hit_ab").map(String::as_str),
            Some("false")
        );
        assert_eq!(params, [("hit_ab", "false".to_string())]);
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[2].response.rust_model.as_deref(),
            Some("LiveWebListData")
        );
        Ok(())
    }

    #[test]
    fn live_follow_up_response_fixtures_parse_declared_models() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/live/account-private-read/follow-up-list/responses/anonymous.requires_login.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());

        let follow_up = ApiEnvelope::<FollowUpLiveData>::from_slice(include_bytes!(
            "../../tests/contracts/live/account-private-read/follow-up-list/responses/authenticated.success.json"
        ))?
        .into_payload()?;
        assert_eq!(follow_up.list.len(), 1);

        let empty_web = ApiEnvelope::<LiveWebListData>::from_slice(include_bytes!(
            "../../tests/contracts/live/account-private-read/follow-up-web-list/responses/normal.empty.success.json"
        ))?
        .into_payload()?;
        assert!(empty_web.rooms.is_empty());

        let vip_web = ApiEnvelope::<LiveWebListData>::from_slice(include_bytes!(
            "../../tests/contracts/live/account-private-read/follow-up-web-list/responses/vip.sample.success.json"
        ))?
        .into_payload()?;
        assert_eq!(vip_web.rooms.len(), 1);
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path = format!(
            "target/bpi-probe-runs/live/account-private-read/{endpoint}/{profile}.response.json"
        );
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn live_follow_up_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body("follow-up-list", profile) {
                let envelope = serde_json::from_value::<ApiEnvelope<FollowUpLiveData>>(body)?;
                if profile == "anonymous" {
                    assert!(envelope.ensure_success().unwrap_err().requires_login());
                } else {
                    let payload = envelope.into_payload()?;
                    assert!(payload.count >= 0);
                }
            }

            if let Some(body) = local_probe_body("follow-up-web-list", profile) {
                let envelope = serde_json::from_value::<ApiEnvelope<LiveWebListData>>(body)?;
                if profile == "anonymous" {
                    assert!(envelope.ensure_success().unwrap_err().requires_login());
                } else {
                    let payload = envelope.into_payload()?;
                    assert!(payload.count >= 0);
                }
            }
        }
        Ok(())
    }
}
