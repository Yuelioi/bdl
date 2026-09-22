use crate::live::danmaku::LiveDanmuInfoData;
use crate::live::emoticons::EmoticonData;
use crate::live::follow_up_live::{FollowUpLiveData, LiveWebListData};
use crate::live::gift::{BlindGiftData, RoomGiftData};
use crate::live::guard::GuardListData;
use crate::live::info::RoomInfoData;
use crate::live::live_area::LiveParentArea;
use crate::live::live_bill::GiftTypeItem;
use crate::live::live_replay::ReplayListData;
use crate::live::live_stream::LiveStreamData;
use crate::live::manage::PcLiveVersionData;
use crate::live::recommend::RecommendData;
use crate::live::redpocket::LotteryInfoData;
use crate::live::report::{HeartBeatData, LiveWebHeartBeatParams};
use crate::live::silent_user_manage::{
    BannedUserListData, LiveBannedUserListParams, LiveShieldKeywordListParams,
    LiveSilentUserListParams, ShieldKeywordListData, SilentUserListData,
};
use crate::live::user::MyMedalsData;
use crate::{BilibiliRequest, BpiClient, BpiResult};

const AREA_LIST_ENDPOINT: &str = "https://api.live.bilibili.com/room/v1/Area/getList";
const ROOM_INFO_ENDPOINT: &str = "https://api.live.bilibili.com/room/v1/Room/get_info";
const STREAM_ENDPOINT: &str = "https://api.live.bilibili.com/room/v1/Room/playUrl";
const RECOMMEND_ENDPOINT: &str =
    "https://api.live.bilibili.com/xlive/web-interface/v1/webMain/getMoreRecList";
const VERSION_ENDPOINT: &str =
    "https://api.live.bilibili.com/xlive/app-blink/v1/liveVersionInfo/getHomePageLiveVersion";
const GIFT_TYPES_ENDPOINT: &str = "https://api.live.bilibili.com/gift/v1/master/getGiftTypes";
const ROOM_GIFT_LIST_ENDPOINT: &str =
    "https://api.live.bilibili.com/xlive/web-room/v1/giftPanel/roomGiftList";
const BLIND_GIFT_INFO_ENDPOINT: &str =
    "https://api.live.bilibili.com/xlive/general-interface/v1/blindFirstWin/getInfo";
const DANMU_INFO_ENDPOINT: &str =
    "https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo";
const EMOTICONS_ENDPOINT: &str =
    "https://api.live.bilibili.com/xlive/web-ucenter/v2/emoticon/GetEmoticons";
const LOTTERY_INFO_ENDPOINT: &str =
    "https://api.live.bilibili.com/xlive/lottery-interface/v1/lottery/getLotteryInfoWeb";
const MY_MEDALS_ENDPOINT: &str =
    "https://api.live.bilibili.com/xlive/app-ucenter/v1/user/GetMyMedals";
const FOLLOW_UP_LIST_ENDPOINT: &str =
    "https://api.live.bilibili.com/xlive/web-ucenter/user/following";
const FOLLOW_UP_WEB_LIST_ENDPOINT: &str =
    "https://api.live.bilibili.com/xlive/web-ucenter/v1/xfetter/GetWebList";
const REPLAY_LIST_ENDPOINT: &str =
    "https://api.live.bilibili.com/xlive/app-blink/v1/anchorVideo/AnchorGetReplayList";
const GUARD_LIST_ENDPOINT: &str =
    "https://api.live.bilibili.com/xlive/app-room/v2/guardTab/topListNew";
const SILENT_USERS_ENDPOINT: &str =
    "https://api.live.bilibili.com/xlive/web-ucenter/v1/banned/GetSilentUserList";
const BANNED_USERS_ENDPOINT: &str =
    "https://api.live.bilibili.com/xlive/app-ucenter/v2/xbanned/banned/GetBlackList";
const SHIELD_KEYWORDS_ENDPOINT: &str =
    "https://api.live.bilibili.com/xlive/app-ucenter/v1/banned/GetShieldKeywordList";
const WEB_HEART_BEAT_ENDPOINT: &str =
    "https://live-trace.bilibili.com/xlive/rdata-interface/v1/heartbeat/webHeartBeat";

