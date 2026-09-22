use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::probe::contract::HttpMethod;
use crate::probe::endpoint_contract::{ApiRisk, ContractStatus, EndpointContract};
use crate::{BpiError, BpiResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiCatalogEntry {
    pub name: String,
    pub module: String,
    pub batch: String,
    pub endpoint: String,
    pub risk: Option<ApiRisk>,
    pub status: Option<ContractStatus>,
    pub profiles: Vec<String>,
    pub method: Option<HttpMethod>,
    pub url: Option<String>,
    pub auth: Vec<String>,
    pub rust_models: Vec<String>,
    pub contract_path: String,
    pub source_path: Option<String>,
    pub function_name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiCatalog {
    pub entries: Vec<ApiCatalogEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceFunction {
    path: String,
    line: usize,
    name: String,
    doc: Option<String>,
}

pub fn generate_api_catalog(contracts_root: &Path, source_root: &Path) -> BpiResult<ApiCatalog> {
    let functions = discover_source_functions(source_root)?;
    let mut entries = Vec::new();

    for path in contract_files(contracts_root)? {
        let bytes = fs::read(&path)
            .map_err(|err| BpiError::parse(format!("failed to read {}: {err}", path.display())))?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|err| BpiError::parse(format!("failed to parse {}: {err}", path.display())))?;

        if value.get("steps").is_some() {
            entries.push(flow_entry(&path, &value, &functions));
            continue;
        }

        let contract = EndpointContract::from_slice(&bytes)?;
        entries.push(endpoint_entry(&path, contract, &functions));
    }

    entries.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(ApiCatalog { entries })
}

pub fn render_api_catalog_markdown(catalog: &ApiCatalog) -> String {
    let mut output = String::new();
    output.push_str("# bpi-rs API 索引\n\n");
    output.push_str("本文件由 `bpi-probe api-doc tests/contracts --output docs/api-index.md` 生成，用于给维护者和 AI 快速查找接口、权限、契约和源码入口。\n\n");
    output.push_str("风险分类定义见 [API 风险分类](api-risk-classification.md)，新增或调整接口前先确认分类和测试门控。\n\n");
    output.push_str(
        "| 模块 | API | 说明 | 风险 | profiles | 方法 | URL | Rust 模型 | 契约 | 函数 |\n",
    );
    output.push_str("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");

    for entry in &catalog.entries {
        let risk = entry.risk.map(risk_name).unwrap_or("-");
        let profiles = if entry.profiles.is_empty() {
            "-".to_string()
        } else {
            entry.profiles.join(", ")
        };
        let method = entry.method.map(method_name).unwrap_or("-");
        let url = entry.url.as_deref().unwrap_or("-");
        let models = if entry.rust_models.is_empty() {
            "-".to_string()
        } else {
            entry.rust_models.join(", ")
        };
        let function = match (&entry.source_path, &entry.function_name) {
            (Some(path), Some(name)) => format!("`{path}` `{name}`"),
            _ => "-".to_string(),
        };
        let description = entry.description.as_deref().unwrap_or("-");

        output.push_str(&format!(
            "| {} | `{}` | {} | `{}` | {} | `{}` | `{}` | {} | `{}` | {} |\n",
            escape_cell(&entry.module),
            escape_cell(&entry.name),
            escape_cell(description),
            risk,
            escape_cell(&profiles),
            method,
            escape_cell(url),
            escape_cell(&models),
            escape_cell(&entry.contract_path),
            function
        ));
    }

    output
}

fn endpoint_entry(
    path: &Path,
    contract: EndpointContract,
    functions: &BTreeMap<String, Vec<SourceFunction>>,
) -> ApiCatalogEntry {
    let module = contract.module.unwrap_or_else(|| "unknown".to_string());
    let function = find_source_function(
        &contract.name,
        Some(&module),
        contract.endpoint.as_deref(),
        functions,
    );
    ApiCatalogEntry {
        module,
        batch: contract.batch.unwrap_or_else(|| "default".to_string()),
        endpoint: contract.endpoint.unwrap_or_else(|| contract.name.clone()),
        risk: contract.risk,
        status: contract.status,
        profiles: contract.profiles,
        method: Some(contract.request.method),
        url: Some(contract.request.url.as_str().to_string()),
        auth: contract
            .request
            .auth
            .requires
            .iter()
            .map(|requirement| format!("{requirement:?}").to_ascii_lowercase())
            .collect(),
        rust_models: rust_models(&contract.cases),
        contract_path: relative_path(path),
        source_path: function
            .as_ref()
            .map(|function| format!("{}:{}", function.path, function.line)),
        function_name: function.map(|function| function.name.clone()),
        description: function.and_then(|function| function.doc.clone()),
        name: contract.name,
    }
}

fn flow_entry(
    path: &Path,
    value: &serde_json::Value,
    functions: &BTreeMap<String, Vec<SourceFunction>>,
) -> ApiCatalogEntry {
    let name = string_field(value, "name").unwrap_or_else(|| "unknown.flow".to_string());
    let endpoint = string_field(value, "endpoint").unwrap_or_else(|| {
        name.rsplit_once('.')
            .map(|(_, value)| value.to_string())
            .unwrap_or_else(|| name.clone())
    });
    let module = string_field(value, "module").unwrap_or_else(|| "unknown".to_string());
    let function = find_source_function(&name, Some(&module), Some(&endpoint), functions);

    ApiCatalogEntry {
        module,
        batch: string_field(value, "batch").unwrap_or_else(|| "flow".to_string()),
        endpoint,
        risk: value
            .get("risk")
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok()),
        status: value
            .get("status")
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok()),
        profiles: value
            .get("profiles")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default(),
        method: None,
        url: None,
        auth: Vec::new(),
        rust_models: Vec::new(),
        contract_path: relative_path(path),
        source_path: function
            .as_ref()
            .map(|function| format!("{}:{}", function.path, function.line)),
        function_name: function.map(|function| function.name.clone()),
        description: function.and_then(|function| function.doc.clone()),
        name,
    }
}

