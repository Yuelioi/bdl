use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::probe::contract::ApiContract;
use crate::probe::endpoint_contract::{ApiRisk, ContractStatus, EndpointContract};
use crate::probe::flow::ProbeFlow;
use crate::{BpiError, BpiResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractAuditLevel {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractAuditFinding {
    pub level: ContractAuditLevel,
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractAuditReport {
    pub contracts: usize,
    pub errors: usize,
    pub warnings: usize,
    pub findings: Vec<ContractAuditFinding>,
}

impl ContractAuditReport {
    pub fn is_success(&self) -> bool {
        self.errors == 0
    }
}

pub fn audit_contracts(root: impl AsRef<Path>) -> BpiResult<ContractAuditReport> {
    let files = contract_files(root.as_ref())?;
    let mut findings = Vec::new();

    for path in &files {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(err) => {
                findings.push(error(path, format!("读取契约失败: {err}")));
                continue;
            }
        };

        let value: serde_json::Value = match serde_json::from_slice(&bytes) {
            Ok(value) => value,
            Err(err) => {
                findings.push(error(path, format!("解析 JSON 失败: {err}")));
                continue;
            }
        };

        if value.get("steps").is_some() {
            audit_flow_contract(path, &bytes, &mut findings);
        } else {
            audit_endpoint_contract(path, &bytes, &mut findings);
        }
    }

    let errors = findings
        .iter()
        .filter(|finding| finding.level == ContractAuditLevel::Error)
        .count();
    let warnings = findings.len() - errors;

    Ok(ContractAuditReport {
        contracts: files.len(),
        errors,
        warnings,
        findings,
    })
}

fn audit_endpoint_contract(path: &Path, bytes: &[u8], findings: &mut Vec<ContractAuditFinding>) {
    let contract = match EndpointContract::from_slice(bytes) {
        Ok(contract) => contract,
        Err(err) => {
            findings.push(error(path, format!("解析契约失败: {err}")));
            return;
        }
    };

    audit_contract(path, &contract, findings);
}

fn audit_flow_contract(path: &Path, bytes: &[u8], findings: &mut Vec<ContractAuditFinding>) {
    let value: serde_json::Value = match serde_json::from_slice(bytes) {
        Ok(value) => value,
        Err(err) => {
            findings.push(error(path, format!("解析流程 JSON 失败: {err}")));
            return;
        }
    };
    let flow = match ProbeFlow::from_slice(bytes) {
        Ok(flow) => flow,
        Err(err) => {
            findings.push(error(path, format!("解析流程契约失败: {err}")));
            return;
        }
    };

    audit_flow_v2_metadata(path, &value, findings);

    for step in &flow.steps {
        if let Err(err) = ApiContract::from_value(step.contract.clone()) {
            findings.push(error(
                path,
                format!("流程步骤 {} 的内嵌请求契约无效: {err}", step.name),
            ));
        }
    }
}

fn audit_flow_v2_metadata(
    path: &Path,
    value: &serde_json::Value,
    findings: &mut Vec<ContractAuditFinding>,
) {
    let Some(version) = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
    else {
        findings.push(warning(path, "流程契约暂未迁移 schema_version 元数据"));
        return;
    };

    if version != 2 {
        findings.push(error(
            path,
            format!("流程契约不支持的 schema_version: {version}"),
        ));
        return;
    }

    for field in ["module", "batch", "endpoint", "status"] {
        if value
            .get(field)
            .and_then(serde_json::Value::as_str)
            .is_none_or(str::is_empty)
        {
            findings.push(error(path, format!("v2 流程契约缺少 {field}")));
        }
    }
    if value.get("risk").is_none() {
        findings.push(error(path, "v2 流程契约缺少 risk"));
    } else if serde_json::from_value::<ApiRisk>(value["risk"].clone()).is_err() {
        findings.push(error(path, "v2 流程契约 risk 非法"));
    }
    if value.get("status").is_some()
        && serde_json::from_value::<ContractStatus>(value["status"].clone()).is_err()
    {
        findings.push(error(path, "v2 流程契约 status 非法"));
    }
    if value
        .get("profiles")
        .and_then(serde_json::Value::as_array)
        .is_none_or(Vec::is_empty)
    {
        findings.push(error(path, "v2 流程契约缺少 profiles"));
    }
}

fn audit_contract(
    path: &Path,
    contract: &EndpointContract,
    findings: &mut Vec<ContractAuditFinding>,
) {
    audit_v2_metadata(path, contract, findings);

    for case in &contract.cases {
        if let Some(fixture) = &case.response.fixture {
            let fixture_path = path.parent().unwrap_or_else(|| Path::new("")).join(fixture);
            if !fixture_path.is_file() {
                findings.push(error(
                    path,
                    format!("用例 {} 引用的 fixture 不存在: {fixture}", case.name),
                ));
            }
        }

        if let Some(kind) = &case.response.fixture_kind
            && !allowed_fixture_kind(kind)
        {
            findings.push(error(
                path,
                format!("用例 {} 使用了未知 fixture_kind: {kind}", case.name),
            ));
        }

        if case.profile.is_some()
            && case.auth.profile.is_some()
            && case.profile != case.auth.profile
        {
            findings.push(error(
                path,
                format!("用例 {} 的 profile 和 auth.profile 不一致", case.name),
            ));
        }
    }
}

fn audit_v2_metadata(
    path: &Path,
    contract: &EndpointContract,
    findings: &mut Vec<ContractAuditFinding>,
) {
    let Some(version) = contract.schema_version else {
        findings.push(warning(path, "缺少 schema_version，当前按旧契约兼容读取"));
        return;
    };

    if version != 2 {
        findings.push(error(path, format!("不支持的 schema_version: {version}")));
        return;
    }

    if contract.module.as_deref().is_none_or(str::is_empty) {
        findings.push(error(path, "v2 契约缺少 module"));
    }
    if contract.batch.as_deref().is_none_or(str::is_empty) {
        findings.push(error(path, "v2 契约缺少 batch"));
    }
    if contract.endpoint.as_deref().is_none_or(str::is_empty) {
        findings.push(error(path, "v2 契约缺少 endpoint"));
    }
    if contract.risk.is_none() {
        findings.push(error(path, "v2 契约缺少 risk"));
    }
    if contract.status.is_none() {
        findings.push(error(path, "v2 契约缺少 status"));
    }
    if contract.profiles.is_empty() {
        findings.push(error(path, "v2 契约缺少 profiles"));
    }

    if matches!(
        contract.risk,
        Some(ApiRisk::PrivateRead | ApiRisk::Mutating | ApiRisk::Spending | ApiRisk::LoginSession)
    ) && contract.sanitize.is_none()
    {
        findings.push(warning(path, "私有或高风险契约建议声明 sanitize 规则"));
    }

    if matches!(contract.status, Some(ContractStatus::Promoted)) && contract.provenance.is_none() {
        findings.push(warning(path, "promoted 契约建议声明 provenance 来源"));
    }
}

fn allowed_fixture_kind(kind: &str) -> bool {
    matches!(
        kind,
        "binary_probe_body"
            | "empty_success_missing_model_fields"
            | "local_probe_blocked"
            | "no_payload_success"
            | "probe_binary_body"
            | "probe_body"
            | "probe_body_no_payload"
            | "probe_errno_error_body"
            | "probe_error_body"
            | "sanitized_probe"
            | "sanitized_probe_body"
            | "trimmed_probe_body"
    )
}

fn contract_files(root: &Path) -> BpiResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_contract_files(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_contract_files(path: &Path, files: &mut Vec<PathBuf>) -> BpiResult<()> {
    if path.is_file() {
        if path.file_name().and_then(|value| value.to_str()) == Some("contract.json") {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }

    if !path.is_dir() {
        return Err(BpiError::invalid_parameter(
            "path",
            "contract audit path must be a contract.json file or directory",
        ));
    }

    let entries = fs::read_dir(path)
        .map_err(|err| BpiError::parse(format!("failed to read {}: {err}", path.display())))?;
    for entry in entries {
        let entry = entry.map_err(|err| BpiError::parse(err.to_string()))?;
        collect_contract_files(&entry.path(), files)?;
    }
    Ok(())
}

fn error(path: &Path, message: impl Into<String>) -> ContractAuditFinding {
    ContractAuditFinding {
        level: ContractAuditLevel::Error,
        path: path.to_path_buf(),
        message: message.into(),
    }
}

fn warning(path: &Path, message: impl Into<String>) -> ContractAuditFinding {
    ContractAuditFinding {
        level: ContractAuditLevel::Warning,
        path: path.to_path_buf(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_audit_reports_missing_v2_metadata() -> Result<(), BpiError> {
        let dir = unique_temp_dir("missing-v2-metadata")?;
        let contract_path = dir.join("contract.json");
        fs::write(
            &contract_path,
            r#"{
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
        .map_err(|err| BpiError::parse(err.to_string()))?;

        let report = audit_contracts(&dir)?;

        assert_eq!(report.contracts, 1);
        assert_eq!(report.errors, 0);
        assert_eq!(report.warnings, 1);
        assert!(report.findings[0].message.contains("缺少 schema_version"));

        fs::remove_dir_all(dir).map_err(|err| BpiError::parse(err.to_string()))?;
        Ok(())
    }

    #[test]
    fn contract_audit_reports_missing_fixture() -> Result<(), BpiError> {
        let dir = unique_temp_dir("missing-fixture")?;
        let contract_path = dir.join("contract.json");
        fs::write(
            &contract_path,
            r#"{
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
                "provenance": {
                    "source": "unit_test"
                },
                "cases": [
                    {
                        "name": "anonymous",
                        "response": {
                            "http_status": 200,
                            "api_code": 0,
                            "fixture": "responses/anonymous.success.json",
                            "fixture_kind": "sanitized_probe_body"
                        }
                    }
                ]
            }"#,
        )
        .map_err(|err| BpiError::parse(err.to_string()))?;

        let report = audit_contracts(&dir)?;

        assert_eq!(report.contracts, 1);
        assert_eq!(report.errors, 1);
        assert_eq!(report.warnings, 0);
        assert!(report.findings[0].message.contains("fixture 不存在"));

        fs::remove_dir_all(dir).map_err(|err| BpiError::parse(err.to_string()))?;
        Ok(())
    }

    #[test]
    fn contract_audit_accepts_flow_contracts() -> Result<(), BpiError> {
        let dir = unique_temp_dir("flow")?;
        let contract_path = dir.join("contract.json");
        fs::write(
            &contract_path,
            r#"{
                "name": "login.qr.flow",
                "steps": [
                    {
                        "name": "generate",
                        "contract": {
                            "name": "login.qr_generate.anonymous",
                            "request": {
                                "method": "GET",
                                "url": "https://passport.bilibili.com/x/passport-login/web/qrcode/generate"
                            },
                            "expect": {
                                "api_code": 0
                            }
                        },
                        "extract": {
                            "qrcode_key": "/response/body/data/qrcode_key"
                        }
                    },
                    {
                        "name": "poll",
                        "contract": {
                            "name": "login.qr_poll.anonymous",
                            "request": {
                                "method": "GET",
                                "url": "https://passport.bilibili.com/x/passport-login/web/qrcode/poll",
                                "query": {
                                    "qrcode_key": "${qrcode_key}"
                                }
                            },
                            "expect": {
                                "api_code": 0
                            }
                        }
                    }
                ]
            }"#,
        )
        .map_err(|err| BpiError::parse(err.to_string()))?;

        let report = audit_contracts(&dir)?;

        assert_eq!(report.contracts, 1);
        assert_eq!(report.errors, 0);
        assert_eq!(report.warnings, 1);

        fs::remove_dir_all(dir).map_err(|err| BpiError::parse(err.to_string()))?;
        Ok(())
    }

    #[test]
    fn contract_audit_accepts_v2_flow_metadata() -> Result<(), BpiError> {
        let dir = unique_temp_dir("flow-v2")?;
        let contract_path = dir.join("contract.json");
        fs::write(
            &contract_path,
            r#"{
                "schema_version": 2,
                "name": "login.qr.flow",
                "module": "login",
                "batch": "qr",
                "endpoint": "flow",
                "risk": "login-session",
                "status": "promoted",
                "profiles": ["anonymous"],
                "steps": [
                    {
                        "name": "generate",
                        "contract": {
                            "name": "login.qr_generate.anonymous",
                            "request": {
                                "method": "GET",
                                "url": "https://passport.bilibili.com/x/passport-login/web/qrcode/generate"
                            },
                            "expect": {
                                "api_code": 0
                            }
                        }
                    }
                ]
            }"#,
        )
        .map_err(|err| BpiError::parse(err.to_string()))?;

        let report = audit_contracts(&dir)?;

        assert_eq!(report.contracts, 1);
        assert_eq!(report.errors, 0);
        assert_eq!(report.warnings, 0);

        fs::remove_dir_all(dir).map_err(|err| BpiError::parse(err.to_string()))?;
        Ok(())
    }

    fn unique_temp_dir(name: &str) -> BpiResult<PathBuf> {
        let dir = std::env::temp_dir().join(format!(
            "bpi-rs-contract-audit-{name}-{}",
            std::process::id()
        ));
        if dir.exists() {
            fs::remove_dir_all(&dir).map_err(|err| BpiError::parse(err.to_string()))?;
        }
        fs::create_dir_all(&dir).map_err(|err| BpiError::parse(err.to_string()))?;
        Ok(dir)
    }
}
