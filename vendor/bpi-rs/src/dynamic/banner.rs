use serde::{Deserialize, Serialize};

/// 动态首页公告栏响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicBannerData {
    /// 横幅列表
    pub banners: Vec<DynamicBanner>,
}

/// 动态横幅数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicBanner {
    /// 横幅 ID
    pub banner_id: u64,
    /// 结束时间（UNIX 秒级时间戳）
    pub end_time: u64,
    /// 图片 URL
    pub img_url: String,
    /// 跳转链接
    pub link: String,
    /// 平台
    pub platform: u64,
    /// 位置
    pub position: String,
    /// 开始时间（UNIX 秒级时间戳）
    pub start_time: u64,
    /// 标题
    pub title: String,
    /// 权重
    pub weight: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use tracing::info;

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/dynamic/feed/banner/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_dynamic_feed_banner() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi.dynamic().feed_banner().await?;

        info!("成功获取到 {} 条公告", data.banners.len());
        assert!(!data.banners.is_empty());

        Ok(())
    }

    #[test]
    fn dynamic_feed_banner_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "dynamic.feed_banner");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/dynamic/feed/dyn/banner"
        );
        assert_eq!(
            contract.request.query.get("platform").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            contract.request.query.get("position").map(String::as_str),
            Some("web动态")
        );
        assert_eq!(
            contract
                .request
                .query
                .get("web_location")
                .map(String::as_str),
            Some("333.1365")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("DynamicBannerData")
        );
        Ok(())
    }

    #[test]
    fn dynamic_feed_banner_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/dynamic/feed/banner/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/dynamic/feed/banner/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!("../../tests/contracts/dynamic/feed/banner/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<DynamicBannerData>::from_slice(bytes)?.into_payload()?;
            assert!(!payload.banners.is_empty());
        }
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!(
            "target/bpi-probe-runs/dynamic/feed-readonly/feed-banner/{profile}.response.json"
        );
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn dynamic_feed_banner_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<DynamicBannerData>>(body)?.into_payload()?;
            assert!(!payload.banners.is_empty());
        }
        Ok(())
    }
}
