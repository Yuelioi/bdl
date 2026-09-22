use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ChargeVipInfo {
    /// 大会员过期时间（恒为 0）
    #[serde(rename = "vipDueMsec")]
    pub vip_due_msec: i64,

    /// 大会员状态（包月充电时恒为 0；自定义充电：0=无, 1=有）
    #[serde(rename = "vipStatus")]
    pub vip_status: i32,

    /// 大会员类型（包月充电时恒为 0；自定义充电：0=无, 1=月大会员, 2=年度及以上大会员）
    #[serde(rename = "vipType")]
    pub vip_type: i32,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ChargeUser {
    /// 充电用户昵称
    pub uname: String,

    /// 充电用户头像 url
    pub avatar: String,

    /// 充电对象 mid
    pub mid: i64,

    /// 充电用户 mid(支付id?)
    pub pay_mid: i64,

    /// 充电用户排名（取决于充电多少）
    pub rank: i32,

    /// 充电用户会员信息
    pub vip_info: ChargeVipInfo,

    /// 充电留言（为空表示无留言）
    pub message: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ChargeMonthUpData {
    /// 本月充电人数
    pub count: i32,

    /// 本月充电用户列表
    #[serde(default)]
    pub list: Vec<ChargeUser>,

    /// 总计充电次数
    pub total_count: i32,
}

/// 视频充电展示信息（高阶信息）
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct VideoShowInfoHighLevel {
    /// 权限类型
    pub privilege_type: i32,
    /// 主标题
    pub title: String,
    /// 副标题
    pub sub_title: String,
    /// 是否显示按钮
    pub show_button: bool,
}

/// 视频充电展示信息
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct VideoShowInfo {
    /// 是否显示
    pub show: bool,

    /// 充电功能开启状态
    /// - `-1`: 未开通
    /// - `1`: 开通
    /// - `2`: 包月、自定义充电
    /// - `3`: 包月高档、自定义充电
    pub state: i32,

    /// 充电按钮显示文字
    pub title: String,

    /// 充电跳转 URL 支付页面
    pub jump_url: String,

    /// 图标 URL
    pub icon: String,

    /// 充电专属视频信息
    pub high_level: VideoShowInfoHighLevel,

    /// 充电问答 ID
    pub with_qa_id: i64,
}

/// 视频充电展示数据
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct VideoElecShowData {
    /// 展示选项
    pub show_info: VideoShowInfo,
    /// 目标视频充电人数
    pub av_count: i32,
    /// 本月充电人数
    pub count: i32,
    /// 总计充电人数
    pub total_count: i32,
    /// 本月充电用户列表
    #[serde(default)]
    pub list: Vec<ChargeUser>,
}

// 充电列表分页信息
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RechargePage {
    /// 当前页数
    pub current_page: u64,
    /// 当前分页大小
    pub page_size: u64,
    /// 记录总数
    pub total_count: u64,
    /// 总页数
    pub total_page: u64,
}

/// 充电信息本体
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RechargeRecord {
    /// 充电人mid
    pub mid: u64,
    /// 充电人昵称
    pub name: String,
    /// 充电人头像
    pub avatar: String,
    /// 原始B币数
    pub original_third_coin: f64,
    /// 实际收到的贝壳数
    pub brokerage: f64,
    /// 充电渠道 Web/安卓/iOS
    pub remark: String,
    /// 充电时间 yyyy-MM-dd HH:mm:ss
    pub ctime: String,
}

/// 充电列表数据
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RechargeData {
    /// 分页信息
    pub page: RechargePage,
    /// 充电信息本体
    pub result: Vec<RechargeRecord>,
}

/// 充电列表分页信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ElecRankPager {
    /// 当前页数
    pub current: u64,
    /// 当前分页大小
    pub size: u64,
    /// 记录总数
    pub total: u64,
}

/// 充电信息本体
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ElecRankRecord {
    /// 0
    pub aid: u64,
    /// 空
    pub bvid: String,
    /// 充电电池数
    pub elec_num: f64,
    /// 空
    pub title: String,
    /// 充电人昵称
    pub uname: String,
    /// 充电人头像
    pub avatar: String,
    /// 充电时间 yyyy-MM-dd HH:mm:ss
    pub ctime: String,
}

/// 历史充电数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ElecRankData {
    /// 充电信息本体
    pub list: Vec<ElecRankRecord>,
    /// 分页信息
    pub pager: ElecRankPager,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiResult};
    use chrono::{Duration, Utc};
    use tracing::info;

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "month-up-list" => include_bytes!(
                "../../tests/contracts/electric/public-read/month-up-list/contract.json"
            )
            .as_slice(),
            "video-show" => include_bytes!(
                "../../tests/contracts/electric/public-read/video-show/contract.json"
            )
            .as_slice(),
            "recharge-list" => include_bytes!(
                "../../tests/contracts/electric/private-read/recharge-list/contract.json"
            )
            .as_slice(),
            "rank-recent" => include_bytes!(
                "../../tests/contracts/electric/private-read/rank-recent/contract.json"
            )
            .as_slice(),
            _ => unreachable!("unknown electric charge-list contract endpoint"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_electric_month_up_list() {
        let bpi = BpiClient::new().expect("client should build");
        let resp = bpi.electric().month_up_list(53456).await;
        assert!(resp.is_ok());
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_electric_video_show() {
        let bpi = BpiClient::new().expect("client should build");
        let resp = bpi
            .electric()
            .video_show(53456, None, Some("BV1Dh411S7sS"))
            .await;
        assert!(resp.is_ok());
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_recharge_list() {
        let bpi = BpiClient::new().expect("client should build");
        // 测试获取第一页，每页10条记录
        let resp = bpi.electric().recharge_list(1, 10, None, None).await;
        info!("响应: {:?}", resp);
        assert!(resp.is_ok());

        if let Ok(data) = resp {
            info!("充电总记录数: {}", data.page.total_count);
            info!("当前页充电记录数: {}", data.result.len());
            if let Some(record) = data.result.first() {
                info!("第一条充电记录信息: {:?}", record);
            }
        }
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_recharge_list_with_dates() {
        let bpi = BpiClient::new().expect("client should build");
        let now = Utc::now().date_naive();
        let start_date = now - Duration::days(30);
        let end_date = now;

        let resp = bpi
            .electric()
            .recharge_list(1, 10, Some(start_date), Some(end_date))
            .await;
        info!("响应: {:?}", resp);
        assert!(resp.is_ok());

        if let Ok(data) = resp {
            info!("在日期范围内获取到的总记录数: {}", data.page.total_count);
        }
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_elec_rank_recent() {
        let bpi = BpiClient::new().expect("client should build");
        // 测试获取第一页，每页10条记录
        let resp = bpi.electric().rank_recent(Some(1), Some(10)).await;
        info!("响应: {:?}", resp);
        assert!(resp.is_ok());

        if let Ok(data) = resp {
            info!("充电总记录数: {}", data.pager.total);
            info!("当前页充电记录数: {}", data.list.len());
            if let Some(record) = data.list.first() {
                info!("第一条充电记录信息: {:?}", record);
            }
        }
    }

    #[test]
    fn electric_month_up_list_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("month-up-list")?;

        assert_eq!(contract.name, "electric.month_up_list");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/ugcpay-rank/elec/month/up"
        );
        assert_eq!(
            contract.request.query.get("up_mid").map(String::as_str),
            Some("53456")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("ChargeMonthUpData")
        );
        Ok(())
    }

    #[test]
    fn electric_video_show_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("video-show")?;

        assert_eq!(contract.name, "electric.video_show");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/web-interface/elec/show"
        );
        assert_eq!(
            contract.request.query.get("mid").map(String::as_str),
            Some("53456")
        );
        assert_eq!(
            contract.request.query.get("bvid").map(String::as_str),
            Some("BV1Dh411S7sS")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("VideoElecShowData")
        );
        Ok(())
    }

    #[test]
    fn electric_recharge_list_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("recharge-list")?;

        assert_eq!(contract.name, "electric.recharge_list");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://pay.bilibili.com/bk/brokerage/listForCustomerRechargeRecord"
        );
        assert_eq!(
            contract.request.query.get("customerId").map(String::as_str),
            Some("10026")
        );
        assert_eq!(
            contract
                .request
                .query
                .get("currentPage")
                .map(String::as_str),
            Some("1")
        );
        assert_eq!(
            contract.request.query.get("pageSize").map(String::as_str),
            Some("10")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(contract.cases[0].response.api_code, Some(800501007));
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("RechargeData")
        );
        Ok(())
    }

    #[test]
    fn electric_rank_recent_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("rank-recent")?;

        assert_eq!(contract.name, "electric.rank_recent");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://member.bilibili.com/x/h5/elec/rank/recent"
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
            contract.cases[1].response.rust_model.as_deref(),
            Some("ElecRankData")
        );
        Ok(())
    }

    #[test]
    fn electric_charge_list_response_fixtures_parse_declared_models() -> BpiResult<()> {
        let month_up = ApiEnvelope::<ChargeMonthUpData>::from_slice(include_bytes!(
            "../../tests/contracts/electric/public-read/month-up-list/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(month_up.list.len(), 1);
        assert_eq!(month_up.list[0].rank, 1);

        let video_show = ApiEnvelope::<VideoElecShowData>::from_slice(include_bytes!(
            "../../tests/contracts/electric/public-read/video-show/responses/success.json"
        ))?
        .into_payload()?;
        assert!(video_show.show_info.show);
        Ok(())
    }

    #[test]
    fn electric_charge_list_private_response_fixtures_parse_declared_models() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/electric/private-read/recharge-list/responses/anonymous.requires_login.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());

        let recharge = ApiEnvelope::<RechargeData>::from_slice(include_bytes!(
            "../../tests/contracts/electric/private-read/recharge-list/responses/authenticated.success.json"
        ))?
        .into_payload()?;
        assert_eq!(recharge.result.len(), 1);

        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/electric/private-read/rank-recent/responses/anonymous.requires_login.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());

        let normal_rank = ApiEnvelope::<ElecRankData>::from_slice(include_bytes!(
            "../../tests/contracts/electric/private-read/rank-recent/responses/normal.success.json"
        ))?
        .into_payload()?;
        assert!(normal_rank.list.is_empty());

        let vip_rank = ApiEnvelope::<ElecRankData>::from_slice(include_bytes!(
            "../../tests/contracts/electric/private-read/rank-recent/responses/vip.success.json"
        ))?
        .into_payload()?;
        assert_eq!(vip_rank.list.len(), 1);
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
    fn electric_charge_list_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body("public-read", "month-up-list", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<ChargeMonthUpData>>(body)?
                    .into_payload()?;
                assert!(payload.total_count >= payload.count);
            }

            if let Some(body) = local_probe_body("public-read", "video-show", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<VideoElecShowData>>(body)?
                    .into_payload()?;
                assert!(payload.total_count >= payload.count);
            }
        }
        Ok(())
    }

    #[test]
    fn electric_charge_list_private_models_match_local_probe_outputs_when_available()
    -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body("private-read", "recharge-list", profile) {
                let envelope = serde_json::from_value::<ApiEnvelope<RechargeData>>(body)?;
                if profile == "anonymous" {
                    let err = envelope.ensure_success().unwrap_err();
                    assert!(err.requires_login());
                } else {
                    let payload = envelope.into_payload()?;
                    assert!(payload.page.total_count >= payload.result.len() as u64);
                }
            }

            if let Some(body) = local_probe_body("private-read", "rank-recent", profile) {
                let envelope = serde_json::from_value::<ApiEnvelope<ElecRankData>>(body)?;
                if profile == "anonymous" {
                    let err = envelope.ensure_success().unwrap_err();
                    assert!(err.requires_login());
                } else {
                    let payload = envelope.into_payload()?;
                    assert!(payload.pager.total >= payload.list.len() as u64);
                }
            }
        }
        Ok(())
    }
}
