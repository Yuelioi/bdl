use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct UserInfo {
    /// 用户UID
    pub uid: i64,
    /// 用户基本信息
    pub base: UserBaseInfo,
    /// 粉丝牌信息
    pub medal: UserMedalInfo,
    /// 财富信息
    pub wealth: Option<serde_json::Value>,
    /// 标题
    pub title: Option<serde_json::Value>,
    /// 大航海信息
    pub guard: UserGuardInfo,
    /// 头像框
    pub uhead_frame: Option<serde_json::Value>,
    /// 大航海队长
    pub guard_leader: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct GuardTabInfo {
    /// 大航海总人数
    pub num: i32,
    /// 总页数
    pub page: i32,
    /// 当前页数
    pub now: i32,
    /// 成就等级
    pub achievement_level: i32,
    /// 主播守护成就等级
    pub anchor_guard_achieve_level: i32,
    /// 成就图标
    pub achievement_icon_src: String,
    /// 购买守护图标
    pub buy_guard_icon_src: String,
    /// 规则文档链接
    pub rule_doc_src: String,
    /// 背景图片
    pub ex_background_src: String,
    /// 颜色开始
    pub color_start: String,
    /// 颜色结束
    pub color_end: String,
    /// 标签颜色
    pub tab_color: Vec<String>,
    /// 标题颜色
    pub title_color: Vec<String>,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct UserOriginInfo {
    /// 用户名
    pub name: String,
    /// 头像
    pub face: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct UserOfficialInfo {
    /// 角色
    pub role: i32,
    /// 标题
    pub title: String,
    /// 描述
    pub desc: String,
    /// 类型
    pub r#type: i32,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct UserBaseInfo {
    /// 用户名
    pub name: String,
    /// 头像
    pub face: String,
    /// 名称颜色
    pub name_color: i32,
    /// 是否匿名
    pub is_mystery: bool,
    /// 风险控制信息
    pub risk_ctrl_info: Option<serde_json::Value>,
    /// 原始信息
    pub origin_info: UserOriginInfo,
    /// 官方信息
    pub official_info: UserOfficialInfo,
    /// 名称颜色字符串
    pub name_color_str: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct UserMedalInfo {
    /// 粉丝牌名称
    pub name: String,
    /// 粉丝牌等级
    pub level: i32,
    /// 颜色开始
    pub color_start: i32,
    /// 颜色结束
    pub color_end: i32,
    /// 边框颜色
    pub color_border: i32,
    /// 颜色
    pub color: i32,
    /// ID
    pub id: i32,
    /// 类型
    pub typ: i32,
    /// 是否点亮
    pub is_light: i32,
    /// 主播UID
    pub ruid: i64,
    /// 大航海等级
    pub guard_level: i32,
    /// 亲密度
    pub score: i32,
    /// 大航海图标
    pub guard_icon: String,
    /// 荣誉图标
    pub honor_icon: String,
    /// V2粉丝牌颜色开始
    pub v2_medal_color_start: String,
    /// V2粉丝牌颜色结束
    pub v2_medal_color_end: String,
    /// V2粉丝牌边框颜色
    pub v2_medal_color_border: String,
    /// V2粉丝牌文本颜色
    pub v2_medal_color_text: String,
    /// V2粉丝牌等级颜色
    pub v2_medal_color_level: String,
    /// 用户接收数量
    pub user_receive_count: i32,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct UserGuardInfo {
    /// 大航海等级
    pub level: i32,
    /// 过期时间字符串
    pub expired_str: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct GuardMember {
    /// 主播UID
    pub ruid: i64,
    /// 排名
    pub rank: i32,
    /// 陪伴天数
    pub accompany: i32,
    /// 用户信息
    pub uinfo: UserInfo,
    /// 亲密度
    pub score: i32,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct GuardListData {
    /// 大航海信息
    pub info: GuardTabInfo,
    /// 前三名
    pub top3: Vec<GuardMember>,
    /// 大航海成员列表
    pub list: Vec<GuardMember>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/live/guard-read/guard-list/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_guard_list() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .live()
            .guard_list(23174842, 504140200, None, None, None)
            .await?;

        assert!(!data.list.is_empty());
        Ok(())
    }

    #[test]
    fn live_guard_list_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params: Vec<(&str, String)> = vec![
            ("roomid", 23174842_i64.to_string()),
            ("ruid", 504140200_i64.to_string()),
            ("page", 1_i32.to_string()),
            ("page_size", 20_i32.to_string()),
            ("typ", 5_i32.to_string()),
        ];

        assert_eq!(contract.name, "live.guard_list");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.live.bilibili.com/xlive/app-room/v2/guardTab/topListNew"
        );
        assert_eq!(
            contract.request.query.get("roomid").map(String::as_str),
            Some("23174842")
        );
        assert_eq!(
            contract.request.query.get("ruid").map(String::as_str),
            Some("504140200")
        );
        assert_eq!(
            params,
            vec![
                ("roomid", "23174842".to_string()),
                ("ruid", "504140200".to_string()),
                ("page", "1".to_string()),
                ("page_size", "20".to_string()),
                ("typ", "5".to_string()),
            ]
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("GuardListData")
        );
        Ok(())
    }

    #[test]
    fn live_guard_list_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<GuardListData>::from_slice(include_bytes!(
            "../../tests/contracts/live/guard-read/guard-list/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.info.now, 1);
        assert_eq!(payload.top3.len(), 1);
        assert_eq!(payload.list.len(), 1);
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/live/guard-read/guard-list/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn live_guard_list_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<GuardListData>>(body)?.into_payload()?;

            assert!(!payload.list.is_empty() || !payload.top3.is_empty());
        }
        Ok(())
    }
}
