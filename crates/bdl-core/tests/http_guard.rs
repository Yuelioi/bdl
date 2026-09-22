use bdl_core::resolver::http_guard::HttpGuard;
use bpi_rs::transport::{
    ReqwestTransport,
    observer::{RequestObserver, scope},
};
use std::{
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

fn path() -> PathBuf {
    std::env::temp_dir().join(format!("bdl-http-{}.json", uuid::Uuid::new_v4()))
}
fn server(status: &str, body: &str, extra: &str) -> (String, std::thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/test", listener.local_addr().unwrap());
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nContent-Type: application/json\r\n{extra}Connection: close\r\n\r\n{body}",
        body.len()
    );
    let handle = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let mut request = [0; 4096];
        stream.read(&mut request).unwrap();
        let _ = stream.write_all(response.as_bytes());
    });
    (url, handle)
}
fn cleanup(path: &PathBuf) {
    std::fs::remove_file(path).unwrap();
    std::fs::remove_file(path.with_extension("lock")).unwrap();
}

#[tokio::test]
async fn actual_transport_counts_attempts_and_blocks_before_second_send() {
    let path = path();
    let guard = Arc::new(HttpGuard::new(path.clone(), Duration::ZERO, 1));
    let (url, server) = server("200 OK", r#"{"code":0,"data":{}}"#, "");
    let client = reqwest::Client::builder().no_proxy().build().unwrap();
    scope(guard, async {
        ReqwestTransport::send_request_builder(client.get(&url), "fixture")
            .await
            .unwrap();
        let error = ReqwestTransport::send_request_builder(client.get(&url), "fixture")
            .await
            .unwrap_err();
        assert!(error.to_string().contains("budget exhausted"));
    })
    .await;
    server.join().unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved["requests"], 1);
    cleanup(&path);
}

#[tokio::test]
async fn transient_get_retries_once_and_both_attempts_count() {
    let path = path();
    let guard = Arc::new(HttpGuard::new(path.clone(), Duration::ZERO, 2));
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/test", listener.local_addr().unwrap());
    let server = std::thread::spawn(move || {
        for status in ["503 Service Unavailable", "200 OK"] {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = [0; 4096];
            socket.read(&mut request).unwrap();
            socket
                .write_all(
                    format!(
                        "HTTP/1.1 {status}\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}"
                    )
                    .as_bytes(),
                )
                .unwrap();
        }
    });
    let client = reqwest::Client::builder()
        .no_proxy()
        .retry(reqwest::retry::never())
        .build()
        .unwrap();
    scope(
        guard,
        ReqwestTransport::send_request_builder(client.get(&url), "fixture"),
    )
    .await
    .unwrap();
    server.join().unwrap();
    let saved: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved["requests"], 2);
    cleanup(&path);
}

#[tokio::test]
async fn http_and_api_restrictions_persist_across_independent_guards() {
    for (status, body, headers) in [
        ("429 Too Many Requests", "{}", "Retry-After: 600\r\n"),
        ("200 OK", r#"{"code":-352,"message":"risk"}"#, ""),
    ] {
        let path = path();
        let guard = Arc::new(HttpGuard::new(path.clone(), Duration::ZERO, 10));
        let (url, server) = server(status, body, headers);
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let result = scope(
            guard,
            ReqwestTransport::send_request_builder(client.get(&url), "fixture"),
        )
        .await;
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("source restriction")
        );
        server.join().unwrap();
        let independent = HttpGuard::new(path.clone(), Duration::ZERO, 10);
        assert!(
            independent
                .before()
                .await
                .err()
                .unwrap()
                .to_string()
                .contains("shared cooldown")
        );
        cleanup(&path);
    }
}

#[tokio::test]
async fn shared_budget_cannot_be_reset_by_new_guard() {
    let path = path();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    std::fs::write(&path, serde_json::to_vec(&serde_json::json!({"window_started_ms":now,"requests":7200,"next_ms":0,"blocked_until_ms":0})).unwrap()).unwrap();
    let guard = HttpGuard::new(path.clone(), Duration::ZERO, 1000);
    assert!(
        guard
            .before()
            .await
            .err()
            .unwrap()
            .to_string()
            .contains("shared HTTP budget")
    );
    cleanup(&path);
}

#[test]
#[ignore = "subprocess fixture; only started by lock_release_after_process_termination"]
fn http_lock_holder() {
    let path = PathBuf::from(std::env::var_os("BDL_HTTP_TEST_STATE").unwrap());
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        let guard = HttpGuard::new(path.clone(), Duration::ZERO, 10);
        let _permit = guard.before().await.unwrap();
        std::fs::write(path.with_extension("ready"), b"ready").unwrap();
        tokio::time::sleep(Duration::from_secs(30)).await;
    });
}

#[tokio::test]
async fn lock_release_after_process_termination() {
    let path = path();
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "http_lock_holder", "--ignored"])
        .env("BDL_HTTP_TEST_STATE", &path)
        .stdout(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let ready = path.with_extension("ready");
    let start = std::time::Instant::now();
    while !ready.exists() && start.elapsed() < Duration::from_secs(10) {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let competing = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path.with_extension("lock"))
        .unwrap();
    let blocked = fs2::FileExt::try_lock_exclusive(&competing).is_err();
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(ready.exists(), "child must obtain the OS lock");
    assert!(
        blocked,
        "second process must not overlap the first HTTP request"
    );
    fs2::FileExt::try_lock_exclusive(&competing).unwrap();
    drop(competing);
    let state: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(state["requests"], 1, "crash must preserve the used budget");
    assert!(state["next_ms"].as_u64().unwrap() > state["window_started_ms"].as_u64().unwrap());
    std::fs::remove_file(ready).unwrap();
    cleanup(&path);
}

#[tokio::test]
async fn shared_half_second_spacing_has_no_tenth_request_pause() {
    let path = path();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    std::fs::write(&path, serde_json::to_vec(&serde_json::json!({"window_started_ms":now,"requests":9,"next_ms":0,"blocked_until_ms":0})).unwrap()).unwrap();
    let first = HttpGuard::new(path.clone(), Duration::from_millis(500), 20);
    let start = std::time::Instant::now();
    drop(first.before().await.unwrap());
    let state: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(state["requests"], 10);
    let next = state["next_ms"].as_u64().unwrap();
    assert!(next >= now + 500 && next < now + 2000);
    let second = HttpGuard::new(path.clone(), Duration::from_millis(500), 20);
    drop(second.before().await.unwrap());
    assert!(start.elapsed() >= Duration::from_millis(450));
    cleanup(&path);
}