fn rust_models(cases: &[crate::probe::endpoint_contract::EndpointCase]) -> Vec<String> {
    let mut models = cases
        .iter()
        .filter_map(|case| case.response.rust_model.clone())
        .collect::<Vec<_>>();
    models.sort();
    models.dedup();
    models
}

fn discover_source_functions(root: &Path) -> BpiResult<BTreeMap<String, Vec<SourceFunction>>> {
    let mut files = Vec::new();
    collect_rs_files(root, &mut files)?;
    let mut functions = BTreeMap::new();

    for path in files {
        let text = fs::read_to_string(&path)
            .map_err(|err| BpiError::parse(format!("failed to read {}: {err}", path.display())))?;
        let lines = text.lines().collect::<Vec<_>>();
        for (index, line) in lines.iter().enumerate() {
            let Some(name) = parse_pub_async_fn_name(line) else {
                continue;
            };
            functions
                .entry(name.clone())
                .or_insert_with(Vec::new)
                .push(SourceFunction {
                    path: relative_path(&path),
                    line: index + 1,
                    name,
                    doc: doc_comment_before(&lines, index),
                });
        }
    }

    Ok(functions)
}

fn parse_pub_async_fn_name(line: &str) -> Option<String> {
    let line = line.trim_start();
    let rest = line.strip_prefix("pub async fn ")?;
    let name = rest
        .split(|ch: char| !(ch == '_' || ch.is_ascii_alphanumeric()))
        .next()?;
    if name.is_empty() {
        None
    } else {
        Some(name.to_string())
    }
}

fn doc_comment_before(lines: &[&str], function_index: usize) -> Option<String> {
    let mut docs = Vec::new();
    let mut index = function_index;

    while index > 0 {
        index -= 1;
        let line = lines[index].trim_start();
        if let Some(doc) = line.strip_prefix("///") {
            docs.push(doc.trim().to_string());
            continue;
        }
        if line.is_empty() || line.starts_with("#[") {
            continue;
        }
        break;
    }

    if docs.is_empty() {
        None
    } else {
        docs.reverse();
        Some(docs.join(" "))
    }
}

