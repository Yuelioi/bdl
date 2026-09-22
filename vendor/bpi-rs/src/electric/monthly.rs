use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// --- `getChargeRecord` 的结构体 ---

/// 充电自动续费详情
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Renew {
    /// 自己的mid
    pub uid: u64,
    /// UP主的mid
    pub ruid: u64,
    /// 充电类型 172:一个月 173:连续包月 174:连续包年
    pub goods_id: u64,
    /// 充电状态 1
    pub status: u8,
    /// 下次续费时间秒级时间戳
    pub next_execute_time: u64,
    /// 签约时间秒级时间戳
    pub signed_time: u64,
    /// 下次续费金额单位为千分之一元人民币
    pub signed_price: u64,
    /// 签约平台 2:微信支付 4:支付宝
    pub pay_channel: u8,
    /// 下次充电天数
    pub period: u64,
    /// 充电渠道
    pub mobile_app: String,
}

/// 充电档位详情
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChargeItem {
    /// 充电档位代码
    pub privilege_type: u64,
    /// 充电图标
    pub icon: String,
    /// 充电档位名称
    pub name: String,
    /// 该档位过期时间秒级时间戳
    pub expire_time: u64,
    /// 充电自动续费详情
    pub renew: Option<Renew>,
    /// 该档位生效时间秒级时间戳
    pub start_time: u64,
    /// 充电自动续费列表
    pub renew_list: Option<Vec<Renew>>,
}

/// 包月充电UP主
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChargeUp {
    /// 充电UP主mid
    pub up_uid: u64,
    /// 充电UP主昵称
    pub user_name: String,
    /// 充电UP主头像url
    pub user_face: String,
    /// 充电详情
    pub item: Vec<ChargeItem>,
    /// 开始充电时间秒级时间戳
    pub start: u64,
    /// 是否可对UP主进行高档充电
    pub high_level_state: u8,
    /// 是否可对UP主进行专属问答 0:否 1:是 2:状态未知
    pub elec_reply_state: u8,
}

/// 包月充电列表数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChargeRecordData {
    /// 包月充电UP主列表
    pub list: Option<Vec<ChargeUp>>,
    /// 当前页数
    pub page: u64,
    /// 当前分页大小
    pub page_size: u64,
    /// 总页数
    pub total_page: u64,
    /// 用户总数
    pub total_num: u64,
    /// 是否有更多用户 0:否 1:是
    pub is_more: u8,
}

// --- `upower/item/detail` 的结构体 ---

/// 充电用户排名
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpowerRankUser {
    /// 充电用户索引
    pub rank: u64,
    /// 充电用户mid
    pub mid: u64,
    /// 充电用户昵称
    pub nickname: String,
    /// 充电用户头像url
    pub avatar: String,
}

/// 充电详情
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpowerRank {
    /// 充电用户总数
    pub total: u64,
    /// 充电总数文字说明
    pub total_desc: String,
    /// 充电用户列表
    pub list: Vec<UpowerRankUser>,
}

/// 充电介绍
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ItemDetailIntro {
    /// 充电介绍视频AV号
    pub intro_video_aid: String,
    /// 充电介绍语
    pub welcomes: String,
}

/// UP主信息卡片
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpUserCard {
    /// UP主头像url
    pub avatar: String,
    /// UP主昵称
    pub nickname: String,
}

/// 不同充电档位下的充电权益数
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpowerRightCount {
    #[serde(flatten)]
    pub counts: HashMap<String, u64>,
}

/// 包月充电详情数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpowerItemDetail {
    /// 充电详情
    pub upower_rank: UpowerRank,
    /// 充电欢迎语信息
    pub item: ItemDetailIntro,
    /// UP主信息
    pub user_card: UpUserCard,
    /// UP主开通的充电等级 1:非高档充电 2:高档充电
    pub upower_level: u8,
    /// 是否可对UP主进行专属问答
    pub elec_reply_state: u8,
    /// 包月充电券信息
    pub voucher_state: serde_json::Value,
    /// 不同充电档位下的充电权益数
    pub upower_right_count: UpowerRightCount,
    /// 享有的权益仅为粉丝勋章
    pub only_contain_medal: bool,
    /// 当前给该UP主包月充电的档位
    pub privilege_type: u64,
}

