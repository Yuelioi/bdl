//! Web / App 弹幕二进制接口（protobuf，见 bilibili-API-collect `danmaku_proto.md`、`danmaku_view_proto.md`）
//!
//! 响应体需使用官方 [`dm.proto`](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/grpc_api/bilibili/community/service/dm/v1)
//! 中的 `DmSegMobileReply`、`DmWebViewReply` 等自行反序列化。

use crate::{BpiError, BpiResult};

/// 实时 protobuf 弹幕分段 endpoint 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DanmakuSegmentParams {
    typ: u8,
    oid: u64,
    segment_index: u32,
    pid: Option<u64>,
    pull_mode: Option<u32>,
    ps: Option<u32>,
    pe: Option<u32>,
}

impl DanmakuSegmentParams {
    /// 为实时弹幕分段请求创建参数。
    pub fn new(typ: u8, oid: u64, segment_index: u32) -> BpiResult<Self> {
        if typ == 0 {
            return Err(BpiError::invalid_parameter(
                "type",
                "danmaku type must be non-zero",
            ));
        }
        if oid == 0 {
            return Err(BpiError::invalid_parameter(
                "oid",
                "danmaku oid must be non-zero",
            ));
        }
        if segment_index == 0 {
            return Err(BpiError::invalid_parameter(
                "segment_index",
                "segment index must be non-zero",
            ));
        }

        Ok(Self {
            typ,
            oid,
            segment_index,
            pid: None,
            pull_mode: None,
            ps: None,
            pe: None,
        })
    }

    /// 设置可选稿件 avid。
    pub fn pid(mut self, pid: u64) -> BpiResult<Self> {
        if pid == 0 {
            return Err(BpiError::invalid_parameter(
                "pid",
                "archive id must be non-zero",
            ));
        }
        self.pid = Some(pid);
        Ok(self)
    }

    /// 设置可选拉取模式。
    pub fn pull_mode(mut self, pull_mode: u32) -> Self {
        self.pull_mode = Some(pull_mode);
        self
    }

    /// 设置分段内容的可选毫秒范围。
    pub fn range(mut self, ps: u32, pe: u32) -> BpiResult<Self> {
        if pe < ps {
            return Err(BpiError::invalid_parameter(
                "pe",
                "range end must be greater than or equal to range start",
            ));
        }
        self.ps = Some(ps);
        self.pe = Some(pe);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(String, String)> {
        let mut q = vec![
            ("type".to_string(), self.typ.to_string()),
            ("oid".to_string(), self.oid.to_string()),
            ("segment_index".to_string(), self.segment_index.to_string()),
        ];

        if let Some(pid) = self.pid {
            q.push(("pid".to_string(), pid.to_string()));
        }
        if let Some(pull_mode) = self.pull_mode {
            q.push(("pull_mode".to_string(), pull_mode.to_string()));
        }
        if let Some(ps) = self.ps {
            q.push(("ps".to_string(), ps.to_string()));
        }
        if let Some(pe) = self.pe {
            q.push(("pe".to_string(), pe.to_string()));
        }

        q
    }
}

/// protobuf 弹幕 web-view 元数据的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DanmakuWebViewParams {
    typ: u8,
    oid: u64,
    pid: Option<u64>,
}

impl DanmakuWebViewParams {
    pub fn new(typ: u8, oid: u64) -> BpiResult<Self> {
        validate_danmaku_type(typ)?;
        validate_oid(oid)?;

        Ok(Self {
            typ,
            oid,
            pid: None,
        })
    }

    pub fn pid(mut self, pid: u64) -> BpiResult<Self> {
        if pid == 0 {
            return Err(BpiError::invalid_parameter(
                "pid",
                "archive id must be non-zero",
            ));
        }
        self.pid = Some(pid);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(String, String)> {
        let mut q = vec![
            ("type".to_string(), self.typ.to_string()),
            ("oid".to_string(), self.oid.to_string()),
        ];
        if let Some(pid) = self.pid {
            q.push(("pid".to_string(), pid.to_string()));
        }
        q
    }
}

/// 指定日期弹幕历史字节 endpoint 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DanmakuHistoryBytesParams {
    typ: u8,
    oid: u64,
    date: String,
}

