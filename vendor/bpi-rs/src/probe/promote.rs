use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::probe::audit::audit_contracts;
use crate::probe::contract::ProbeResult;
use crate::probe::endpoint_contract::ApiRisk;
use crate::probe::sanitize::{audit_value, sanitize_value};
use crate::{BpiError, BpiResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromoteSummary {
    pub contract_path: PathBuf,
    pub fixture_path: PathBuf,
    pub case_name: String,
    pub api_code: Option<i32>,
}

pub fn promote_probe_output(input: impl AsRef<Path>) -> BpiResult<PromoteSummary> {
    promote_probe_output_with_contract_root(input, Path::new("tests/contracts"))
}

pub fn promote_probe_output_with_contract_root(
    input: impl AsRef<Path>,
    contract_root: &Path,
) -> BpiResult<PromoteSummary> {
    let input = input.as_ref();
    let target = PromoteTarget::from_probe_path(input, contract_root)?;
    let bytes = fs::read(input)
        .map_err(|err| BpiError::parse(format!("failed to read {}: {err}", input.display())))?;
    let mut result: ProbeResult = serde_json::from_slice(&bytes)
        .map_err(|err| BpiError::parse(format!("failed to parse {}: {err}", input.display())))?;
    let risk = read_contract_risk(&target.contract_path).unwrap_or(ApiRisk::PublicRead);

    sanitize_probe_result(&mut result, risk);
    let body = result.response.body;
    let findings = audit_value(&body, Some(ApiRisk::PublicRead));
    if !findings.is_empty() {
        return Err(BpiError::unsupported_response(format!(
            "promoted fixture still contains sensitive fields: {findings:?}"
        )));
    }

    let api_code = body
        .get("code")
        .or_else(|| body.get("errno"))
        .and_then(serde_json::Value::as_i64)
        .map(|value| value as i32);
    let fixture_name = fixture_name(&target.case_name, api_code);
    let fixture_path = target
        .contract_path
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join("responses")
        .join(&fixture_name);

    if let Some(parent) = fixture_path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            BpiError::parse(format!("failed to create {}: {err}", parent.display()))
        })?;
    }
    fs::write(
        &fixture_path,
        format!("{}\n", serde_json::to_string_pretty(&body)?),
    )
    .map_err(|err| BpiError::parse(format!("failed to write {}: {err}", fixture_path.display())))?;

    update_contract_case(
        &target.contract_path,
        &target.case_name,
        result.response.status,
        api_code,
        &format!("responses/{fixture_name}"),
    )?;

    let report = audit_contracts(
        target
            .contract_path
            .parent()
            .unwrap_or_else(|| Path::new("")),
    )?;
    if !report.is_success() {
        return Err(BpiError::unsupported_response(format!(
            "promote contract audit failed: {:?}",
            report.findings
        )));
    }

    Ok(PromoteSummary {
        contract_path: target.contract_path,
        fixture_path,
        case_name: target.case_name,
        api_code,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PromoteTarget {
    contract_path: PathBuf,
    case_name: String,
}

impl PromoteTarget {
    fn from_probe_path(path: &Path, contract_root: &Path) -> BpiResult<Self> {
        let file_stem = path
            .file_name()
            .and_then(|value| value.to_str())
            .and_then(|name| name.strip_suffix(".response.json"))
            .ok_or_else(|| {
                BpiError::invalid_parameter(
                    "input",
                    "probe output file name must end with .response.json",
                )
            })?;
        let endpoint_dir = path.parent().ok_or_else(|| {
            BpiError::invalid_parameter("input", "probe output path must include endpoint")
        })?;
        let endpoint = endpoint_dir
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| BpiError::invalid_parameter("input", "missing endpoint path"))?;
        let batch_dir = endpoint_dir
            .parent()
            .ok_or_else(|| BpiError::invalid_parameter("input", "missing batch path"))?;
        let batch = batch_dir
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| BpiError::invalid_parameter("input", "missing batch path"))?;
        let module_dir = batch_dir
            .parent()
            .ok_or_else(|| BpiError::invalid_parameter("input", "missing module path"))?;
        let module = module_dir
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| BpiError::invalid_parameter("input", "missing module path"))?;

        Ok(Self {
            contract_path: contract_root
                .join(module)
                .join(batch)
                .join(endpoint)
                .join("contract.json"),
            case_name: file_stem.to_string(),
        })
    }
}

fn read_contract_risk(path: &Path) -> Option<ApiRisk> {
    let bytes = fs::read(path).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    serde_json::from_value(value.get("risk")?.clone()).ok()
}

fn fixture_name(case_name: &str, api_code: Option<i32>) -> String {
    match api_code {
        Some(0) => format!("{case_name}.success.json"),
        Some(code) => format!("{case_name}.{}.json", error_slug(code)),
        None => format!("{case_name}.response.json"),
    }
}

fn error_slug(code: i32) -> &'static str {
    match BpiError::from_code(code).semantic_error() {
        Some("requires_login") => "requires_login",
        Some("requires_vip") => "requires_vip",
        Some("risk_control") => "risk_control",
        Some("permission_denied") => "permission_denied",
        Some("business_error") => "business_error",
        _ => "error",
    }
}

