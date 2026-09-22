//! B站分区轮播图相关接口
//!
//! [查看 API 文档](https://socialsisteryi.github.io/bilibili-API-collect/docs/web_widget/banner.html)
use serde::{Deserialize, Serialize};

/// 轮播图对象
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegionBanner {
    pub image: String,     // 封面资源路径
    pub title: String,     // 封面标题
    pub sub_title: String, // 封面子标题
    pub url: String,       // 点击后的跳转链接
    pub rid: i64,          // 分区 ID
}

/// 轮播图响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegionBannerData {
    pub region_banner_list: Vec<RegionBanner>, // 轮播图列表
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::video::video_zone_v2::{Douga, VideoPartitionV2};
    use crate::web_widget::params::WebWidgetRegionBannerParams;
    use crate::{ApiEnvelope, BpiClient, BpiResult};

    use tracing::info;

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/web_widget/region-banner/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_region_banner() {
        let bpi = BpiClient::new().expect("client should build");
        // 例如 region_id = 1 (动画)
        let resp = bpi
            .web_widget()
            .region_banner(WebWidgetRegionBannerParams::new(VideoPartitionV2::Douga(
                Douga::Douga,
            )))
            .await;
        info!("响应: {:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("分区轮播图: {:?}", data);
    }

    #[test]
    fn web_widget_region_banner_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let partition = VideoPartitionV2::Douga(Douga::Douga);

        assert_eq!(contract.name, "web_widget.region_banner");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/web-show/region/banner"
        );
        assert_eq!(
            contract.request.query.get("region_id").map(String::as_str),
            Some(partition.tid().to_string().as_str())
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("RegionBannerData")
        );
        Ok(())
    }

    #[test]
    fn web_widget_region_banner_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/web_widget/region-banner/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/web_widget/region-banner/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/web_widget/region-banner/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<RegionBannerData>::from_slice(bytes)?.into_payload()?;

            assert!(!payload.region_banner_list.is_empty());
        }
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!(
            "target/bpi-probe-runs/web_widget/public/region-banner/{profile}.response.json"
        );
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn web_widget_region_banner_model_matches_local_probe_outputs_when_available() -> BpiResult<()>
    {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<RegionBannerData>>(body)?.into_payload()?;

            assert!(!payload.region_banner_list.is_empty());
        }
        Ok(())
    }
}
