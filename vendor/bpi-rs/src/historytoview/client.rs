use crate::historytoview::history::HistoryListData;
use crate::historytoview::params::HistoryListParams;
use crate::historytoview::toview::ToViewListData;
use crate::{BilibiliRequest, BpiClient, BpiResult};

const HISTORY_LIST_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/history/cursor";
const HISTORY_SHADOW_ENDPOINT: &str = "https://api.bilibili.com/x/v2/history/shadow";
const TOVIEW_LIST_ENDPOINT: &str = "https://api.bilibili.com/x/v2/history/toview";

/// 历史记录和稍后再看 API 客户端。
#[derive(Clone, Copy)]
pub struct HistoryToViewClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> HistoryToViewClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn history_list_endpoint(&self) -> &'static str {
        HISTORY_LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn history_shadow_endpoint(&self) -> &'static str {
        HISTORY_SHADOW_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn toview_list_endpoint(&self) -> &'static str {
        TOVIEW_LIST_ENDPOINT
    }

    /// 获取账号历史记录。
    pub async fn history_list(&self, params: HistoryListParams) -> BpiResult<HistoryListData> {
        self.client
            .get(HISTORY_LIST_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("historytoview.history_list")
            .await
    }

    /// 获取历史记录是否已禁用。
    pub async fn history_shadow(&self) -> BpiResult<bool> {
        self.client
            .get(HISTORY_SHADOW_ENDPOINT)
            .send_bpi_payload("historytoview.history_shadow")
            .await
    }

    /// 获取账号稍后再看视频。
    pub async fn toview_list(&self) -> BpiResult<ToViewListData> {
        self.client
            .get(TOVIEW_LIST_ENDPOINT)
            .send_bpi_payload("historytoview.toview_list")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::historytoview::history::HistoryListData;
    use crate::historytoview::params::HistoryListParams;
    use crate::historytoview::toview::ToViewListData;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiResult};

    fn history_list_params() -> BpiResult<HistoryListParams> {
        HistoryListParams::new().with_page_size(5)
    }

    fn assert_history_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<HistoryListData>>,
    {
    }

    fn assert_history_shadow_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<bool>>,
    {
    }

    fn assert_toview_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ToViewListData>>,
    {
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "history-list" => include_bytes!(
                "../../tests/contracts/historytoview/read/history-list/contract.json"
            )
            .as_slice(),
            "history-shadow" => include_bytes!(
                "../../tests/contracts/historytoview/read/history-shadow/contract.json"
            )
            .as_slice(),
            "toview-list" => {
                include_bytes!("../../tests/contracts/historytoview/read/toview-list/contract.json")
                    .as_slice()
            }
            _ => unreachable!("unknown historytoview read contract"),
        };
        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn historytoview_client_exposes_promoted_endpoint_urls() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let historytoview = client.historytoview();

        assert_eq!(
            historytoview.history_list_endpoint(),
            "https://api.bilibili.com/x/web-interface/history/cursor"
        );
        assert_eq!(
            historytoview.history_shadow_endpoint(),
            "https://api.bilibili.com/x/v2/history/shadow"
        );
        assert_eq!(
            historytoview.toview_list_endpoint(),
            "https://api.bilibili.com/x/v2/history/toview"
        );
        Ok(())
    }

    #[test]
    fn historytoview_methods_return_payload_futures() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let historytoview = client.historytoview();

        assert_history_list_future(historytoview.history_list(history_list_params()?));
        assert_history_shadow_future(historytoview.history_shadow());
        assert_toview_list_future(historytoview.toview_list());
        Ok(())
    }

    #[test]
    fn historytoview_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let historytoview = client.historytoview();
        let history_list = contract("history-list")?;
        let history_shadow = contract("history-shadow")?;
        let toview_list = contract("toview-list")?;

        assert_eq!(history_list.name, "historytoview.history_list");
        assert_eq!(history_list.request.method, HttpMethod::Get);
        assert_eq!(
            history_list.request.url.as_str(),
            historytoview.history_list_endpoint()
        );
        assert_eq!(
            history_list.request.query.get("ps").map(String::as_str),
            Some("5")
        );

        assert_eq!(history_shadow.name, "historytoview.history_shadow");
        assert_eq!(history_shadow.request.method, HttpMethod::Get);
        assert_eq!(
            history_shadow.request.url.as_str(),
            historytoview.history_shadow_endpoint()
        );
        assert!(history_shadow.request.query.is_empty());

        assert_eq!(toview_list.name, "historytoview.toview_list");
        assert_eq!(toview_list.request.method, HttpMethod::Get);
        assert_eq!(
            toview_list.request.url.as_str(),
            historytoview.toview_list_endpoint()
        );
        assert!(toview_list.request.query.is_empty());
        Ok(())
    }
}
