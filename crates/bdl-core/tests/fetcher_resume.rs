use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use bdl_core::fetcher::{
    FetchConfig, FetchState, Fetcher, ReqwestFetcher, state_path_for, write_fetch_state,
};
use bdl_core::model::HeaderPair;
use bdl_core::queue::{
    DownloadResource, DownloadResourceIntent, DownloadResourceKind, ResourceStatus,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use uuid::Uuid;

#[tokio::test]
async fn fetcher_downloads_full_resource_and_sends_headers() {
    let server = TestServer::spawn(b"hello from bdl".to_vec(), 0).await;
    let dir = temp_case_dir("full").await;
    let resource = resource(server.url(), &dir, "full.bin");
    let fetcher = ReqwestFetcher::with_config(FetchConfig { max_retries: 0 });
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();

    let outcome = fetcher
        .fetch(&resource, Some(tx))
        .await
        .expect("resource should download");

    assert_eq!(outcome.bytes_written, 14);
    assert_eq!(
        tokio::fs::read(&resource.target_path).await.unwrap(),
        b"hello from bdl"
    );
    assert!(!resource.temp_path.exists());
    assert!(!state_path_for(&resource.temp_path).exists());

    let progress = rx.recv().await.expect("progress should be emitted");
    assert_eq!(progress.resource_id, resource.id);
    assert_eq!(progress.total_bytes, Some(14));

    let headers = server.last_headers().await;
    assert_eq!(
        headers.get("referer").map(String::as_str),
        Some("https://www.bilibili.com/")
    );
    assert_eq!(
        headers.get("cookie").map(String::as_str),
        Some("SESSDATA=test")
    );
}

#[tokio::test]
async fn fetcher_resumes_existing_bdlpart_with_range_request() {
    let server = TestServer::spawn(b"abcdefghij".to_vec(), 0).await;
    let dir = temp_case_dir("resume").await;
    let resource = resource(server.url(), &dir, "resume.bin");
    tokio::fs::write(&resource.temp_path, b"abcd")
        .await
        .unwrap();
    write_fetch_state(
        &resource.temp_path,
        &FetchState {
            total_bytes: Some(10),
            downloaded_bytes: 4,
        },
    )
    .await
    .unwrap();
    let fetcher = ReqwestFetcher::with_config(FetchConfig { max_retries: 0 });

    fetcher
        .fetch(&resource, None)
        .await
        .expect("resource should resume");

    assert_eq!(
        tokio::fs::read(&resource.target_path).await.unwrap(),
        b"abcdefghij"
    );
    assert_eq!(server.ranges().await, vec![Some("bytes=4-".to_owned())]);
}

#[tokio::test]
async fn fetcher_stops_after_configured_retry_count() {
    let server = TestServer::spawn(b"retry".to_vec(), 3).await;
    let dir = temp_case_dir("retry").await;
    let resource = resource(server.url(), &dir, "retry.bin");
    let fetcher = ReqwestFetcher::with_config(FetchConfig { max_retries: 2 });

    let error = fetcher
        .fetch(&resource, None)
        .await
        .expect_err("forced failures should exhaust retries");

    assert!(error.to_string().contains("3 attempts"));
    assert_eq!(server.get_count(), 3);
    assert!(!resource.target_path.exists());
}

#[tokio::test]
async fn fetcher_restarts_when_content_length_changes_between_attempts() {
    let server = TestServer::spawn(b"abcdefghijkl".to_vec(), 0).await;
    let dir = temp_case_dir("changed-length").await;
    let resource = resource(server.url(), &dir, "changed.bin");
    tokio::fs::write(&resource.temp_path, b"abcd")
        .await
        .unwrap();
    write_fetch_state(
        &resource.temp_path,
        &FetchState {
            total_bytes: Some(10),
            downloaded_bytes: 4,
        },
    )
    .await
    .unwrap();
    let fetcher = ReqwestFetcher::with_config(FetchConfig { max_retries: 0 });

    fetcher
        .fetch(&resource, None)
        .await
        .expect("resource should restart cleanly");

    assert_eq!(
        tokio::fs::read(&resource.target_path).await.unwrap(),
        b"abcdefghijkl"
    );
    assert_eq!(server.ranges().await, vec![None]);
}

fn resource(url: String, dir: &Path, file_name: &str) -> DownloadResource {
    let target_path = dir.join(file_name);
    DownloadResource {
        id: format!("resource:{file_name}"),
        kind: DownloadResourceKind::Video,
        intent: DownloadResourceIntent::Video,
        current_urls: vec![url],
        headers: vec![
            HeaderPair {
                name: "Referer".to_owned(),
                value: "https://www.bilibili.com/".to_owned(),
            },
            HeaderPair {
                name: "Cookie".to_owned(),
                value: "SESSDATA=test".to_owned(),
            },
        ],
        temp_path: target_path.with_extension("bin.bdlpart"),
        target_path,
        status: ResourceStatus::Pending,
    }
}

async fn temp_case_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("bdl-{name}-{}", Uuid::new_v4()));
    tokio::fs::create_dir_all(&dir).await.unwrap();
    dir
}

