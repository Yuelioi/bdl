use crate::activity::info::{ActivityInfoData, ActivityInfoParams};
use crate::activity::list::{ActivityListData, ActivityListParams};
use crate::{BilibiliRequest, BpiClient, BpiResult};

const INFO_ENDPOINT: &str = "https://api.bilibili.com/x/activity/subject/info";
const LIST_ENDPOINT: &str = "https://api.bilibili.com/x/activity/page/list";

/// 活动 API 客户端。
#[derive(Clone, Copy)]
pub struct ActivityClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> ActivityClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn info_endpoint(&self) -> &'static str {
        INFO_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn list_endpoint(&self) -> &'static str {
        LIST_ENDPOINT
    }

    /// 获取活动主题信息。
    pub async fn info(&self, params: ActivityInfoParams) -> BpiResult<ActivityInfoData> {
        self.client
            .get(INFO_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("activity.info")
            .await
    }

    /// 获取活动列表。
    pub async fn list(&self, params: ActivityListParams) -> BpiResult<ActivityListData> {
        self.client
            .get(LIST_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("activity.list")
            .await
    }

    /// 使用 Bilibili Web 默认值获取活动列表。
    pub async fn list_default(&self) -> BpiResult<ActivityListData> {
        self.list(ActivityListParams::default()).await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::activity::info::{ActivityInfoData, ActivityInfoParams};
    use crate::activity::list::{ActivityListData, ActivityListParams};
    use crate::ids::Bvid;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiResult};

    fn assert_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ActivityInfoData>>,
    {
    }

    fn assert_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ActivityListData>>,
    {
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "info" => {
                include_bytes!("../../tests/contracts/activity/info/contract.json").as_slice()
            }
            "list" => {
                include_bytes!("../../tests/contracts/activity/list/contract.json").as_slice()
            }
            _ => unreachable!("unknown activity endpoint fixture"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn activity_client_exposes_promoted_endpoint_urls() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let activity = client.activity();

        assert_eq!(
            activity.info_endpoint(),
            "https://api.bilibili.com/x/activity/subject/info"
        );
        assert_eq!(
            activity.list_endpoint(),
            "https://api.bilibili.com/x/activity/page/list"
        );
        Ok(())
    }

    #[test]
    fn activity_methods_return_payload_futures() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let activity = client.activity();
        let bvid: Bvid = "BV1mKY4e8ELy".parse()?;

        assert_info_future(activity.info(ActivityInfoParams::new(4_017_552)?.with_bvid(bvid)));
        assert_list_future(activity.list(ActivityListParams::new().page_size(1)?));
        assert_list_future(activity.list_default());
        Ok(())
    }

    #[test]
    fn activity_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let activity = client.activity();
        let info = contract("info")?;
        let list = contract("list")?;

        assert_eq!(info.name, "activity.info");
        assert_eq!(info.request.method, HttpMethod::Get);
        assert_eq!(info.request.url.as_str(), activity.info_endpoint());
        assert_eq!(
            info.request.query.get("sid").map(String::as_str),
            Some("4017552")
        );
        assert_eq!(
            info.request.query.get("bvid").map(String::as_str),
            Some("BV1mKY4e8ELy")
        );

        assert_eq!(list.name, "activity.list");
        assert_eq!(list.request.method, HttpMethod::Get);
        assert_eq!(list.request.url.as_str(), activity.list_endpoint());
        assert_eq!(
            list.request.query.get("plat").map(String::as_str),
            Some("1,3")
        );
        assert_eq!(list.request.query.get("ps").map(String::as_str), Some("1"));
        Ok(())
    }
}
