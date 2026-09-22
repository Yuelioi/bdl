use crate::vip::center::VipCenterData;
use crate::vip::params::VipCenterInfoParams;
use crate::{BilibiliRequest, BpiClient, BpiResult};

const CENTER_INFO_ENDPOINT: &str = "https://api.bilibili.com/x/vip/web/vip_center/combine";

/// VIP API 客户端。
#[derive(Clone, Copy)]
pub struct VipClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> VipClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn center_info_endpoint(&self) -> &'static str {
        CENTER_INFO_ENDPOINT
    }

    /// 获取 VIP 中心信息。
    pub async fn center_info(&self, params: VipCenterInfoParams) -> BpiResult<VipCenterData> {
        self.client
            .get(CENTER_INFO_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("vip.center_info")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::vip::center::VipCenterData;
    use crate::vip::params::VipCenterInfoParams;
    use crate::{BpiClient, BpiResult};

    fn assert_center_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<VipCenterData>>,
    {
    }

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/vip/read/center-info/contract.json"
        ))
    }

    #[test]
    fn vip_client_exposes_promoted_endpoint_url() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let vip = client.vip();

        assert_eq!(
            vip.center_info_endpoint(),
            "https://api.bilibili.com/x/vip/web/vip_center/combine"
        );
        Ok(())
    }

    #[test]
    fn vip_methods_return_payload_futures() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let vip = client.vip();

        assert_center_info_future(vip.center_info(VipCenterInfoParams::new()));
        Ok(())
    }

    #[test]
    fn vip_contract_matches_module_client_endpoint() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let vip = client.vip();
        let contract = contract()?;

        assert_eq!(contract.name, "vip.center_info");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), vip.center_info_endpoint());
        assert_eq!(
            contract.request.query.get("build").map(String::as_str),
            Some("0")
        );
        Ok(())
    }
}
