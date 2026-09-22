use crate::dynamic::all::{DynamicAllData, DynamicUpdateData};
use crate::dynamic::banner::DynamicBannerData;
use crate::dynamic::content::{DynUpUsersData, LiveUsersData};
use crate::dynamic::detail::{
    DynamicDetailData, DynamicForwardData, DynamicForwardInfoData, DynamicLotteryData, DynamicPic,
    DynamicReactionData,
};
use crate::dynamic::get_dynamic_detail::RecentUpData;
use crate::dynamic::nav::DynamicNavData;
use crate::dynamic::{
    DynamicAllParams, DynamicCheckNewParams, DynamicDetailParams, DynamicForwardItemParams,
    DynamicForwardsParams, DynamicLiveUsersParams, DynamicLotteryNoticeParams,
    DynamicNavFeedParams, DynamicPicsParams, DynamicReactionsParams, DynamicUpUsersParams,
};
use crate::{BilibiliRequest, BpiClient, BpiResult};

const ALL_ENDPOINT: &str = "https://api.bilibili.com/x/polymer/web-dynamic/v1/feed/all";
const CHECK_NEW_ENDPOINT: &str =
    "https://api.bilibili.com/x/polymer/web-dynamic/v1/feed/all/update";
const NAV_FEED_ENDPOINT: &str = "https://api.bilibili.com/x/polymer/web-dynamic/v1/feed/nav";
const FEED_BANNER_ENDPOINT: &str = "https://api.bilibili.com/x/dynamic/feed/dyn/banner";
const DETAIL_ENDPOINT: &str = "https://api.bilibili.com/x/polymer/web-dynamic/v1/detail";
const REACTIONS_ENDPOINT: &str =
    "https://api.bilibili.com/x/polymer/web-dynamic/v1/detail/reaction";
const LOTTERY_NOTICE_ENDPOINT: &str =
    "https://api.vc.bilibili.com/lottery_svr/v1/lottery_svr/lottery_notice";
const FORWARDS_ENDPOINT: &str = "https://api.bilibili.com/x/polymer/web-dynamic/v1/detail/forward";
const PICS_ENDPOINT: &str = "https://api.bilibili.com/x/polymer/web-dynamic/v1/detail/pic";
const FORWARD_ITEM_ENDPOINT: &str =
    "https://api.bilibili.com/x/polymer/web-dynamic/v1/detail/forward/item";
const LIVE_USERS_ENDPOINT: &str =
    "https://api.vc.bilibili.com/dynamic_svr/v1/dynamic_svr/w_live_users";
const UP_USERS_ENDPOINT: &str =
    "https://api.vc.bilibili.com/dynamic_svr/v1/dynamic_svr/w_dyn_uplist";
const RECENT_UP_ENDPOINT: &str = "https://api.bilibili.com/x/polymer/web-dynamic/v1/portal";

