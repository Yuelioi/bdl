//! 漫画赛季
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/manga/Season.md)

use crate::response::BpiResponse;
use serde::{Deserialize, Serialize};

// ================= 数据结构 =================

/// 赛季任务信息
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct SeasonTask {
    // 任务相关字段
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    // 其他字段根据实际需要添加
}

/// 赛季奖励信息
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct SeasonWelfare {
    // 奖励相关字段
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: String,
    // 其他字段根据实际需要添加
}

/// 赛季文案信息
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct SeasonText {
    // 文案相关字段
    #[serde(default)]
    pub title: String,
    // 其他字段根据实际需要添加
}

/// 赛季排名信息
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct SeasonRank {
    // 排名相关字段
    // 根据实际需要添加
}

/// 赛季信息数据
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct SeasonInfoData {
    /// 当前时间字符串，ISO 8601格式
    pub current_time: String,
    /// 赛季开始时间，ISO 8601格式
    pub start_time: String,
    /// 赛季结束时间，ISO 8601格式
    pub end_time: String,
    /// 拥有积分，未登录为0
    pub remain_amount: i32,
    /// 第几个赛季
    pub season_id: String,
    /// 待领取奖励的任务，未登录/没有可领取时为空数组
    pub tasks: Vec<SeasonTask>,
    /// 赛季奖励
    pub welfare: Vec<SeasonWelfare>,
    /// 版头图片
    pub cover: String,
    /// 今日的任务完成情况
    pub today_tasks: Vec<SeasonTask>,
    /// 赛季相关文案，未登录为null
    #[serde(default)]
    pub text: Option<SeasonText>,
    /// 赛季标题
    pub season_title: String,
    /// 排名信息
    #[serde(default)]
    pub rank: Option<SeasonRank>,
    // 其他字段根据实际需要添加
}

pub type SeasonInfoResponse = BpiResponse<SeasonInfoData>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/manga/read-core/season-info/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_manga_season_info() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");

        let data = bpi.manga().season_info().await?;

        // 不需要登录也可以获取基本信息

        tracing::info!("{:#?}", data);

        Ok(())
    }

    #[test]
    fn manga_season_info_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "manga.season_info");
        assert_eq!(contract.request.method, HttpMethod::Post);
        assert_eq!(
            contract.request.url.as_str(),
            "https://manga.bilibili.com/twirp/user.v1.Season/GetSeasonInfo"
        );
        assert!(contract.request.query.is_empty());
        assert_eq!(contract.cases.len(), 3);
        for case in &contract.cases {
            assert_eq!(case.response.api_code, Some(0));
            assert_eq!(case.response.rust_model.as_deref(), Some("SeasonInfoData"));
        }
        Ok(())
    }

    #[test]
    fn manga_season_info_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<SeasonInfoData>::from_slice(include_bytes!(
            "../../tests/contracts/manga/read-core/season-info/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.season_id, "0");
        assert!(!payload.current_time.is_empty());
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/manga/read-core/season-info/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn manga_season_info_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<SeasonInfoData>>(body)?.into_payload()?;

            assert!(!payload.season_id.is_empty());
            assert!(!payload.current_time.is_empty());
        }
        Ok(())
    }
}
