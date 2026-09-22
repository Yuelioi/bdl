use crate::message::msg::{ReplyFeedData, UnreadCountData};
use crate::message::params::{
    MessageReplyFeedParams, MessageSingleUnreadParams, MessageUnreadCountParams,
};
use crate::message::private_msg::SingleUnreadData;
use crate::{BilibiliRequest, BpiClient, BpiResult};

const UNREAD_COUNT_ENDPOINT: &str = "https://api.vc.bilibili.com/x/im/web/msgfeed/unread";
const REPLY_FEED_ENDPOINT: &str = "https://api.bilibili.com/x/msgfeed/reply";
const SINGLE_UNREAD_ENDPOINT: &str =
    "https://api.vc.bilibili.com/session_svr/v1/session_svr/single_unread";

/// 消息 API 客户端。
#[derive(Clone, Copy)]
pub struct MessageClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> MessageClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn unread_count_endpoint(&self) -> &'static str {
        UNREAD_COUNT_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn reply_feed_endpoint(&self) -> &'static str {
        REPLY_FEED_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn single_unread_endpoint(&self) -> &'static str {
        SINGLE_UNREAD_ENDPOINT
    }

    /// 获取未读消息计数。
    pub async fn unread_count(
        &self,
        params: MessageUnreadCountParams,
    ) -> BpiResult<UnreadCountData> {
        self.client
            .get(UNREAD_COUNT_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("message.unread_count")
            .await
    }

    /// 获取回复通知流条目。
    pub async fn reply_feed(&self, params: MessageReplyFeedParams) -> BpiResult<ReplyFeedData> {
        self.client
            .get(REPLY_FEED_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("message.reply_feed")
            .await
    }

    /// 获取未读私信计数。
    pub async fn single_unread(
        &self,
        params: MessageSingleUnreadParams,
    ) -> BpiResult<SingleUnreadData> {
        self.client
            .get(SINGLE_UNREAD_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("message.single_unread")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::message::msg::{ReplyFeedData, UnreadCountData};
    use crate::message::params::{
        MessageReplyFeedParams, MessageSingleUnreadParams, MessageUnreadCountParams,
    };
    use crate::message::private_msg::SingleUnreadData;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiResult};

    fn assert_unread_count_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<UnreadCountData>>,
    {
    }

    fn assert_reply_feed_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ReplyFeedData>>,
    {
    }

    fn assert_single_unread_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<SingleUnreadData>>,
    {
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "unread-count" => {
                include_bytes!("../../tests/contracts/message/read/unread-count/contract.json")
                    .as_slice()
            }
            "reply-feed" => {
                include_bytes!("../../tests/contracts/message/read/reply-feed/contract.json")
                    .as_slice()
            }
            "single-unread" => {
                include_bytes!("../../tests/contracts/message/read/single-unread/contract.json")
                    .as_slice()
            }
            _ => unreachable!("unknown message endpoint fixture"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn message_client_exposes_promoted_endpoint_urls() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let message = client.message();

        assert_eq!(
            message.unread_count_endpoint(),
            "https://api.vc.bilibili.com/x/im/web/msgfeed/unread"
        );
        assert_eq!(
            message.reply_feed_endpoint(),
            "https://api.bilibili.com/x/msgfeed/reply"
        );
        assert_eq!(
            message.single_unread_endpoint(),
            "https://api.vc.bilibili.com/session_svr/v1/session_svr/single_unread"
        );
        Ok(())
    }

    #[test]
    fn message_methods_return_payload_futures() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let message = client.message();

        assert_unread_count_future(message.unread_count(MessageUnreadCountParams::new()));
        assert_reply_feed_future(message.reply_feed(MessageReplyFeedParams::new()));
        assert_single_unread_future(message.single_unread(MessageSingleUnreadParams::new()));
        Ok(())
    }

    #[test]
    fn message_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let message = client.message();
        let unread_count = contract("unread-count")?;
        let reply_feed = contract("reply-feed")?;
        let single_unread = contract("single-unread")?;

        assert_eq!(unread_count.name, "message.unread_count");
        assert_eq!(unread_count.request.method, HttpMethod::Get);
        assert_eq!(
            unread_count.request.url.as_str(),
            message.unread_count_endpoint()
        );
        assert_eq!(
            unread_count
                .request
                .query
                .get("mobi_app")
                .map(String::as_str),
            Some("web")
        );

        assert_eq!(reply_feed.name, "message.reply_feed");
        assert_eq!(reply_feed.request.method, HttpMethod::Get);
        assert_eq!(
            reply_feed.request.url.as_str(),
            message.reply_feed_endpoint()
        );
        assert_eq!(
            reply_feed.request.query.get("platform").map(String::as_str),
            Some("web")
        );

        assert_eq!(single_unread.name, "message.single_unread");
        assert_eq!(single_unread.request.method, HttpMethod::Get);
        assert_eq!(
            single_unread.request.url.as_str(),
            message.single_unread_endpoint()
        );
        assert_eq!(
            single_unread
                .request
                .query
                .get("unread_type")
                .map(String::as_str),
            Some("0")
        );
        Ok(())
    }
}