// --- `charge/follow/info` 的结构体 ---

/// UP主信息卡片
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpCard {
    /// UP主mid
    pub mid: u64,
    /// UP主昵称
    pub nickname: String,
    /// UP主认证信息
    pub official_title: String,
    /// UP主头像url
    pub avatar: String,
}

/// 用户信息卡片
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UserCard {
    /// 用户头像url
    pub avatar: String,
    /// 用户昵称
    pub nickname: String,
}

/// 与UP主的包月充电关系数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChargeFollowInfo {
    /// 已保持多少天包月充电状态
    pub days: u64,
    /// UP主信息
    pub up_card: UpCard,
    /// 自己的信息
    pub user_card: UserCard,
    /// 剩余天数 未处于包月充电状态为-1
    pub remain_days: i64,
    /// 剩余的天数是否小于1天 0:否 1:是 未处于包月充电状态为0
    pub remain_less_1day: u8,
    /// 充电详情
    pub upower_rank: UpowerRank,
    /// 充电图标url 仅在处于包月充电状态时有内容
    pub upower_icon: String,
    /// 当前自己享有该UP主的充电权益数
    pub upower_right_count: i64,
    /// 享有的权益仅为粉丝勋章
    pub only_contain_medal: bool,
    /// 当前给该UP主包月充电的档位代码
    pub privilege_type: u64,
    /// 充电挑战信息
    pub challenge_info: ChallengeInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChallengeInfo {
    pub challenge_id: String,
    pub description: String,
    pub challenge_type: i64,
    pub remaining_days: i64,
    pub end_time: String,
    pub progress: i64,
    pub targets: Vec<serde_json::Value>,
    pub state: i64,
    pub end_time_unix: i64,
    pub pub_dyn: i64,
    pub dyn_content: String,
}

/// UP主信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpInfo {
    /// UP主mid
    pub mid: u64,
    /// UP主昵称
    pub nickname: String,
    /// UP主头像url
    pub avatar: String,
    /// UP主认证类型
    pub r#type: i32,
    /// UP主认证文字
    pub title: String,
    /// UP主充电功能开启状态
    pub upower_state: u8,
}

/// 充电用户排名
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RankInfo {
    /// 充电用户mid
    pub mid: u64,
    /// 充电用户昵称
    pub nickname: String,
    /// 充电用户头像url
    pub avatar: String,
    /// 充电用户排名
    pub rank: u64,
    /// 包月充电天数
    pub day: u64,
    /// 包月充电过期时间恒为0
    pub expire_at: u64,
    /// 剩余天数恒为0
    pub remain_days: u64,
}

/// 自己的充电关系信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemberUserInfo {
    /// 用户mid
    pub mid: u64,
    /// 用户昵称
    pub nickname: String,
    /// 用户头像url
    pub avatar: String,
    /// 包月充电排名
    pub rank: i64,
    /// 包月充电天数
    pub day: u64,
    /// 包月充电过期时间秒级时间戳
    pub expire_at: u64,
    /// 剩余天数
    pub remain_days: u64,
}

/// 充电档位信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LevelInfo {
    /// 充电档位代码
    pub privilege_type: u64,
    /// 档位名称
    pub name: String,
    /// 档位价格单位为百分之一元人民币
    pub price: u64,
    /// 当前档位的用户总数
    pub member_total: u64,
}