impl DanmakuHistoryBytesParams {
    pub fn new(typ: u8, oid: u64, date: impl Into<String>) -> BpiResult<Self> {
        validate_danmaku_type(typ)?;
        validate_oid(oid)?;
        let date = date.into();
        validate_date(&date)?;

        Ok(Self { typ, oid, date })
    }

    pub(crate) fn query_pairs(&self) -> Vec<(String, String)> {
        vec![
            ("type".to_string(), self.typ.to_string()),
            ("oid".to_string(), self.oid.to_string()),
            ("date".to_string(), self.date.clone()),
        ]
    }
}

fn validate_danmaku_type(typ: u8) -> BpiResult<()> {
    if typ == 0 {
        return Err(BpiError::invalid_parameter(
            "type",
            "danmaku type must be non-zero",
        ));
    }
    Ok(())
}

fn validate_oid(oid: u64) -> BpiResult<()> {
    if oid == 0 {
        return Err(BpiError::invalid_parameter(
            "oid",
            "danmaku oid must be non-zero",
        ));
    }
    Ok(())
}

fn validate_date(value: &str) -> BpiResult<()> {
    let bytes = value.as_bytes();
    let valid = bytes.len() == 10
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[7] == b'-'
        && bytes[8..10].iter().all(u8::is_ascii_digit);
    if !valid {
        return Err(BpiError::invalid_parameter(
            "date",
            "date must use YYYY-MM-DD format",
        ));
    }

    let month = value[5..7]
        .parse::<u8>()
        .map_err(|_| BpiError::invalid_parameter("date", "date must use YYYY-MM-DD format"))?;
    let day = value[8..10]
        .parse::<u8>()
        .map_err(|_| BpiError::invalid_parameter("date", "date must use YYYY-MM-DD format"))?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return Err(BpiError::invalid_parameter(
            "date",
            "date month/day is out of range",
        ));
    }

    Ok(())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::BpiClient;
    use base64::{Engine as _, engine::general_purpose};
    use serde::Deserialize;
    use std::collections::BTreeMap;

    const TEST_OID: u64 = 16546;
    const TEST_DATE: &str = "2022-01-01";

    #[derive(Debug, Deserialize)]
    struct BinaryProbeBody {
        kind: String,
        encoding: String,
        content_type: Option<String>,
        length: usize,
        body_base64: String,
    }

    fn contract(endpoint: &str) -> BpiResult<crate::probe::endpoint_contract::EndpointContract> {
        let bytes = match endpoint {
            "web-seg" => {
                include_bytes!("../../tests/contracts/danmaku/non-json-read/web-seg/contract.json")
                    .as_slice()
            }
            "web-seg-wbi" => include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/web-seg-wbi/contract.json"
            )
            .as_slice(),
            "web-view" => {
                include_bytes!("../../tests/contracts/danmaku/non-json-read/web-view/contract.json")
                    .as_slice()
            }
            "mobile-seg" => include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/mobile-seg/contract.json"
            )
            .as_slice(),
            "web-history-seg" => include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/web-history-seg/contract.json"
            )
            .as_slice(),
            "history-xml" => {
                include_bytes!("../../tests/contracts/danmaku/history-xml/contract.json").as_slice()
            }
            _ => unreachable!("unknown danmaku non-json contract"),
        };
        crate::probe::endpoint_contract::EndpointContract::from_slice(bytes)
    }

    fn query_map(params: Vec<(String, String)>) -> BTreeMap<String, String> {
        params.into_iter().collect()
    }

    fn assert_binary_fixture(bytes: &[u8]) -> BpiResult<Vec<u8>> {
        assert_binary_fixture_with_content_type(bytes, Some("application/octet-stream"))
    }

    fn assert_binary_fixture_with_content_type(
        bytes: &[u8],
        content_type: Option<&str>,
    ) -> BpiResult<Vec<u8>> {
        let body: BinaryProbeBody = serde_json::from_slice(bytes)?;
        assert_eq!(body.kind, "binary");
        assert_eq!(body.encoding, "base64");
        assert_eq!(body.content_type.as_deref(), content_type);

        let decoded = general_purpose::STANDARD
            .decode(body.body_base64)
            .map_err(|err| BpiError::parse(err.to_string()))?;
        assert_eq!(decoded.len(), body.length);
        assert!(!decoded.is_empty());
        Ok(decoded)
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_danmaku_web_seg_proto() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let params = DanmakuSegmentParams::new(1, TEST_OID, 1)?;
        let data = bpi.danmaku().web_seg_proto(params).await?;

        assert!(!data.is_empty(), "protobuf 响应不应为空");
        tracing::info!("web seg.so 响应字节数: {}", data.len());

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_danmaku_web_seg_wbi_proto() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let params = DanmakuSegmentParams::new(1, TEST_OID, 1)?;
        let data = bpi.danmaku().web_seg_wbi_proto(params).await?;

        assert!(!data.is_empty(), "protobuf 响应不应为空");
        tracing::info!("wbi web seg.so 响应字节数: {}", data.len());

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_danmaku_web_view_proto() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let params = DanmakuWebViewParams::new(1, TEST_OID)?;
        let data = bpi.danmaku().web_view_proto(params).await?;

        assert!(!data.is_empty(), "protobuf 响应不应为空");
        tracing::info!("web/view 响应字节数: {}", data.len());

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_danmaku_mobile_seg_proto() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let params = DanmakuSegmentParams::new(1, TEST_OID, 1)?;
        let data = bpi.danmaku().mobile_seg_proto(params).await?;

        assert!(!data.is_empty(), "protobuf 响应不应为空");
        tracing::info!("mobile seg.so 响应字节数: {}", data.len());

        Ok(())
    }

    #[test]
    fn danmaku_segment_params_serializes_required_query() -> Result<(), BpiError> {
        let params = DanmakuSegmentParams::new(1, TEST_OID, 1)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("type".to_string(), "1".to_string()),
                ("oid".to_string(), TEST_OID.to_string()),
                ("segment_index".to_string(), "1".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn danmaku_segment_params_serializes_optional_query() -> Result<(), BpiError> {
        let params = DanmakuSegmentParams::new(1, TEST_OID, 2)?
            .pid(590635620)?
            .pull_mode(1)
            .range(0, 360_000)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("type".to_string(), "1".to_string()),
                ("oid".to_string(), TEST_OID.to_string()),
                ("segment_index".to_string(), "2".to_string()),
                ("pid".to_string(), "590635620".to_string()),
                ("pull_mode".to_string(), "1".to_string()),
                ("ps".to_string(), "0".to_string()),
                ("pe".to_string(), "360000".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn danmaku_segment_params_rejects_zero_segment_index() {
        let err = DanmakuSegmentParams::new(1, TEST_OID, 0).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "segment_index",
                ..
            }
        ));
    }

    #[test]
    fn danmaku_web_view_params_serializes_query() -> Result<(), BpiError> {
        let params = DanmakuWebViewParams::new(1, TEST_OID)?.pid(590635620)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("type".to_string(), "1".to_string()),
                ("oid".to_string(), TEST_OID.to_string()),
                ("pid".to_string(), "590635620".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn danmaku_history_bytes_params_serializes_query() -> Result<(), BpiError> {
        let params = DanmakuHistoryBytesParams::new(1, TEST_OID, TEST_DATE)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("type".to_string(), "1".to_string()),
                ("oid".to_string(), TEST_OID.to_string()),
                ("date".to_string(), TEST_DATE.to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn danmaku_history_bytes_params_rejects_invalid_date() -> Result<(), BpiError> {
        let err = DanmakuHistoryBytesParams::new(1, TEST_OID, "2022-13-01").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "date", .. }
        ));
        Ok(())
    }

    #[test]
    fn danmaku_non_json_contracts_match_endpoint_requests() -> BpiResult<()> {
        let web_seg = contract("web-seg")?;
        let segment_params = DanmakuSegmentParams::new(1, TEST_OID, 1)?;
        assert_eq!(web_seg.name, "danmaku.web.seg");
        assert_eq!(
            web_seg.request.url.as_str(),
            "https://api.bilibili.com/x/v2/dm/web/seg.so"
        );
        assert_eq!(
            query_map(segment_params.query_pairs()),
            web_seg.request.query
        );

        let web_seg_wbi = contract("web-seg-wbi")?;
        assert_eq!(web_seg_wbi.name, "danmaku.web.seg_wbi");
        assert_eq!(
            web_seg_wbi.request.url.as_str(),
            "https://api.bilibili.com/x/v2/dm/wbi/web/seg.so"
        );
        assert_eq!(
            query_map(DanmakuSegmentParams::new(1, TEST_OID, 1)?.query_pairs()),
            web_seg_wbi.request.query
        );
        assert!(web_seg_wbi.request.auth.requires_wbi());

        let web_view = contract("web-view")?;
        let view_params = DanmakuWebViewParams::new(1, TEST_OID)?;
        assert_eq!(web_view.name, "danmaku.web.view");
        assert_eq!(
            web_view.request.url.as_str(),
            "https://api.bilibili.com/x/v2/dm/web/view"
        );
        assert_eq!(query_map(view_params.query_pairs()), web_view.request.query);

        let mobile_seg = contract("mobile-seg")?;
        assert_eq!(mobile_seg.name, "danmaku.mobile.seg");
        assert_eq!(
            mobile_seg.request.url.as_str(),
            "https://api.bilibili.com/x/v2/dm/list/seg.so"
        );
        assert_eq!(
            query_map(DanmakuSegmentParams::new(1, TEST_OID, 1)?.query_pairs()),
            mobile_seg.request.query
        );

        let history_seg = contract("web-history-seg")?;
        let history_params = DanmakuHistoryBytesParams::new(1, TEST_OID, TEST_DATE)?;
        assert_eq!(history_seg.name, "danmaku.web.history_seg");
        assert_eq!(
            history_seg.request.url.as_str(),
            "https://api.bilibili.com/x/v2/dm/web/history/seg.so"
        );
        assert_eq!(
            query_map(history_params.query_pairs()),
            history_seg.request.query
        );

        let history_xml = contract("history-xml")?;
        assert_eq!(history_xml.name, "danmaku.history.xml");
        assert_eq!(
            history_xml.request.url.as_str(),
            "https://api.bilibili.com/x/v2/dm/history"
        );
        assert_eq!(
            query_map(DanmakuHistoryBytesParams::new(1, TEST_OID, TEST_DATE)?.query_pairs()),
            history_xml.request.query
        );
        assert_eq!(
            history_xml.request.response_decoding,
            crate::probe::contract::ResponseDecoding::Disabled
        );
        Ok(())
    }

    #[test]
    fn danmaku_non_json_response_fixtures_preserve_binary_bodies() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/web-seg/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/web-seg/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/web-seg/responses/vip.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/web-seg-wbi/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/web-seg-wbi/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/web-seg-wbi/responses/vip.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/web-view/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/web-view/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/web-view/responses/vip.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/mobile-seg/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/mobile-seg/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/mobile-seg/responses/vip.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/web-history-seg/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/non-json-read/web-history-seg/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            assert_binary_fixture(bytes)?;
        }
        Ok(())
    }

    #[test]
    fn danmaku_history_xml_fixtures_preserve_raw_compressed_xml() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/danmaku/history-xml/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!("../../tests/contracts/danmaku/history-xml/responses/vip.success.json")
                .as_slice(),
        ] {
            let decoded = assert_binary_fixture_with_content_type(bytes, Some("text/xml"))?;
            assert!(!decoded.starts_with(b"{"));
        }
        Ok(())
    }

    #[test]
    fn danmaku_web_history_seg_anonymous_fixture_records_login_error() -> BpiResult<()> {
        let err = crate::response::ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/danmaku/non-json-read/web-history-seg/responses/anonymous.requires_login.json"
        ))
        .and_then(crate::response::ApiEnvelope::ensure_success)
        .unwrap_err();

        assert!(err.requires_login());
        Ok(())
    }

    #[test]
    fn danmaku_history_xml_anonymous_fixture_records_login_error() -> BpiResult<()> {
        let err = crate::response::ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/danmaku/history-xml/responses/anonymous.requires_login.json"
        ))
        .and_then(crate::response::ApiEnvelope::ensure_success)
        .unwrap_err();

        assert!(err.requires_login());
        Ok(())
    }
}
