use std::collections::HashMap;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use bdl_core::fetcher::{
    BandwidthLimiter, FetchCancelToken, FetchConfig, FetchState, Fetcher, ReqwestFetcher,
    state_path_for, write_fetch_state,
};
use bdl_core::model::HeaderPair;
use bdl_core::queue::{
    DownloadResource, DownloadResourceIntent, DownloadResourceKind, ResourceStatus,
};
use flate2::Compression;
use flate2::write::DeflateEncoder;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::sync::oneshot;
use tokio::time::{Duration, timeout};
use uuid::Uuid;

#[tokio::test]
async fn fetcher_downloads_full_resource_and_sends_headers() {
    let server = TestServer::spawn(b"hello from bdl".to_vec(), 0).await;
    let dir = temp_case_dir("full").await;
    let resource = resource(server.url(), &dir, "full.bin");
    let fetcher = ReqwestFetcher::with_config(FetchConfig {
        max_retries: 0,
        ..FetchConfig::default()
    })
    .expect("fetcher should be created");
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
async fn cli_restriction_mode_stops_before_cdn_fallback_or_retry() {
    let blocked = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}/blocked", blocked.local_addr().unwrap());
    let first = tokio::spawn(async move {
        let (mut stream, _) = blocked.accept().await.unwrap();
        let mut request = [0; 4096];
        stream.read(&mut request).await.unwrap();
        stream
            .write_all(
                b"HTTP/1.1 429 Too Many Requests\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            )
            .await
            .unwrap();
    });
    let backup = TestServer::spawn(b"must not fetch".to_vec(), 0).await;
    let dir = temp_case_dir("restriction").await;
    let mut resource = resource(url, &dir, "blocked.bin");
    resource.current_urls.push(backup.url());
    let fetcher = ReqwestFetcher::new().unwrap().with_stop_on_restriction();
    let error = fetcher.fetch(&resource, None).await.unwrap_err();
    assert!(error.to_string().contains("429"));
    assert_eq!(backup.get_count(), 0);
    first.await.unwrap();
}

#[tokio::test]
async fn fetcher_downloads_raw_deflate_response() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8"?><i><chatid>39092555045</chatid></i>"#;
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(xml).unwrap();
    let compressed = encoder.finish().unwrap();
    let server = TestServer::spawn_encoded(compressed, "deflate").await;
    let dir = temp_case_dir("raw-deflate").await;
    let mut resource = resource(server.url(), &dir, "danmaku.xml");
    resource.kind = DownloadResourceKind::Asset;
    resource.intent = DownloadResourceIntent::Danmaku;
    let fetcher = ReqwestFetcher::with_config(FetchConfig {
        max_retries: 0,
        ..FetchConfig::default()
    })
    .expect("fetcher should be created");

    fetcher
        .fetch(&resource, None)
        .await
        .expect("raw-deflate danmaku should download");

    assert_eq!(tokio::fs::read(&resource.target_path).await.unwrap(), xml);
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
            etag: None,
            last_modified: None,
        },
    )
    .await
    .unwrap();
    let fetcher = ReqwestFetcher::with_config(FetchConfig {
        max_retries: 0,
        ..FetchConfig::default()
    })
    .expect("fetcher should be created");

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
    let fetcher = ReqwestFetcher::with_config(FetchConfig {
        max_retries: 2,
        ..FetchConfig::default()
    })
    .expect("fetcher should be created");

    let error = fetcher
        .fetch(&resource, None)
        .await
        .expect_err("forced failures should exhaust retries");

    assert!(error.to_string().contains("3 attempts"));
    assert_eq!(server.get_count(), 3);
    assert!(!resource.target_path.exists());
}