/// 包月充电用户排名数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MemberRankData {
    /// UP主信息
    pub up_info: UpInfo,
    /// 当前档位的充电用户排名
    pub rank_info: Vec<RankInfo>,
    /// 自己在该档位下与UP主的充电关系
    pub user_info: MemberUserInfo,
    /// 当前档位充电用户总数
    pub member_total: u64,
    /// 当前充电档位代码
    pub privilege_type: u64,
    /// 自己是否给该UP主包月充电过
    pub is_charge: bool,
    /// 可显示排名的充电档位代码列表
    pub tabs: Vec<u64>,
    /// 可显示排名的充电档位信息
    pub level_info: Vec<LevelInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiResult};
    use tracing::info;

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "upower-item-detail" => include_bytes!(
                "../../tests/contracts/electric/public-read/upower-item-detail/contract.json"
            )
            .as_slice(),
            "upower-member-rank" => include_bytes!(
                "../../tests/contracts/electric/public-read/upower-member-rank/contract.json"
            )
            .as_slice(),
            "charge-record" => include_bytes!(
                "../../tests/contracts/electric/private-read/charge-record/contract.json"
            )
            .as_slice(),
            "charge-follow-info" => include_bytes!(
                "../../tests/contracts/electric/private-read/charge-follow-info/contract.json"
            )
            .as_slice(),
            _ => unreachable!("unknown electric monthly contract endpoint"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_charge_record() {
        let bpi = BpiClient::new().expect("client should build");
        // 获取自己使用中的包月充电列表
        let resp = bpi.electric().charge_record(1, 1).await;
        info!("响应: {:?}", resp);
        assert!(resp.is_ok());

        if let Ok(data) = resp {
            if let Some(list) = data.list {
                info!("找到 {} 个正在充电的UP主", list.len());
            } else {
                info!("没有正在充电的UP主");
            }
        }
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_upower_item_detail() {
        let bpi = BpiClient::new().expect("client should build");
        // 替换为有效的UP主mid
        let up_mid = 1265680561;
        let resp = bpi.electric().upower_item_detail(up_mid).await;
        info!("响应: {:?}", resp);
        assert!(resp.is_ok());

        if let Ok(data) = resp {
            info!(
                "UP主 {} 的充电总人数: {}",
                data.user_card.nickname, data.upower_rank.total
            );
        }
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_charge_follow_info() {
        let bpi = BpiClient::new().expect("client should build");
        let up_mid = 293793435;
        let resp = bpi.electric().charge_follow_info(up_mid).await;
        info!("响应: {:?}", resp);
        assert!(resp.is_ok());

        if let Ok(data) = resp {
            info!(
                "与UP主 {} 的充电关系：已保持 {} 天",
                data.up_card.nickname, data.days
            );
        }
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_upower_member_rank() {
        let bpi = BpiClient::new().expect("client should build");
        // 替换为有效的UP主mid
        let up_mid = 1265680561;
        // 获取所有档位的用户排名
        let resp = bpi.electric().upower_member_rank(up_mid, 1, 10, None).await;
        info!("响应: {:?}", resp);
        assert!(resp.is_ok());

        if let Ok(data) = resp {
            info!("当前档位充电用户总数: {}", data.member_total);
            if let Some(first_rank) = data.rank_info.first() {
                info!("排名第一的用户: {}", first_rank.nickname);
            }
        }
    }

    #[test]
    fn electric_upower_item_detail_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("upower-item-detail")?;

        assert_eq!(contract.name, "electric.upower_item_detail");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/upower/item/detail"
        );
        assert_eq!(
            contract.request.query.get("up_mid").map(String::as_str),
            Some("1265680561")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("UpowerItemDetail")
        );
        Ok(())
    }

    #[test]
    fn electric_upower_member_rank_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("upower-member-rank")?;

        assert_eq!(contract.name, "electric.upower_member_rank");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/upower/up/member/rank/v2"
        );
        assert_eq!(
            contract.request.query.get("up_mid").map(String::as_str),
            Some("1265680561")
        );
        assert_eq!(
            contract.request.query.get("pn").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            contract.request.query.get("ps").map(String::as_str),
            Some("10")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("MemberRankData")
        );
        Ok(())
    }

    #[test]
    fn electric_charge_record_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("charge-record")?;

        assert_eq!(contract.name, "electric.charge_record");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.live.bilibili.com/xlive/revenue/v1/guard/getChargeRecord"
        );
        assert_eq!(
            contract.request.query.get("page").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            contract.request.query.get("type").map(String::as_str),
            Some("1")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("ChargeRecordData")
        );
        Ok(())
    }

    #[test]
    fn electric_charge_follow_info_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("charge-follow-info")?;

        assert_eq!(contract.name, "electric.charge_follow_info");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/upower/charge/follow/info"
        );
        assert_eq!(
            contract.request.query.get("up_mid").map(String::as_str),
            Some("1265680561")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("ChargeFollowInfo")
        );
        Ok(())
    }

    #[test]
    fn electric_monthly_response_fixtures_parse_declared_models() -> BpiResult<()> {
        let item_detail = ApiEnvelope::<UpowerItemDetail>::from_slice(include_bytes!(
            "../../tests/contracts/electric/public-read/upower-item-detail/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(item_detail.upower_rank.list.len(), 1);
        assert_eq!(item_detail.upower_right_count.counts["100"], 5);

        let anonymous_rank = ApiEnvelope::<MemberRankData>::from_slice(include_bytes!(
            "../../tests/contracts/electric/public-read/upower-member-rank/responses/anonymous.success.json"
        ))?
        .into_payload()?;
        assert_eq!(anonymous_rank.user_info.mid, 0);

        let authenticated_rank = ApiEnvelope::<MemberRankData>::from_slice(include_bytes!(
            "../../tests/contracts/electric/public-read/upower-member-rank/responses/authenticated.success.json"
        ))?
        .into_payload()?;
        assert_eq!(authenticated_rank.user_info.mid, 1);
        Ok(())
    }

    #[test]
    fn electric_monthly_private_response_fixtures_parse_declared_models() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/electric/private-read/charge-record/responses/anonymous.requires_login.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());

        let charge_record = ApiEnvelope::<ChargeRecordData>::from_slice(include_bytes!(
            "../../tests/contracts/electric/private-read/charge-record/responses/authenticated.success.json"
        ))?
        .into_payload()?;
        assert_eq!(charge_record.total_num, 0);

        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/electric/private-read/charge-follow-info/responses/anonymous.requires_login.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());

        let follow_info = ApiEnvelope::<ChargeFollowInfo>::from_slice(include_bytes!(
            "../../tests/contracts/electric/private-read/charge-follow-info/responses/authenticated.success.json"
        ))?
        .into_payload()?;
        assert_eq!(follow_info.up_card.mid, 1265680561);
        Ok(())
    }

    fn local_probe_body(batch: &str, endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/electric/{batch}/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn electric_monthly_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body("public-read", "upower-item-detail", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<UpowerItemDetail>>(body)?
                    .into_payload()?;
                assert!(payload.upower_rank.total >= payload.upower_rank.list.len() as u64);
            }

            if let Some(body) = local_probe_body("public-read", "upower-member-rank", profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<MemberRankData>>(body)?.into_payload()?;
                assert!(payload.member_total >= payload.rank_info.len() as u64);
            }
        }
        Ok(())
    }

    #[test]
    fn electric_monthly_private_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body("private-read", "charge-record", profile) {
                let envelope = serde_json::from_value::<ApiEnvelope<ChargeRecordData>>(body)?;
                if profile == "anonymous" {
                    let err = envelope.ensure_success().unwrap_err();
                    assert!(err.requires_login());
                } else {
                    let payload = envelope.into_payload()?;
                    assert!(payload.total_num >= payload.list.as_ref().map_or(0, Vec::len) as u64);
                }
            }

            if let Some(body) = local_probe_body("private-read", "charge-follow-info", profile) {
                let envelope = serde_json::from_value::<ApiEnvelope<ChargeFollowInfo>>(body)?;
                if profile == "anonymous" {
                    let err = envelope.ensure_success().unwrap_err();
                    assert!(err.requires_login());
                } else {
                    let payload = envelope.into_payload()?;
                    assert!(payload.upower_rank.total >= payload.upower_rank.list.len() as u64);
                }
            }
        }
        Ok(())
    }
}
