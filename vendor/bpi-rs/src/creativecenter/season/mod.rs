pub mod action;
pub mod aid;

pub mod edit;

pub mod info;
pub mod list;
pub mod models;
pub mod params;

pub mod section;

pub use aid::SeasonByAidData;
pub use info::{SeasonInfoData, Sections};
pub use models::{Season, Section};
pub use params::{
    SeasonByAidParams, SeasonInfoParams, SeasonListOrder, SeasonListParams, SeasonListSort,
    SeasonSectionEpisodesParams,
};

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::BpiResult;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::response::ApiEnvelope;

    use super::aid::SeasonByAidData;
    use super::info::SeasonInfoData;
    use super::list::SeasonListData;
    use super::section::SeasonSectionEpisodesData;
    use super::{
        SeasonByAidParams, SeasonInfoParams, SeasonListOrder, SeasonListParams, SeasonListSort,
        SeasonSectionEpisodesParams,
    };
    use crate::ids::{Aid, SeasonId};

    const TEST_AID: u64 = 113602455409683;
    const TEST_SEASON_ID: u64 = 4294056;
    const TEST_SECTION_ID: u64 = 176088;

    fn contract(name: &str) -> BpiResult<EndpointContract> {
        let bytes = match name {
            "list" => {
                include_bytes!("../../../tests/contracts/creativecenter/season/list/contract.json")
                    .as_slice()
            }
            "info" => {
                include_bytes!("../../../tests/contracts/creativecenter/season/info/contract.json")
                    .as_slice()
            }
            "aid" => {
                include_bytes!("../../../tests/contracts/creativecenter/season/aid/contract.json")
                    .as_slice()
            }
            "section" => include_bytes!(
                "../../../tests/contracts/creativecenter/season/section/contract.json"
            )
            .as_slice(),
            _ => unreachable!("unknown creativecenter season contract"),
        };
        EndpointContract::from_slice(bytes)
    }

    fn query_map<I>(params: I) -> BTreeMap<String, String>
    where
        I: IntoIterator<Item = (&'static str, String)>,
    {
        params
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect()
    }

    #[test]
    fn creativecenter_season_contracts_match_endpoint_requests() -> BpiResult<()> {
        let list = contract("list")?;
        let list_params = SeasonListParams::new(1, 10)?
            .with_order(SeasonListOrder::CreatedAt)
            .with_sort(SeasonListSort::Desc);
        assert_eq!(list.name, "creativecenter.season.list");
        assert_eq!(list.request.method, HttpMethod::Get);
        assert_eq!(
            list.request.url.as_str(),
            "https://member.bilibili.com/x2/creative/web/seasons"
        );
        assert_eq!(query_map(list_params.query_pairs()), list.request.query);

        let info = contract("info")?;
        let info_params = SeasonInfoParams::new(SeasonId::new(TEST_SEASON_ID)?);
        assert_eq!(info.name, "creativecenter.season.info");
        assert_eq!(
            info.request.url.as_str(),
            "https://member.bilibili.com/x2/creative/web/season"
        );
        assert_eq!(query_map(info_params.query_pairs()), info.request.query);

        let aid = contract("aid")?;
        let aid_params = SeasonByAidParams::new(Aid::new(TEST_AID)?);
        assert_eq!(aid.name, "creativecenter.season.aid");
        assert_eq!(
            aid.request.url.as_str(),
            "https://member.bilibili.com/x2/creative/web/season/aid"
        );
        assert_eq!(query_map(aid_params.query_pairs()), aid.request.query);

        let section = contract("section")?;
        let section_params = SeasonSectionEpisodesParams::new(SeasonId::new(TEST_SECTION_ID)?);
        assert_eq!(section.name, "creativecenter.season.section");
        assert_eq!(
            section.request.url.as_str(),
            "https://member.bilibili.com/x2/creative/web/season/section"
        );
        assert_eq!(
            query_map(section_params.query_pairs()),
            section.request.query
        );
        Ok(())
    }

    #[test]
    fn creativecenter_season_response_fixtures_parse_declared_models() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../../tests/contracts/creativecenter/season/list/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../../tests/contracts/creativecenter/season/list/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<SeasonListData>::from_slice(bytes)?.into_payload()?;
            assert_eq!(payload.seasons.len(), 1);
        }

        let info = ApiEnvelope::<SeasonInfoData>::from_slice(include_bytes!(
            "../../../tests/contracts/creativecenter/season/info/responses/vip.success.json"
        ))?
        .into_payload()?;
        assert_eq!(info.season.id, 1);

        let aid = ApiEnvelope::<SeasonByAidData>::from_slice(include_bytes!(
            "../../../tests/contracts/creativecenter/season/aid/responses/vip.success.json"
        ))?
        .into_payload()?;
        assert_eq!(aid.id, 1);

        for bytes in [
            include_bytes!(
                "../../../tests/contracts/creativecenter/season/section/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../../tests/contracts/creativecenter/season/section/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload =
                ApiEnvelope::<SeasonSectionEpisodesData>::from_slice(bytes)?.into_payload()?;
            assert_eq!(payload.episodes.as_ref().map(Vec::len), Some(1));
        }
        Ok(())
    }

    #[test]
    fn creativecenter_season_error_fixtures_preserve_observed_api_errors() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../../tests/contracts/creativecenter/season/list/responses/anonymous.requires_login.json"
            )
            .as_slice(),
            include_bytes!(
                "../../../tests/contracts/creativecenter/season/info/responses/anonymous.requires_login.json"
            )
            .as_slice(),
            include_bytes!(
                "../../../tests/contracts/creativecenter/season/aid/responses/anonymous.requires_login.json"
            )
            .as_slice(),
            include_bytes!(
                "../../../tests/contracts/creativecenter/season/section/responses/anonymous.requires_login.json"
            )
            .as_slice(),
        ] {
            let err = ApiEnvelope::<serde_json::Value>::from_slice(bytes)
                .and_then(ApiEnvelope::ensure_success)
                .unwrap_err();
            assert!(err.requires_login());
        }

        let not_found = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../../tests/contracts/creativecenter/season/info/responses/normal.not_found.json"
        ))
        .and_then(ApiEnvelope::ensure_success)
        .unwrap_err();
        assert_eq!(not_found.code(), Some(-404));

        let not_owner = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../../tests/contracts/creativecenter/season/aid/responses/normal.not_owner.json"
        ))
        .and_then(ApiEnvelope::ensure_success)
        .unwrap_err();
        assert_eq!(not_owner.code(), Some(20103));

        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path = format!(
            "target/bpi-probe-runs/creativecenter/season-read/{endpoint}/{profile}.response.json"
        );
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn creativecenter_season_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["normal", "vip"] {
            if let Some(body) = local_probe_body("list", profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<SeasonListData>>(body)?.into_payload()?;
                assert!(!payload.seasons.is_empty());
            }

            if let Some(body) = local_probe_body("section", profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<SeasonSectionEpisodesData>>(body)?
                        .into_payload()?;
                assert!(
                    payload
                        .episodes
                        .as_ref()
                        .is_some_and(|episodes| !episodes.is_empty())
                );
            }
        }

        if let Some(body) = local_probe_body("info", "vip") {
            let payload =
                serde_json::from_value::<ApiEnvelope<SeasonInfoData>>(body)?.into_payload()?;
            assert_eq!(payload.season.id, TEST_SEASON_ID);
        }

        if let Some(body) = local_probe_body("aid", "vip") {
            let payload =
                serde_json::from_value::<ApiEnvelope<SeasonByAidData>>(body)?.into_payload()?;
            assert_eq!(payload.id, TEST_SEASON_ID);
        }
        Ok(())
    }
}
