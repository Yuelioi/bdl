use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::probe::account::RawProbeConfig;
use crate::probe::contract::{ApiContract, ProbeResult};
use crate::probe::endpoint_contract::{ApiRisk, EndpointCase, EndpointContract};
use crate::probe::model::parse_registered_response_model;
use crate::probe::run::execute_contract;
use crate::{BpiError, BpiResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BatchRunConfig {
    pub root: PathBuf,
    pub account_path: PathBuf,
    pub output_root: PathBuf,
    pub profiles: BTreeSet<String>,
    pub pages: usize,
    pub risk_mode: BatchRiskMode,
    pub gate: BatchRunGate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BatchRiskMode {
    #[default]
    All,
    ReadOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BatchRunGate {
    pub probe: bool,
    pub mutating: bool,
    pub spending: bool,
    pub login_session: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BatchRunSummary {
    pub attempted: usize,
    pub written: usize,
    pub skipped: usize,
}

impl BatchRunConfig {
    pub fn from_env(
        root: PathBuf,
        account_path: PathBuf,
        output_root: PathBuf,
        profiles: BTreeSet<String>,
        pages: usize,
    ) -> Self {
        Self {
            root,
            account_path,
            output_root,
            profiles,
            pages,
            risk_mode: BatchRiskMode::All,
            gate: BatchRunGate {
                probe: env_flag("BPI_PROBE"),
                mutating: env_flag("BPI_MUTATING_TEST"),
                spending: env_flag("BPI_SPENDING_TEST"),
                login_session: env_flag("BPI_LOGIN_SESSION_TEST"),
            },
        }
    }
}

pub fn parse_profile_set(input: &str) -> BpiResult<BTreeSet<String>> {
    let profiles = input
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    if profiles.is_empty() {
        return Err(BpiError::invalid_parameter(
            "profiles",
            "profiles must not be empty",
        ));
    }
    Ok(profiles)
}

pub fn parse_page_count(input: &str) -> BpiResult<usize> {
    let pages = input
        .parse::<usize>()
        .map_err(|_| BpiError::invalid_parameter("pages", "pages must be a positive integer"))?;
    if pages == 0 {
        return Err(BpiError::invalid_parameter(
            "pages",
            "pages must be greater than 0",
        ));
    }
    Ok(pages)
}

pub async fn run_batch(config: BatchRunConfig) -> BpiResult<BatchRunSummary> {
    if !config.gate.probe {
        return Err(BpiError::invalid_parameter(
            "BPI_PROBE",
            "set BPI_PROBE=1 to run network probes",
        ));
    }

    let accounts = if config.account_path.is_file() {
        RawProbeConfig::load(&config.account_path)?
    } else {
        RawProbeConfig::default()
    };
    let mut summary = BatchRunSummary {
        attempted: 0,
        written: 0,
        skipped: 0,
    };

    for path in contract_files(&config.root)? {
        let bytes = fs::read(&path)
            .map_err(|err| BpiError::parse(format!("failed to read {}: {err}", path.display())))?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|err| BpiError::parse(format!("failed to parse {}: {err}", path.display())))?;
        if value.get("steps").is_some() {
            summary.skipped += 1;
            continue;
        }

        let contract = EndpointContract::from_slice(&bytes)?;
        if should_skip_for_risk_mode(contract.risk, config.risk_mode) {
            summary.skipped += 1;
            continue;
        }
        enforce_risk_gate(contract.risk, config.gate)?;

        for case in &contract.cases {
            let profile = case_profile(case);
            if !config.profiles.contains(profile) {
                summary.skipped += 1;
                continue;
            }

            if is_history_cursor_contract(&contract) {
                run_history_cursor_case(&config, &accounts, &contract, case, &mut summary).await?;
                continue;
            }

            for probe_case in probe_contracts_for_case(&contract, case, config.pages)? {
                summary.attempted += 1;
                let result = execute_contract(&probe_case.contract, &accounts).await?;
                validate_case_model(case, &result)?;
                write_probe_result(
                    &config.output_root,
                    &contract,
                    case,
                    probe_case.page,
                    &result,
                )?;
                summary.written += 1;
            }
        }
    }

    Ok(summary)
}

fn should_skip_for_risk_mode(risk: Option<ApiRisk>, mode: BatchRiskMode) -> bool {
    matches!(mode, BatchRiskMode::ReadOnly)
        && matches!(
            risk,
            Some(ApiRisk::Mutating | ApiRisk::Spending | ApiRisk::LoginSession)
        )
}

pub fn enforce_risk_gate(risk: Option<ApiRisk>, gate: BatchRunGate) -> BpiResult<()> {
    match risk {
        Some(ApiRisk::Mutating) if !gate.mutating => Err(BpiError::invalid_parameter(
            "BPI_MUTATING_TEST",
            "set BPI_MUTATING_TEST=1 to run mutating API probes",
        )),
        Some(ApiRisk::Spending) if !(gate.mutating && gate.spending) => {
            Err(BpiError::invalid_parameter(
                "BPI_SPENDING_TEST",
                "set BPI_MUTATING_TEST=1 and BPI_SPENDING_TEST=1 to run spending API probes",
            ))
        }
        Some(ApiRisk::LoginSession) if !gate.login_session => Err(BpiError::invalid_parameter(
            "BPI_LOGIN_SESSION_TEST",
            "set BPI_LOGIN_SESSION_TEST=1 to run login session probes",
        )),
        _ => Ok(()),
    }
}

#[derive(Debug, Clone)]
pub struct BatchProbeCase {
    pub contract: ApiContract,
    pub page: Option<i64>,
}

pub fn probe_contracts_for_case(
    contract: &EndpointContract,
    case: &EndpointCase,
    pages: usize,
) -> BpiResult<Vec<BatchProbeCase>> {
    let base_request = request_value_for_case(contract, case)?;
    let page_state = pagination_state(&base_request);
    let page_values = page_state
        .map(|state| {
            (0..pages)
                .map(|offset| state.start + offset as i64)
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec![-1]);

    page_values
        .into_iter()
        .map(|page| {
            let mut request = base_request.clone();
            let page = if page >= 0 {
                apply_page_value(&mut request, page);
                Some(page)
            } else {
                None
            };
            Ok(BatchProbeCase {
                contract: api_contract_for_request(contract, case, request)?,
                page,
            })
        })
        .collect()
}

pub fn probe_contract_for_case(
    contract: &EndpointContract,
    case: &EndpointCase,
) -> BpiResult<ApiContract> {
    let request = request_value_for_case(contract, case)?;
    api_contract_for_request(contract, case, request)
}

async fn run_history_cursor_case(
    config: &BatchRunConfig,
    accounts: &RawProbeConfig,
    contract: &EndpointContract,
    case: &EndpointCase,
    summary: &mut BatchRunSummary,
) -> BpiResult<()> {
    let mut probe_contract = probe_contract_for_case(contract, case)?;
    let mut last_cursor = None;

    for page_index in 1..=config.pages {
        summary.attempted += 1;
        let result = execute_contract(&probe_contract, accounts).await?;
        validate_case_model(case, &result)?;
        write_probe_result(
            &config.output_root,
            contract,
            case,
            Some(page_index as i64),
            &result,
        )?;
        summary.written += 1;

        let Some(cursor) = history_cursor_from_body(&result.response.body) else {
            break;
        };
        if cursor.max <= 0 || last_cursor.as_ref() == Some(&cursor) {
            break;
        }

        apply_history_cursor(&mut probe_contract, &cursor);
        last_cursor = Some(cursor);
    }

    Ok(())
}

fn validate_case_model(case: &EndpointCase, result: &ProbeResult) -> BpiResult<()> {
    if let Some(model) = case.response.rust_model.as_deref() {
        parse_registered_response_model(model, &result.response.body)?;
    }
    Ok(())
}

fn write_probe_result(
    root: &Path,
    contract: &EndpointContract,
    case: &EndpointCase,
    page: Option<i64>,
    result: &ProbeResult,
) -> BpiResult<()> {
    let output_path = output_path_for_case(root, contract, case, page);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            BpiError::parse(format!("failed to create {}: {err}", parent.display()))
        })?;
    }
    let output = serde_json::to_string_pretty(result)?;
    fs::write(&output_path, format!("{output}\n"))
        .map_err(|err| BpiError::parse(format!("failed to write {}: {err}", output_path.display())))
}

fn request_value_for_case(
    contract: &EndpointContract,
    case: &EndpointCase,
) -> BpiResult<serde_json::Value> {
    let mut request = serde_json::to_value(&contract.request)?;
    request["auth"] = serde_json::to_value(&case.auth)?;
    Ok(request)
}

fn api_contract_for_request(
    contract: &EndpointContract,
    case: &EndpointCase,
    request: serde_json::Value,
) -> BpiResult<ApiContract> {
    let mut expect = serde_json::Map::new();
    if let Some(api_code) = case.response.api_code {
        expect.insert("api_code".to_string(), api_code.into());
    }

    ApiContract::from_value(serde_json::json!({
        "name": format!("{}.{}", contract.name, case.name),
        "request": request,
        "expect": expect
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PaginationState {
    start: i64,
}

fn pagination_state(request: &serde_json::Value) -> Option<PaginationState> {
    find_page_value(request)
        .and_then(page_value_as_i64)
        .map(|start| PaginationState { start })
}

fn apply_page_value(request: &mut serde_json::Value, page: i64) {
    replace_page_value(request, page);
}

fn find_page_value(value: &serde_json::Value) -> Option<&serde_json::Value> {
    value
        .get("query")
        .and_then(find_page_value_in_object)
        .or_else(|| value.get("body").and_then(find_page_value_in_object))
}

fn find_page_value_in_object(value: &serde_json::Value) -> Option<&serde_json::Value> {
    let object = value.as_object()?;
    for key in ["page", "pn", "pageNum"] {
        if let Some(value) = object.get(key) {
            return Some(value);
        }
    }
    None
}

fn replace_page_value(value: &mut serde_json::Value, page: i64) -> bool {
    replace_page_value_in_object(&mut value["query"], page)
        || replace_page_value_in_object(&mut value["body"], page)
}

fn replace_page_value_in_object(value: &mut serde_json::Value, page: i64) -> bool {
    let Some(object) = value.as_object_mut() else {
        return false;
    };
    for key in ["page", "pn", "pageNum"] {
        if let Some(value) = object.get_mut(key) {
            *value = match value {
                serde_json::Value::String(_) => serde_json::Value::String(page.to_string()),
                _ => serde_json::Value::Number(page.into()),
            };
            return true;
        }
    }
    false
}

fn page_value_as_i64(value: &serde_json::Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_str().and_then(|value| value.parse().ok()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HistoryCursorPage {
    max: i64,
    view_at: i64,
    business: Option<String>,
}

fn is_history_cursor_contract(contract: &EndpointContract) -> bool {
    contract.name == "historytoview.history_list"
        || (contract.module.as_deref() == Some("historytoview")
            && contract.endpoint.as_deref() == Some("history-list"))
}

fn history_cursor_from_body(body: &serde_json::Value) -> Option<HistoryCursorPage> {
    let cursor = body.pointer("/data/cursor")?;
    Some(HistoryCursorPage {
        max: value_as_i64(cursor.get("max")?)?,
        view_at: value_as_i64(cursor.get("view_at")?)?,
        business: cursor.get("business").and_then(value_as_string),
    })
}

fn apply_history_cursor(contract: &mut ApiContract, cursor: &HistoryCursorPage) {
    contract
        .request
        .query
        .insert("max".to_string(), cursor.max.to_string());
    contract
        .request
        .query
        .insert("view_at".to_string(), cursor.view_at.to_string());
    if let Some(business) = &cursor.business {
        contract
            .request
            .query
            .insert("business".to_string(), business.clone());
    }
}

fn value_as_i64(value: &serde_json::Value) -> Option<i64> {
    value
        .as_i64()
        .or_else(|| value.as_str().and_then(|value| value.parse().ok()))
}

fn value_as_string(value: &serde_json::Value) -> Option<String> {
    value
        .as_str()
        .map(str::to_string)
        .or_else(|| value.as_i64().map(|value| value.to_string()))
}

fn output_path_for_case(
    root: &Path,
    contract: &EndpointContract,
    case: &EndpointCase,
    page: Option<i64>,
) -> PathBuf {
    let module = contract.module.as_deref().unwrap_or("unknown");
    let batch = contract.batch.as_deref().unwrap_or("default");
    let endpoint = contract.endpoint.as_deref().unwrap_or(&contract.name);
    let file_name = match page {
        Some(page) => format!("{}.page{page}.response.json", case.name),
        None => format!("{}.response.json", case.name),
    };
    root.join(module).join(batch).join(endpoint).join(file_name)
}

fn case_profile(case: &EndpointCase) -> &str {
    case.profile
        .as_deref()
        .or(case.auth.profile.as_deref())
        .unwrap_or("anonymous")
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
            "batch-run path must be a contract.json file or directory",
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

fn env_flag(name: &str) -> bool {
    std::env::var(name).as_deref() == Ok("1")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn risk_gate_blocks_spending_without_double_opt_in() {
        let gate = BatchRunGate {
            probe: true,
            mutating: true,
            spending: false,
            login_session: false,
        };

        let err = enforce_risk_gate(Some(ApiRisk::Spending), gate).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "BPI_SPENDING_TEST",
                ..
            }
        ));
    }

    #[test]
    fn risk_gate_allows_public_read_without_extra_opt_in() -> Result<(), BpiError> {
        enforce_risk_gate(Some(ApiRisk::PublicRead), BatchRunGate::default())
    }

    #[test]
    fn read_only_mode_skips_side_effect_risks() {
        assert!(should_skip_for_risk_mode(
            Some(ApiRisk::Mutating),
            BatchRiskMode::ReadOnly
        ));
        assert!(should_skip_for_risk_mode(
            Some(ApiRisk::Spending),
            BatchRiskMode::ReadOnly
        ));
        assert!(should_skip_for_risk_mode(
            Some(ApiRisk::LoginSession),
            BatchRiskMode::ReadOnly
        ));
        assert!(!should_skip_for_risk_mode(
            Some(ApiRisk::PrivateRead),
            BatchRiskMode::ReadOnly
        ));
    }

    #[test]
    fn probe_contract_for_case_uses_case_auth_and_expectation() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(
            br#"{
                "name": "login.vip_info",
                "request": {
                    "method": "GET",
                    "url": "https://api.bilibili.com/x/vip/web/user/info"
                },
                "cases": [
                    {
                        "name": "vip",
                        "auth": {
                            "profile": "vip",
                            "requires": ["cookie"]
                        },
                        "response": {
                            "api_code": 0
                        }
                    }
                ]
            }"#,
        )?;
        let probe = probe_contract_for_case(&contract, &contract.cases[0])?;

        assert_eq!(probe.name, "login.vip_info.vip");
        assert_eq!(probe.request.auth.profile.as_deref(), Some("vip"));
        assert!(probe.request.auth.requires_cookie());
        assert_eq!(probe.expect["api_code"], 0);
        Ok(())
    }

    #[test]
    fn probe_contracts_for_case_expands_query_pages() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(
            br#"{
                "schema_version": 2,
                "name": "history.list",
                "module": "historytoview",
                "batch": "read",
                "endpoint": "history-list",
                "risk": "private-read",
                "status": "promoted",
                "profiles": ["normal"],
                "request": {
                    "method": "GET",
                    "url": "https://api.bilibili.com/x/web-interface/history/cursor",
                    "query": {
                        "page": "1"
                    }
                },
                "cases": [
                    {
                        "name": "normal",
                        "auth": {
                            "profile": "normal",
                            "requires": ["cookie"]
                        },
                        "response": {
                            "api_code": 0
                        }
                    }
                ]
            }"#,
        )?;

        let cases = probe_contracts_for_case(&contract, &contract.cases[0], 3)?;

        assert_eq!(cases.len(), 3);
        assert_eq!(cases[0].page, Some(1));
        assert_eq!(cases[1].contract.request.query["page"], "2");
        assert_eq!(cases[2].contract.request.query["page"], "3");
        Ok(())
    }

    #[test]
    fn probe_contracts_for_case_expands_zero_based_body_pages() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(
            br#"{
                "schema_version": 2,
                "name": "manga.coupons",
                "module": "manga",
                "batch": "read-core",
                "endpoint": "coupons",
                "risk": "public-read",
                "status": "promoted",
                "profiles": ["anonymous"],
                "request": {
                    "method": "POST",
                    "url": "https://manga.bilibili.com/twirp/user.v1.User/GetCoupons",
                    "body": {
                        "pageNum": 0
                    }
                },
                "cases": [
                    {
                        "name": "anonymous",
                        "response": {
                            "api_code": 0
                        }
                    }
                ]
            }"#,
        )?;

        let cases = probe_contracts_for_case(&contract, &contract.cases[0], 2)?;

        assert_eq!(cases.len(), 2);
        assert_eq!(cases[0].page, Some(0));
        assert_eq!(
            cases[1].contract.request.body.as_ref().unwrap()["pageNum"],
            1
        );
        Ok(())
    }

    #[test]
    fn probe_contracts_for_case_keeps_non_paginated_single_case() -> Result<(), BpiError> {
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
                    "url": "https://api.bilibili.com/x/web-interface/view",
                    "query": {
                        "bvid": "BV1xx411c7mD"
                    }
                },
                "cases": [
                    {
                        "name": "anonymous",
                        "response": {
                            "api_code": 0
                        }
                    }
                ]
            }"#,
        )?;

        let cases = probe_contracts_for_case(&contract, &contract.cases[0], 10)?;

        assert_eq!(cases.len(), 1);
        assert_eq!(cases[0].page, None);
        Ok(())
    }

    #[test]
    fn parse_profile_set_trims_and_deduplicates() -> Result<(), BpiError> {
        let profiles = parse_profile_set(" anonymous,normal,normal ")?;

        assert_eq!(
            profiles,
            ["anonymous".to_string(), "normal".to_string()]
                .into_iter()
                .collect()
        );
        Ok(())
    }

    #[test]
    fn parse_page_count_rejects_zero() {
        let err = parse_page_count("0").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "pages", .. }
        ));
    }

    #[test]
    fn history_cursor_from_body_extracts_next_query_values() {
        let body = serde_json::json!({
            "code": 0,
            "data": {
                "cursor": {
                    "max": 42,
                    "view_at": "1234567890",
                    "business": "archive",
                    "ps": 5
                }
            }
        });

        let cursor = history_cursor_from_body(&body).unwrap();

        assert_eq!(cursor.max, 42);
        assert_eq!(cursor.view_at, 1234567890);
        assert_eq!(cursor.business.as_deref(), Some("archive"));
    }

    #[test]
    fn apply_history_cursor_updates_probe_query() -> Result<(), BpiError> {
        let contract = EndpointContract::from_slice(
            br#"{
                "schema_version": 2,
                "name": "historytoview.history_list",
                "module": "historytoview",
                "batch": "read",
                "endpoint": "history-list",
                "risk": "private-read",
                "status": "promoted",
                "profiles": ["normal"],
                "request": {
                    "method": "GET",
                    "url": "https://api.bilibili.com/x/web-interface/history/cursor",
                    "query": {
                        "ps": "5"
                    }
                },
                "cases": [
                    {
                        "name": "normal",
                        "auth": {
                            "profile": "normal",
                            "requires": ["cookie"]
                        },
                        "response": {
                            "api_code": 0
                        }
                    }
                ]
            }"#,
        )?;
        let mut probe = probe_contract_for_case(&contract, &contract.cases[0])?;

        apply_history_cursor(
            &mut probe,
            &HistoryCursorPage {
                max: 42,
                view_at: 1234567890,
                business: Some("archive".to_string()),
            },
        );

        assert_eq!(probe.request.query["ps"], "5");
        assert_eq!(probe.request.query["max"], "42");
        assert_eq!(probe.request.query["view_at"], "1234567890");
        assert_eq!(probe.request.query["business"], "archive");
        Ok(())
    }
}