fn update_contract_case(
    path: &Path,
    case_name: &str,
    http_status: u16,
    api_code: Option<i32>,
    fixture: &str,
) -> BpiResult<()> {
    let bytes = fs::read(path)
        .map_err(|err| BpiError::parse(format!("failed to read {}: {err}", path.display())))?;
    let mut value: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|err| BpiError::parse(format!("failed to parse {}: {err}", path.display())))?;
    let cases = value
        .get_mut("cases")
        .and_then(serde_json::Value::as_array_mut)
        .ok_or_else(|| BpiError::unsupported_response("contract missing cases array"))?;
    let case = cases
        .iter_mut()
        .find(|case| case.get("name").and_then(serde_json::Value::as_str) == Some(case_name))
        .ok_or_else(|| {
            BpiError::unsupported_response(format!("contract missing case {case_name}"))
        })?;
    let response = case
        .get_mut("response")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or_else(|| BpiError::unsupported_response("case missing response object"))?;

    response.insert("http_status".to_string(), http_status.into());
    if let Some(api_code) = api_code {
        response.insert("api_code".to_string(), api_code.into());
        if api_code != 0 {
            response.insert("error".to_string(), error_slug(api_code).into());
        }
    }
    response.insert("fixture".to_string(), fixture.into());
    response.insert("fixture_kind".to_string(), "sanitized_probe_body".into());
    if api_code == Some(0) {
        response.remove("error");
    }
    response.insert(
        "observed_at".to_string(),
        chrono::Utc::now().date_naive().to_string().into(),
    );

    fs::write(path, format!("{}\n", serde_json::to_string_pretty(&value)?))
        .map_err(|err| BpiError::parse(format!("failed to write {}: {err}", path.display())))?;
    Ok(())
}

fn sanitize_probe_result(result: &mut ProbeResult, risk: ApiRisk) {
    redact_sensitive_headers(&mut result.request.headers);
    redact_sensitive_headers(&mut result.response.headers);
    if let Some(body) = &mut result.request.body {
        sanitize_value(body, Some(risk));
    }
    sanitize_value(&mut result.response.body, Some(risk));
}

fn redact_sensitive_headers(headers: &mut BTreeMap<String, String>) {
    for (name, value) in headers {
        let normalized = name.to_ascii_lowercase();
        if matches!(
            normalized.as_str(),
            "cookie" | "authorization" | "set-cookie" | "x-csrf-token"
        ) {
            *value = "<redacted>".to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promote_writes_sanitized_fixture_and_updates_contract() -> Result<(), BpiError> {
        let dir = unique_temp_dir("promote")?;
        let contract_root = dir.join("contracts");
        let contract_dir = contract_root
            .join("login")
            .join("vip-info")
            .join("vip-info");
        fs::create_dir_all(&contract_dir).map_err(|err| BpiError::parse(err.to_string()))?;
        fs::write(
            contract_dir.join("contract.json"),
            r#"{
                "schema_version": 2,
                "name": "login.vip_info",
                "module": "login",
                "batch": "vip-info",
                "endpoint": "vip-info",
                "risk": "authenticated-read",
                "status": "promoted",
                "profiles": ["vip"],
                "request": {
                    "method": "GET",
                    "url": "https://api.bilibili.com/x/vip/web/user/info"
                },
                "sanitize": { "preset": ["account"] },
                "provenance": { "source": "unit_test" },
                "cases": [
                    {
                        "name": "vip",
                        "auth": { "profile": "vip", "requires": ["cookie"] },
                        "response": { "http_status": 200, "api_code": 0 }
                    }
                ]
            }"#,
        )
        .map_err(|err| BpiError::parse(err.to_string()))?;

        let probe_path = dir
            .join("runs")
            .join("login")
            .join("vip-info")
            .join("vip-info")
            .join("vip.response.json");
        fs::create_dir_all(probe_path.parent().unwrap())
            .map_err(|err| BpiError::parse(err.to_string()))?;
        fs::write(
            &probe_path,
            r#"{
                "contract": "login.vip_info.vip",
                "request": {
                    "method": "GET",
                    "url": "https://api.bilibili.com/x/vip/web/user/info",
                    "headers": { "cookie": "<redacted>" },
                    "query": {},
                    "body": null
                },
                "response": {
                    "status": 200,
                    "headers": {},
                    "body": {
                        "code": 0,
                        "message": "0",
                        "ttl": 1,
                        "data": {
                            "mid": 42,
                            "name": "real-user",
                            "csrf": "secret"
                        }
                    }
                }
            }"#,
        )
        .map_err(|err| BpiError::parse(err.to_string()))?;

        let summary = promote_probe_output_with_contract_root(&probe_path, &contract_root)?;
        let fixture = fs::read_to_string(&summary.fixture_path)
            .map_err(|err| BpiError::parse(err.to_string()))?;

        assert!(fixture.contains("1000001"));
        assert!(fixture.contains("sanitized-user"));
        assert!(fixture.contains("<redacted>"));

        let contract = fs::read_to_string(contract_dir.join("contract.json"))
            .map_err(|err| BpiError::parse(err.to_string()))?;
        assert!(contract.contains("responses/vip.success.json"));
        assert!(contract.contains("observed_at"));

        fs::remove_dir_all(dir).map_err(|err| BpiError::parse(err.to_string()))?;
        Ok(())
    }

    fn unique_temp_dir(name: &str) -> BpiResult<PathBuf> {
        let dir =
            std::env::temp_dir().join(format!("bpi-rs-promote-{name}-{}", std::process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir).map_err(|err| BpiError::parse(err.to_string()))?;
        }
        fs::create_dir_all(&dir).map_err(|err| BpiError::parse(err.to_string()))?;
        Ok(dir)
    }
}
