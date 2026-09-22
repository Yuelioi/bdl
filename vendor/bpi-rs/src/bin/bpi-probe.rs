use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use bpi_rs::BpiError;
use bpi_rs::probe::account::RawProbeConfig;
use bpi_rs::probe::audit::{ContractAuditLevel, audit_contracts};
use bpi_rs::probe::batch::{
    BatchRiskMode, BatchRunConfig, parse_page_count, parse_profile_set, run_batch,
};
use bpi_rs::probe::catalog::{generate_api_catalog, render_api_catalog_markdown};
use bpi_rs::probe::contract::ApiContract;
use bpi_rs::probe::endpoint_contract::ApiRisk;
use bpi_rs::probe::field_audit::audit_contract_fields;
use bpi_rs::probe::flow::{ProbeFlow, execute_flow};
use bpi_rs::probe::promote::promote_probe_output;
use bpi_rs::probe::run::execute_contract;
use bpi_rs::probe::sanitize::audit_value;

const USAGE: &str = "usage: bpi-probe fields <path> | sanitize-audit <path> | contract-audit <path> | api-doc <path> [--output docs/api-index.md] | batch-run <path> [--account account.toml] [--profiles anonymous,normal,vip] [--pages 10] [--read-only] [--output target/bpi-probe-runs] | promote <probe-output.response.json> | bpi-probe <contract.json> [account.toml] [output.json]";
const BATCH_RUN_USAGE: &str = "usage: bpi-probe batch-run <path> [--account account.toml] [--profiles anonymous,normal,vip] [--pages 10] [--read-only] [--output target/bpi-probe-runs]";

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), BpiError> {
    let mut args = env::args().skip(1);
    let first = args
        .next()
        .ok_or_else(|| BpiError::invalid_parameter("command", USAGE))?;

    if first == "-h" || first == "--help" {
        println!("{USAGE}");
        return Ok(());
    }

    if first == "fields" {
        return run_fields(args);
    }
    if first == "sanitize-audit" {
        return run_sanitize_audit(args);
    }
    if first == "contract-audit" {
        return run_contract_audit(args);
    }
    if first == "api-doc" {
        return run_api_doc(args);
    }
    if first == "batch-run" {
        return run_batch_cli(args).await;
    }
    if first == "promote" {
        return run_promote(args);
    }

    if env::var("BPI_PROBE").as_deref() != Ok("1") {
        return Err(BpiError::invalid_parameter(
            "BPI_PROBE",
            "set BPI_PROBE=1 to run network probes",
        ));
    }

    let contract_path = first;
    let account_path = args.next().unwrap_or_else(|| "account.toml".to_string());
    let output_path = args.next();

    if args.next().is_some() {
        return Err(BpiError::invalid_parameter(
            "args",
            "usage: bpi-probe <contract.json> [account.toml] [output.json]",
        ));
    }

    let contract_bytes = fs::read(&contract_path).map_err(|err| {
        BpiError::parse(format!(
            "failed to read contract file {contract_path}: {err}"
        ))
    })?;
    let accounts = RawProbeConfig::load(&account_path)?;
    let input: serde_json::Value = serde_json::from_slice(&contract_bytes)?;
    let output = if input.get("steps").is_some() {
        let flow = ProbeFlow::from_slice(&contract_bytes)?;
        let result = execute_flow(&flow, &accounts).await?;
        serde_json::to_string_pretty(&result)?
    } else {
        let contract = ApiContract::from_value(input)?;
        let result = execute_contract(&contract, &accounts).await?;
        serde_json::to_string_pretty(&result)?
    };

    if let Some(output_path) = output_path {
        fs::write(&output_path, format!("{output}\n")).map_err(|err| {
            BpiError::parse(format!(
                "failed to write probe output file {output_path}: {err}"
            ))
        })?;
    } else {
        println!("{output}");
    }
    Ok(())
}

