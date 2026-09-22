use bytes::Bytes;

use crate::danmaku::action::{DanmakuAdvState, DanmakuAdvStateParams};
use crate::danmaku::danmaku_xml::{DanmakuXml, DanmakuXmlListParams, parse_deflate_danmaku_xml};
use crate::danmaku::history::DanmakuHistoryDatesParams;
use crate::danmaku::snapshot::DanmakuSnapshotParams;
use crate::danmaku::thumbup::{DanmakuThumbupStatsParams, ThumbupStatsMap};
use crate::danmaku::web::{DanmakuHistoryBytesParams, DanmakuSegmentParams, DanmakuWebViewParams};
use crate::{BilibiliRequest, BpiClient, BpiResult};

const HISTORY_DATES_ENDPOINT: &str = "https://api.bilibili.com/x/v2/dm/history/index";
const SNAPSHOT_ENDPOINT: &str = "https://api.bilibili.com/x/v2/dm/ajax";
const THUMBUP_STATS_ENDPOINT: &str = "https://api.bilibili.com/x/v2/dm/thumbup/stats";
const ADV_STATE_ENDPOINT: &str = "https://api.bilibili.com/x/dm/adv/state";
const WEB_SEG_ENDPOINT: &str = "https://api.bilibili.com/x/v2/dm/web/seg.so";
const WEB_SEG_WBI_ENDPOINT: &str = "https://api.bilibili.com/x/v2/dm/wbi/web/seg.so";
const WEB_VIEW_ENDPOINT: &str = "https://api.bilibili.com/x/v2/dm/web/view";
const MOBILE_SEG_ENDPOINT: &str = "https://api.bilibili.com/x/v2/dm/list/seg.so";
const WEB_HISTORY_SEG_ENDPOINT: &str = "https://api.bilibili.com/x/v2/dm/web/history/seg.so";
const HISTORY_XML_ENDPOINT: &str = "https://api.bilibili.com/x/v2/dm/history";
const XML_LIST_SO_ENDPOINT: &str = "https://api.bilibili.com/x/v1/dm/list.so";