#[tokio::test]
async fn fetcher_uses_backup_url_before_exhausting_resource() {
    let primary = TestServer::spawn(b"primary".to_vec(), 1).await;
    let backup = TestServer::spawn(b"backup".to_vec(), 0).await;
    let dir = temp_case_dir("backup-url").await;
    let mut resource = resource(primary.url(), &dir, "backup.bin");
    resource.current_urls.push(backup.url());
    let fetcher = ReqwestFetcher::with_config(FetchConfig {
        max_retries: 0,
        ..FetchConfig::default()
    })
    .expect("fetcher should be created");

    fetcher
        .fetch(&resource, None)
        .await
        .expect("backup URL should download");

    assert_eq!(
        tokio::fs::read(&resource.target_path).await.unwrap(),
        b"backup"
    );
    assert_eq!(primary.get_count(), 1);
    assert_eq!(backup.get_count(), 1);
}

#[tokio::test]
async fn fetcher_downloads_segments_with_configured_segment_count() {
    let server = TestServer::spawn(b"abcdefgh".to_vec(), 0).await;
    let dir = temp_case_dir("segments").await;
    let resource = resource(server.url(), &dir, "segments.bin");
    let fetcher = ReqwestFetcher::with_config(FetchConfig {
        max_retries: 0,
        segment_count: 4,
        ..FetchConfig::default()
    })
    .expect("fetcher should be created");

    fetcher
        .fetch(&resource, None)
        .await
        .expect("resource should download in segments");

    assert_eq!(
        tokio::fs::read(&resource.target_path).await.unwrap(),
        b"abcdefgh"
    );
    assert_eq!(
        sorted_range_headers(server.ranges().await),
        vec![
            "bytes=0-1".to_owned(),
            "bytes=2-3".to_owned(),
            "bytes=4-5".to_owned(),
            "bytes=6-7".to_owned(),
        ]
    );
}

#[tokio::test]
async fn segmented_fetch_shares_one_task_speed_limit() {
    let server = TestServer::spawn(b"abcdefgh".to_vec(), 0).await;
    let dir = temp_case_dir("segment-limit").await;
    let resource = resource(server.url(), &dir, "segment-limit.bin");
    let fetcher = ReqwestFetcher::with_config(FetchConfig {
        max_retries: 0,
        segment_count: 4,
        speed_limit_bytes_per_second: Some(4),
        ..FetchConfig::default()
    })
    .expect("fetcher should be created");
    let started = std::time::Instant::now();

    fetcher
        .fetch(&resource, None)
        .await
        .expect("segmented resource should download within one task budget");

    assert!(started.elapsed() >= Duration::from_millis(800));
}

#[tokio::test]
async fn concurrent_fetchers_share_one_global_speed_limit() {
    let server = TestServer::spawn(b"abcdefgh".to_vec(), 0).await;
    let dir = temp_case_dir("global-limit").await;
    let first = resource(server.url(), &dir, "global-first.bin");
    let second = resource(server.url(), &dir, "global-second.bin");
    let limiter = Arc::new(BandwidthLimiter::new(Some(8)));
    let first_fetcher = ReqwestFetcher::with_global_limiter(
        FetchConfig {
            max_retries: 0,
            ..FetchConfig::default()
        },
        limiter.clone(),
    )
    .expect("first fetcher should be created");
    let second_fetcher = ReqwestFetcher::with_global_limiter(
        FetchConfig {
            max_retries: 0,
            ..FetchConfig::default()
        },
        limiter,
    )
    .expect("second fetcher should be created");
    let started = std::time::Instant::now();

    let (first_result, second_result) = tokio::join!(
        first_fetcher.fetch(&first, None),
        second_fetcher.fetch(&second, None),
    );

    first_result.expect("first resource should download");
    second_result.expect("second resource should download");
    assert!(started.elapsed() >= Duration::from_millis(800));
}