fn find_source_function<'a>(
    contract_name: &str,
    module: Option<&str>,
    endpoint: Option<&str>,
    functions: &'a BTreeMap<String, Vec<SourceFunction>>,
) -> Option<&'a SourceFunction> {
    for candidate in function_candidates(contract_name, endpoint) {
        if let Some(matches) = functions.get(&candidate) {
            if let Some(module) = module
                && let Some(function) = matches
                    .iter()
                    .find(|function| function.path.starts_with(&format!("src/{module}/")))
            {
                return Some(function);
            }
            if let Some(function) = matches.first() {
                return Some(function);
            }
        }
    }
    None
}

fn function_candidates(contract_name: &str, endpoint: Option<&str>) -> Vec<String> {
    let mut candidates = Vec::new();
    if let Some((_, name)) = contract_name.rsplit_once('.') {
        candidates.push(name.replace('-', "_"));
    }
    if let Some(endpoint) = endpoint {
        candidates.push(endpoint.replace('-', "_"));
    }
    candidates.sort();
    candidates.dedup();
    candidates
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
            "api-doc path must be a contract.json file or directory",
        ));
    }

    for entry in fs::read_dir(path)
        .map_err(|err| BpiError::parse(format!("failed to read {}: {err}", path.display())))?
    {
        let entry = entry.map_err(|err| BpiError::parse(err.to_string()))?;
        collect_contract_files(&entry.path(), files)?;
    }
    Ok(())
}

fn collect_rs_files(path: &Path, files: &mut Vec<PathBuf>) -> BpiResult<()> {
    if path.is_file() {
        if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }

    if !path.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(path)
        .map_err(|err| BpiError::parse(format!("failed to read {}: {err}", path.display())))?
    {
        let entry = entry.map_err(|err| BpiError::parse(err.to_string()))?;
        collect_rs_files(&entry.path(), files)?;
    }
    Ok(())
}

fn string_field(value: &serde_json::Value, field: &str) -> Option<String> {
    value.get(field)?.as_str().map(str::to_string)
}

fn relative_path(path: &Path) -> String {
    let cwd = std::env::current_dir().ok();
    let path = cwd
        .as_ref()
        .and_then(|cwd| path.strip_prefix(cwd).ok())
        .unwrap_or(path);
    path.to_string_lossy().replace('\\', "/")
}

fn method_name(method: HttpMethod) -> &'static str {
    match method {
        HttpMethod::Get => "GET",
        HttpMethod::Post => "POST",
    }
}

fn risk_name(risk: ApiRisk) -> &'static str {
    match risk {
        ApiRisk::PublicRead => "public-read",
        ApiRisk::AuthenticatedRead => "authenticated-read",
        ApiRisk::PrivateRead => "private-read",
        ApiRisk::Mutating => "mutating",
        ApiRisk::Spending => "spending",
        ApiRisk::LoginSession => "login-session",
    }
}

fn escape_cell(value: &str) -> String {
    value.replace('|', "\\|")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_public_async_function_name() {
        assert_eq!(
            parse_pub_async_fn_name("    pub async fn history_list(&self) -> BpiResult<()> {"),
            Some("history_list".to_string())
        );
    }

    #[test]
    fn extracts_doc_comment_before_function() {
        let lines = [
            "    /// 获取专栏概要信息。",
            "    pub async fn info(&self) -> BpiResult<()> {",
        ];

        assert_eq!(
            doc_comment_before(&lines, 1),
            Some("获取专栏概要信息。".to_string())
        );
    }

    #[test]
    fn function_candidates_include_contract_and_endpoint_names() {
        assert_eq!(
            function_candidates("historytoview.history_list", Some("history-list")),
            vec!["history_list".to_string()]
        );
    }

    #[test]
    fn catalog_generation_finds_existing_contracts() -> BpiResult<()> {
        let catalog =
            generate_api_catalog(Path::new("tests/contracts/historytoview"), Path::new("src"))?;

        assert!(
            catalog
                .entries
                .iter()
                .any(|entry| entry.name == "historytoview.history_list"
                    && entry.function_name.as_deref() == Some("history_list")
                    && entry.description.is_some())
        );
        Ok(())
    }
}
