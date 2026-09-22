use crate::models::{Account, Vip};
use serde::Deserialize;

#[cfg(test)]
const VIP_CENTER_INFO_ENDPOINT: &str = "https://api.bilibili.com/x/vip/web/vip_center/combine";

/// 大会员中心信息响应结构体
#[derive(Debug, Clone, Deserialize)]
pub struct VipCenterData {
    /// 用户信息
    pub user: User,
    /// 钱包信息
    pub wallet: WalletInfo,

    pub in_review: bool,
}

/// 用户信息结构体
#[derive(Debug, Clone, Deserialize)]
pub struct User {
    /// 账号基本信息
    pub account: Option<Account>,
    /// 账号会员信息
    pub vip: Option<Vip>,
    /// 电视会员信息
    pub tv: Option<TvVipInfo>,
    /// 空字段
    pub background_image_small: String,
    /// 空字段
    pub background_image_big: String,
    /// 用户昵称
    pub panel_title: String,

    /// 大会员提示文案（有效期/到期）
    pub vip_overdue_explain: String,
    /// 电视大会员提示文案（有效期/到期）
    pub tv_overdue_explain: String,
    /// 空字段
    pub account_exception_text: String,
    /// 是否自动续费
    pub is_auto_renew: bool,
    /// 是否自动续费电视大会员
    pub is_tv_auto_renew: bool,
    /// 大会员到期剩余时间（单位为秒）
    pub surplus_seconds: u64,
    /// 持续开通大会员时间（单位为秒）
    pub vip_keep_time: u64,
}

/// 电视会员信息结构体
#[derive(Debug, Clone, Deserialize)]
pub struct TvVipInfo {
    /// 电视大会员类型（0：无，1：月大会员，2：年度及以上大会员）
    #[serde(rename = "type")]
    pub tv_type: u32,
    /// 电视大支付类型（0：未支付，1：已支付）
    pub vip_pay_type: u32,
    /// 电视大会员状态（0：无，1：有）
    pub status: u32,
    /// 电视大会员过期时间（毫秒时间戳）
    pub due_date: u64,
}