#[tokio::test]
async fn fetcher_resumes_existing_segment_parts() {
    let server = TestServer::spawn(b"abcdefgh".to_vec(), 0).await;
    let dir = temp_case_dir("segment-resume").await;
    let resource = resource(server.url(), &dir, "segment-resume.bin");
    tokio::fs::write(segment_path(&resource.temp_path, 0), b"ab")
        .await
        .unwrap();
    tokio::fs::write(segment_path(&resource.temp_path, 1), b"c")
        .await
        .unwrap();
    write_fetch_state(
        &resource.temp_path,
        &FetchState {
            total_bytes: Some(8),
            downloaded_bytes: 0,
            etag: None,
            last_modified: None,
        },
    )
    .await
    .unwrap();
    let fetcher = ReqwestFetcher::with_config(FetchConfig {
        max_retries: 0,
        segment_count: 4,
        ..FetchConfig::default()
    })
    .expect("fetcher should be created");

    fetcher
        .fetch(&resource, None)
        .await
        .expect("existing segment parts should resume");

    assert_eq!(
        tokio::fs::read(&resource.target_path).await.unwrap(),
        b"abcdefgh"
    );
    assert_eq!(
        sorted_range_headers(server.ranges().await),
        vec![
            "bytes=3-3".to_owned(),
            "bytes=4-5".to_owned(),
            "bytes=6-7".to_owned(),
        ]
    );
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
            etag: None,
            last_modified: None,
        },
    )
    .await
    .unwrap();
    let fetcher = ReqwestFetcher::with_config(FetchConfig {
        max_retries: 0,
        ..FetchConfig::default()
    })
    .expect("fetcher should be created");

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

#[tokio::test]
async fn fetcher_restarts_when_saved_etag_differs_from_remote_etag() {
    let server =
        TestServer::spawn_with_validators(b"abcdefghij".to_vec(), 0, Some("\"new\""), None).await;
    let dir = temp_case_dir("changed-etag").await;
    let resource = resource(server.url(), &dir, "changed-etag.bin");
    tokio::fs::write(&resource.temp_path, b"abcd")
        .await
        .unwrap();
    write_fetch_state(
        &resource.temp_path,
        &FetchState {
            total_bytes: Some(10),
            downloaded_bytes: 4,
            etag: Some("\"old\"".to_owned()),
            last_modified: None,
        },
    )
    .await
    .unwrap();
    let fetcher = ReqwestFetcher::with_config(FetchConfig {
        max_retries: 0,
        ..FetchConfig::default()
    })
    .expect("fetcher should be created");

    fetcher
        .fetch(&resource, None)
        .await
        .expect("resource should restart when etag changes");

    assert_eq!(
        tokio::fs::read(&resource.target_path).await.unwrap(),
        b"abcdefghij"
    );
    assert_eq!(server.ranges().await, vec![None]);
}

#[tokio::test]
async fn cancellation_interrupts_a_stalled_download_request() {
    let (url, request_started) = spawn_stalled_server().await;
    let dir = temp_case_dir("cancel-stalled").await;
    let resource = resource(url, &dir, "cancel-stalled.bin");
    let fetcher = ReqwestFetcher::with_config(FetchConfig {
        max_retries: 0,
        ..FetchConfig::default()
    })
    .expect("fetcher should be created");
    let cancel_token = FetchCancelToken::new();
    let fetch_cancel_token = cancel_token.clone();

    let fetch = tokio::spawn(async move {
        fetcher
            .fetch_cancelable(&resource, None, fetch_cancel_token)
            .await
    });
    request_started
        .await
        .expect("download request should reach the server");

    cancel_token.cancel();
    let result = timeout(Duration::from_millis(250), fetch)
        .await
        .expect("cancellation should wake a stalled request")
        .expect("fetch task should not panic");

    assert!(
        result
            .expect_err("cancelled fetch should stop")
            .to_string()
            .contains("暂停")
    );
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

fn sorted_range_headers(ranges: Vec<Option<String>>) -> Vec<String> {
    let mut ranges = ranges.into_iter().flatten().collect::<Vec<_>>();
    ranges.sort_by_key(|range| {
        range
            .strip_prefix("bytes=")
            .and_then(|value| value.split_once('-'))
            .and_then(|(start, _)| start.parse::<usize>().ok())
            .unwrap_or(0)
    });
    ranges
}

fn segment_path(temp_path: &Path, index: usize) -> PathBuf {
    PathBuf::from(format!("{}.seg{index}", temp_path.display()))
}

async fn temp_case_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("bdl-{name}-{}", Uuid::new_v4()));
    tokio::fs::create_dir_all(&dir).await.unwrap();
    dir
}

