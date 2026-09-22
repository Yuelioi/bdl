use crate::request::BilibiliRequest;
use crate::{BpiClient, BpiResult};

use super::{ClientInfoIpParams, IpInfo};

const IP_ENDPOINT: &str = "https://api.live.bilibili.com/ip_service/v1/ip_service/get_ip_addr";

/// 客户端信息 API 客户端。
#[derive(Clone, Copy)]
pub struct ClientInfoClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> ClientInfoClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn ip_endpoint(&self) -> &'static str {
        IP_ENDPOINT
    }

    /// 查询 IP 地理位置信息。
    pub async fn ip(&self, params: ClientInfoIpParams) -> BpiResult<IpInfo> {
        self.client
            .get(IP_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("clientinfo.ip")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiResult};

    use super::*;

    fn assert_ip_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<IpInfo>>,
    {
    }

    #[test]
    fn clientinfo_client_exposes_ip_endpoint() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let clientinfo = client.clientinfo();

        assert_eq!(
            clientinfo.ip_endpoint(),
            "https://api.live.bilibili.com/ip_service/v1/ip_service/get_ip_addr"
        );
        Ok(())
    }

    #[test]
    fn clientinfo_ip_returns_payload_future() -> BpiResult<()> {
        let client = BpiClient::new()?;

        assert_ip_future(client.clientinfo().ip(ClientInfoIpParams::new()));
        Ok(())
    }

    #[test]
    fn clientinfo_ip_contract_matches_module_client_endpoint() -> BpiResult<()> {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/clientinfo/ip/contract.json"
        ))?;

        assert_eq!(contract.name, "clientinfo.ip");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), IP_ENDPOINT);
        assert_eq!(
            contract.request.query.get("ip").map(String::as_str),
            Some("8.8.8.8")
        );
        Ok(())
    }
}
