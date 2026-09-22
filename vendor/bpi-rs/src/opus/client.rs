use crate::opus::{OpusSpaceFeedParams, SpaceData};
use crate::{BilibiliRequest, BpiClient, BpiResult};

const SPACE_FEED_ENDPOINT: &str =
    "https://api.bilibili.com/x/polymer/web-dynamic/v1/opus/feed/space";

/// Opus API 客户端。
#[derive(Clone, Copy)]
pub struct OpusClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> OpusClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn space_feed_endpoint(&self) -> &'static str {
        SPACE_FEED_ENDPOINT
    }

    /// 获取用户空间 feed 中的 opus 条目。
    pub async fn space_feed(&self, params: OpusSpaceFeedParams) -> BpiResult<SpaceData> {
        self.client
            .get(SPACE_FEED_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("opus.space_feed")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::ids::Mid;
    use crate::opus::{OpusSpaceFeedParams, SpaceData};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiResult};

    fn assert_space_feed_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<SpaceData>>,
    {
    }

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/opus/space-read/space-feed/contract.json"
        ))
    }

    #[test]
    fn opus_client_exposes_promoted_endpoint_url() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let opus = client.opus();

        assert_eq!(
            opus.space_feed_endpoint(),
            "https://api.bilibili.com/x/polymer/web-dynamic/v1/opus/feed/space"
        );
        Ok(())
    }

    #[test]
    fn opus_space_feed_returns_payload_future() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let opus = client.opus();

        assert_space_feed_future(opus.space_feed(OpusSpaceFeedParams::new(Mid::new(1_000_001)?)));
        Ok(())
    }

    #[test]
    fn opus_contract_matches_module_client_endpoint() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let opus = client.opus();
        let contract = contract()?;

        assert_eq!(contract.name, "opus.space_feed");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), opus.space_feed_endpoint());
        assert_eq!(
            contract.request.query.get("host_mid").map(String::as_str),
            Some("1000001")
        );
        assert_eq!(
            contract.request.query.get("page").map(String::as_str),
            Some("0")
        );
        assert_eq!(
            contract.request.query.get("type").map(String::as_str),
            Some("all")
        );
        assert_eq!(
            contract
                .request
                .query
                .get("web_location")
                .map(String::as_str),
            Some("333.1387")
        );
        Ok(())
    }
}
