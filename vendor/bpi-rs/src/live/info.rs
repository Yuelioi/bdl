use serde::{Deserialize, Serialize};

// ================= 数据结构 =================

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct RoomPendantFrame {
    /// 名称
    pub name: String,
    /// 值
    pub value: String,
    /// 位置
    pub position: i32,
    /// 描述
    pub desc: String,
    /// 分区
    pub area: i32,
    /// 旧分区
    pub area_old: i32,
    /// 背景色
    pub bg_color: String,
    /// 背景图
    pub bg_pic: String,
    /// 是否旧分区号
    pub use_old_area: bool,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct RoomPendantBadge {
    /// 类型
    pub name: String,
    /// 位置
    pub position: i32,
    /// 值
    pub value: String,
    /// 描述
    pub desc: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct RoomPendants {
    /// 头像框
    pub frame: RoomPendantFrame,
    /// 手机版头像框
    pub mobile_frame: Option<RoomPendantFrame>,
    /// 大v
    pub badge: Option<RoomPendantBadge>,
    /// 手机版大v
    pub mobile_badge: Option<RoomPendantBadge>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct RoomStudioInfo {
    // 根据实际情况添加字段
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct RoomInfoData {
    /// 主播mid
    pub uid: i64,
    /// 直播间长号
    pub room_id: i64,
    /// 直播间短号
    pub short_id: i64,
    /// 关注数量
    pub attention: i64,
    /// 观看人数
    pub online: i64,
    /// 是否竖屏
    pub is_portrait: bool,
    /// 描述
    pub description: String,
    /// 直播状态
    pub live_status: i32,
    /// 分区id
    pub area_id: i32,
    /// 父分区id
    pub parent_area_id: i32,
    /// 父分区名称
    pub parent_area_name: String,
    /// 旧版分区id
    pub old_area_id: i32,
    /// 背景图片链接
    pub background: String,
    /// 标题
    pub title: String,
    /// 封面
    pub user_cover: String,
    /// 关键帧
    pub keyframe: String,
    /// 直播开始时间
    pub live_time: String,
    /// 标签
    pub tags: String,
    /// 禁言状态
    pub room_silent_type: String,
    /// 禁言等级
    pub room_silent_level: i32,
    /// 禁言时间
    pub room_silent_second: i64,
    /// 分区名称
    pub area_name: String,
    /// 热词
    pub hot_words: Vec<String>,
    /// 热词状态
    pub hot_words_status: i32,
    /// 头像框\大v
    pub new_pendants: RoomPendants,
    /// pk状态
    pub pk_status: i32,
    /// pk id
    pub pk_id: i64,
    /// 允许更改分区时间
    pub allow_change_area_time: i64,
    /// 允许上传封面时间
    pub allow_upload_cover_time: i64,
    /// 工作室信息
    pub studio_info: Option<RoomStudioInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/live/public-core/room-info/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_room_info() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi.live().room_info(23174842).await?;

        assert_eq!(data.room_id, 23174842);
        Ok(())
    }

    #[test]
    fn live_room_info_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "live.room_info");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.live.bilibili.com/room/v1/Room/get_info"
        );
        assert_eq!(
            contract.request.query.get("room_id").map(String::as_str),
            Some("23174842")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("RoomInfoData")
        );
        Ok(())
    }

    #[test]
    fn live_room_info_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<RoomInfoData>::from_slice(include_bytes!(
            "../../tests/contracts/live/public-core/room-info/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.room_id, 23174842);
        assert_eq!(payload.live_status, 1);
        Ok(())
    }

    #[test]
    fn room_pendants_accepts_null_badge() -> BpiResult<()> {
        let pendants = serde_json::from_str::<RoomPendants>(
            r#"{
                "frame": {
                    "name": "",
                    "value": "",
                    "position": 0,
                    "desc": "",
                    "area": 0,
                    "area_old": 0,
                    "bg_color": "",
                    "bg_pic": "",
                    "use_old_area": false
                },
                "mobile_frame": null,
                "badge": null,
                "mobile_badge": null
            }"#,
        )?;

        assert!(pendants.badge.is_none());
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/live/public-core/room-info/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn live_room_info_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body(profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<RoomInfoData>>(body)?.into_payload()?;
                assert_eq!(payload.room_id, 23174842);
            }
        }
        Ok(())
    }
}
