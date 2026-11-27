use std::{env, error::Error, fs::OpenOptions, io::Write, path::Path};

use bdl_core::{
    BdlError,
    muxer::MuxError,
    public_error::{ErrorCode, ErrorCondition, PublicError, find_solution},
};
use serde_json::{Value, json};
use uuid::Uuid;

pub struct Invocation {
    trace_id: Uuid,
    chinese: bool,
}

impl Invocation {
    pub fn new() -> Self {
        Self {
            trace_id: Uuid::new_v4(),
            chinese: env::var("BDL_LANG").is_ok_and(|lang| lang.eq_ignore_ascii_case("zh-CN")),
        }
    }

    pub fn report(
        &self,
        operation: &str,
        error: &anyhow::Error,
        default_code: ErrorCode,
        json_output: bool,
    ) {
        let condition = error
            .downcast_ref::<ErrorCondition>()
            .cloned()
            .or_else(|| error.chain().find_map(typed_condition))
            .unwrap_or_else(|| ErrorCondition::new(default_code));
        let problem = PublicError {
            condition: condition.clone(),
            trace_id: self.trace_id,
        };
        let (matched, solution) =
            find_solution(condition.code.as_str()).expect("build-checked solution coverage");
        let advice = solution.select(self.chinese);
        let message = condition.message(self.chinese);
        let mut output = json!({
            "ok": false,
            "error": message,
            "problem": problem,
            "solution": { "matchedCode": matched, "advice": advice },
        });
        if let Some(dir) = env::var_os("BDL_DIAGNOSTICS_DIR") {
            let result = write_diagnostic(Path::new(&dir), &problem, operation, error);
            if let Err(io) = result {
                output["diagnosticWriteError"] = json!(format!("{:?}", io.kind()));
            }
        }
        if json_output {
            eprintln!("{output}");
        } else {
            eprintln!(
                "[{}] {message}\n{}\ntraceId={}",
                condition.code,
                advice,
                self.trace_id.simple()
            );
            if let Some(kind) = output["diagnosticWriteError"].as_str() {
                eprintln!("diagnosticWriteError={kind}");
            }
        }
    }
}

fn typed_condition(error: &(dyn Error + 'static)) -> Option<ErrorCondition> {
    if let Some(error) = error.downcast_ref::<BdlError>() {
        return Some(error.public_condition());
    }
    if let Some(error) = error.downcast_ref::<MuxError>() {
        return Some(ErrorCondition::from_mux(error));
    }
    error
        .downcast_ref::<std::io::Error>()
        .map(ErrorCondition::from_io)
}

fn diagnostic_cause(source: &(dyn Error + 'static)) -> Option<Value> {
    if let Some(io) = source.downcast_ref::<std::io::Error>() {
        return Some(json!({
            "type": "io",
            "kind": format!("{:?}", io.kind()),
            "osCode": io.raw_os_error(),
        }));
    }
    if let Some(bdl) = source.downcast_ref::<BdlError>() {
        if let BdlError::BpiHttp { status, origin, .. } = bdl {
            return Some(json!({
                "type": "BpiHttp",
                "status": status,
                "line": origin.line(),
                "file": Path::new(origin.file()).file_name().and_then(|s| s.to_str()),
            }));
        }
        return Some(json!({ "type": "BdlError", "code": bdl.public_condition().code }));
    }
    if let Some(clap) = source.downcast_ref::<clap::Error>() {
        return Some(json!({ "type": "clap", "kind": format!("{:?}", clap.kind()) }));
    }
    if let Some(data) = source.downcast_ref::<serde_json::Error>() {
        return Some(json!({ "type": "json", "line": data.line(), "column": data.column() }));
    }
    source
        .downcast_ref::<MuxError>()
        .map(|mux| json!({ "type": "MuxError", "code": ErrorCondition::from_mux(mux).code }))
}

fn write_diagnostic(
    dir: &Path,
    problem: &PublicError,
    operation: &str,
    error: &anyhow::Error,
) -> std::io::Result<()> {
    // Record only typed evidence. CLI arguments, upstream responses and error Display strings
    // can contain arbitrary user input, URLs, credentials and private paths.
    let causes: Vec<_> = error
        .chain()
        .take(16)
        .filter_map(diagnostic_cause)
        .collect();
    let bytes = serde_json::to_vec_pretty(&json!({
        "problem": problem, "component": "bdl-cli", "operation": operation, "causes": causes
    }))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(dir.join(format!("{}.json", problem.trace_id.simple())))?;
    file.write_all(&bytes)
}
