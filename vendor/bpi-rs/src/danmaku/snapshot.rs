//! 弹幕快照（最近产生的几条弹幕，最多20条）
//!
//! [文档入口](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/danmaku)

use crate::ids::{Aid, Bvid};

/// `/x/v2/dm/ajax` 弹幕快照的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DanmakuSnapshotParams {
    target: DanmakuSnapshotTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DanmakuSnapshotTarget {
    Aid(Aid),
    Bvid(Bvid),
}

impl DanmakuSnapshotParams {
    pub fn from_aid(aid: Aid) -> Self {
        Self {
            target: DanmakuSnapshotTarget::Aid(aid),
        }
    }

    pub fn from_bvid(bvid: Bvid) -> Self {
        Self {
            target: DanmakuSnapshotTarget::Bvid(bvid),
        }
    }

    pub fn query_pairs(&self) -> [(&'static str, String); 1] {
        let value = match &self.target {
            DanmakuSnapshotTarget::Aid(aid) => aid.to_string(),
            DanmakuSnapshotTarget::Bvid(bvid) => bvid.to_string(),
        };

        [("aid", value)]
    }
}

#[cfg(test)]
pub mod tests {
    use crate::danmaku::DanmakuSnapshotParams;
    use crate::ids::{Aid, Bvid};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::response::ApiEnvelope;
    use crate::{BpiClient, BpiError, BpiResult};
    use std::collections::BTreeMap;

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/danmaku/json-read/snapshot/contract.json"
        ))
    }

    fn query_map(params: [(&'static str, String); 1]) -> BTreeMap<String, String> {
        params
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect()
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_danmaku_snapshot() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let bvid: Bvid = "BV1fK4y1t741".parse()?;
        let params = DanmakuSnapshotParams::from_bvid(bvid);
        let data = bpi.danmaku().snapshot(params).await?;
        tracing::info!("{:#?}", data);

        Ok(())
    }

    #[test]
    fn danmaku_snapshot_params_serializes_bvid_query() -> Result<(), BpiError> {
        let bvid: Bvid = "BV1fK4y1t741".parse()?;
        let params = DanmakuSnapshotParams::from_bvid(bvid);

        assert_eq!(params.query_pairs(), [("aid", "BV1fK4y1t741".to_string())]);
        Ok(())
    }

    #[test]
    fn danmaku_snapshot_params_serializes_aid_query() -> Result<(), BpiError> {
        let params = DanmakuSnapshotParams::from_aid(Aid::new(170001)?);

        assert_eq!(params.query_pairs(), [("aid", "170001".to_string())]);
        Ok(())
    }

    #[test]
    fn danmaku_snapshot_contract_matches_endpoint_request() -> Result<(), BpiError> {
        let contract = contract()?;
        let bvid: Bvid = "BV1fK4y1t741".parse()?;
        let params = DanmakuSnapshotParams::from_bvid(bvid);

        assert_eq!(contract.name, "danmaku.snapshot");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/v2/dm/ajax"
        );
        assert_eq!(query_map(params.query_pairs()), contract.request.query);
        Ok(())
    }

    #[test]
    fn danmaku_snapshot_response_fixtures_parse_declared_model() -> Result<(), BpiError> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/danmaku/json-read/snapshot/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/json-read/snapshot/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/danmaku/json-read/snapshot/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<Vec<String>>::from_slice(bytes)?.into_payload()?;
            assert!(payload.is_empty());
        }
        Ok(())
    }
}
