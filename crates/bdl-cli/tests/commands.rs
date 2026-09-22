use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bdl"))
        .env_remove("BDL_COOKIE")
        .args(args)
        .output()
        .expect("CLI should start")
}

fn json_error(output: &Output, code: i32) -> serde_json::Value {
    assert_eq!(output.status.code(), Some(code));
    assert!(output.stdout.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stderr).expect("JSON stderr");
    assert_eq!(value["ok"], false);
    value
}

#[test]
fn invalid_arguments_have_json_errors_and_usage_exit_code() {
    let output = run(&["download", "BV1xx411c7mD", "--json"]);
    let error = json_error(&output, 2);
    assert!(error["error"].as_str().unwrap().contains("--output"));
}

#[test]
fn invalid_quality_fails_before_network_or_ffmpeg_lookup() {
    let output = run(&[
        "--json",
        "download",
        "BV1xx411c7mD",
        "--parts",
        "1",
        "-o",
        ".",
        "--quality",
        "invalid",
        "--ffmpeg",
        "missing-ffmpeg",
    ]);
    let error = json_error(&output, 1);
    assert!(!error["error"].as_str().unwrap().contains("ffmpeg"));
}

#[test]
fn unbounded_download_and_all_without_checkpoint_fail_before_network() {
    for args in [
        vec!["download", "BV1xx411c7mD", "-o", ".", "--json"],
        vec!["download", "BV1xx411c7mD", "-o", ".", "--all", "--json"],
        vec!["parse", "BV1xx411c7mD", "--all", "--json"],
    ] {
        let error = json_error(&run(&args), 1);
        assert!(error["error"].as_str().unwrap().contains("--state"));
    }
    json_error(
        &run(&["parse", "BV1xx411c7mD", "--max-operations", "0", "--json"]),
        2,
    );
    json_error(
        &run(&["parse", "BV1xx411c7mD", "--interval-seconds", "0", "--json"]),
        2,
    );
}

#[test]
fn cookie_check_does_not_print_secrets() {
    for cookie in [
        "SESSDATA=fixture-secret",
        "SESSDATA=",
        "other=fixture-secret",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_bdl"))
            .env("BDL_COOKIE", cookie)
            .args(["verify-cookie", "--json"])
            .output()
            .unwrap();
        assert!(!String::from_utf8_lossy(&output.stderr).contains("fixture-secret"));
        if cookie == "SESSDATA=fixture-secret" {
            assert!(output.status.success());
            let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            assert_eq!(value, serde_json::json!({"valid": true}));
        } else {
            json_error(&output, 1);
        }
    }
}

#[test]
fn missing_ffmpeg_is_a_structured_diagnostic() {
    let output = run(&["ffmpeg-check", "--ffmpeg", "missing-ffmpeg", "--json"]);
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["available"], false);
}

#[test]
fn help_is_available_without_cookie_or_network() {
    let output = run(&["--help"]);
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("download"));
}

#[test]
fn fractional_interval_accepts_half_second_and_rejects_invalid_values() {
    let output = run(&["verify-cookie", "--interval-seconds", "0.5", "--json"]);
    assert_ne!(output.status.code(), Some(2));
    for value in ["0.49", "NaN", "inf", "121"] {
        json_error(
            &run(&["verify-cookie", "--interval-seconds", value, "--json"]),
            2,
        );
    }
}
