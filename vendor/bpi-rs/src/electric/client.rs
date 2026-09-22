use chrono::NaiveDate;

use crate::electric::charge_list::{
    ChargeMonthUpData, ElecRankData, RechargeData, VideoElecShowData,
};
use crate::electric::charge_msg::{ElecRemarkDetail, ElecRemarkList};
use crate::electric::monthly::{
    ChargeFollowInfo, ChargeRecordData, MemberRankData, UpowerItemDetail,
};
use crate::{BilibiliRequest, BpiClient, BpiResult};

const MONTH_UP_LIST_ENDPOINT: &str = "https://api.bilibili.com/x/ugcpay-rank/elec/month/up";
const VIDEO_SHOW_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/elec/show";
const RECHARGE_LIST_ENDPOINT: &str =
    "https://pay.bilibili.com/bk/brokerage/listForCustomerRechargeRecord";
const RANK_RECENT_ENDPOINT: &str = "https://member.bilibili.com/x/h5/elec/rank/recent";
const CHARGE_RECORD_ENDPOINT: &str =
    "https://api.live.bilibili.com/xlive/revenue/v1/guard/getChargeRecord";
const UPOWER_ITEM_DETAIL_ENDPOINT: &str = "https://api.bilibili.com/x/upower/item/detail";
const CHARGE_FOLLOW_INFO_ENDPOINT: &str = "https://api.bilibili.com/x/upower/charge/follow/info";
const UPOWER_MEMBER_RANK_ENDPOINT: &str = "https://api.bilibili.com/x/upower/up/member/rank/v2";
const REMARK_LIST_ENDPOINT: &str = "https://member.bilibili.com/x/web/elec/remark/list";
const REMARK_DETAIL_ENDPOINT: &str = "https://member.bilibili.com/x/web/elec/remark/detail";