/// 弹幕 API 客户端。
#[derive(Clone, Copy)]
pub struct DanmakuClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> DanmakuClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn history_dates_endpoint(&self) -> &'static str {
        HISTORY_DATES_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn snapshot_endpoint(&self) -> &'static str {
        SNAPSHOT_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn thumbup_stats_endpoint(&self) -> &'static str {
        THUMBUP_STATS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn adv_state_endpoint(&self) -> &'static str {
        ADV_STATE_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn web_seg_endpoint(&self) -> &'static str {
        WEB_SEG_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn web_seg_wbi_endpoint(&self) -> &'static str {
        WEB_SEG_WBI_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn web_view_endpoint(&self) -> &'static str {
        WEB_VIEW_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn mobile_seg_endpoint(&self) -> &'static str {
        MOBILE_SEG_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn web_history_seg_endpoint(&self) -> &'static str {
        WEB_HISTORY_SEG_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn history_xml_endpoint(&self) -> &'static str {
        HISTORY_XML_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn xml_list_so_endpoint(&self) -> &'static str {
        XML_LIST_SO_ENDPOINT
    }

    /// 查询某个月内有历史弹幕的日期。
    pub async fn history_dates(
        &self,
        params: DanmakuHistoryDatesParams,
    ) -> BpiResult<Option<Vec<String>>> {
        self.client
            .get(HISTORY_DATES_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_optional_payload("danmaku.history.dates")
            .await
    }

    /// 获取稿件的近期弹幕快照行。
    pub async fn snapshot(&self, params: DanmakuSnapshotParams) -> BpiResult<Vec<String>> {
        self.client
            .get(SNAPSHOT_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("danmaku.snapshot")
            .await
    }

    /// 查询指定弹幕 ID 的点赞统计。
    pub async fn thumbup_stats(
        &self,
        params: DanmakuThumbupStatsParams,
    ) -> BpiResult<ThumbupStatsMap> {
        self.client
            .get(THUMBUP_STATS_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("danmaku.thumbup.stats")
            .await
    }

    /// 检查高级弹幕购买/发送状态。
    pub async fn adv_state(&self, params: DanmakuAdvStateParams) -> BpiResult<DanmakuAdvState> {
        self.client
            .get(ADV_STATE_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("danmaku.adv.state")
            .await
    }

    /// 获取实时 Web protobuf 弹幕分段字节。
    pub async fn web_seg_proto(&self, params: DanmakuSegmentParams) -> BpiResult<Bytes> {
        self.client
            .get(WEB_SEG_ENDPOINT)
            .with_bilibili_headers()
            .query(&params.query_pairs())
            .send_request("danmaku.web.seg")
            .await
    }

    /// 获取 WBI 签名的实时 Web protobuf 弹幕分段字节。
    pub async fn web_seg_wbi_proto(&self, params: DanmakuSegmentParams) -> BpiResult<Bytes> {
        let signed = self.client.get_wbi_sign2(params.query_pairs()).await?;

        self.client
            .get(WEB_SEG_WBI_ENDPOINT)
            .with_bilibili_headers()
            .query(&signed)
            .send_request("danmaku.web.seg_wbi")
            .await
    }

    /// 获取 protobuf 弹幕 web-view 元数据字节。
    pub async fn web_view_proto(&self, params: DanmakuWebViewParams) -> BpiResult<Bytes> {
        self.client
            .get(WEB_VIEW_ENDPOINT)
            .with_bilibili_headers()
            .query(&params.query_pairs())
            .send_request("danmaku.web.view")
            .await
    }

    /// 获取移动端 protobuf 弹幕分段字节。
    pub async fn mobile_seg_proto(&self, params: DanmakuSegmentParams) -> BpiResult<Bytes> {
        self.client
            .get(MOBILE_SEG_ENDPOINT)
            .with_bilibili_headers()
            .query(&params.query_pairs())
            .send_request("danmaku.mobile.seg")
            .await
    }

    /// 获取指定日期的历史 protobuf 弹幕分段字节。
    pub async fn web_history_seg_proto(
        &self,
        params: DanmakuHistoryBytesParams,
    ) -> BpiResult<Bytes> {
        self.client
            .get(WEB_HISTORY_SEG_ENDPOINT)
            .with_bilibili_headers()
            .query(&params.query_pairs())
            .send_request("danmaku.web.history_seg")
            .await
    }

    /// 获取指定日期的原始压缩历史 XML 弹幕字节。
    pub async fn history_xml_bytes(&self, params: DanmakuHistoryBytesParams) -> BpiResult<Bytes> {
        self.client
            .get_without_response_decoding(HISTORY_XML_ENDPOINT)?
            .query(&params.query_pairs())
            .send_request("danmaku.history.xml")
            .await
    }

    /// 从 `/x/v1/dm/list.so` 获取并解析实时 XML 弹幕。
    pub async fn xml_list_so(&self, params: DanmakuXmlListParams) -> BpiResult<DanmakuXml> {
        let response = self
            .client
            .get_without_response_decoding(XML_LIST_SO_ENDPOINT)?
            .query(&params.query_pairs())
            .send()
            .await?;
        let bytes = response.bytes().await?;

        parse_deflate_danmaku_xml(&bytes)
    }

    /// 从 comment host 获取并解析实时 XML 弹幕。
    pub async fn xml_list(&self, params: DanmakuXmlListParams) -> BpiResult<DanmakuXml> {
        let response = self
            .client
            .get_without_response_decoding(&params.comment_xml_url())?
            .send()
            .await?;
        let bytes = response.bytes().await?;

        parse_deflate_danmaku_xml(&bytes)
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use bytes::Bytes;

    use crate::danmaku::action::{DanmakuAdvState, DanmakuAdvStateParams};
    use crate::danmaku::danmaku_xml::{DanmakuXml, DanmakuXmlListParams};
    use crate::danmaku::history::DanmakuHistoryDatesParams;
    use crate::danmaku::snapshot::DanmakuSnapshotParams;
    use crate::danmaku::thumbup::{DanmakuThumbupStatsParams, ThumbupStatsMap};
    use crate::danmaku::web::{
        DanmakuHistoryBytesParams, DanmakuSegmentParams, DanmakuWebViewParams,
    };
    use crate::ids::{Aid, Bvid, Cid};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiError, BpiResult};

    const TEST_AID: u64 = 170_001;
    const TEST_BVID: &str = "BV1fK4y1t741";
    const TEST_CID: u64 = 413_195_701;
    const TEST_DMID: u64 = 1_932_011_031_958_944_000;
    const TEST_OID: u64 = 16_546;

    fn assert_history_dates_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<Option<Vec<String>>>>,
    {
    }

    fn assert_snapshot_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<Vec<String>>>,
    {
    }

    fn assert_thumbup_stats_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ThumbupStatsMap>>,
    {
    }

    fn assert_adv_state_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<DanmakuAdvState>>,
    {
    }

    fn assert_bytes_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<Bytes>>,
    {
    }

    fn assert_xml_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<DanmakuXml>>,
    {
    }

    fn contract(path: &str) -> BpiResult<EndpointContract> {
        let bytes = match path {
            "json-read/history-dates" => include_bytes!(
                "../../tests/contracts/danmaku/json-read/history-dates/contract.json"
            )
            .as_slice(),
            "json-read/snapshot" => {
                include_bytes!("../../tests/contracts/danmaku/json-read/snapshot/contract.json")
                    .as_slice()
            }
            "json-read/thumbup-stats" => include_bytes!(
                "../../tests/contracts/danmaku/json-read/thumbup-stats/contract.json"
            )
            .as_slice(),
            "json-read/adv-state" => {
                include_bytes!("../../tests/contracts/danmaku/json-read/adv-state/contract.json")
                    .as_slice()
            }
            "non-json-read/web-seg" => {
                include_bytes!("../../tests/contracts/danmaku/non-json-read/web-seg/contract.json")
                    .as_slice()
            }
            "non-json-read/web-seg-wbi" => include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/web-seg-wbi/contract.json"
            )
            .as_slice(),
            "non-json-read/web-view" => {
                include_bytes!("../../tests/contracts/danmaku/non-json-read/web-view/contract.json")
                    .as_slice()
            }
            "non-json-read/mobile-seg" => include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/mobile-seg/contract.json"
            )
            .as_slice(),
            "non-json-read/web-history-seg" => include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/web-history-seg/contract.json"
            )
            .as_slice(),
            "history-xml" => {
                include_bytes!("../../tests/contracts/danmaku/history-xml/contract.json").as_slice()
            }
            "xml-read/list-so" => {
                include_bytes!("../../tests/contracts/danmaku/xml-read/list-so/contract.json")
                    .as_slice()
            }
            "xml-read/comment-xml" => {
                include_bytes!("../../tests/contracts/danmaku/xml-read/comment-xml/contract.json")
                    .as_slice()
            }
            _ => unreachable!("unknown danmaku contract"),
        };
        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn danmaku_methods_return_read_futures() -> Result<(), BpiError> {
        let client = BpiClient::new()?;
        let danmaku = client.danmaku();
        let cid = Cid::new(TEST_CID)?;
        let xml_cid = Cid::new(TEST_OID)?;
        let bvid = Bvid::new(TEST_BVID)?;

        assert_history_dates_future(
            danmaku.history_dates(DanmakuHistoryDatesParams::new(cid, "2022-01")?),
        );
        assert_snapshot_future(danmaku.snapshot(DanmakuSnapshotParams::from_bvid(bvid)));
        assert_snapshot_future(
            danmaku.snapshot(DanmakuSnapshotParams::from_aid(Aid::new(TEST_AID)?)),
        );
        assert_thumbup_stats_future(danmaku.thumbup_stats(DanmakuThumbupStatsParams::new(
            Cid::new(TEST_CID)?,
            [TEST_DMID],
        )?));
        assert_adv_state_future(danmaku.adv_state(DanmakuAdvStateParams::new(Cid::new(TEST_CID)?)));
        assert_bytes_future(danmaku.web_seg_proto(DanmakuSegmentParams::new(1, TEST_OID, 1)?));
        assert_bytes_future(danmaku.web_seg_wbi_proto(DanmakuSegmentParams::new(1, TEST_OID, 1)?));
        assert_bytes_future(danmaku.web_view_proto(DanmakuWebViewParams::new(1, TEST_OID)?));
        assert_bytes_future(danmaku.mobile_seg_proto(DanmakuSegmentParams::new(1, TEST_OID, 1)?));
        assert_bytes_future(
            danmaku.web_history_seg_proto(DanmakuHistoryBytesParams::new(
                1,
                TEST_OID,
                "2022-01-01",
            )?),
        );
        assert_bytes_future(danmaku.history_xml_bytes(DanmakuHistoryBytesParams::new(
            1,
            TEST_OID,
            "2022-01-01",
        )?));
        assert_xml_future(danmaku.xml_list_so(DanmakuXmlListParams::new(xml_cid)));
        assert_xml_future(danmaku.xml_list(DanmakuXmlListParams::new(xml_cid)));
        Ok(())
    }

    #[test]
    fn danmaku_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let danmaku = client.danmaku();
        let cases = [
            (
                "json-read/history-dates",
                "danmaku.history.dates",
                danmaku.history_dates_endpoint(),
            ),
            (
                "json-read/snapshot",
                "danmaku.snapshot",
                danmaku.snapshot_endpoint(),
            ),
            (
                "json-read/thumbup-stats",
                "danmaku.thumbup.stats",
                danmaku.thumbup_stats_endpoint(),
            ),
            (
                "json-read/adv-state",
                "danmaku.adv.state",
                danmaku.adv_state_endpoint(),
            ),
            (
                "non-json-read/web-seg",
                "danmaku.web.seg",
                danmaku.web_seg_endpoint(),
            ),
            (
                "non-json-read/web-seg-wbi",
                "danmaku.web.seg_wbi",
                danmaku.web_seg_wbi_endpoint(),
            ),
            (
                "non-json-read/web-view",
                "danmaku.web.view",
                danmaku.web_view_endpoint(),
            ),
            (
                "non-json-read/mobile-seg",
                "danmaku.mobile.seg",
                danmaku.mobile_seg_endpoint(),
            ),
            (
                "non-json-read/web-history-seg",
                "danmaku.web.history_seg",
                danmaku.web_history_seg_endpoint(),
            ),
            (
                "history-xml",
                "danmaku.history.xml",
                danmaku.history_xml_endpoint(),
            ),
            (
                "xml-read/list-so",
                "danmaku.xml.list_so",
                danmaku.xml_list_so_endpoint(),
            ),
        ];

        for (path, name, endpoint) in cases {
            let contract = contract(path)?;
            assert_eq!(contract.name, name);
            assert_eq!(contract.request.method, HttpMethod::Get);
            assert_eq!(contract.request.url.as_str(), endpoint);
        }

        let comment_xml = contract("xml-read/comment-xml")?;
        let params = DanmakuXmlListParams::new(Cid::new(TEST_OID)?);
        assert_eq!(comment_xml.name, "danmaku.xml.comment_xml");
        assert_eq!(comment_xml.request.method, HttpMethod::Get);
        assert_eq!(comment_xml.request.url.as_str(), params.comment_xml_url());
        Ok(())
    }
}
