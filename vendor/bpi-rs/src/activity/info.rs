//! 活动主题信息
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/activity/info.md)
use crate::ids::Bvid;
use crate::{BpiError, BpiResult};
use serde::{Deserialize, Serialize};

/// 活动主题信息数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityInfoData {
    /// 活动 id
    pub id: u64,
    /// 开始时间 UNIX 秒级时间戳
    pub stime: i64,
    /// 结束时间 UNIX 秒级时间戳
    pub etime: i64,
    /// 创建时间 UNIX 秒级时间戳
    pub ctime: i64,
    /// 修改时间 UNIX 秒级时间戳
    pub mtime: i64,
    /// 活动名称
    pub name: String,
    /// 活动链接
    pub act_url: String,
    /// 封面图片
    pub cover: String,
    /// 简介
    pub dic: String,
    /// H5 封面
    pub h5_cover: String,
    /// Android 端活动链接
    pub android_url: String,
    /// iOS 端活动链接
    pub ios_url: String,
    /// 子活动 id?
    pub child_sids: String,
    /// 仅在传入 bvid 时存在
    pub lid: Option<i64>,
}

/// 获取活动主题信息的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityInfoParams {
    sid: u64,
    bvid: Option<Bvid>,
}

impl ActivityInfoParams {
    /// 使用非零活动 ID 创建活动主题参数。
    pub fn new(sid: u64) -> BpiResult<Self> {
        if sid == 0 {
            return Err(BpiError::invalid_parameter("sid", "sid must be non-zero"));
        }

        Ok(Self { sid, bvid: None })
    }

    /// 设置可选来源视频 ID。
    pub fn with_bvid(mut self, bvid: Bvid) -> Self {
        self.bvid = Some(bvid);
        self
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![("sid", self.sid.to_string())];

        if let Some(bvid) = self.bvid.as_ref() {
            params.push(("bvid", bvid.to_string()));
        }

        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiResult};

    const TEST_ACTIVITY_ID: u64 = 4_017_552;
    const TEST_ACTIVITY_BVID: &str = "BV1mKY4e8ELy";

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/activity/info/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_activity_info() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let params = ActivityInfoParams::new(4017552)?
            .with_bvid("BV1mKY4e8ELy".parse().expect("bvid should be valid"));

        let data = bpi.activity().info(params).await?;
        tracing::info!("{:#?}", data);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_activity_info_without_bvid() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let sid = 4017552;
        let params = ActivityInfoParams::new(sid)?;

        let data = bpi.activity().info(params).await?;
        tracing::info!("{:#?}", data);

        assert_eq!(data.id, sid);

        Ok(())
    }

    #[test]
    fn activity_info_params_serializes_required_query() -> Result<(), BpiError> {
        let params = ActivityInfoParams::new(4017552)?;

        assert_eq!(params.query_pairs(), vec![("sid", "4017552".to_string())]);
        Ok(())
    }

    #[test]
    fn activity_info_params_serializes_bvid_query() -> Result<(), BpiError> {
        let params = ActivityInfoParams::new(4017552)?.with_bvid("BV1mKY4e8ELy".parse()?);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("sid", "4017552".to_string()),
                ("bvid", "BV1mKY4e8ELy".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn activity_info_params_rejects_zero_sid() {
        let err = ActivityInfoParams::new(0).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "sid", .. }
        ));
    }

    #[test]
    fn activity_info_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "activity.info");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/activity/subject/info"
        );
        assert_eq!(
            contract.request.query.get("sid").map(String::as_str),
            Some("4017552")
        );
        assert_eq!(
            contract.request.query.get("bvid").map(String::as_str),
            Some(TEST_ACTIVITY_BVID)
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("ActivityInfoData")
        );
        Ok(())
    }

    #[test]
    fn activity_info_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!("../../tests/contracts/activity/info/responses/anonymous.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/activity/info/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/activity/info/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<ActivityInfoData>::from_slice(bytes)?.into_payload()?;

            assert_eq!(payload.id, TEST_ACTIVITY_ID);
            assert!(payload.lid.is_some());
        }
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/activity/public/info/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn activity_info_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<ActivityInfoData>>(body)?.into_payload()?;

            assert_eq!(payload.id, TEST_ACTIVITY_ID);
            assert!(payload.lid.is_some());
        }
        Ok(())
    }
}
