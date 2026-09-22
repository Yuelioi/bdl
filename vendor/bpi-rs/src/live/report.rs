use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};

use crate::{BpiError, BpiResult};

// ================= 数据结构 =================

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct HeartBeatData {
    /// 下次心跳间隔
    pub next_interval: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveWebHeartBeatParams {
    room_id: i64,
    next_interval: i32,
    platform: String,
}

impl LiveWebHeartBeatParams {
    pub fn new(room_id: i64) -> BpiResult<Self> {
        let params = Self {
            room_id,
            next_interval: 60,
            platform: "web".to_string(),
        };
        params.validate()?;
        Ok(params)
    }

    pub fn next_interval(mut self, next_interval: i32) -> BpiResult<Self> {
        self.next_interval = next_interval;
        self.validate()?;
        Ok(self)
    }

    pub fn platform(mut self, platform: impl Into<String>) -> BpiResult<Self> {
        self.platform = platform.into();
        self.validate()?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let heart_beat_data = format!("{}|{}|1|0", self.next_interval, self.room_id);
        vec![
            ("hb", general_purpose::STANDARD.encode(heart_beat_data)),
            ("pf", self.platform.clone()),
        ]
    }

    fn validate(&self) -> BpiResult<()> {
        if self.room_id <= 0 {
            return Err(BpiError::invalid_parameter(
                "room_id",
                "room_id must be positive",
            ));
        }

        if self.next_interval <= 0 {
            return Err(BpiError::invalid_parameter(
                "next_interval",
                "next_interval must be positive",
            ));
        }

        if self.platform.trim().is_empty() {
            return Err(BpiError::invalid_parameter(
                "platform",
                "platform cannot be blank",
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/live/telemetry-read/heartbeat/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_web_heart_beat() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .live()
            .web_heart_beat(LiveWebHeartBeatParams::new(23174842)?)
            .await?;

        assert!(data.next_interval > 0);
        Ok(())
    }

    #[test]
    fn live_web_heart_beat_params_serializes_default_query() -> BpiResult<()> {
        let params = LiveWebHeartBeatParams::new(23174842)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("hb", "NjB8MjMxNzQ4NDJ8MXww".to_string()),
                ("pf", "web".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn live_web_heart_beat_params_rejects_invalid_room_id() {
        let err = LiveWebHeartBeatParams::new(0).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "room_id",
                ..
            }
        ));
    }

    #[test]
    fn live_web_heart_beat_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params = LiveWebHeartBeatParams::new(23174842)?;

        assert_eq!(contract.name, "live.web_heart_beat");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://live-trace.bilibili.com/xlive/rdata-interface/v1/heartbeat/webHeartBeat"
        );
        assert_eq!(
            contract.request.query.get("hb").map(String::as_str),
            Some("NjB8MjMxNzQ4NDJ8MXww")
        );
        assert_eq!(
            contract.request.query.get("pf").map(String::as_str),
            Some("web")
        );
        assert_eq!(
            params.query_pairs()[0],
            ("hb", "NjB8MjMxNzQ4NDJ8MXww".to_string())
        );
        assert_eq!(contract.cases.len(), 3);
        for case in &contract.cases {
            assert_eq!(case.response.api_code, Some(0));
            assert_eq!(case.response.rust_model.as_deref(), Some("HeartBeatData"));
        }
        Ok(())
    }

    #[test]
    fn live_web_heart_beat_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<HeartBeatData>::from_slice(include_bytes!(
            "../../tests/contracts/live/telemetry-read/heartbeat/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.next_interval, 60);
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/live/telemetry-read/heartbeat/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn live_web_heart_beat_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<HeartBeatData>>(body)?.into_payload()?;

            assert!(payload.next_interval > 0);
        }
        Ok(())
    }
}