/// 直播 API 客户端。
#[derive(Clone, Copy)]
pub struct LiveClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> LiveClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn area_list_endpoint(&self) -> &'static str {
        AREA_LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn room_info_endpoint(&self) -> &'static str {
        ROOM_INFO_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn stream_endpoint(&self) -> &'static str {
        STREAM_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn recommend_endpoint(&self) -> &'static str {
        RECOMMEND_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn version_endpoint(&self) -> &'static str {
        VERSION_ENDPOINT
    }

    /// 获取全部直播分区分类。
    pub async fn area_list(&self) -> BpiResult<Vec<LiveParentArea>> {
        self.client
            .get(AREA_LIST_ENDPOINT)
            .with_bilibili_headers()
            .send_bpi_payload("live.area_list")
            .await
    }

    /// 按真实房间 ID 获取公开房间信息。
    pub async fn room_info(&self, room_id: i64) -> BpiResult<RoomInfoData> {
        self.client
            .get(ROOM_INFO_ENDPOINT)
            .with_bilibili_headers()
            .query(&[("room_id", room_id.to_string())])
            .send_bpi_payload("live.room_info")
            .await
    }

    /// 按真实房间 ID 获取直播流 URL。
    pub async fn stream(
        &self,
        cid: i64,
        platform: Option<&str>,
        quality: Option<i32>,
        qn: Option<i32>,
    ) -> BpiResult<LiveStreamData> {
        let mut query = vec![("cid", cid.to_string())];

        if let Some(platform) = platform {
            query.push(("platform", platform.to_string()));
        }
        if let Some(quality) = quality {
            query.push(("quality", quality.to_string()));
        }
        if let Some(qn) = qn {
            query.push(("qn", qn.to_string()));
        }

        self.client
            .get(STREAM_ENDPOINT)
            .with_bilibili_headers()
            .query(&query)
            .send_bpi_payload("live.stream")
            .await
    }

    /// 获取 Web 首页直播推荐列表。
    pub async fn recommend(&self) -> BpiResult<RecommendData> {
        self.client
            .get(RECOMMEND_ENDPOINT)
            .with_bilibili_headers()
            .query(&[("platform", "web"), ("web_location", "333.1007")])
            .send_bpi_payload("live.recommend")
            .await
    }

    /// 获取当前 PC 直播客户端版本元数据。
    pub async fn version(&self) -> BpiResult<PcLiveVersionData> {
        self.client
            .get(VERSION_ENDPOINT)
            .with_bilibili_headers()
            .query(&[("system_version", "2")])
            .send_bpi_payload("live.version")
            .await
    }

    /// 获取已登录账号的直播礼物类型列表。
    pub async fn gift_types(&self) -> BpiResult<Vec<GiftTypeItem>> {
        self.client
            .get(GIFT_TYPES_ENDPOINT)
            .with_bilibili_headers()
            .send_bpi_payload("live.gift_types")
            .await
    }

    /// 获取直播间礼物面板。
    pub async fn room_gift_list(
        &self,
        room_id: i64,
        area_parent_id: Option<i32>,
        area_id: Option<i32>,
    ) -> BpiResult<RoomGiftData> {
        let mut query = vec![
            ("room_id", room_id.to_string()),
            ("platform", "web".to_string()),
        ];

        if let Some(area_parent_id) = area_parent_id {
            query.push(("area_parent_id", area_parent_id.to_string()));
        }

        if let Some(area_id) = area_id {
            query.push(("area_id", area_id.to_string()));
        }

        self.client
            .get(ROOM_GIFT_LIST_ENDPOINT)
            .with_bilibili_headers()
            .query(&query)
            .send_bpi_payload("live.room_gift_list")
            .await
    }

    /// 获取盲盒礼物概率详情。
    pub async fn blind_gift_info(&self, gift_id: i64) -> BpiResult<BlindGiftData> {
        self.client
            .get(BLIND_GIFT_INFO_ENDPOINT)
            .with_bilibili_headers()
            .query(&[("gift_id", gift_id.to_string())])
            .send_bpi_payload("live.blind_gift_info")
            .await
    }

    /// 获取直播 WebSocket 弹幕 token 和主机信息。
    pub async fn danmu_info(&self, room_id: u64, info_type: u8) -> BpiResult<LiveDanmuInfoData> {
        let query = self
            .client
            .get_wbi_sign2(vec![
                ("id", room_id.to_string()),
                ("type", info_type.to_string()),
            ])
            .await?;

        self.client
            .get(DANMU_INFO_ENDPOINT)
            .with_bilibili_headers()
            .query(&query)
            .send_bpi_payload("live.danmu_info")
            .await
    }

    /// 获取直播间表情包。
    pub async fn emoticons(&self, room_id: i64, platform: &str) -> BpiResult<EmoticonData> {
        self.client
            .get(EMOTICONS_ENDPOINT)
            .with_bilibili_headers()
            .query(&[
                ("room_id", room_id.to_string()),
                ("platform", platform.to_string()),
            ])
            .send_bpi_payload("live.emoticons")
            .await
    }

    /// 获取直播间抽奖信息。
    pub async fn lottery_info(&self, room_id: i64) -> BpiResult<LotteryInfoData> {
        let query = self
            .client
            .get_wbi_sign2(vec![("roomid", room_id.to_string())])
            .await?;

        self.client
            .get(LOTTERY_INFO_ENDPOINT)
            .with_bilibili_headers()
            .query(&query)
            .send_bpi_payload("live.lottery_info")
            .await
    }

    /// 获取当前账号的直播粉丝勋章。
    pub async fn my_medals(&self, page: i32, page_size: i32) -> BpiResult<MyMedalsData> {
        self.client
            .get(MY_MEDALS_ENDPOINT)
            .with_bilibili_headers()
            .query(&[
                ("page", page.to_string()),
                ("page_size", page_size.to_string()),
            ])
            .send_bpi_payload("live.my_medals")
            .await
    }

    /// 获取已关注主播及其直播状态。
    pub async fn follow_up_list(
        &self,
        page: Option<i32>,
        page_size: Option<i32>,
        ignore_record: Option<i32>,
        hit_ab: Option<bool>,
    ) -> BpiResult<FollowUpLiveData> {
        let mut query = Vec::new();

        if let Some(page) = page {
            query.push(("page", page.to_string()));
        }
        if let Some(page_size) = page_size {
            query.push(("page_size", page_size.to_string()));
        }
        if let Some(ignore_record) = ignore_record {
            query.push(("ignoreRecord", ignore_record.to_string()));
        }
        if let Some(hit_ab) = hit_ab {
            query.push(("hit_ab", hit_ab.to_string()));
        }

        self.client
            .get(FOLLOW_UP_LIST_ENDPOINT)
            .with_bilibili_headers()
            .query(&query)
            .send_bpi_payload("live.follow_up_list")
            .await
    }

    /// 获取当前正在直播的已关注主播。
    pub async fn follow_up_web_list(&self, hit_ab: Option<bool>) -> BpiResult<LiveWebListData> {
        let mut query = Vec::new();

        if let Some(hit_ab) = hit_ab {
            query.push(("hit_ab", hit_ab.to_string()));
        }

        self.client
            .get(FOLLOW_UP_WEB_LIST_ENDPOINT)
            .with_bilibili_headers()
            .query(&query)
            .send_bpi_payload("live.follow_up_web_list")
            .await
    }

    /// 获取当前账号的直播回放列表。
    pub async fn replay_list(
        &self,
        page: Option<i32>,
        page_size: Option<i32>,
    ) -> BpiResult<ReplayListData> {
        let mut query = Vec::new();

        if let Some(page) = page {
            query.push(("page", page.to_string()));
        }
        if let Some(page_size) = page_size {
            query.push(("page_size", page_size.to_string()));
        }

        self.client
            .get(REPLAY_LIST_ENDPOINT)
            .with_bilibili_headers()
            .query(&query)
            .send_bpi_payload("live.replay_list")
            .await
    }

    /// 获取直播间的大航海成员。
    pub async fn guard_list(
        &self,
        room_id: i64,
        ruid: i64,
        page: Option<i32>,
        page_size: Option<i32>,
        typ: Option<i32>,
    ) -> BpiResult<GuardListData> {
        let query = [
            ("roomid", room_id.to_string()),
            ("ruid", ruid.to_string()),
            ("page", page.unwrap_or(1).to_string()),
            ("page_size", page_size.unwrap_or(20).to_string()),
            ("typ", typ.unwrap_or(5).to_string()),
        ];

        self.client
            .get(GUARD_LIST_ENDPOINT)
            .with_bilibili_headers()
            .query(&query)
            .send_bpi_payload("live.guard_list")
            .await
    }

    /// 获取直播间禁言用户。
    pub async fn silent_users(
        &self,
        params: LiveSilentUserListParams,
    ) -> BpiResult<SilentUserListData> {
        let csrf = self.client.csrf().unwrap_or_default();
        let form = params.form_pairs(&csrf);

        self.client
            .post(SILENT_USERS_ENDPOINT)
            .with_bilibili_headers()
            .form(&form)
            .send_bpi_payload("live.silent_users")
            .await
    }

    /// 获取直播主播的封禁用户。
    pub async fn banned_users(
        &self,
        params: LiveBannedUserListParams,
    ) -> BpiResult<BannedUserListData> {
        let csrf = self.client.csrf().unwrap_or_default();
        let query = params.query_pairs(&csrf);

        self.client
            .get(BANNED_USERS_ENDPOINT)
            .with_bilibili_headers()
            .query(&query)
            .send_bpi_payload("live.banned_users")
            .await
    }

    /// 获取直播间屏蔽关键词。
    pub async fn shield_keywords(
        &self,
        params: LiveShieldKeywordListParams,
    ) -> BpiResult<ShieldKeywordListData> {
        let csrf = self.client.csrf().unwrap_or_default();
        let form = params.form_pairs(&csrf);

        self.client
            .post(SHIELD_KEYWORDS_ENDPOINT)
            .with_bilibili_headers()
            .form(&form)
            .send_bpi_payload("live.shield_keywords")
            .await
    }

    /// 发送用于直播遥测的 Web 心跳。
    pub async fn web_heart_beat(&self, params: LiveWebHeartBeatParams) -> BpiResult<HeartBeatData> {
        self.client
            .get(WEB_HEART_BEAT_ENDPOINT)
            .with_bilibili_headers()
            .query(&params.query_pairs())
            .send_bpi_payload("live.web_heart_beat")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::ids::{Mid, RoomId};
    use crate::live::danmaku::LiveDanmuInfoData;
    use crate::live::emoticons::EmoticonData;
    use crate::live::follow_up_live::{FollowUpLiveData, LiveWebListData};
    use crate::live::gift::{BlindGiftData, RoomGiftData};
    use crate::live::guard::GuardListData;
    use crate::live::info::RoomInfoData;
    use crate::live::live_area::LiveParentArea;
    use crate::live::live_bill::GiftTypeItem;
    use crate::live::live_replay::ReplayListData;
    use crate::live::live_stream::LiveStreamData;
    use crate::live::manage::PcLiveVersionData;
    use crate::live::recommend::RecommendData;
    use crate::live::redpocket::LotteryInfoData;
    use crate::live::report::{HeartBeatData, LiveWebHeartBeatParams};
    use crate::live::silent_user_manage::{
        BannedUserListData, LiveBannedUserListParams, LiveShieldKeywordListParams,
        LiveSilentUserListParams, ShieldKeywordListData, SilentUserListData,
    };
    use crate::live::user::MyMedalsData;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiError, BpiResult};

    fn assert_area_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<Vec<LiveParentArea>>>,
    {
    }

    fn assert_room_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<RoomInfoData>>,
    {
    }

    fn assert_stream_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<LiveStreamData>>,
    {
    }

    fn assert_recommend_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<RecommendData>>,
    {
    }

    fn assert_version_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<PcLiveVersionData>>,
    {
    }

    fn assert_gift_types_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<Vec<GiftTypeItem>>>,
    {
    }

    fn assert_room_gift_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<RoomGiftData>>,
    {
    }

    fn assert_blind_gift_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<BlindGiftData>>,
    {
    }

    fn assert_danmu_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<LiveDanmuInfoData>>,
    {
    }

    fn assert_emoticons_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<EmoticonData>>,
    {
    }

    fn assert_lottery_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<LotteryInfoData>>,
    {
    }

    fn assert_my_medals_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<MyMedalsData>>,
    {
    }

    fn assert_follow_up_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<FollowUpLiveData>>,
    {
    }

    fn assert_follow_up_web_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<LiveWebListData>>,
    {
    }

    fn assert_replay_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ReplayListData>>,
    {
    }

    fn assert_guard_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<GuardListData>>,
    {
    }

    fn assert_silent_users_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<SilentUserListData>>,
    {
    }

    fn assert_banned_users_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<BannedUserListData>>,
    {
    }

    fn assert_shield_keywords_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ShieldKeywordListData>>,
    {
    }

    fn assert_web_heart_beat_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<HeartBeatData>>,
    {
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes: &[u8] = match endpoint {
            "area-list" => {
                include_bytes!("../../tests/contracts/live/public-core/area-list/contract.json")
            }
            "room-info" => {
                include_bytes!("../../tests/contracts/live/public-core/room-info/contract.json")
            }
            "stream" => {
                include_bytes!("../../tests/contracts/live/public-core/stream/contract.json")
            }
            "recommend" => {
                include_bytes!("../../tests/contracts/live/public-core/recommend/contract.json")
            }
            "version" => {
                include_bytes!("../../tests/contracts/live/public-core/version/contract.json")
            }
            _ => {
                return Err(BpiError::invalid_parameter(
                    "endpoint",
                    "unknown live contract",
                ));
            }
        };

        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn live_public_core_methods_return_payload_futures() -> Result<(), BpiError> {
        let client = BpiClient::new()?;
        let live = client.live();

        assert_area_list_future(live.area_list());
        assert_room_info_future(live.room_info(23_174_842));
        assert_stream_future(live.stream(14_073_662, Some("web"), None, Some(10_000)));
        assert_recommend_future(live.recommend());
        assert_version_future(live.version());
        Ok(())
    }

    #[test]
    fn live_client_exposes_remaining_read_methods() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let live = client.live();

        assert_gift_types_future(live.gift_types());
        assert_room_gift_list_future(live.room_gift_list(23_174_842, None, None));
        assert_blind_gift_info_future(live.blind_gift_info(32_251));
        assert_danmu_info_future(live.danmu_info(21_733_448, 0));
        assert_emoticons_future(live.emoticons(14_047, "pc"));
        assert_lottery_info_future(live.lottery_info(23_174_842));
        assert_my_medals_future(live.my_medals(1, 10));
        assert_follow_up_list_future(live.follow_up_list(Some(1), Some(2), Some(1), Some(true)));
        assert_follow_up_web_list_future(live.follow_up_web_list(Some(false)));
        assert_replay_list_future(live.replay_list(Some(1), Some(2)));
        assert_guard_list_future(live.guard_list(23_174_842, 504_140_200, None, None, None));
        assert_silent_users_future(
            live.silent_users(
                LiveSilentUserListParams::new(RoomId::new(3_818_081)?).page_size(10)?,
            ),
        );
        assert_banned_users_future(
            live.banned_users(LiveBannedUserListParams::new(Mid::new(1_000_001)?).page_size(10)?),
        );
        assert_shield_keywords_future(
            live.shield_keywords(LiveShieldKeywordListParams::new(RoomId::new(3_818_081)?)),
        );
        assert_web_heart_beat_future(live.web_heart_beat(LiveWebHeartBeatParams::new(23_174_842)?));

        let source = include_str!("client.rs");
        let payload_helper = concat!(".send_", "bpi_payload");
        assert!(
            source.matches(payload_helper).count() >= 20,
            "LiveClient should use payload helpers for public-core and remaining promoted read methods"
        );

        Ok(())
    }

    #[test]
    fn live_public_core_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let live = client.live();
        let cases = [
            ("area-list", "live.area_list", live.area_list_endpoint()),
            ("room-info", "live.room_info", live.room_info_endpoint()),
            ("stream", "live.stream", live.stream_endpoint()),
            ("recommend", "live.recommend", live.recommend_endpoint()),
            ("version", "live.version", live.version_endpoint()),
        ];

        for (endpoint, name, url) in cases {
            let contract = contract(endpoint)?;

            assert_eq!(contract.name, name);
            assert_eq!(contract.request.method, HttpMethod::Get);
            assert_eq!(contract.request.url.as_str(), url);
        }

        Ok(())
    }
}
