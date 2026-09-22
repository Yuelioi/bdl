use crate::wallet::{UserWallet, WalletInfoParams};
use crate::{BilibiliRequest, BpiClient, BpiResult};

const INFO_ENDPOINT: &str = "https://pay.bilibili.com/paywallet/wallet/getUserWallet";

/// 钱包 API 客户端。
#[derive(Clone, Copy)]
pub struct WalletClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> WalletClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn info_endpoint(&self) -> &'static str {
        INFO_ENDPOINT
    }

    /// 获取已认证钱包信息。
    pub async fn info(&self, params: WalletInfoParams) -> BpiResult<UserWallet> {
        let csrf = self.client.csrf()?;
        let body = params.body(&csrf);

        self.client
            .post(INFO_ENDPOINT)
            .json(&body)
            .send_bpi_payload("wallet.info")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::wallet::{UserWallet, WalletInfoParams};
    use crate::{BpiClient, BpiResult};

    fn assert_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<UserWallet>>,
    {
    }

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/wallet/read/info/contract.json"
        ))
    }

    #[test]
    fn wallet_client_exposes_promoted_endpoint_url() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let wallet = client.wallet();

        assert_eq!(
            wallet.info_endpoint(),
            "https://pay.bilibili.com/paywallet/wallet/getUserWallet"
        );
        Ok(())
    }

    #[test]
    fn wallet_info_returns_payload_future() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let wallet = client.wallet();

        assert_info_future(wallet.info(WalletInfoParams::at_timestamp(1_700_000_000_000)));
        Ok(())
    }

    #[test]
    fn wallet_contract_matches_module_client_endpoint() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let wallet = client.wallet();
        let contract = contract()?;

        assert_eq!(contract.name, "wallet.info");
        assert_eq!(contract.request.method, HttpMethod::Post);
        assert_eq!(contract.request.url.as_str(), wallet.info_endpoint());
        assert!(contract.request.query.is_empty());
        assert_eq!(contract.cases.len(), 2);

        let body =
            contract.request.body.as_ref().ok_or_else(|| {
                crate::BpiError::unsupported_response("missing wallet contract body")
            })?;
        assert_eq!(body["csrf"], "${csrf}");
        assert_eq!(body["platformType"], 3);
        assert_eq!(body["timestamp"], 1_700_000_000_000_i64);
        assert_eq!(body["traceId"], 1_700_000_000_000_i64);
        assert_eq!(body["version"], "1.0");
        Ok(())
    }
}