fn run_fields(mut args: impl Iterator<Item = String>) -> Result<(), BpiError> {
    let path = args
        .next()
        .ok_or_else(|| BpiError::invalid_parameter("path", "usage: bpi-probe fields <path>"))?;
    if args.next().is_some() {
        return Err(BpiError::invalid_parameter(
            "args",
            "usage: bpi-probe fields <path>",
        ));
    }

    let report = audit_contract_fields(path)?;
    let stdout = io::stdout();
    let mut output = stdout.lock();
    if !write_line(&mut output, &format!("files\t{}", report.files))? {
        return Ok(());
    }
    if !write_line(&mut output, "count\tkinds\tpath")? {
        return Ok(());
    }
    for field in report.fields {
        let kinds = field
            .value_kinds
            .iter()
            .map(|kind| format!("{kind:?}").to_ascii_lowercase())
            .collect::<Vec<_>>()
            .join(",");
        if !write_line(
            &mut output,
            &format!("{}\t{}\t{}", field.count, kinds, field.path),
        )? {
            return Ok(());
        }
    }

    Ok(())
}

fn run_sanitize_audit(mut args: impl Iterator<Item = String>) -> Result<(), BpiError> {
    let path = args.next().ok_or_else(|| {
        BpiError::invalid_parameter("path", "usage: bpi-probe sanitize-audit <path>")
    })?;
    if args.next().is_some() {
        return Err(BpiError::invalid_parameter(
            "args",
            "usage: bpi-probe sanitize-audit <path>",
        ));
    }

    let files = response_json_files(Path::new(&path))?;
    let mut finding_count = 0usize;
    for file in files {
        let bytes = fs::read(&file)
            .map_err(|err| BpiError::parse(format!("failed to read {}: {err}", file.display())))?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|err| BpiError::parse(format!("failed to parse {}: {err}", file.display())))?;
        let findings = audit_value(&value, Some(ApiRisk::PublicRead));
        for finding in findings {
            finding_count += 1;
            println!("{}\t{}\t{}", file.display(), finding.path, finding.reason);
        }
    }

    if finding_count == 0 {
        println!("sanitize-audit ok: no high-confidence sensitive fields found");
        Ok(())
    } else {
        Err(BpiError::unsupported_response(format!(
            "sanitize-audit found {finding_count} sensitive field candidates"
        )))
    }
}

fn run_contract_audit(mut args: impl Iterator<Item = String>) -> Result<(), BpiError> {
    let path = args.next().ok_or_else(|| {
        BpiError::invalid_parameter("path", "usage: bpi-probe contract-audit <path>")
    })?;
    if args.next().is_some() {
        return Err(BpiError::invalid_parameter(
            "args",
            "usage: bpi-probe contract-audit <path>",
        ));
    }

    let report = audit_contracts(path)?;
    for finding in &report.findings {
        let level = match finding.level {
            ContractAuditLevel::Error => "error",
            ContractAuditLevel::Warning => "warning",
        };
        println!("{}\t{}\t{}", level, finding.path.display(), finding.message);
    }
    println!(
        "contract-audit summary: contracts={} errors={} warnings={}",
        report.contracts, report.errors, report.warnings
    );

    if report.is_success() {
        Ok(())
    } else {
        Err(BpiError::unsupported_response(format!(
            "contract-audit found {} errors",
            report.errors
        )))
    }
}