struct TestServer {
    base_url: String,
    state: Arc<TestServerState>,
}

struct TestServerState {
    data: Vec<u8>,
    fail_gets: AtomicUsize,
    get_count: AtomicUsize,
    ranges: Mutex<Vec<Option<String>>>,
    headers: Mutex<Vec<HashMap<String, String>>>,
}

impl TestServer {
    async fn spawn(data: Vec<u8>, fail_gets: usize) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let state = Arc::new(TestServerState {
            data,
            fail_gets: AtomicUsize::new(fail_gets),
            get_count: AtomicUsize::new(0),
            ranges: Mutex::new(Vec::new()),
            headers: Mutex::new(Vec::new()),
        });
        let server_state = state.clone();

        tokio::spawn(async move {
            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                let state = server_state.clone();
                tokio::spawn(async move {
                    handle_connection(stream, state).await;
                });
            }
        });

        Self {
            base_url: format!("http://{addr}/file"),
            state,
        }
    }

    fn url(&self) -> String {
        self.base_url.clone()
    }

    async fn ranges(&self) -> Vec<Option<String>> {
        self.state.ranges.lock().await.clone()
    }

    async fn last_headers(&self) -> HashMap<String, String> {
        self.state
            .headers
            .lock()
            .await
            .last()
            .cloned()
            .unwrap_or_default()
    }

    fn get_count(&self) -> usize {
        self.state.get_count.load(Ordering::SeqCst)
    }
}

async fn handle_connection(mut stream: TcpStream, state: Arc<TestServerState>) {
    let mut buffer = vec![0_u8; 4096];
    let Ok(read) = stream.read(&mut buffer).await else {
        return;
    };
    let request = String::from_utf8_lossy(&buffer[..read]);
    let mut lines = request.split("\r\n");
    let Some(request_line) = lines.next() else {
        return;
    };
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next().unwrap_or_default();
    let headers = parse_headers(lines);

    match method {
        "HEAD" => {
            write_response(
                &mut stream,
                200,
                "OK",
                &[("Content-Length", state.data.len())],
                b"",
            )
            .await;
        }
        "GET" => {
            state.get_count.fetch_add(1, Ordering::SeqCst);
            state.headers.lock().await.push(headers.clone());
            state
                .ranges
                .lock()
                .await
                .push(headers.get("range").cloned());

            if state.fail_gets.load(Ordering::SeqCst) > 0 {
                state.fail_gets.fetch_sub(1, Ordering::SeqCst);
                write_response(&mut stream, 500, "FAIL", &[("Content-Length", 0)], b"").await;
                return;
            }

            let start = headers
                .get("range")
                .and_then(|range| range.strip_prefix("bytes="))
                .and_then(|range| range.strip_suffix('-'))
                .and_then(|start| start.parse::<usize>().ok())
                .unwrap_or(0);
            if start > 0 {
                let body = &state.data[start..];
                write_response(
                    &mut stream,
                    206,
                    "PARTIAL",
                    &[("Content-Length", body.len())],
                    body,
                )
                .await;
            } else {
                write_response(
                    &mut stream,
                    200,
                    "OK",
                    &[("Content-Length", state.data.len())],
                    &state.data,
                )
                .await;
            }
        }
        _ => {
            write_response(&mut stream, 405, "METHOD", &[("Content-Length", 0)], b"").await;
        }
    }
}

fn parse_headers<'a>(lines: impl Iterator<Item = &'a str>) -> HashMap<String, String> {
    lines
        .take_while(|line| !line.is_empty())
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.to_ascii_lowercase(), value.trim().to_owned()))
        .collect()
}

async fn write_response(
    stream: &mut TcpStream,
    status: u16,
    reason: &str,
    headers: &[(&str, usize)],
    body: &[u8],
) {
    let mut response = format!("HTTP/1.1 {status} {reason}\r\nConnection: close\r\n");
    for (name, value) in headers {
        response.push_str(&format!("{name}: {value}\r\n"));
    }
    response.push_str("\r\n");
    stream.write_all(response.as_bytes()).await.unwrap();
    stream.write_all(body).await.unwrap();
}
