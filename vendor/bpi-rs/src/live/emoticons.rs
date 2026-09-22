use serde::{Deserialize, Serialize};

// ================= 数据结构 =================

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct EmoticonItem {
    /// 突出展示
    pub bulge_display: i32,
    /// 描述
    pub descript: String,
    /// 触发关键词
    pub emoji: String,
    /// 表情ID
    pub emoticon_id: i64,
    /// 表情唯一标识
    pub emoticon_unique: String,
    /// 表情值类型
    pub emoticon_value_type: i32,
    /// 表情图片高度
    pub height: i32,
    /// 身份限制标识
    pub identity: i32,
    /// 播放器区域内展示
    pub in_player_area: i32,
    /// 是否为动态表情
    pub is_dynamic: i32,
    /// 使用权限
    pub perm: i32,
    /// 解锁需求礼物
    pub unlock_need_gift: i32,
    /// 解锁需求等级
    pub unlock_need_level: i32,
    /// 解锁展示颜色
    pub unlock_show_color: String,
    /// 解锁展示图片
    pub unlock_show_image: String,
    /// 解锁展示文字
    pub unlock_show_text: String,
    /// 表情图片URL
    pub url: String,
    /// 表情图片宽度
    pub width: i32,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct TopShowItem {
    /// 图片
    pub image: String,
    /// 文字
    pub text: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct TopShow {
    /// 左上
    pub top_left: TopShowItem,
    /// 右上
    pub top_right: TopShowItem,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct EmoticonPackage {
    /// 封面URL
    pub current_cover: String,
    /// 表情列表
    pub emoticons: Vec<EmoticonItem>,
    /// 文字描述
    pub pkg_descript: String,
    /// 包ID
    pub pkg_id: i64,
    /// 包名称
    pub pkg_name: String,
    /// 使用权限
    pub pkg_perm: i32,
    /// 包类型
    pub pkg_type: i32,
    /// 最近使用的表情
    pub recently_used_emoticons: Vec<serde_json::Value>,
    /// 顶部展示信息
    pub top_show: Option<TopShow>,
    /// 最近使用的顶部展示信息
    pub top_show_recent: Option<TopShow>,
    /// 解锁所需身份标识
    pub unlock_identity: i32,
    /// 解锁所需礼物
    pub unlock_need_gift: i32,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct EmoticonData {
    /// 表情包数据
    pub data: Vec<EmoticonPackage>,
    /// 品牌标识
    pub fans_brand: i32,
    /// 购买链接
    pub purchase_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/live/room-interaction-read/emoticons/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_live_emoticons() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi.live().emoticons(14047, "pc").await?;

        assert!(!data.data.is_empty());
        Ok(())
    }

    #[test]
    fn live_emoticons_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params = [
            ("room_id", 14047_i64.to_string()),
            ("platform", "pc".to_string()),
        ];

        assert_eq!(contract.name, "live.emoticons");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.live.bilibili.com/xlive/web-ucenter/v2/emoticon/GetEmoticons"
        );
        assert_eq!(
            contract.request.query.get("room_id").map(String::as_str),
            Some("14047")
        );
        assert_eq!(
            contract.request.query.get("platform").map(String::as_str),
            Some("pc")
        );
        assert_eq!(
            params,
            [
                ("room_id", "14047".to_string()),
                ("platform", "pc".to_string())
            ]
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.error.as_deref(),
            Some("requires_login")
        );
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("EmoticonData")
        );
        Ok(())
    }

    #[test]
    fn live_emoticons_response_fixtures_parse_declared_model() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/live/room-interaction-read/emoticons/responses/anonymous.requires_login.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());

        let payload = ApiEnvelope::<EmoticonData>::from_slice(include_bytes!(
            "../../tests/contracts/live/room-interaction-read/emoticons/responses/authenticated.success.json"
        ))?
        .into_payload()?;
        assert_eq!(payload.data.len(), 1);
        assert_eq!(payload.data[0].emoticons.len(), 1);
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!(
            "target/bpi-probe-runs/live/room-interaction-read/emoticons/{profile}.response.json"
        );
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn live_emoticons_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let envelope = serde_json::from_value::<ApiEnvelope<EmoticonData>>(body)?;

            if profile == "anonymous" {
                assert!(envelope.ensure_success().unwrap_err().requires_login());
            } else {
                let payload = envelope.into_payload()?;
                assert!(!payload.data.is_empty());
            }
        }
        Ok(())
    }
}