async fn run_batch_cli(mut args: impl Iterator<Item = String>) -> Result<(), BpiError> {
    let root = args
        .next()
        .ok_or_else(|| BpiError::invalid_parameter("path", BATCH_RUN_USAGE))?;
    if root == "-h" || root == "--help" {
        println!("{BATCH_RUN_USAGE}");
        return Ok(());
    }
    let mut account_path = PathBuf::from("account.toml");
    let mut output_root = PathBuf::from("target/bpi-probe-runs");
    let mut profiles = parse_profile_set("anonymous,normal,vip")?;
    let mut pages = 10usize;
    let mut risk_mode = BatchRiskMode::All;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--account" => {
                account_path = PathBuf::from(args.next().ok_or_else(|| {
                    BpiError::invalid_parameter("account", "--account requires a path")
                })?);
            }
            "--profiles" => {
                profiles = parse_profile_set(&args.next().ok_or_else(|| {
                    BpiError::invalid_parameter("profiles", "--profiles requires a comma list")
                })?)?;
            }
            "--output" => {
                output_root = PathBuf::from(args.next().ok_or_else(|| {
                    BpiError::invalid_parameter("output", "--output requires a path")
                })?);
            }
            "--pages" => {
                pages = parse_page_count(&args.next().ok_or_else(|| {
                    BpiError::invalid_parameter("pages", "--pages requires a positive integer")
                })?)?;
            }
            "--read-only" => {
                risk_mode = BatchRiskMode::ReadOnly;
            }
            _ => {
                return Err(BpiError::invalid_parameter(
                    "args",
                    "supported batch-run flags are --account, --profiles, --pages, --read-only, and --output",
                ));
            }
        }
    }

    let mut config = BatchRunConfig::from_env(
        PathBuf::from(root),
        account_path,
        output_root,
        profiles,
        pages,
    );
    config.risk_mode = risk_mode;
    let summary = run_batch(config).await?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

fn run_api_doc(mut args: impl Iterator<Item = String>) -> Result<(), BpiError> {
    let root = args.next().ok_or_else(|| {
        BpiError::invalid_parameter(
            "path",
            "usage: bpi-probe api-doc <path> [--output docs/api-index.md]",
        )
    })?;
    if root == "-h" || root == "--help" {
        println!("usage: bpi-probe api-doc <path> [--output docs/api-index.md]");
        return Ok(());
    }

    let mut output_path = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => {
                output_path = Some(PathBuf::from(args.next().ok_or_else(|| {
                    BpiError::invalid_parameter("output", "--output requires a path")
                })?));
            }
            _ => {
                return Err(BpiError::invalid_parameter(
                    "args",
                    "supported api-doc flags are --output",
                ));
            }
        }
    }

    let catalog = generate_api_catalog(Path::new(&root), Path::new("src"))?;
    let output = render_api_catalog_markdown(&catalog);
    if let Some(output_path) = output_path {
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                BpiError::parse(format!("failed to create {}: {err}", parent.display()))
            })?;
        }
        fs::write(&output_path, output).map_err(|err| {
            BpiError::parse(format!("failed to write {}: {err}", output_path.display()))
        })?;
    } else {
        print!("{output}");
    }
    Ok(())
}

fn run_promote(mut args: impl Iterator<Item = String>) -> Result<(), BpiError> {
    let input = args.next().ok_or_else(|| {
        BpiError::invalid_parameter(
            "input",
            "usage: bpi-probe promote <probe-output.response.json>",
        )
    })?;
    if args.next().is_some() {
        return Err(BpiError::invalid_parameter(
            "args",
            "usage: bpi-probe promote <probe-output.response.json>",
        ));
    }

    let summary = promote_probe_output(input)?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

fn response_json_files(root: &Path) -> Result<Vec<PathBuf>, BpiError> {
    let mut files = Vec::new();
    collect_response_json_files(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_response_json_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), BpiError> {
    if path.is_file() {
        if path.extension().and_then(|value| value.to_str()) == Some("json")
            && path
                .components()
                .any(|component| component.as_os_str() == "responses")
        {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }

    if !path.is_dir() {
        return Err(BpiError::invalid_parameter(
            "path",
            "sanitize audit path must be a JSON response file or directory",
        ));
    }

    let entries = fs::read_dir(path)
        .map_err(|err| BpiError::parse(format!("failed to read {}: {err}", path.display())))?;
    for entry in entries {
        let entry = entry.map_err(|err| BpiError::parse(err.to_string()))?;
        collect_response_json_files(&entry.path(), files)?;
    }
    Ok(())
}

fn write_line(output: &mut impl Write, line: &str) -> Result<bool, BpiError> {
    match writeln!(output, "{line}") {
        Ok(()) => Ok(true),
        Err(err) if err.kind() == io::ErrorKind::BrokenPipe => Ok(false),
        Err(err) => Err(BpiError::parse(err.to_string())),
    }
}