/// 钱包信息结构体
#[derive(Debug, Clone, Deserialize)]
pub struct WalletInfo {
    /// 当前B币券
    pub coupon: u64,
    /// 积分
    pub point: u64,
    /// 是否已领取特权
    pub privilege_received: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::vip::params::VipCenterInfoParams;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    const PROFILES: [&str; 3] = ["anonymous", "normal", "vip"];

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/vip/read/center-info/contract.json"
        ))
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/vip/read/center-info/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_vip_center_info_comprehensive() {
        tracing::info!("开始测试大会员中心信息的综合功能");

        let bpi = BpiClient::new().expect("client should build");
        let resp = bpi.vip().center_info(VipCenterInfoParams::new()).await;

        match resp {
            Ok(data) => {
                tracing::info!("开始详细分析大会员中心信息数据结构");

                // 1. 用户账户信息详细检查
                tracing::info!("=== 用户账户信息 ===");
                let Some(account) = &data.user.account else {
                    tracing::info!("未登录状态返回空账号信息，测试通过");
                    return;
                };
                tracing::info!("用户ID: {}", account.mid);
                tracing::info!("用户昵称: {}", account.name);
                tracing::info!("用户性别: {}", account.sex);
                tracing::info!("用户等级: {}", account.rank);
                tracing::info!("用户签名: {}", account.sign);
                tracing::info!(
                    "是否注销: {}",
                    if account.is_deleted == 0 {
                        "正常"
                    } else {
                        "已注销"
                    }
                );
                tracing::info!(
                    "是否转正: {}",
                    if account.is_senior_member == 1 {
                        "正式会员"
                    } else {
                        "未转正"
                    }
                );

                // 2. 会员信息详细检查
                tracing::info!("=== 会员信息 ===");
                let Some(vip) = &data.user.vip else {
                    tracing::info!("未登录状态返回空会员信息，测试通过");
                    return;
                };
                let vip_type_text = match vip.vip_type {
                    0 => "无会员",
                    1 => "月大会员",
                    2 => "年度及以上大会员",
                    _ => "未知类型",
                };
                tracing::info!("会员类型: {} ({})", vip.vip_type, vip_type_text);
                tracing::info!(
                    "会员状态: {}",
                    if vip.vip_status == 1 {
                        "有效"
                    } else {
                        "无效"
                    }
                );

                if vip.vip_due_date > 0 {
                    let due_date = chrono::DateTime::from_timestamp_millis(vip.vip_due_date as i64);
                    if let Some(date) = due_date {
                        tracing::info!("会员到期时间: {}", date.format("%Y-%m-%d %H:%M:%S"));
                    }
                }

                tracing::info!("会员标签主题: {}", vip.label.label_theme);
                tracing::info!("会员标签文案: {}", vip.label.text);
                tracing::info!("昵称颜色: {}", vip.nickname_color);

                // 3. 电视会员信息
                tracing::info!("=== 电视会员信息 ===");
                let Some(tv) = &data.user.tv else {
                    tracing::info!("未登录状态返回空电视会员信息，测试通过");
                    return;
                };
                let tv_type_text = match tv.tv_type {
                    0 => "无电视会员",
                    1 => "月电视会员",
                    2 => "年度及以上电视会员",
                    _ => "未知类型",
                };
                tracing::info!("电视会员类型: {} ({})", tv.tv_type, tv_type_text);
                tracing::info!(
                    "电视会员状态: {}",
                    if tv.status == 1 { "有效" } else { "无效" }
                );

                // 4. 头像框信息
                tracing::info!("=== 头像框信息 ===");
                // let pendant = &data.user.as_ref().avatar_pendant.unwrap();
                // tracing::info!("头像框URL: {}", pendant.image);
                // tracing::info!("动态头像框URL: {}", pendant.image_enhance);

                // 5. 续费和通知信息
                tracing::info!("=== 续费和通知信息 ===");
                tracing::info!("自动续费状态: {}", data.user.is_auto_renew);
                tracing::info!("电视会员自动续费: {}", data.user.is_tv_auto_renew);
                tracing::info!("大会员提示: {}", data.user.vip_overdue_explain);
                tracing::info!("电视会员提示: {}", data.user.tv_overdue_explain);

                // 6. 钱包详细信息
                tracing::info!("=== 钱包信息 ===");
                let wallet = &data.wallet;
                tracing::info!("B币券: {}", wallet.coupon);
                tracing::info!("积分: {}", wallet.point);
                tracing::info!("特权已领取: {}", wallet.privilege_received);

                // 验证数据合理性
                assert!(account.mid > 0, "用户mid应该大于0");
                assert!(!account.name.is_empty(), "用户昵称不应为空");

                tracing::info!("大会员中心信息综合测试通过");
            }
            Err(e) => {
                if let BpiError::Api { code: -101, .. } = e {
                    tracing::info!("账号未登录，无法获取详细信息，测试通过");
                } else {
                    tracing::error!("综合测试失败: {:?}", e);
                    panic!("综合测试失败");
                }
            }
        }
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_time_calculation() {
        tracing::info!("开始测试时间计算功能");

        let bpi = BpiClient::new().expect("client should build");
        let resp = bpi.vip().center_info(VipCenterInfoParams::new()).await;

        match resp {
            Ok(data) => {
                let user = &data.user;

                // 计算剩余时间
                if user.surplus_seconds > 0 {
                    let total_seconds = user.surplus_seconds;
                    let days = total_seconds / (24 * 3600);
                    let hours = (total_seconds % (24 * 3600)) / 3600;
                    let minutes = (total_seconds % 3600) / 60;
                    let seconds = total_seconds % 60;

                    tracing::info!(
                        "大会员剩余时间详细: {}天{}小时{}分钟{}秒",
                        days,
                        hours,
                        minutes,
                        seconds
                    );

                    if days > 0 {
                        tracing::info!("剩余时间充足（超过1天）");
                    } else if hours > 0 {
                        tracing::info!("剩余时间不足1天但超过1小时");
                    } else {
                        tracing::info!("剩余时间不足1小时，即将到期");
                    }
                } else {
                    tracing::info!("没有大会员或已过期");
                }

                // 计算持续开通时间
                if user.vip_keep_time > 0 {
                    let keep_days = user.vip_keep_time / (24 * 3600);
                    let keep_years = keep_days / 365;
                    let keep_months = (keep_days % 365) / 30;
                    let remaining_days = keep_days % 30;

                    tracing::info!(
                        "持续开通大会员时间: {}年{}个月{}天",
                        keep_years,
                        keep_months,
                        remaining_days
                    );

                    if keep_years > 0 {
                        tracing::info!("长期忠实用户（超过1年）");
                    } else if keep_days > 30 {
                        tracing::info!("短期用户（超过1个月）");
                    } else {
                        tracing::info!("新用户（少于1个月）");
                    }
                } else {
                    tracing::info!("没有持续开通记录");
                }

                // 分析会员到期时间
                let Some(vip) = &user.vip else {
                    tracing::info!("未登录状态返回空会员信息，跳过到期时间分析");
                    return;
                };
                if vip.vip_due_date > 0
                    && let Some(due_date) =
                        chrono::DateTime::from_timestamp_millis(vip.vip_due_date as i64)
                {
                    let now = chrono::Utc::now();
                    let duration = due_date.signed_duration_since(now);

                    if duration.num_days() > 0 {
                        tracing::info!("会员还有{}天到期", duration.num_days());
                    } else if duration.num_hours() > 0 {
                        tracing::info!("会员还有{}小时到期", duration.num_hours());
                    } else if duration.num_minutes() > 0 {
                        tracing::info!("会员还有{}分钟到期", duration.num_minutes());
                    } else if duration.num_seconds() > 0 {
                        tracing::info!("会员还有{}秒到期", duration.num_seconds());
                    } else {
                        tracing::info!("会员已过期");
                    }

                    tracing::info!("会员到期时间: {}", due_date.format("%Y-%m-%d %H:%M:%S UTC"));
                }

                tracing::info!("时间计算测试通过");
            }
            Err(e) => {
                if let BpiError::Api { code: -101, .. } = e {
                    tracing::info!("账号未登录，无法进行时间计算测试");
                } else {
                    tracing::error!("时间计算测试失败: {:?}", e);
                    panic!("时间计算测试失败");
                }
            }
        }
    }

    #[test]
    fn vip_center_info_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "vip.center_info");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), VIP_CENTER_INFO_ENDPOINT);
        assert_eq!(
            contract.request.query.get("build").map(String::as_str),
            Some("0")
        );
        assert_eq!(contract.cases.len(), 3);

        for case in &contract.cases {
            assert_eq!(case.response.http_status, Some(200));
            assert_eq!(case.response.api_code, Some(0));
            assert_eq!(case.response.rust_model.as_deref(), Some("VipCenterData"));
            if case.name == "anonymous" {
                assert!(!case.auth.requires_cookie());
            } else {
                assert!(case.auth.requires_cookie());
            }
        }

        Ok(())
    }

    #[test]
    fn vip_center_info_response_fixtures_parse_declared_model() -> BpiResult<()> {
        let anonymous = ApiEnvelope::<VipCenterData>::from_slice(include_bytes!(
            "../../tests/contracts/vip/read/center-info/responses/anonymous.success.json"
        ))?
        .into_payload()?;
        assert!(anonymous.user.account.is_none());
        assert!(anonymous.user.vip.is_none());
        assert!(anonymous.user.tv.is_none());

        let normal = ApiEnvelope::<VipCenterData>::from_slice(include_bytes!(
            "../../tests/contracts/vip/read/center-info/responses/normal.success.json"
        ))?
        .into_payload()?;
        let normal_account = normal.user.account.expect("normal account should exist");
        assert_eq!(normal_account.birthday, -1);
        assert_eq!(
            normal
                .user
                .vip
                .expect("normal vip info should exist")
                .vip_status,
            0
        );

        let vip = ApiEnvelope::<VipCenterData>::from_slice(include_bytes!(
            "../../tests/contracts/vip/read/center-info/responses/vip.success.json"
        ))?
        .into_payload()?;
        assert_eq!(vip.user.vip.expect("vip info should exist").vip_status, 1);
        Ok(())
    }

    #[test]
    fn vip_center_info_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in PROFILES {
            if let Some(body) = local_probe_body(profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<VipCenterData>>(body)?.into_payload()?;
                if profile == "anonymous" {
                    assert!(payload.user.account.is_none());
                    assert!(payload.user.vip.is_none());
                    assert!(payload.user.tv.is_none());
                } else {
                    assert!(payload.user.account.is_some());
                    assert!(payload.user.vip.is_some());
                    assert!(payload.user.tv.is_some());
                }
            }
        }

        Ok(())
    }
}
