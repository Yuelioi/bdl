use crate::comment::list::{
    CommentCountParams, CommentHotParams, CommentListData, CommentListParams, CommentRepliesParams,
    CountData, HotCommentData,
};
use crate::{BilibiliRequest, BpiClient, BpiResult};

const LIST_ENDPOINT: &str = "https://api.bilibili.com/x/v2/reply";
const REPLIES_ENDPOINT: &str = "https://api.bilibili.com/x/v2/reply/reply";
const HOT_ENDPOINT: &str = "https://api.bilibili.com/x/v2/reply/hot";
const COUNT_ENDPOINT: &str = "https://api.bilibili.com/x/v2/reply/count";

/// 评论 API 客户端。
#[derive(Clone, Copy)]
pub struct CommentClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> CommentClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn list_endpoint(&self) -> &'static str {
        LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn replies_endpoint(&self) -> &'static str {
        REPLIES_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn hot_endpoint(&self) -> &'static str {
        HOT_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn count_endpoint(&self) -> &'static str {
        COUNT_ENDPOINT
    }

    /// 获取目标评论区的主评论列表。
    pub async fn list(&self, params: CommentListParams) -> BpiResult<CommentListData> {
        self.client
            .get(LIST_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("comment.read.list")
            .await
    }

    /// 获取根评论下的回复。
    pub async fn replies(&self, params: CommentRepliesParams) -> BpiResult<CommentListData> {
        self.client
            .get(REPLIES_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("comment.read.replies")
            .await
    }

    /// 当 API 返回 payload 时，获取根评论下的热评。
    pub async fn hot(&self, params: CommentHotParams) -> BpiResult<Option<HotCommentData>> {
        self.client
            .get(HOT_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_optional_payload("comment.read.hot")
            .await
    }

    /// 获取目标评论区的评论总数。
    pub async fn count(&self, params: CommentCountParams) -> BpiResult<CountData> {
        self.client
            .get(COUNT_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("comment.read.count")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::comment::list::{
        CommentCountParams, CommentHotParams, CommentListData, CommentListParams,
        CommentRepliesParams, CommentSort, CommentTarget, CountData, HotCommentData,
    };
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiResult};

    const TEST_TYPE: i32 = 1;
    const TEST_OID: i64 = 23199;
    const TEST_ROOT_RPID: i64 = 2554491176;

    fn target() -> BpiResult<CommentTarget> {
        CommentTarget::new(TEST_TYPE, TEST_OID)
    }

    fn list_params() -> BpiResult<CommentListParams> {
        Ok(CommentListParams::new(target()?)
            .with_page(1)?
            .with_page_size(5)?
            .with_sort(CommentSort::Time)
            .without_hot(false))
    }

    fn replies_params() -> BpiResult<CommentRepliesParams> {
        CommentRepliesParams::new(target()?, TEST_ROOT_RPID)?
            .with_page(1)?
            .with_page_size(5)
    }

    fn hot_params() -> BpiResult<CommentHotParams> {
        CommentHotParams::new(target()?, TEST_ROOT_RPID)?
            .with_page(1)?
            .with_page_size(5)
    }

    fn assert_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<CommentListData>>,
    {
    }

    fn assert_hot_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<Option<HotCommentData>>>,
    {
    }

    fn assert_count_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<CountData>>,
    {
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "list" => {
                include_bytes!("../../tests/contracts/comment/read/list/contract.json").as_slice()
            }
            "replies" => include_bytes!("../../tests/contracts/comment/read/replies/contract.json")
                .as_slice(),
            "hot" => {
                include_bytes!("../../tests/contracts/comment/read/hot/contract.json").as_slice()
            }
            "count" => {
                include_bytes!("../../tests/contracts/comment/read/count/contract.json").as_slice()
            }
            _ => unreachable!("unknown comment read contract"),
        };
        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn comment_client_exposes_promoted_endpoint_urls() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let comment = client.comment();

        assert_eq!(
            comment.list_endpoint(),
            "https://api.bilibili.com/x/v2/reply"
        );
        assert_eq!(
            comment.replies_endpoint(),
            "https://api.bilibili.com/x/v2/reply/reply"
        );
        assert_eq!(
            comment.hot_endpoint(),
            "https://api.bilibili.com/x/v2/reply/hot"
        );
        assert_eq!(
            comment.count_endpoint(),
            "https://api.bilibili.com/x/v2/reply/count"
        );
        Ok(())
    }

    #[test]
    fn comment_methods_return_payload_futures() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let comment = client.comment();

        assert_list_future(comment.list(list_params()?));
        assert_list_future(comment.replies(replies_params()?));
        assert_hot_future(comment.hot(hot_params()?));
        assert_count_future(comment.count(CommentCountParams::new(target()?)));
        Ok(())
    }

    #[test]
    fn comment_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let comment = client.comment();
        let list = contract("list")?;
        let replies = contract("replies")?;
        let hot = contract("hot")?;
        let count = contract("count")?;

        assert_eq!(list.name, "comment.read.list");
        assert_eq!(list.request.method, HttpMethod::Get);
        assert_eq!(list.request.url.as_str(), comment.list_endpoint());
        assert_eq!(
            list.request.query.get("sort").map(String::as_str),
            Some("0")
        );

        assert_eq!(replies.name, "comment.read.replies");
        assert_eq!(replies.request.method, HttpMethod::Get);
        assert_eq!(replies.request.url.as_str(), comment.replies_endpoint());
        assert_eq!(
            replies.request.query.get("root").map(String::as_str),
            Some("2554491176")
        );

        assert_eq!(hot.name, "comment.read.hot");
        assert_eq!(hot.request.method, HttpMethod::Get);
        assert_eq!(hot.request.url.as_str(), comment.hot_endpoint());
        assert!(
            hot.cases
                .iter()
                .all(|case| case.response.rust_model.is_none())
        );

        assert_eq!(count.name, "comment.read.count");
        assert_eq!(count.request.method, HttpMethod::Get);
        assert_eq!(count.request.url.as_str(), comment.count_endpoint());
        assert_eq!(
            count.request.query.get("oid").map(String::as_str),
            Some("23199")
        );
        Ok(())
    }
}