/// 充电 API 客户端。
#[derive(Clone, Copy)]
pub struct ElectricClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> ElectricClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn month_up_list_endpoint(&self) -> &'static str {
        MONTH_UP_LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn video_show_endpoint(&self) -> &'static str {
        VIDEO_SHOW_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn recharge_list_endpoint(&self) -> &'static str {
        RECHARGE_LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn rank_recent_endpoint(&self) -> &'static str {
        RANK_RECENT_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn charge_record_endpoint(&self) -> &'static str {
        CHARGE_RECORD_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn upower_item_detail_endpoint(&self) -> &'static str {
        UPOWER_ITEM_DETAIL_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn charge_follow_info_endpoint(&self) -> &'static str {
        CHARGE_FOLLOW_INFO_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn upower_member_rank_endpoint(&self) -> &'static str {
        UPOWER_MEMBER_RANK_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn remark_list_endpoint(&self) -> &'static str {
        REMARK_LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn remark_detail_endpoint(&self) -> &'static str {
        REMARK_DETAIL_ENDPOINT
    }

    /// 获取 UP 用户的月度公开充电列表。
    pub async fn month_up_list(&self, up_mid: i64) -> BpiResult<ChargeMonthUpData> {
        self.client
            .get(MONTH_UP_LIST_ENDPOINT)
            .query(&[("up_mid", up_mid)])
            .send_bpi_payload("electric.month_up_list")
            .await
    }

    /// 获取视频充电鸣谢。
    pub async fn video_show(
        &self,
        mid: i64,
        aid: Option<i64>,
        bvid: Option<&str>,
    ) -> BpiResult<VideoElecShowData> {
        let mut request = self.client.get(VIDEO_SHOW_ENDPOINT).query(&[("mid", mid)]);

        if let Some(aid) = aid {
            request = request.query(&[("aid", aid)]);
        }
        if let Some(bvid) = bvid {
            request = request.query(&[("bvid", bvid)]);
        }

        request.send_bpi_payload("electric.video_show").await
    }

    /// 获取已认证账号收到的充电记录。
    pub async fn recharge_list(
        &self,
        page: u64,
        page_size: u64,
        begin_time: Option<NaiveDate>,
        end_time: Option<NaiveDate>,
    ) -> BpiResult<RechargeData> {
        let mut request = self
            .client
            .get(RECHARGE_LIST_ENDPOINT)
            .query(&[("customerId", "10026")])
            .query(&[("currentPage", page), ("pageSize", page_size)]);

        if let Some(begin_time) = begin_time {
            request = request.query(&[("beginTime", begin_time.format("%Y-%m-%d").to_string())]);
        }
        if let Some(end_time) = end_time {
            request = request.query(&[("endTime", end_time.format("%Y-%m-%d").to_string())]);
        }

        request.send_bpi_payload("electric.recharge_list").await
    }

    /// 获取近期充电排行历史。
    pub async fn rank_recent(&self, pn: Option<u64>, ps: Option<u64>) -> BpiResult<ElecRankData> {
        let mut request = self.client.get(RANK_RECENT_ENDPOINT);

        if let Some(pn) = pn {
            request = request.query(&[("pn", pn)]);
        }
        if let Some(ps) = ps {
            request = request.query(&[("ps", ps)]);
        }

        request.send_bpi_payload("electric.rank_recent").await
    }

    /// 获取已认证账号的月度充电记录。
    pub async fn charge_record(&self, page: u64, charge_type: u32) -> BpiResult<ChargeRecordData> {
        self.client
            .get(CHARGE_RECORD_ENDPOINT)
            .query(&[("page", page)])
            .query(&[("type", charge_type)])
            .send_bpi_payload("electric.charge_record")
            .await
    }

    /// 获取 UP 用户的月度充电项目详情。
    pub async fn upower_item_detail(&self, up_mid: u64) -> BpiResult<UpowerItemDetail> {
        self.client
            .get(UPOWER_ITEM_DETAIL_ENDPOINT)
            .query(&[("up_mid", up_mid)])
            .send_bpi_payload("electric.upower_item_detail")
            .await
    }

    /// 获取已认证账号与 UP 用户的月度充电关系。
    pub async fn charge_follow_info(&self, up_mid: u64) -> BpiResult<ChargeFollowInfo> {
        self.client
            .get(CHARGE_FOLLOW_INFO_ENDPOINT)
            .query(&[("up_mid", up_mid)])
            .send_bpi_payload("electric.charge_follow_info")
            .await
    }

    /// 获取 UP 用户的月度充电成员排行。
    pub async fn upower_member_rank(
        &self,
        up_mid: u64,
        pn: u64,
        ps: u64,
        privilege_type: Option<u64>,
    ) -> BpiResult<MemberRankData> {
        let mut request = self.client.get(UPOWER_MEMBER_RANK_ENDPOINT).query(&[
            ("up_mid", up_mid),
            ("pn", pn),
            ("ps", ps),
        ]);

        if let Some(privilege_type) = privilege_type {
            request = request.query(&[("privilege_type", privilege_type)]);
        }

        request
            .send_bpi_payload("electric.upower_member_rank")
            .await
    }

    /// 列出已认证账号收到的充电留言。
    pub async fn remark_list(
        &self,
        pn: Option<u64>,
        ps: Option<u64>,
        begin: Option<NaiveDate>,
        end: Option<NaiveDate>,
    ) -> BpiResult<ElecRemarkList> {
        let mut request = self.client.get(REMARK_LIST_ENDPOINT);

        if let Some(pn) = pn {
            request = request.query(&[("pn", pn)]);
        }
        if let Some(ps) = ps {
            request = request.query(&[("ps", ps)]);
        }
        if let Some(begin) = begin {
            request = request.query(&[("begin", begin.format("%Y-%m-%d").to_string())]);
        }
        if let Some(end) = end {
            request = request.query(&[("end", end.format("%Y-%m-%d").to_string())]);
        }

        request.send_bpi_payload("electric.remark_list").await
    }

    /// 按 id 获取一条充电留言。
    pub async fn remark_detail(&self, id: u64) -> BpiResult<ElecRemarkDetail> {
        self.client
            .get(REMARK_DETAIL_ENDPOINT)
            .query(&[("id", id)])
            .send_bpi_payload("electric.remark_detail")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::electric::charge_list::{
        ChargeMonthUpData, ElecRankData, RechargeData, VideoElecShowData,
    };
    use crate::electric::charge_msg::{ElecRemarkDetail, ElecRemarkList};
    use crate::electric::monthly::{
        ChargeFollowInfo, ChargeRecordData, MemberRankData, UpowerItemDetail,
    };
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiResult};

    fn assert_month_up_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ChargeMonthUpData>>,
    {
    }

    fn assert_video_show_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<VideoElecShowData>>,
    {
    }

    fn assert_recharge_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<RechargeData>>,
    {
    }

    fn assert_rank_recent_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ElecRankData>>,
    {
    }

    fn assert_charge_record_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ChargeRecordData>>,
    {
    }

    fn assert_upower_item_detail_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<UpowerItemDetail>>,
    {
    }

    fn assert_charge_follow_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ChargeFollowInfo>>,
    {
    }

    fn assert_upower_member_rank_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<MemberRankData>>,
    {
    }

    fn assert_remark_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ElecRemarkList>>,
    {
    }

    fn assert_remark_detail_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ElecRemarkDetail>>,
    {
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "month-up-list" => include_bytes!(
                "../../tests/contracts/electric/public-read/month-up-list/contract.json"
            )
            .as_slice(),
            "video-show" => include_bytes!(
                "../../tests/contracts/electric/public-read/video-show/contract.json"
            )
            .as_slice(),
            "upower-item-detail" => include_bytes!(
                "../../tests/contracts/electric/public-read/upower-item-detail/contract.json"
            )
            .as_slice(),
            "upower-member-rank" => include_bytes!(
                "../../tests/contracts/electric/public-read/upower-member-rank/contract.json"
            )
            .as_slice(),
            "recharge-list" => include_bytes!(
                "../../tests/contracts/electric/private-read/recharge-list/contract.json"
            )
            .as_slice(),
            "rank-recent" => include_bytes!(
                "../../tests/contracts/electric/private-read/rank-recent/contract.json"
            )
            .as_slice(),
            "charge-record" => include_bytes!(
                "../../tests/contracts/electric/private-read/charge-record/contract.json"
            )
            .as_slice(),
            "charge-follow-info" => include_bytes!(
                "../../tests/contracts/electric/private-read/charge-follow-info/contract.json"
            )
            .as_slice(),
            "remark-list" => include_bytes!(
                "../../tests/contracts/electric/private-read/remark-list/contract.json"
            )
            .as_slice(),
            "remark-detail" => include_bytes!(
                "../../tests/contracts/electric/private-read/remark-detail/contract.json"
            )
            .as_slice(),
            _ => unreachable!("unknown electric contract"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn electric_client_exposes_promoted_endpoint_urls() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let electric = client.electric();

        assert_eq!(
            electric.month_up_list_endpoint(),
            "https://api.bilibili.com/x/ugcpay-rank/elec/month/up"
        );
        assert_eq!(
            electric.video_show_endpoint(),
            "https://api.bilibili.com/x/web-interface/elec/show"
        );
        assert_eq!(
            electric.recharge_list_endpoint(),
            "https://pay.bilibili.com/bk/brokerage/listForCustomerRechargeRecord"
        );
        assert_eq!(
            electric.rank_recent_endpoint(),
            "https://member.bilibili.com/x/h5/elec/rank/recent"
        );
        assert_eq!(
            electric.charge_record_endpoint(),
            "https://api.live.bilibili.com/xlive/revenue/v1/guard/getChargeRecord"
        );
        assert_eq!(
            electric.upower_item_detail_endpoint(),
            "https://api.bilibili.com/x/upower/item/detail"
        );
        assert_eq!(
            electric.charge_follow_info_endpoint(),
            "https://api.bilibili.com/x/upower/charge/follow/info"
        );
        assert_eq!(
            electric.upower_member_rank_endpoint(),
            "https://api.bilibili.com/x/upower/up/member/rank/v2"
        );
        assert_eq!(
            electric.remark_list_endpoint(),
            "https://member.bilibili.com/x/web/elec/remark/list"
        );
        assert_eq!(
            electric.remark_detail_endpoint(),
            "https://member.bilibili.com/x/web/elec/remark/detail"
        );
        Ok(())
    }

    #[test]
    fn electric_methods_return_payload_futures() {
        let client = BpiClient::new().expect("client should build");
        let electric = client.electric();

        assert_month_up_list_future(electric.month_up_list(53456));
        assert_video_show_future(electric.video_show(53456, None, Some("BV1Dh411S7sS")));
        assert_recharge_list_future(electric.recharge_list(1, 10, None, None));
        assert_rank_recent_future(electric.rank_recent(Some(1), Some(10)));
        assert_charge_record_future(electric.charge_record(1, 1));
        assert_upower_item_detail_future(electric.upower_item_detail(1265680561));
        assert_charge_follow_info_future(electric.charge_follow_info(1265680561));
        assert_upower_member_rank_future(electric.upower_member_rank(1265680561, 1, 10, None));
        assert_remark_list_future(electric.remark_list(Some(1), Some(10), None, None));
        assert_remark_detail_future(electric.remark_detail(1));
    }

    #[test]
    fn electric_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let electric = client.electric();

        let expectations = [
            (
                "month-up-list",
                "electric.month_up_list",
                electric.month_up_list_endpoint(),
            ),
            (
                "video-show",
                "electric.video_show",
                electric.video_show_endpoint(),
            ),
            (
                "upower-item-detail",
                "electric.upower_item_detail",
                electric.upower_item_detail_endpoint(),
            ),
            (
                "upower-member-rank",
                "electric.upower_member_rank",
                electric.upower_member_rank_endpoint(),
            ),
            (
                "recharge-list",
                "electric.recharge_list",
                electric.recharge_list_endpoint(),
            ),
            (
                "rank-recent",
                "electric.rank_recent",
                electric.rank_recent_endpoint(),
            ),
            (
                "charge-record",
                "electric.charge_record",
                electric.charge_record_endpoint(),
            ),
            (
                "charge-follow-info",
                "electric.charge_follow_info",
                electric.charge_follow_info_endpoint(),
            ),
            (
                "remark-list",
                "electric.remark_list",
                electric.remark_list_endpoint(),
            ),
            (
                "remark-detail",
                "electric.remark_detail",
                electric.remark_detail_endpoint(),
            ),
        ];

        for (endpoint, name, url) in expectations {
            let contract = contract(endpoint)?;

            assert_eq!(contract.name, name);
            assert_eq!(contract.request.method, HttpMethod::Get);
            assert_eq!(contract.request.url.as_str(), url);
        }

        Ok(())
    }
}
