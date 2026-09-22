//! 弹幕点赞查询
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/danmaku/thumbup.md)

use crate::ids::Cid;
use crate::{BpiError, BpiResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbupStatsItem {
    /// 对应弹幕所获得的点赞数
    pub likes: i64,
    /// 当前用户是否点赞
    pub user_like: i32,
    pub id_str: String,
}

pub type ThumbupStatsMap = HashMap<String, ThumbupStatsItem>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DanmakuThumbupStatsParams {
    oid: Cid,
    ids: Vec<u64>,
}

impl DanmakuThumbupStatsParams {
    pub fn new(oid: Cid, ids: impl IntoIterator<Item = u64>) -> BpiResult<Self> {
        let ids = ids.into_iter().collect::<Vec<_>>();
        if ids.is_empty() {
            return Err(BpiError::invalid_parameter(
                "ids",
                "at least one danmaku id is required",
            ));
        }
        if ids.contains(&0) {
            return Err(BpiError::invalid_parameter(
                "ids",
                "danmaku ids must be non-zero",
            ));
        }

        Ok(Self { oid, ids })
    }

    pub(crate) fn query_pairs(&self) -> [(&'static str, String); 2] {
        let ids = self
            .ids
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        [("oid", self.oid.to_string()), ("ids", ids)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::response::ApiEnvelope;
    use std::collections::BTreeMap;

    const TEST_CID: u64 = 413195701;
    const TEST_DMID: u64 = 1932011031958944000;

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/danmaku/json-read/thumbup-stats/contract.json"
        ))
    }

    fn query_map(params: [(&'static str, String); 2]) -> BTreeMap<String, String> {
        params
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect()
    }

    #[test]
    fn danmaku_thumbup_stats_params_serializes_query() -> BpiResult<()> {
        let params = DanmakuThumbupStatsParams::new(Cid::new(TEST_CID)?, [TEST_DMID])?;

        assert_eq!(
            params.query_pairs(),
            [
                ("oid", TEST_CID.to_string()),
                ("ids", TEST_DMID.to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn danmaku_thumbup_stats_params_rejects_empty_ids() -> BpiResult<()> {
        let err = DanmakuThumbupStatsParams::new(Cid::new(TEST_CID)?, []).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "ids", .. }
        ));
        Ok(())
    }

    #[test]
    fn danmaku_thumbup_stats_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params = DanmakuThumbupStatsParams::new(Cid::new(TEST_CID)?, [TEST_DMID])?;

        assert_eq!(contract.name, "danmaku.thumbup.stats");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/v2/dm/thumbup/stats"
        );
        assert_eq!(query_map(params.query_pairs()), contract.request.query);
        Ok(())
    }

    #[test]
    fn danmaku_thumbup_stats_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/danmaku/json-read/thumbup-stats/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/json-read/thumbup-stats/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/json-read/thumbup-stats/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<ThumbupStatsMap>::from_slice(bytes)?.into_payload()?;
            let item = payload
                .get(&TEST_DMID.to_string())
                .ok_or_else(|| BpiError::unsupported_response("missing probed danmaku id"))?;
            assert_eq!(item.id_str, TEST_DMID.to_string());
        }
        Ok(())
    }
}