async fn spawn_stalled_server() -> (String, oneshot::Receiver<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (started_tx, started_rx) = oneshot::channel();

    tokio::spawn(async move {
        let mut started_tx = Some(started_tx);
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                return;
            };
            let mut buffer = vec![0_u8; 4096];
            let Ok(_) = stream.read(&mut buffer).await else {
                return;
            };
            if let Some(sender) = started_tx.take() {
                let _ = sender.send(());
            }
            std::future::pending::<()>().await;
        }
    });

    (format!("http://{addr}/file"), started_rx)
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
    etag: Option<String>,
    last_modified: Option<String>,
    content_encoding: Option<String>,
}

impl TestServer {
    async fn spawn(data: Vec<u8>, fail_gets: usize) -> Self {
        Self::spawn_with_options(data, fail_gets, None, None, None).await
    }

    async fn spawn_encoded(data: Vec<u8>, content_encoding: &str) -> Self {
        Self::spawn_with_options(data, 0, None, None, Some(content_encoding)).await
    }

    async fn spawn_with_validators(
        data: Vec<u8>,
        fail_gets: usize,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Self {
        Self::spawn_with_options(data, fail_gets, etag, last_modified, None).await
    }

    async fn spawn_with_options(
        data: Vec<u8>,
        fail_gets: usize,
        etag: Option<&str>,
        last_modified: Option<&str>,
        content_encoding: Option<&str>,
    ) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let state = Arc::new(TestServerState {
            data,
            fail_gets: AtomicUsize::new(fail_gets),
            get_count: AtomicUsize::new(0),
            ranges: Mutex::new(Vec::new()),
            headers: Mutex::new(Vec::new()),
            etag: etag.map(str::to_owned),
            last_modified: last_modified.map(str::to_owned),
            content_encoding: content_encoding.map(str::to_owned),
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
                &response_headers(&state, state.data.len()),
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
                write_response(&mut stream, 500, "FAIL", &response_headers(&state, 0), b"").await;
                return;
            }

            if let Some((start, end)) = headers
                .get("range")
                .and_then(|range| parse_byte_range(range, state.data.len()))
            {
                let body = &state.data[start..=end];
                write_response(
                    &mut stream,
                    206,
                    "PARTIAL",
                    &response_headers(&state, body.len()),
                    body,
                )
                .await;
            } else {
                write_response(
                    &mut stream,
                    200,
                    "OK",
                    &response_headers(&state, state.data.len()),
                    &state.data,
                )
                .await;
            }
        }
        _ => {
            write_response(
                &mut stream,
                405,
                "METHOD",
                &response_headers(&state, 0),
                b"",
            )
            .await;
        }
    }
}

fn parse_byte_range(value: &str, data_len: usize) -> Option<(usize, usize)> {
    if data_len == 0 {
        return None;
    }

    let value = value.strip_prefix("bytes=")?;
    let (start, end) = value.split_once('-')?;
    let start = start.parse::<usize>().ok()?;
    if start >= data_len {
        return None;
    }

    let end = if end.is_empty() {
        data_len - 1
    } else {
        end.parse::<usize>().ok()?.min(data_len - 1)
    };
    (end >= start).then_some((start, end))
}

fn response_headers(state: &TestServerState, content_length: usize) -> Vec<(&'static str, String)> {
    let mut headers = vec![("Content-Length", content_length.to_string())];
    if let Some(etag) = &state.etag {
        headers.push(("ETag", etag.clone()));
    }
    if let Some(last_modified) = &state.last_modified {
        headers.push(("Last-Modified", last_modified.clone()));
    }
    if let Some(content_encoding) = &state.content_encoding {
        headers.push(("Content-Encoding", content_encoding.clone()));
    }

    headers
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
    headers: &[(&str, String)],
    body: &[u8],
) {
    let mut response = format!("HTTP/1.1 {status} {reason}\r\nConnection: close\r\n");
    for (name, value) in headers {
        response.push_str(&format!("{name}: {value}\r\n"));
    }
    response.push_str("\r\n");
    stream.write_all(response.as_bytes()).await.unwrap();
    stream.write_all(body).await.unwrap();
    stream.flush().await.unwrap();
    stream.shutdown().await.unwrap();
}
