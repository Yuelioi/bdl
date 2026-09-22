//! 历史弹幕 API
//!
//! [文档入口](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/danmaku)

use crate::ids::Cid;
use crate::{BpiError, BpiResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DanmakuHistoryDatesParams {
    typ: u8,
    oid: Cid,
    month: String,
}

impl DanmakuHistoryDatesParams {
    pub fn new(oid: Cid, month: impl Into<String>) -> BpiResult<Self> {
        let month = month.into();
        validate_month(&month)?;

        Ok(Self { typ: 1, oid, month })
    }

    pub fn danmaku_type(mut self, typ: u8) -> BpiResult<Self> {
        if typ == 0 {
            return Err(BpiError::invalid_parameter(
                "type",
                "danmaku type must be non-zero",
            ));
        }
        self.typ = typ;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> [(&'static str, String); 3] {
        [
            ("type", self.typ.to_string()),
            ("oid", self.oid.to_string()),
            ("month", self.month.clone()),
        ]
    }
}

fn validate_month(value: &str) -> BpiResult<()> {
    let bytes = value.as_bytes();
    let valid = bytes.len() == 7
        && bytes[0..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit);
    if !valid {
        return Err(BpiError::invalid_parameter(
            "month",
            "month must use YYYY-MM format",
        ));
    }

    let month = value[5..7]
        .parse::<u8>()
        .map_err(|_| BpiError::invalid_parameter("month", "month must use YYYY-MM format"))?;
    if !(1..=12).contains(&month) {
        return Err(BpiError::invalid_parameter(
            "month",
            "month must be between 01 and 12",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BpiClient;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::response::ApiEnvelope;
    use std::collections::BTreeMap;

    const TEST_CID: u64 = 772096113;
    const TEST_MONTH: &str = "2022-01";

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/danmaku/json-read/history-dates/contract.json"
        ))
    }

    fn query_map(params: [(&'static str, String); 3]) -> BTreeMap<String, String> {
        params
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect()
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_history_dates() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let params = DanmakuHistoryDatesParams::new(Cid::new(TEST_CID)?, TEST_MONTH)?;
        let data = bpi.danmaku().history_dates(params).await?;
        tracing::info!("{:#?}", data);

        Ok(())
    }

    #[test]
    fn danmaku_history_dates_params_serializes_query() -> BpiResult<()> {
        let params = DanmakuHistoryDatesParams::new(Cid::new(TEST_CID)?, TEST_MONTH)?;

        assert_eq!(
            params.query_pairs(),
            [
                ("type", "1".to_string()),
                ("oid", TEST_CID.to_string()),
                ("month", TEST_MONTH.to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn danmaku_history_dates_params_rejects_invalid_month() -> BpiResult<()> {
        let err = DanmakuHistoryDatesParams::new(Cid::new(TEST_CID)?, "2022-13").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "month", .. }
        ));
        Ok(())
    }

    #[test]
    fn danmaku_history_dates_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params = DanmakuHistoryDatesParams::new(Cid::new(TEST_CID)?, TEST_MONTH)?;

        assert_eq!(contract.name, "danmaku.history.dates");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/v2/dm/history/index"
        );
        assert_eq!(query_map(params.query_pairs()), contract.request.query);
        Ok(())
    }

    #[test]
    fn danmaku_history_dates_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/danmaku/json-read/history-dates/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/json-read/history-dates/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<Vec<String>>::from_slice(bytes)?.into_optional_payload()?;
            assert!(payload.is_none());
        }
        Ok(())
    }

    #[test]
    fn danmaku_history_dates_anonymous_fixture_records_login_error() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/danmaku/json-read/history-dates/responses/anonymous.requires_login.json"
        ))
        .and_then(ApiEnvelope::ensure_success)
        .unwrap_err();

        assert!(err.requires_login());
        Ok(())
    }
}
