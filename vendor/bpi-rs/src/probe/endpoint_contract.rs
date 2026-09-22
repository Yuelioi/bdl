use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::probe::contract::{ApiContract, ContractAuth, ContractRequest};
use crate::{BpiError, BpiResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EndpointContract {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_version: Option<u32>,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk: Option<ApiRisk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ContractStatus>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<String>,
    pub request: ContractRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sanitize: Option<SanitizeSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<ContractProvenance>,
    pub cases: Vec<EndpointCase>,
}

impl EndpointContract {
    pub fn from_slice(bytes: &[u8]) -> BpiResult<Self> {
        let raw: RawEndpointContract = serde_json::from_slice(bytes)?;
        let contract: EndpointContract = raw.try_into()?;
        if contract.cases.is_empty() {
            return Err(BpiError::invalid_parameter(
                "cases",
                "endpoint contract must contain at least one response case",
            ));
        }
        Ok(contract)
    }
}

#[derive(Debug, Deserialize)]
struct RawEndpointContract {
    #[serde(default)]
    schema_version: Option<u32>,
    name: String,
    #[serde(default)]
    module: Option<String>,
    #[serde(default)]
    batch: Option<String>,
    #[serde(default)]
    endpoint: Option<String>,
    #[serde(default)]
    risk: Option<ApiRisk>,
    #[serde(default)]
    status: Option<ContractStatus>,
    #[serde(default)]
    profiles: Vec<String>,
    request: serde_json::Value,
    #[serde(default)]
    sanitize: Option<SanitizeSpec>,
    #[serde(default)]
    provenance: Option<ContractProvenance>,
    cases: Vec<EndpointCase>,
}

impl TryFrom<RawEndpointContract> for EndpointContract {
    type Error = BpiError;

    fn try_from(raw: RawEndpointContract) -> Result<Self, Self::Error> {
        let request_contract = ApiContract::from_value(serde_json::json!({
            "name": raw.name,
            "request": raw.request,
            "expect": {}
        }))?;

        let contract = Self {
            schema_version: raw.schema_version,
            name: request_contract.name,
            module: raw.module,
            batch: raw.batch,
            endpoint: raw.endpoint,
            risk: raw.risk,
            status: raw.status,
            profiles: raw.profiles,
            request: request_contract.request,
            sanitize: raw.sanitize,
            provenance: raw.provenance,
            cases: raw.cases,
        };
        contract.validate_metadata()?;
        contract.validate_response_errors()?;
        Ok(contract)
    }
}

impl EndpointContract {
    fn validate_metadata(&self) -> BpiResult<()> {
        match self.schema_version {
            None => Ok(()),
            Some(2) => self.validate_v2_metadata(),
            Some(_) => Err(BpiError::invalid_parameter(
                "schema_version",
                "supported endpoint contract schema versions are 2 or omitted legacy v1",
            )),
        }
    }

    fn validate_v2_metadata(&self) -> BpiResult<()> {
        if self.module.as_deref().is_none_or(str::is_empty) {
            return Err(BpiError::invalid_parameter(
                "module",
                "v2 endpoint contract must declare module",
            ));
        }
        if self.batch.as_deref().is_none_or(str::is_empty) {
            return Err(BpiError::invalid_parameter(
                "batch",
                "v2 endpoint contract must declare batch",
            ));
        }
        if self.endpoint.as_deref().is_none_or(str::is_empty) {
            return Err(BpiError::invalid_parameter(
                "endpoint",
                "v2 endpoint contract must declare endpoint",
            ));
        }
        if self.risk.is_none() {
            return Err(BpiError::invalid_parameter(
                "risk",
                "v2 endpoint contract must declare risk",
            ));
        }
        if self.status.is_none() {
            return Err(BpiError::invalid_parameter(
                "status",
                "v2 endpoint contract must declare status",
            ));
        }
        if self.profiles.is_empty() {
            return Err(BpiError::invalid_parameter(
                "profiles",
                "v2 endpoint contract must declare profiles",
            ));
        }
        Ok(())
    }

    fn validate_response_errors(&self) -> BpiResult<()> {
        for case in &self.cases {
            case.response.validate_error_label()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApiRisk {
    PublicRead,
    AuthenticatedRead,
    PrivateRead,
    Mutating,
    Spending,
    LoginSession,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractStatus {
    Draft,
    Probed,
    Promoted,
    Blocked,
    Deprecated,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SanitizeSpec {
    #[serde(default)]
    pub preset: Vec<String>,
    #[serde(default)]
    pub replace: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub drop: Vec<String>,
    #[serde(default)]
    pub keep: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ContractProvenance {
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub observed_at: Option<String>,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub tool_version: Option<String>,
    #[serde(default)]
    pub privacy: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointCase {
    pub name: String,
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub auth: ContractAuth,
    pub response: EndpointResponse,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EndpointResponse {
    #[serde(default)]
    pub http_status: Option<u16>,
    #[serde(default)]
    pub api_code: Option<i32>,
    #[serde(default)]
    pub api_code_text: Option<String>,
    #[serde(default)]
    pub fixture: Option<String>,
    #[serde(default)]
    pub fixture_kind: Option<String>,
    #[serde(default)]
    pub rust_model: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

const STABLE_SEMANTIC_ERROR_LABELS: &[&str] = &[
    "requires_login",
    "requires_vip",
    "risk_control",
    "permission_denied",
    "business_error",
];

const DOMAIN_ERROR_LABELS: &[&str] = &[
    "missing_model_fields",
    "not_admin",
    "not_found",
    "not_note_owner",
    "not_owner",
    "wbi_risk_control",
];

impl EndpointResponse {
    fn validate_error_label(&self) -> BpiResult<()> {
        let Some(label) = self.error.as_deref() else {
            return Ok(());
        };

        if !is_allowed_error_label(label) {
            return Err(invalid_error_label());
        }

        if STABLE_SEMANTIC_ERROR_LABELS.contains(&label)
            && let Some(observed) = self.observed_semantic_error()
            && observed != label
        {
            return Err(invalid_error_label());
        }

        Ok(())
    }

    fn observed_semantic_error(&self) -> Option<&'static str> {
        self.api_code
            .and_then(|code| BpiError::from_code(code).semantic_error())
            .or_else(|| {
                self.http_status
                    .and_then(|status| BpiError::http(status).semantic_error())
            })
    }
}

fn is_allowed_error_label(label: &str) -> bool {
    STABLE_SEMANTIC_ERROR_LABELS.contains(&label) || DOMAIN_ERROR_LABELS.contains(&label)
}

fn invalid_error_label() -> BpiError {
    BpiError::invalid_parameter(
        "response.error",
        "response error label must be a supported stable semantic or domain-specific label",
    )
}

#[cfg(all(test, feature = "login"))]
mod tests {
    use super::*;
    use crate::login::LoginVipInfo;
    use crate::response::ApiEnvelope;

    #[test]
    fn endpoint_contract_groups_profiles_under_one_request() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/login/vip-info/contract.json"
        ))?;

        assert_eq!(contract.schema_version, Some(2));
        assert_eq!(contract.name, "login.vip_info");
        assert_eq!(contract.module.as_deref(), Some("login"));
        assert_eq!(contract.batch.as_deref(), Some("vip-info"));
        assert_eq!(contract.endpoint.as_deref(), Some("vip-info"));
        assert_eq!(contract.risk, Some(ApiRisk::AuthenticatedRead));
        assert_eq!(contract.status, Some(ContractStatus::Promoted));
        assert_eq!(contract.profiles, ["anonymous", "normal", "vip"]);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/vip/web/user/info"
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(contract.cases[0].name, "anonymous");
        assert_eq!(contract.cases[0].response.api_code, Some(-101));
        assert_eq!(
            contract.cases[2].response.fixture.as_deref(),
            Some("responses/vip.success.json")
        );
        Ok(())
    }

    #[test]
    fn endpoint_contract_deserializes_v2_metadata() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(
            br#"{
                "schema_version": 2,
                "name": "video.view",
                "module": "video",
                "batch": "info-read",
                "endpoint": "view",
                "risk": "public-read",
                "status": "promoted",
                "profiles": ["anonymous"],
                "request": {
                    "method": "GET",
                    "url": "https://api.bilibili.com/x/web-interface/view"
                },
                "sanitize": {
                    "preset": ["account"],
                    "replace": {
                        "$.data.owner.mid": 1000001,
                        "$.data.owner.name": "sanitized-user"
                    },
                    "drop": ["$.data.email"],
                    "keep": ["$.data.title"]
                },
                "provenance": {
                    "source": "local_probe_output",
                    "observed_at": "2026-07-06",
                    "tool": "bpi-probe",
                    "tool_version": "0.2.0",
                    "privacy": "account fields sanitized before commit"
                },
                "cases": [
                    {
                        "name": "anonymous",
                        "response": {
                            "http_status": 200,
                            "api_code": 0
                        }
                    }
                ]
            }"#,
        )?;

        assert_eq!(contract.schema_version, Some(2));
        assert_eq!(contract.risk, Some(ApiRisk::PublicRead));
        assert_eq!(contract.status, Some(ContractStatus::Promoted));
        assert_eq!(contract.profiles, ["anonymous"]);

        let sanitize = contract
            .sanitize
            .as_ref()
            .ok_or_else(|| BpiError::unsupported_response("missing sanitize"))?;
        assert_eq!(sanitize.preset, ["account"]);
        assert_eq!(sanitize.replace["$.data.owner.mid"], 1000001);
        assert_eq!(sanitize.replace["$.data.owner.name"], "sanitized-user");
        assert_eq!(sanitize.drop, ["$.data.email"]);
        assert_eq!(sanitize.keep, ["$.data.title"]);

        let provenance = contract
            .provenance
            .as_ref()
            .ok_or_else(|| BpiError::unsupported_response("missing provenance"))?;
        assert_eq!(provenance.source.as_deref(), Some("local_probe_output"));
        assert_eq!(provenance.tool.as_deref(), Some("bpi-probe"));
        Ok(())
    }

    #[test]
    fn endpoint_contract_success_fixture_parses_declared_rust_model() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/login/vip-info/contract.json"
        ))?;
        let vip_case = contract
            .cases
            .iter()
            .find(|case| case.name == "vip")
            .ok_or_else(|| BpiError::unsupported_response("missing vip contract case"))?;

        assert_eq!(
            vip_case.response.rust_model.as_deref(),
            Some("LoginVipInfo")
        );

        let payload = ApiEnvelope::<LoginVipInfo>::from_slice(include_bytes!(
            "../../tests/contracts/login/vip-info/responses/vip.success.json"
        ))?
        .into_payload()?;

        assert!(payload.is_active());
        Ok(())
    }

    #[test]
    fn endpoint_contract_supports_non_numeric_api_code_response() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(
            br#"{
                "name": "manga.coupons",
                "request": {
                    "method": "POST",
                    "url": "https://manga.bilibili.com/twirp/user.v1.User/GetCoupons",
                    "auth": { "requires": [] }
                },
                "cases": [
                    {
                        "name": "anonymous",
                        "profile": "anonymous",
                        "auth": { "requires": [] },
                        "response": {
                            "http_status": 401,
                            "api_code_text": "unauthenticated",
                            "fixture": "responses/anonymous.requires_login.json",
                            "fixture_kind": "probe_error_body",
                            "error": "requires_login"
                        }
                    }
                ]
            }"#,
        )?;

        let response = &contract.cases[0].response;

        assert_eq!(response.http_status, Some(401));
        assert_eq!(response.api_code, None);
        assert_eq!(response.api_code_text.as_deref(), Some("unauthenticated"));
        Ok(())
    }

    #[test]
    fn endpoint_contract_rejects_error_label_that_disagrees_with_observed_api_code() {
        let err = EndpointContract::from_slice(
            br#"{
                "name": "login.vip_info",
                "request": {
                    "method": "GET",
                    "url": "https://api.bilibili.com/x/vip/web/user/info"
                },
                "cases": [
                    {
                        "name": "anonymous",
                        "response": {
                            "http_status": 200,
                            "api_code": -101,
                            "error": "risk_control"
                        }
                    }
                ]
            }"#,
        )
        .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "response.error",
                ..
            }
        ));
    }

    #[test]
    fn endpoint_contract_rejects_unknown_error_label() {
        let err = EndpointContract::from_slice(
            br#"{
                "name": "login.vip_info",
                "request": {
                    "method": "GET",
                    "url": "https://api.bilibili.com/x/vip/web/user/info"
                },
                "cases": [
                    {
                        "name": "anonymous",
                        "response": {
                            "http_status": 401,
                            "error": "login_required"
                        }
                    }
                ]
            }"#,
        )
        .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "response.error",
                ..
            }
        ));
    }

    #[test]
    fn endpoint_contract_rejects_incomplete_v2_metadata() {
        let err = EndpointContract::from_slice(
            br#"{
                "schema_version": 2,
                "name": "video.view",
                "request": {
                    "method": "GET",
                    "url": "https://api.bilibili.com/x/web-interface/view"
                },
                "cases": [
                    {
                        "name": "anonymous",
                        "response": {
                            "http_status": 200,
                            "api_code": 0
                        }
                    }
                ]
            }"#,
        )
        .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "module",
                ..
            }
        ));
    }

    #[test]
    fn endpoint_contract_rejects_unknown_schema_version() {
        let err = EndpointContract::from_slice(
            br#"{
                "schema_version": 99,
                "name": "video.view",
                "request": {
                    "method": "GET",
                    "url": "https://api.bilibili.com/x/web-interface/view"
                },
                "cases": [
                    {
                        "name": "anonymous",
                        "response": {
                            "http_status": 200,
                            "api_code": 0
                        }
                    }
                ]
            }"#,
        )
        .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "schema_version",
                ..
            }
        ));
    }
}
