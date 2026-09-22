use serde::{Deserialize, Serialize};

// ================= 数据结构 =================

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct WatchedShow {
    /// 开关
    pub switch: bool,
    /// 看过人数
    pub num: i32,
    /// 小文本
    pub text_small: String,
    /// 大文本
    pub text_large: String,
    /// 图标URL
    pub icon: String,
    /// 图标位置
    pub icon_location: i32,
    /// Web端图标URL
    pub icon_web: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct RecommendRoom {
    /// 头像框
    pub head_box: Option<serde_json::Value>,
    /// 分区ID
    pub area_v2_id: i32,
    /// 父分区ID
    pub area_v2_parent_id: i32,
    /// 分区名称
    pub area_v2_name: String,
    /// 父分区名称
    pub area_v2_parent_name: String,
    /// 广播类型
    pub broadcast_type: i32,
    /// 封面URL
    pub cover: String,
    /// 直播间链接
    pub link: String,
    /// 观看人数
    pub online: i32,
    /// 挂件信息
    #[serde(rename = "pendant_Info")]
    pub pendant_info: serde_json::Value,
    /// 直播间ID
    pub roomid: i64,
    /// 直播间标题
    pub title: String,
    /// 主播用户名
    pub uname: String,
    /// 主播头像URL
    pub face: String,
    /// 认证信息
    pub verify: serde_json::Value,
    /// 主播用户mid
    pub uid: i64,
    /// 关键帧URL
    pub keyframe: String,
    /// 是否自动播放
    pub is_auto_play: i32,
    /// 头像框类型
    pub head_box_type: i32,
    /// 标记
    pub flag: i32,
    /// 会话ID
    pub session_id: String,
    /// 展示回调URL
    pub show_callback: String,
    /// 点击回调URL
    pub click_callback: String,
    /// 特殊ID
    pub special_id: i32,
    /// 观看展示
    pub watched_show: WatchedShow,
    /// 是否为NFT头像
    pub is_nft: i32,
    /// NFT标记
    pub nft_dmark: String,
    /// 是否为广告
    pub is_ad: bool,
    /// 广告透明内容
    pub ad_transparent_content: Option<serde_json::Value>,
    /// 显示广告图标
    pub show_ad_icon: bool,
    /// 状态
    pub status: bool,
    /// 关注者数量
    pub followers: i32,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct RecommendData {
    /// 推荐房间列表
    pub recommend_room_list: Vec<RecommendRoom>,
    /// 置顶直播间号
    pub top_room_id: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/live/public-core/recommend/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_live_recommend() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi.live().recommend().await?;

        assert!(!data.recommend_room_list.is_empty());
        Ok(())
    }

    #[test]
    fn live_recommend_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "live.recommend");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.live.bilibili.com/xlive/web-interface/v1/webMain/getMoreRecList"
        );
        assert_eq!(
            contract.request.query.get("platform").map(String::as_str),
            Some("web")
        );
        assert_eq!(
            contract
                .request
                .query
                .get("web_location")
                .map(String::as_str),
            Some("333.1007")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("RecommendData")
        );
        Ok(())
    }

    #[test]
    fn live_recommend_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<RecommendData>::from_slice(include_bytes!(
            "../../tests/contracts/live/public-core/recommend/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.recommend_room_list.len(), 1);
        assert_eq!(payload.recommend_room_list[0].roomid, 1);
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/live/public-core/recommend/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn live_recommend_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body(profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<RecommendData>>(body)?.into_payload()?;
                assert!(!payload.recommend_room_list.is_empty());
            }
        }
        Ok(())
    }
}