/// 动态 API 客户端。
#[derive(Clone, Copy)]
pub struct DynamicClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> DynamicClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn all_endpoint(&self) -> &'static str {
        ALL_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn check_new_endpoint(&self) -> &'static str {
        CHECK_NEW_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn nav_feed_endpoint(&self) -> &'static str {
        NAV_FEED_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn feed_banner_endpoint(&self) -> &'static str {
        FEED_BANNER_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn detail_endpoint(&self) -> &'static str {
        DETAIL_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn reactions_endpoint(&self) -> &'static str {
        REACTIONS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn lottery_notice_endpoint(&self) -> &'static str {
        LOTTERY_NOTICE_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn forwards_endpoint(&self) -> &'static str {
        FORWARDS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn pics_endpoint(&self) -> &'static str {
        PICS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn forward_item_endpoint(&self) -> &'static str {
        FORWARD_ITEM_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn live_users_endpoint(&self) -> &'static str {
        LIVE_USERS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn up_users_endpoint(&self) -> &'static str {
        UP_USERS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn recent_up_endpoint(&self) -> &'static str {
        RECENT_UP_ENDPOINT
    }

    /// 获取已关注动态流。
    pub async fn all(&self, params: DynamicAllParams) -> BpiResult<DynamicAllData> {
        self.client
            .get(ALL_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("dynamic.feed_all")
            .await
    }

    /// 检查动态流是否有新内容。
    pub async fn check_new(&self, params: DynamicCheckNewParams) -> BpiResult<DynamicUpdateData> {
        self.client
            .get(CHECK_NEW_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("dynamic.feed_all_update")
            .await
    }

    /// 获取导航动态流展示的动态条目。
    pub async fn nav_feed(&self, params: DynamicNavFeedParams) -> BpiResult<DynamicNavData> {
        self.client
            .get(NAV_FEED_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("dynamic.feed_nav")
            .await
    }

    /// 获取动态流 banner。
    pub async fn feed_banner(&self) -> BpiResult<DynamicBannerData> {
        self.client
            .get(FEED_BANNER_ENDPOINT)
            .query(&[
                ("platform", "1"),
                ("position", "web动态"),
                ("web_location", "333.1365"),
            ])
            .send_bpi_payload("dynamic.feed_banner")
            .await
    }

    /// 获取动态条目详情。
    pub async fn detail(&self, params: DynamicDetailParams) -> BpiResult<DynamicDetailData> {
        self.client
            .get(DETAIL_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("dynamic.detail")
            .await
    }

    /// 获取动态条目的互动用户。
    pub async fn reactions(
        &self,
        params: DynamicReactionsParams,
    ) -> BpiResult<DynamicReactionData> {
        self.client
            .get(REACTIONS_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("dynamic.detail_reaction")
            .await
    }

    /// 获取动态条目的抽奖通知详情。
    pub async fn lottery_notice(
        &self,
        params: DynamicLotteryNoticeParams,
    ) -> BpiResult<DynamicLotteryData> {
        let csrf = self.client.csrf()?;

        self.client
            .get(LOTTERY_NOTICE_ENDPOINT)
            .query(&params.query_pairs(&csrf))
            .send_bpi_payload("dynamic.lottery_notice")
            .await
    }

    /// 获取动态条目的转发列表。
    pub async fn forwards(&self, params: DynamicForwardsParams) -> BpiResult<DynamicForwardData> {
        self.client
            .get(FORWARDS_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("dynamic.detail_forward")
            .await
    }

    /// 获取动态条目的图片。
    pub async fn pics(&self, params: DynamicPicsParams) -> BpiResult<Vec<DynamicPic>> {
        self.client
            .get(PICS_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("dynamic.detail_pic")
            .await
    }

    /// 获取被转发的动态条目。
    pub async fn forward_item(
        &self,
        params: DynamicForwardItemParams,
    ) -> BpiResult<DynamicForwardInfoData> {
        self.client
            .get(FORWARD_ITEM_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("dynamic.detail_forward_item")
            .await
    }

    /// 获取当前正在直播的已关注用户。
    pub async fn live_users(&self, params: DynamicLiveUsersParams) -> BpiResult<LiveUsersData> {
        self.client
            .get(LIVE_USERS_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("dynamic.live_users")
            .await
    }

    /// 获取有新动态内容的已关注用户。
    pub async fn up_users(&self, params: DynamicUpUsersParams) -> BpiResult<DynUpUsersData> {
        self.client
            .get(UP_USERS_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("dynamic.up_users")
            .await
    }

    /// 获取最近更新的已关注用户。
    pub async fn recent_up(&self) -> BpiResult<RecentUpData> {
        self.client
            .get(RECENT_UP_ENDPOINT)
            .send_bpi_payload("dynamic.recent_up")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::dynamic::all::{DynamicAllData, DynamicUpdateData};
    use crate::dynamic::banner::DynamicBannerData;
    use crate::dynamic::content::{DynUpUsersData, LiveUsersData};
    use crate::dynamic::detail::{
        DynamicDetailData, DynamicForwardData, DynamicForwardInfoData, DynamicLotteryData,
        DynamicPic, DynamicReactionData,
    };
    use crate::dynamic::get_dynamic_detail::RecentUpData;
    use crate::dynamic::nav::DynamicNavData;
    use crate::dynamic::{
        DynamicAllParams, DynamicCheckNewParams, DynamicDetailParams, DynamicForwardItemParams,
        DynamicForwardsParams, DynamicLiveUsersParams, DynamicLotteryNoticeParams,
        DynamicNavFeedParams, DynamicPicsParams, DynamicReactionsParams, DynamicUpUsersParams,
    };
    use crate::ids::DynamicId;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiError, BpiResult};

    fn dynamic_id(value: &str) -> Result<DynamicId, BpiError> {
        value.parse()
    }

    fn assert_all_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<DynamicAllData>>,
    {
    }

    fn assert_check_new_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<DynamicUpdateData>>,
    {
    }

    fn assert_nav_feed_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<DynamicNavData>>,
    {
    }

    fn assert_feed_banner_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<DynamicBannerData>>,
    {
    }

    fn assert_detail_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<DynamicDetailData>>,
    {
    }

    fn assert_reactions_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<DynamicReactionData>>,
    {
    }

    fn assert_lottery_notice_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<DynamicLotteryData>>,
    {
    }

    fn assert_forwards_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<DynamicForwardData>>,
    {
    }

    fn assert_pics_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<Vec<DynamicPic>>>,
    {
    }

    fn assert_forward_item_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<DynamicForwardInfoData>>,
    {
    }

    fn assert_live_users_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<LiveUsersData>>,
    {
    }

    fn assert_up_users_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<DynUpUsersData>>,
    {
    }

    fn assert_recent_up_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<RecentUpData>>,
    {
    }

    fn contract(path: &str) -> BpiResult<EndpointContract> {
        let bytes = match path {
            "feed/all" => {
                include_bytes!("../../tests/contracts/dynamic/feed/all/contract.json").as_slice()
            }
            "feed/check-new" => {
                include_bytes!("../../tests/contracts/dynamic/feed/check-new/contract.json")
                    .as_slice()
            }
            "feed/nav" => {
                include_bytes!("../../tests/contracts/dynamic/feed/nav/contract.json").as_slice()
            }
            "feed/banner" => {
                include_bytes!("../../tests/contracts/dynamic/feed/banner/contract.json").as_slice()
            }
            "detail/detail" => {
                include_bytes!("../../tests/contracts/dynamic/detail/detail/contract.json")
                    .as_slice()
            }
            "detail/reactions" => {
                include_bytes!("../../tests/contracts/dynamic/detail/reactions/contract.json")
                    .as_slice()
            }
            "detail/forwards" => {
                include_bytes!("../../tests/contracts/dynamic/detail/forwards/contract.json")
                    .as_slice()
            }
            "detail/pics" => {
                include_bytes!("../../tests/contracts/dynamic/detail/pics/contract.json").as_slice()
            }
            "detail/forward-item" => {
                include_bytes!("../../tests/contracts/dynamic/detail/forward-item/contract.json")
                    .as_slice()
            }
            "content/live-users" => {
                include_bytes!("../../tests/contracts/dynamic/content/live-users/contract.json")
                    .as_slice()
            }
            "content/up-users" => {
                include_bytes!("../../tests/contracts/dynamic/content/up-users/contract.json")
                    .as_slice()
            }
            "content/recent-up" => {
                include_bytes!("../../tests/contracts/dynamic/content/recent-up/contract.json")
                    .as_slice()
            }
            "lottery-notice-read/lottery-notice" => include_bytes!(
                "../../tests/contracts/dynamic/lottery-notice-read/lottery-notice/contract.json"
            )
            .as_slice(),
            _ => unreachable!("unknown dynamic contract"),
        };
        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn dynamic_methods_return_payload_futures() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let dynamic = client.dynamic();
        let detail_id = dynamic_id("1099138163191840776")?;
        let forward_item_id = dynamic_id("1110902525317349376")?;
        let lottery_id = dynamic_id("969916293954142214")?;

        assert_all_future(dynamic.all(DynamicAllParams::new()));
        assert_check_new_future(dynamic.check_new(DynamicCheckNewParams::new("0")?));
        assert_nav_feed_future(dynamic.nav_feed(DynamicNavFeedParams::new()));
        assert_feed_banner_future(dynamic.feed_banner());
        assert_detail_future(dynamic.detail(DynamicDetailParams::new(detail_id.clone())));
        assert_reactions_future(dynamic.reactions(DynamicReactionsParams::new(detail_id.clone())));
        assert_lottery_notice_future(
            dynamic.lottery_notice(DynamicLotteryNoticeParams::new(lottery_id)),
        );
        assert_forwards_future(dynamic.forwards(DynamicForwardsParams::new(detail_id.clone())));
        assert_pics_future(dynamic.pics(DynamicPicsParams::new(detail_id)));
        assert_forward_item_future(
            dynamic.forward_item(DynamicForwardItemParams::new(forward_item_id)),
        );
        assert_live_users_future(dynamic.live_users(DynamicLiveUsersParams::new().with_size(1)?));
        assert_up_users_future(dynamic.up_users(DynamicUpUsersParams::new()));
        assert_recent_up_future(dynamic.recent_up());
        Ok(())
    }

    #[test]
    fn dynamic_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let dynamic = client.dynamic();

        let cases = [
            ("feed/all", "dynamic.feed_all", dynamic.all_endpoint()),
            (
                "feed/check-new",
                "dynamic.feed_all_update",
                dynamic.check_new_endpoint(),
            ),
            ("feed/nav", "dynamic.feed_nav", dynamic.nav_feed_endpoint()),
            (
                "feed/banner",
                "dynamic.feed_banner",
                dynamic.feed_banner_endpoint(),
            ),
            ("detail/detail", "dynamic.detail", dynamic.detail_endpoint()),
            (
                "detail/reactions",
                "dynamic.detail_reaction",
                dynamic.reactions_endpoint(),
            ),
            (
                "detail/forwards",
                "dynamic.detail_forward",
                dynamic.forwards_endpoint(),
            ),
            ("detail/pics", "dynamic.detail_pic", dynamic.pics_endpoint()),
            (
                "detail/forward-item",
                "dynamic.detail_forward_item",
                dynamic.forward_item_endpoint(),
            ),
            (
                "content/live-users",
                "dynamic.live_users",
                dynamic.live_users_endpoint(),
            ),
            (
                "content/up-users",
                "dynamic.up_users",
                dynamic.up_users_endpoint(),
            ),
            (
                "content/recent-up",
                "dynamic.recent_up",
                dynamic.recent_up_endpoint(),
            ),
            (
                "lottery-notice-read/lottery-notice",
                "dynamic.lottery_notice",
                dynamic.lottery_notice_endpoint(),
            ),
        ];

        for (path, name, endpoint) in cases {
            let contract = contract(path)?;
            assert_eq!(contract.name, name);
            assert_eq!(contract.request.method, HttpMethod::Get);
            assert_eq!(contract.request.url.as_str(), endpoint);
        }
        Ok(())
    }
}
