use crate::web_widget::{
    HeaderData, OnlineData, RegionBannerData, WebWidgetHeaderPageParams,
    WebWidgetRegionBannerParams,
};
use crate::{BilibiliRequest, BpiClient, BpiResult};

const REGION_BANNER_ENDPOINT: &str = "https://api.bilibili.com/x/web-show/region/banner";
const HEADER_PAGE_ENDPOINT: &str = "https://api.bilibili.com/x/web-show/page/header";
const ONLINE_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/online";

/// Web 组件 API 客户端。
#[derive(Clone, Copy)]
pub struct WebWidgetClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> WebWidgetClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn region_banner_endpoint(&self) -> &'static str {
        REGION_BANNER_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn header_page_endpoint(&self) -> &'static str {
        HEADER_PAGE_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn online_endpoint(&self) -> &'static str {
        ONLINE_ENDPOINT
    }

    /// 获取分区轮播 banner 数据。
    pub async fn region_banner(
        &self,
        params: WebWidgetRegionBannerParams,
    ) -> BpiResult<RegionBannerData> {
        self.client
            .get(REGION_BANNER_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("web_widget.region_banner")
            .await
    }

    /// 获取首页头图数据。
    pub async fn header_page(&self, params: WebWidgetHeaderPageParams) -> BpiResult<HeaderData> {
        let mut header = self
            .client
            .get(HEADER_PAGE_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload::<HeaderData>("web_widget.header_page")
            .await?;
        header.parse_split_layer()?;
        Ok(header)
    }

    /// 获取当前按分区统计的在线投稿数。
    pub async fn online(&self) -> BpiResult<OnlineData> {
        self.client
            .get(ONLINE_ENDPOINT)
            .send_bpi_payload("web_widget.online")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::video::video_zone_v2::{Douga, VideoPartitionV2};
    use crate::{BpiClient, BpiResult};

    use super::*;

    fn assert_region_banner_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<RegionBannerData>>,
    {
    }

    fn assert_header_page_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<HeaderData>>,
    {
    }

    fn assert_online_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<OnlineData>>,
    {
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "region-banner" => {
                include_bytes!("../../tests/contracts/web_widget/region-banner/contract.json")
                    .as_slice()
            }
            "header-page" => {
                include_bytes!("../../tests/contracts/web_widget/header-page/contract.json")
                    .as_slice()
            }
            "online" => {
                include_bytes!("../../tests/contracts/web_widget/online/contract.json").as_slice()
            }
            _ => unreachable!("unknown web_widget endpoint fixture"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn web_widget_client_exposes_promoted_endpoint_urls() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let web_widget = client.web_widget();

        assert_eq!(
            web_widget.region_banner_endpoint(),
            "https://api.bilibili.com/x/web-show/region/banner"
        );
        assert_eq!(
            web_widget.header_page_endpoint(),
            "https://api.bilibili.com/x/web-show/page/header"
        );
        assert_eq!(
            web_widget.online_endpoint(),
            "https://api.bilibili.com/x/web-interface/online"
        );
        Ok(())
    }

    #[test]
    fn web_widget_methods_return_payload_futures() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let web_widget = client.web_widget();

        assert_region_banner_future(web_widget.region_banner(WebWidgetRegionBannerParams::new(
            VideoPartitionV2::Douga(Douga::Douga),
        )));
        assert_header_page_future(web_widget.header_page(WebWidgetHeaderPageParams::new()));
        assert_online_future(web_widget.online());
        Ok(())
    }

    #[test]
    fn web_widget_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let web_widget = client.web_widget();
        let region_banner = contract("region-banner")?;
        let header_page = contract("header-page")?;
        let online = contract("online")?;

        assert_eq!(region_banner.name, "web_widget.region_banner");
        assert_eq!(region_banner.request.method, HttpMethod::Get);
        assert_eq!(
            region_banner.request.url.as_str(),
            web_widget.region_banner_endpoint()
        );
        assert_eq!(
            region_banner
                .request
                .query
                .get("region_id")
                .map(String::as_str),
            Some("1005")
        );

        assert_eq!(header_page.name, "web_widget.header_page");
        assert_eq!(header_page.request.method, HttpMethod::Get);
        assert_eq!(
            header_page.request.url.as_str(),
            web_widget.header_page_endpoint()
        );
        assert_eq!(
            header_page
                .request
                .query
                .get("resource_id")
                .map(String::as_str),
            Some("142")
        );

        assert_eq!(online.name, "web_widget.online");
        assert_eq!(online.request.method, HttpMethod::Get);
        assert_eq!(online.request.url.as_str(), web_widget.online_endpoint());
        assert!(online.request.query.is_empty());
        Ok(())
    }
}
