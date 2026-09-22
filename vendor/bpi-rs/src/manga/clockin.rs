// 签到
//
// [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/manga/ClockIn.md)

// ================= 数据结构 =================

use crate::BpiError;
use crate::manga::MangaClient;
use crate::request::send_bpi_envelope;
use crate::response::BpiResponse;
use serde::{Deserialize, Serialize};

/// 补签请求参数

#[derive(Debug, Clone, Serialize)]
pub struct ClockInMakeupRequest {
    /// 补签类型
    pub r#type: i32,
    /// 补签日期，格式：YYYY-MM-DD
    pub date: String,
}

/// 签到状态信息中的积分信息
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct PointInfo {
    /// 签到可获取积分
    pub point: i32,
    /// 原始积分
    pub origin_point: i32,
    /// 是否为活动
    pub is_activity: bool,
    /// 签到奖励描述
    pub title: String,
}

/// 签到状态信息
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ClockInInfoData {
    /// 连续签到天数
    pub day_count: i32,
    /// 今日是否已签到，0：未签到，1：已签到
    pub status: i32,
    /// 一次签到周期中每次签到可获得点数
    pub points: Vec<i32>,
    /// 积分图标
    pub credit_icon: String,
    /// 签到前图标
    pub sign_before_icon: String,
    /// 今日签到图标
    pub sign_today_icon: String,
    /// 呼吸图标
    pub breathe_icon: String,
    /// 新积分图标
    #[serde(default)]
    pub new_credit_x_icon: String,
    /// 优惠券图片
    #[serde(default)]
    pub coupon_pic: String,
    /// 积分信息
    pub point_infos: Vec<PointInfo>,
}

pub type ClockInInfoResponse = BpiResponse<ClockInInfoData>;

// ================= 实现 =================

impl<'a> MangaClient<'a> {
    /// 漫画签到
    ///
    /// # 文档
    /// [查看API文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/manga)
    pub async fn manga_clock_in(&self) -> Result<BpiResponse<serde_json::Value>, BpiError> {
        let params = [("platform", "android")];
        let request = self
            .client
            .post("https://manga.bilibili.com/twirp/activity.v1.Activity/ClockIn")
            .form(&params);

        send_bpi_envelope(request, "漫画签到").await
    }

    /// 漫画补签
    ///
    /// # 文档
    /// [查看API文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/manga)
    ///
    /// # 参数
    ///
    /// | 名称 | 类型 | 说明 |
    /// | ---- | ---- | ---- |
    /// | `date` | &str | 补签日期，YYYY-MM-DD |
    pub async fn manga_clock_in_makeup(
        &self,
        date: &str,
    ) -> Result<BpiResponse<serde_json::Value>, BpiError> {
        let params = ClockInMakeupRequest {
            r#type: 0,
            date: date.to_string(),
        };
        let request = self
            .client
            .post("https://manga.bilibili.com/twirp/activity.v1.Activity/ClockIn?platform=android")
            .json(&params);

        send_bpi_envelope(request, "漫画补签").await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/manga/read-core/clock-in-info/contract.json"
        ))
    }

    #[test]
    fn manga_clock_in_info_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "manga.clock_in_info");
        assert_eq!(contract.request.method, HttpMethod::Post);
        assert_eq!(
            contract.request.url.as_str(),
            "https://manga.bilibili.com/twirp/activity.v1.Activity/GetClockInInfo"
        );
        assert!(contract.request.query.is_empty());
        assert_eq!(contract.cases.len(), 3);
        for case in &contract.cases {
            assert_eq!(case.response.api_code, Some(0));
            assert_eq!(case.response.rust_model.as_deref(), Some("ClockInInfoData"));
        }
        Ok(())
    }

    #[test]
    fn manga_clock_in_info_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<ClockInInfoData>::from_slice(include_bytes!(
            "../../tests/contracts/manga/read-core/clock-in-info/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.points.len(), 7);
        assert_eq!(payload.point_infos[0].point, 10);
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/manga/read-core/clock-in-info/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn manga_clock_in_info_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<ClockInInfoData>>(body)?.into_payload()?;

            assert_eq!(payload.points.len(), 7);
            assert!(!payload.point_infos.is_empty());
        }
        Ok(())
    }
}
