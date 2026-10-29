use std::ffi::OsString;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};

use async_trait::async_trait;
use flate2::read::{DeflateDecoder, ZlibDecoder};
use futures::{StreamExt, future::try_join_all};
use reqwest::header::{
    CONTENT_ENCODING, CONTENT_LENGTH, CONTENT_RANGE, ETAG, HeaderMap, HeaderName, HeaderValue,
    LAST_MODIFIED, RANGE,
};
use reqwest::{Client, Proxy, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::fs::{self, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{Mutex, Notify};
use tokio::time::Instant;

use crate::error::{BdlError, BdlResult};
use crate::queue::DownloadResource;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchConfig {
    pub max_retries: usize,
    pub proxy_url: Option<String>,
    pub segment_count: usize,
    pub speed_limit_bytes_per_second: Option<u64>,
}

impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            proxy_url: None,
            segment_count: 1,
            speed_limit_bytes_per_second: None,
        }
    }
}

#[derive(Debug)]
pub struct BandwidthLimiter {
    bytes_per_second: AtomicU64,
    state: Mutex<BandwidthLimiterState>,
}

#[derive(Debug)]
struct BandwidthLimiterState {
    configured_bytes_per_second: u64,
    available_bytes: f64,
    last_refill: Instant,
}

impl BandwidthLimiter {
    pub fn new(bytes_per_second: Option<u64>) -> Self {
        let bytes_per_second = bytes_per_second.unwrap_or_default();
        Self {
            bytes_per_second: AtomicU64::new(bytes_per_second),
            state: Mutex::new(BandwidthLimiterState {
                configured_bytes_per_second: bytes_per_second,
                available_bytes: bytes_per_second as f64,
                last_refill: Instant::now(),
            }),
        }
    }

    pub fn set_limit(&self, bytes_per_second: Option<u64>) {
        self.bytes_per_second
            .store(bytes_per_second.unwrap_or_default(), Ordering::Relaxed);
    }

    pub async fn acquire(&self, bytes: u64) {
        let mut remaining = bytes as f64;
        while remaining > 0.0 {
            let bytes_per_second = self.bytes_per_second.load(Ordering::Relaxed);
            if bytes_per_second == 0 {
                return;
            }

            let mut state = self.state.lock().await;
            let bytes_per_second = self.bytes_per_second.load(Ordering::Relaxed);
            if bytes_per_second == 0 {
                return;
            }

            let now = Instant::now();
            let capacity = bytes_per_second as f64;
            if state.configured_bytes_per_second != bytes_per_second {
                state.configured_bytes_per_second = bytes_per_second;
                state.available_bytes = capacity;
                state.last_refill = now;
            }

            let replenished = now.duration_since(state.last_refill).as_secs_f64() * capacity;
            state.available_bytes = (state.available_bytes + replenished).min(capacity);
            state.last_refill = now;

            let available = state.available_bytes.min(remaining);
            state.available_bytes -= available;
            remaining -= available;
            if remaining <= 0.0 {
                return;
            }

            let wait_seconds = (remaining / capacity).min(0.1);
            let wait = std::time::Duration::from_secs_f64(wait_seconds);
            tokio::time::sleep(wait).await;
            remaining -= wait_seconds * capacity;
            state.last_refill = Instant::now();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FetchState {
    pub total_bytes: Option<u64>,
    pub downloaded_bytes: u64,
    #[serde(default)]
    pub etag: Option<String>,
    #[serde(default)]
    pub last_modified: Option<String>,
    #[serde(default)]
    pub segment_ranges: Option<Vec<(u64, u64)>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchProgress {
    pub resource_id: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchOutcome {
    pub bytes_written: u64,
    pub target_path: PathBuf,
}

pub type ProgressSender = UnboundedSender<FetchProgress>;

#[derive(Debug, Clone)]
pub struct FetchCancelToken {
    cancelled: Arc<AtomicBool>,
    notify: Arc<Notify>,
}

impl FetchCancelToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        self.notify.notify_waiters();
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub async fn cancelled(&self) {
        if self.is_cancelled() {
            return;
        }

        let notified = self.notify.notified();
        if self.is_cancelled() {
            return;
        }
        notified.await;
    }
}

impl Default for FetchCancelToken {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RemoteResourceMetadata {
    total_bytes: Option<u64>,
    etag: Option<String>,
    last_modified: Option<String>,
    content_encoding: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SegmentRequest {
    index: usize,
    start: u64,
    end: u64,
}

struct SegmentFetchRequest<'a> {
    resource: &'a DownloadResource,
    url: &'a str,
    headers: HeaderMap,
    metadata: &'a RemoteResourceMetadata,
    progress: Option<ProgressSender>,
    progress_by_segment: Arc<Mutex<Vec<u64>>>,
    segment: SegmentRequest,
    cancel_token: FetchCancelToken,
}

#[async_trait]
pub trait Fetcher {
    async fn fetch(
        &self,
        resource: &DownloadResource,
        progress: Option<ProgressSender>,
    ) -> BdlResult<FetchOutcome>;
}

#[derive(Debug, Clone)]
pub struct ReqwestFetcher {
    client: Client,
    config: FetchConfig,
    global_limiter: Option<Arc<BandwidthLimiter>>,
    task_limiter: Option<Arc<BandwidthLimiter>>,
    stop_on_restriction: bool,
}

impl ReqwestFetcher {
    /// CLI mode: stop before retries/CDN fallback when the server restricts access.
    pub fn with_stop_on_restriction(mut self) -> Self {
        self.stop_on_restriction = true;
        self
    }
    pub fn new() -> BdlResult<Self> {
        Self::with_config(FetchConfig::default())
    }

    pub fn with_config(config: FetchConfig) -> BdlResult<Self> {
        Self::with_optional_global_limiter(config, None)
    }

    pub fn with_global_limiter(
        config: FetchConfig,
        global_limiter: Arc<BandwidthLimiter>,
    ) -> BdlResult<Self> {
        Self::with_optional_global_limiter(config, Some(global_limiter))
    }

    fn with_optional_global_limiter(
        config: FetchConfig,
        global_limiter: Option<Arc<BandwidthLimiter>>,
    ) -> BdlResult<Self> {
        let mut client = Client::builder();
        if let Some(proxy_url) = config
            .proxy_url
            .as_deref()
            .filter(|value| !value.is_empty())
        {
            let proxy = Proxy::all(proxy_url)
                .map_err(|error| fetch_error(format!("代理设置无效 `{proxy_url}`: {error}")))?;
            client = client.proxy(proxy);
        }

        let client = client
            .no_deflate()
            .build()
            .map_err(|error| fetch_error(format!("创建下载客户端失败: {error}")))?;

        let task_limiter = config
            .speed_limit_bytes_per_second
            .map(|limit| BandwidthLimiter::new(Some(limit)))
            .map(Arc::new);

        Ok(Self {
            client,
            config,
            global_limiter,
            task_limiter,
            stop_on_restriction: false,
        })
    }
}

#[async_trait]
impl Fetcher for ReqwestFetcher {
    async fn fetch(
        &self,
        resource: &DownloadResource,
        progress: Option<ProgressSender>,
    ) -> BdlResult<FetchOutcome> {
        self.fetch_cancelable(resource, progress, FetchCancelToken::default())
            .await
    }
}

impl ReqwestFetcher {
    pub async fn fetch_cancelable(
        &self,
        resource: &DownloadResource,
        progress: Option<ProgressSender>,
        cancel_token: FetchCancelToken,
    ) -> BdlResult<FetchOutcome> {
        let urls = resource
            .current_urls
            .iter()
            .map(String::as_str)
            .filter(|url| !url.trim().is_empty())
            .collect::<Vec<_>>();
        if urls.is_empty() {
            return Err(fetch_error("资源没有可用下载地址，请重新解析后再试。"));
        }

        let mut last_error = None;
        let mut source_errors = vec![None; urls.len()];
        let attempts = self.config.max_retries + 1;
        let mut attempted = 0;
        for _ in 0..attempts {
            ensure_not_cancelled(&cancel_token)?;
            let mut retryable_error_seen = false;
            for (source_index, url) in urls.iter().enumerate() {
                ensure_not_cancelled(&cancel_token)?;
                attempted += 1;
                match self
                    .fetch_once(resource, url, progress.clone(), cancel_token.clone())
                    .await
                {
                    Ok(outcome) => return Ok(outcome),
                    Err(error) => {
                        if self.stop_on_restriction
                            && crate::resolver::pacing::is_source_restriction(&error.to_string())
                        {
                            return Err(error);
                        }
                        let message = crate::account::redact_sensitive(&error.to_string());
                        let host = url::Url::parse(url)
                            .ok()
                            .and_then(|url| url.host_str().map(str::to_owned))
                            .unwrap_or_else(|| "unknown".to_owned());
                        source_errors[source_index] = Some(format!(
                            "source={} host={host}: {message}",
                            source_index + 1
                        ));
                        retryable_error_seen |= !message.contains("HTTP 404");
                        last_error = Some(message);
                    }
                }
            }
            if !retryable_error_seen {
                break;
            }
        }

        let failure_code = last_error
            .as_deref()
            .and_then(crate::diagnostics::classify_failure)
            .map_or("download_failed", |rule| rule.code.as_str());
        Err(fetch_error(format!(
            "[stage=fetch_resource code={failure_code}] 下载 `{}` 失败，已尝试 {attempted} attempts across {} URLs: {}\n失败时配置: segment_count={} max_retries={} proxy_enabled={}\n各下载源最后错误：\n{}",
            resource.id,
            urls.len(),
            last_error.unwrap_or_else(|| "unknown error".to_owned()),
            self.config.segment_count,
            self.config.max_retries,
            self.config
                .proxy_url
                .as_ref()
                .is_some_and(|url| !url.trim().is_empty()),
            source_errors
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()
                .join("\n")
        )))
    }
}

impl ReqwestFetcher {
    async fn fetch_once(
        &self,
        resource: &DownloadResource,
        url: &str,
        progress: Option<ProgressSender>,
        cancel_token: FetchCancelToken,
    ) -> BdlResult<FetchOutcome> {
        ensure_not_cancelled(&cancel_token)?;
        ensure_parent_dir(&resource.target_path).await?;
        ensure_parent_dir(&resource.temp_path).await?;

        let headers = request_headers(resource)?;
        let metadata = self
            .resource_metadata(url, headers.clone(), &cancel_token)
            .await?;
        ensure_not_cancelled(&cancel_token)?;
        let resume_from = if metadata.content_encoding.as_deref() == Some("deflate") {
            remove_if_exists(&resource.temp_path).await?;
            remove_if_exists(&state_path_for(&resource.temp_path)).await?;
            0
        } else {
            resume_offset(&resource.temp_path, &metadata).await?
        };
        if should_fetch_segmented(&metadata, resume_from, self.config.segment_count) {
            return self
                .fetch_segmented(resource, url, headers, metadata, progress, cancel_token)
                .await;
        }

        ensure_not_cancelled(&cancel_token)?;
        let mut request = self.client.get(url).headers(headers);
        if resume_from > 0 {
            request = request.header(RANGE, format!("bytes={resume_from}-"));
        }

        let response = tokio::select! {
            _ = cancel_token.cancelled() => return Err(cancelled_error()),
            response = request.send() => response
                .map_err(|error| request_send_error("请求下载地址失败", &format!("method=GET range_start={resume_from}"), error))?,
        };

        validate_get_status(response.status(), resume_from)?;
        let response_info = response_diagnostic_context(&response);

        if response_uses_deflate(&response, &metadata) {
            return self
                .fetch_deflate_response(resource, response, metadata, progress, cancel_token)
                .await;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(resume_from > 0)
            .truncate(resume_from == 0)
            .open(&resource.temp_path)
            .await?;

        let mut downloaded_bytes = resume_from;
        let mut stream = response.bytes_stream();
        loop {
            let chunk = tokio::select! {
                _ = cancel_token.cancelled() => return Err(cancelled_error()),
                chunk = stream.next() => chunk,
            };
            let Some(chunk) = chunk else { break };
            let chunk = chunk.map_err(|error| {
                response_body_error(
                    "读取响应失败",
                    &format!("received={downloaded_bytes} {response_info}"),
                    error,
                )
            })?;
            self.wait_for_bandwidth(chunk.len() as u64, &cancel_token)
                .await?;
            file.write_all(&chunk).await?;
            downloaded_bytes += chunk.len() as u64;

            persist_fetch_progress(&resource.temp_path, &metadata, downloaded_bytes).await?;
            send_progress(&progress, resource, metadata.total_bytes, downloaded_bytes);
        }

        if downloaded_bytes == resume_from {
            persist_fetch_progress(&resource.temp_path, &metadata, downloaded_bytes).await?;
            send_progress(&progress, resource, metadata.total_bytes, downloaded_bytes);
        }
        file.flush().await?;
        drop(file);

        if let Some(total_bytes) = metadata.total_bytes
            && downloaded_bytes != total_bytes
        {
            return Err(fetch_error(format!(
                "下载长度不完整: expected {total_bytes}, got {downloaded_bytes}"
            )));
        }

        if resource.target_path.exists() {
            fs::remove_file(&resource.target_path).await?;
        }
        fs::rename(&resource.temp_path, &resource.target_path).await?;
        remove_if_exists(&state_path_for(&resource.temp_path)).await?;

        Ok(FetchOutcome {
            bytes_written: downloaded_bytes,
            target_path: resource.target_path.clone(),
        })
    }

    async fn fetch_deflate_response(
        &self,
        resource: &DownloadResource,
        response: reqwest::Response,
        metadata: RemoteResourceMetadata,
        progress: Option<ProgressSender>,
        cancel_token: FetchCancelToken,
    ) -> BdlResult<FetchOutcome> {
        let mut encoded = Vec::new();
        let mut downloaded_bytes = 0_u64;
        let response_info = response_diagnostic_context(&response);
        let mut stream = response.bytes_stream();
        loop {
            let chunk = tokio::select! {
                _ = cancel_token.cancelled() => return Err(cancelled_error()),
                chunk = stream.next() => chunk,
            };
            let Some(chunk) = chunk else { break };
            let chunk = chunk.map_err(|error| {
                response_body_error(
                    "读取压缩响应失败",
                    &format!("received={downloaded_bytes} {response_info}"),
                    error,
                )
            })?;
            self.wait_for_bandwidth(chunk.len() as u64, &cancel_token)
                .await?;
            encoded.extend_from_slice(&chunk);
            downloaded_bytes += chunk.len() as u64;
            send_progress(&progress, resource, metadata.total_bytes, downloaded_bytes);
        }

        if let Some(total_bytes) = metadata.total_bytes
            && downloaded_bytes != total_bytes
        {
            return Err(fetch_error(format!(
                "下载长度不完整: expected {total_bytes}, got {downloaded_bytes}"
            )));
        }

        let decoded = decode_deflate_body(&encoded)?;
        fs::write(&resource.temp_path, &decoded).await?;
        if resource.target_path.exists() {
            fs::remove_file(&resource.target_path).await?;
        }
        fs::rename(&resource.temp_path, &resource.target_path).await?;
        remove_if_exists(&state_path_for(&resource.temp_path)).await?;

        Ok(FetchOutcome {
            bytes_written: decoded.len() as u64,
            target_path: resource.target_path.clone(),
        })
    }

    async fn fetch_segmented(
        &self,
        resource: &DownloadResource,
        url: &str,
        headers: HeaderMap,
        metadata: RemoteResourceMetadata,
        progress: Option<ProgressSender>,
        cancel_token: FetchCancelToken,
    ) -> BdlResult<FetchOutcome> {
        ensure_not_cancelled(&cancel_token)?;
        let total_bytes = metadata
            .total_bytes
            .expect("segmented fetch requires content length");
        let ranges = segment_ranges(total_bytes, self.config.segment_count);
        let can_resume_segments =
            segmented_resume_is_valid(&resource.temp_path, &metadata, &ranges).await?;
        if !can_resume_segments {
            for index in 0..8 {
                remove_if_exists(&segment_path_for(&resource.temp_path, index)).await?;
            }
        }
        let mut resumed_by_segment = Vec::with_capacity(ranges.len());
        for (index, (start, end)) in ranges.iter().enumerate() {
            let expected_len = end - start + 1;
            let segment_path = segment_path_for(&resource.temp_path, index);
            let existing_len = fs::metadata(&segment_path)
                .await
                .map(|metadata| metadata.len())
                .unwrap_or(0);
            if existing_len > expected_len {
                remove_if_exists(&segment_path).await?;
                resumed_by_segment.push(0);
            } else {
                resumed_by_segment.push(existing_len);
            }
        }
        let progress_by_segment = Arc::new(Mutex::new(resumed_by_segment));

        remove_if_exists(&resource.temp_path).await?;
        write_fetch_state(
            &resource.temp_path,
            &FetchState {
                total_bytes: metadata.total_bytes,
                downloaded_bytes: 0,
                etag: metadata.etag.clone(),
                last_modified: metadata.last_modified.clone(),
                segment_ranges: Some(ranges.clone()),
            },
        )
        .await?;

        try_join_all(ranges.iter().enumerate().map(|(index, (start, end))| {
            self.fetch_segment(SegmentFetchRequest {
                resource,
                url,
                headers: headers.clone(),
                metadata: &metadata,
                progress: progress.clone(),
                progress_by_segment: progress_by_segment.clone(),
                segment: SegmentRequest {
                    index,
                    start: *start,
                    end: *end,
                },
                cancel_token: cancel_token.clone(),
            })
        }))
        .await?;

        ensure_not_cancelled(&cancel_token)?;
        let mut temp_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&resource.temp_path)
            .await?;
        for (index, (start, end)) in ranges.iter().enumerate() {
            let segment_path = segment_path_for(&resource.temp_path, index);
            let mut segment_file = fs::File::open(&segment_path).await?;
            let copied = tokio::io::copy(&mut segment_file, &mut temp_file).await?;
            if copied != end - start + 1 {
                return Err(fetch_error("分段文件长度异常，已停止拼接，请重试下载。"));
            }
        }
        temp_file.flush().await?;
        drop(temp_file);

        if fs::metadata(&resource.temp_path).await?.len() != total_bytes {
            return Err(fetch_error("拼接后的文件长度异常，请重试下载。"));
        }

        if resource.target_path.exists() {
            fs::remove_file(&resource.target_path).await?;
        }
        fs::rename(&resource.temp_path, &resource.target_path).await?;
        for index in 0..ranges.len() {
            remove_if_exists(&segment_path_for(&resource.temp_path, index)).await?;
        }
        remove_if_exists(&state_path_for(&resource.temp_path)).await?;

        Ok(FetchOutcome {
            bytes_written: total_bytes,
            target_path: resource.target_path.clone(),
        })
    }

    async fn fetch_segment(&self, request: SegmentFetchRequest<'_>) -> BdlResult<()> {
        let SegmentFetchRequest {
            resource,
            url,
            headers,
            metadata,
            progress,
            progress_by_segment,
            segment,
            cancel_token,
        } = request;
        ensure_not_cancelled(&cancel_token)?;
        let segment_path = segment_path_for(&resource.temp_path, segment.index);
        let expected_len = segment.end - segment.start + 1;
        let resume_from = fs::metadata(&segment_path)
            .await
            .map(|metadata| metadata.len())
            .unwrap_or(0);
        if resume_from == expected_len {
            return Ok(());
        }
        let request_start = segment.start + resume_from;
        let request = self
            .client
            .get(url)
            .headers(headers)
            .header(RANGE, format!("bytes={request_start}-{}", segment.end));
        let response = tokio::select! {
            _ = cancel_token.cancelled() => return Err(cancelled_error()),
            response = request.send() => response
                .map_err(|error| request_send_error("请求分段下载地址失败", &format!("method=GET segment={} range=bytes={request_start}-{}", segment.index, segment.end), error))?,
        };

        if response.status() != StatusCode::PARTIAL_CONTENT {
            return Err(fetch_error(format!(
                "分段下载请求失败: HTTP {}",
                response.status()
            )));
        }
        let response_info = response_diagnostic_context(&response);

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(resume_from > 0)
            .truncate(resume_from == 0)
            .open(&segment_path)
            .await?;
        let mut downloaded_bytes = resume_from;
        let mut stream = response.bytes_stream();
        loop {
            let chunk = tokio::select! {
                _ = cancel_token.cancelled() => return Err(cancelled_error()),
                chunk = stream.next() => chunk,
            };
            let Some(chunk) = chunk else { break };
            let chunk = chunk.map_err(|error| response_body_error("读取分段响应失败", &format!(
                "segment={} range={request_start}-{} received={downloaded_bytes} {response_info}", segment.index, segment.end
            ), error))?;
            self.wait_for_bandwidth(chunk.len() as u64, &cancel_token)
                .await?;
            file.write_all(&chunk).await?;
            downloaded_bytes += chunk.len() as u64;
            let total_downloaded = {
                let mut progress_guard = progress_by_segment.lock().await;
                progress_guard[segment.index] = downloaded_bytes;
                progress_guard.iter().sum()
            };

            send_progress(&progress, resource, metadata.total_bytes, total_downloaded);
        }
        file.flush().await?;

        if downloaded_bytes != expected_len {
            return Err(fetch_error(format!(
                "分段下载长度不完整: expected {expected_len}, got {downloaded_bytes}"
            )));
        }

        Ok(())
    }

    async fn wait_for_bandwidth(
        &self,
        bytes: u64,
        cancel_token: &FetchCancelToken,
    ) -> BdlResult<()> {
        let acquire = async {
            match (&self.global_limiter, &self.task_limiter) {
                (Some(global), Some(task)) => {
                    tokio::join!(global.acquire(bytes), task.acquire(bytes));
                }
                (Some(limiter), None) | (None, Some(limiter)) => limiter.acquire(bytes).await,
                (None, None) => {}
            }
        };

        tokio::select! {
            _ = cancel_token.cancelled() => Err(cancelled_error()),
            () = acquire => Ok(()),
        }
    }

    async fn resource_metadata(
        &self,
        url: &str,
        headers: HeaderMap,
        cancel_token: &FetchCancelToken,
    ) -> BdlResult<RemoteResourceMetadata> {
        let request = self.client.head(url).headers(headers.clone());
        let response = tokio::select! {
            _ = cancel_token.cancelled() => return Err(cancelled_error()),
            response = request.send() => response
                .map_err(|error| request_send_error("请求资源长度失败", "method=HEAD", error))?,
        };

        if response.status().is_success() {
            return remote_resource_metadata(response.headers(), false);
        }
        if self.stop_on_restriction
            && crate::resolver::pacing::is_source_restriction(&response.status().to_string())
        {
            return Err(fetch_error(format!(
                "请求资源长度失败: HTTP {}",
                response.status()
            )));
        }

        let request = self
            .client
            .get(url)
            .headers(headers)
            .header(RANGE, "bytes=0-0");
        let response = tokio::select! {
            _ = cancel_token.cancelled() => return Err(cancelled_error()),
            response = request.send() => response
                .map_err(|error| request_send_error("请求资源长度失败", "method=GET range=bytes=0-0", error))?,
        };

        if !response.status().is_success() {
            return Err(fetch_error(format!(
                "请求资源长度失败: HTTP {}",
                response.status()
            )));
        }

        remote_resource_metadata(
            response.headers(),
            response.status() == StatusCode::PARTIAL_CONTENT,
        )
    }
}

fn response_diagnostic_context(response: &reqwest::Response) -> String {
    let encoding = response
        .headers()
        .get(CONTENT_ENCODING)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("identity_or_auto_decoded");
    format!(
        "http_status={} protocol={:?} content_encoding={encoding}",
        response.status().as_u16(),
        response.version()
    )
}

fn request_send_error(phase: &str, context: &str, error: reqwest::Error) -> BdlError {
    let error = error.without_url();
    let code = if error.is_timeout() {
        "network_timeout"
    } else {
        "download_failed"
    };
    fetch_error(crate::account::redact_sensitive(&format!(
        "[stage=request_send code={code}] {phase}: {error}; {context}; is_connect={} is_timeout={} is_request={}; cause={}",
        error.is_connect(),
        error.is_timeout(),
        error.is_request(),
        request_error_causes(&error)
    )))
}

fn request_error_causes(error: &reqwest::Error) -> String {
    let mut causes = Vec::new();
    let mut current = std::error::Error::source(error);
    while let Some(cause) = current {
        causes.push(cause.to_string());
        if causes.len() >= 8 {
            break;
        }
        current = cause.source();
    }
    causes.join(" <- ")
}

fn response_body_error(phase: &str, context: &str, error: reqwest::Error) -> BdlError {
    let error = error.without_url();
    let code = if error.is_timeout() {
        "network_timeout"
    } else {
        "response_body_failed"
    };
    fetch_error(crate::account::redact_sensitive(&format!(
        "[stage=response_body code={code}] {phase}: {error}; {context}; is_decode={} is_body={} is_timeout={}; cause={}",
        error.is_decode(),
        error.is_body(),
        error.is_timeout(),
        request_error_causes(&error)
    )))
}

fn remote_resource_metadata(
    headers: &HeaderMap,
    range_probe: bool,
) -> BdlResult<RemoteResourceMetadata> {
    let total_bytes = if range_probe {
        headers
            .get(CONTENT_RANGE)
            .map(|value| {
                value
                    .to_str()
                    .map_err(|error| fetch_error(format!("无效 Content-Range: {error}")))
                    .and_then(content_range_total)
            })
            .transpose()?
    } else {
        headers
            .get(CONTENT_LENGTH)
            .map(|value| {
                value
                    .to_str()
                    .map_err(|error| fetch_error(format!("无效 Content-Length: {error}")))?
                    .parse::<u64>()
                    .map_err(|error| fetch_error(format!("无效 Content-Length: {error}")))
            })
            .transpose()?
    };

    Ok(RemoteResourceMetadata {
        total_bytes,
        etag: header_to_string(headers, ETAG)?,
        last_modified: header_to_string(headers, LAST_MODIFIED)?,
        content_encoding: header_to_string(headers, CONTENT_ENCODING)?
            .map(|value| value.to_ascii_lowercase()),
    })
}

fn content_range_total(value: &str) -> BdlResult<u64> {
    value
        .rsplit_once('/')
        .map(|(_, total)| total)
        .filter(|total| *total != "*")
        .ok_or_else(|| fetch_error(format!("无效 Content-Range: {value}")))?
        .parse::<u64>()
        .map_err(|error| fetch_error(format!("无效 Content-Range: {error}")))
}

pub async fn write_fetch_state(temp_path: &Path, state: &FetchState) -> BdlResult<()> {
    let path = state_path_for(temp_path);
    ensure_parent_dir(&path).await?;
    let bytes = serde_json::to_vec(state)
        .map_err(|error| fetch_error(format!("序列化下载状态失败: {error}")))?;
    let temporary = state_temp_path_for(temp_path);
    fs::write(&temporary, bytes).await?;
    fs::rename(temporary, path).await?;
    Ok(())
}

async fn persist_fetch_progress(
    temp_path: &Path,
    metadata: &RemoteResourceMetadata,
    downloaded_bytes: u64,
) -> BdlResult<()> {
    write_fetch_state(
        temp_path,
        &FetchState {
            total_bytes: metadata.total_bytes,
            downloaded_bytes,
            etag: metadata.etag.clone(),
            last_modified: metadata.last_modified.clone(),
            segment_ranges: None,
        },
    )
    .await
}

fn send_progress(
    progress: &Option<ProgressSender>,
    resource: &DownloadResource,
    total_bytes: Option<u64>,
    downloaded_bytes: u64,
) {
    if let Some(sender) = progress {
        let _ = sender.send(FetchProgress {
            resource_id: resource.id.clone(),
            downloaded_bytes,
            total_bytes,
        });
    }
}

fn ensure_not_cancelled(cancel_token: &FetchCancelToken) -> BdlResult<()> {
    if cancel_token.is_cancelled() {
        return Err(cancelled_error());
    }

    Ok(())
}

fn cancelled_error() -> BdlError {
    fetch_error("下载已暂停或取消。")
}

pub fn state_path_for(temp_path: &Path) -> PathBuf {
    auxiliary_path_for(temp_path, ".state")
}

pub fn state_temp_path_for(temp_path: &Path) -> PathBuf {
    auxiliary_path_for(temp_path, ".state.tmp")
}

// Keep existing short-path checkpoints compatible. Long title paths leave too
// little room for suffixes on some Android runtimes; use an internal stable ID.
fn auxiliary_path_for(temp_path: &Path, suffix: &str) -> PathBuf {
    let mut value = OsString::from(temp_path.as_os_str());
    value.push(suffix);
    if value.as_encoded_bytes().len() <= 240 {
        return PathBuf::from(value);
    }
    let id = uuid::Uuid::new_v5(
        &uuid::Uuid::NAMESPACE_OID,
        temp_path.as_os_str().as_encoded_bytes(),
    );
    temp_path.with_file_name(format!(".bdl-{id}{suffix}"))
}

/// Completed media may come from an older corrupted segmented download. Only
/// those inputs need fetching again; valid completed tracks remain reusable.
pub async fn completed_media_needs_redownload(resource: &DownloadResource) -> BdlResult<bool> {
    if resource.status != crate::queue::ResourceStatus::Completed
        || !matches!(
            resource.intent,
            crate::queue::DownloadResourceIntent::Video
                | crate::queue::DownloadResourceIntent::Audio
        )
    {
        return Ok(false);
    }
    let Ok(mut file) = fs::File::open(&resource.target_path).await else {
        return Ok(true);
    };
    let mut header = [0; 16];
    let count = file.read(&mut header).await?;
    let bytes = &header[..count];
    let iso_bmff = bytes.len() >= 8
        && matches!(
            &bytes[4..8],
            b"ftyp" | b"styp" | b"moov" | b"moof" | b"sidx" | b"free" | b"skip" | b"mdat"
        );
    let other_container = [
        b"FLV".as_slice(),
        b"ID3",
        b"OggS",
        b"fLaC",
        b"RIFF",
        &[0x1a, 0x45, 0xdf, 0xa3],
        &[0, 0, 1],
        &[0, 0, 0, 1],
    ]
    .iter()
    .any(|prefix| bytes.starts_with(prefix));
    let audio_frame = bytes.len() >= 2 && bytes[0] == 0xff && bytes[1] & 0xe0 == 0xe0;
    Ok(!(iso_bmff || other_container || audio_frame))
}

fn should_fetch_segmented(
    metadata: &RemoteResourceMetadata,
    resume_from: u64,
    segment_count: usize,
) -> bool {
    metadata.content_encoding.is_none()
        && resume_from == 0
        && segment_count > 1
        && metadata
            .total_bytes
            .is_some_and(|total_bytes| total_bytes > 1)
}

fn response_uses_deflate(response: &reqwest::Response, metadata: &RemoteResourceMetadata) -> bool {
    metadata.content_encoding.as_deref() == Some("deflate")
        || response
            .headers()
            .get(CONTENT_ENCODING)
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.eq_ignore_ascii_case("deflate"))
}

fn decode_deflate_body(encoded: &[u8]) -> BdlResult<Vec<u8>> {
    let mut zlib_decoded = Vec::new();
    if ZlibDecoder::new(encoded)
        .read_to_end(&mut zlib_decoded)
        .is_ok()
    {
        return Ok(zlib_decoded);
    }

    let mut raw_decoded = Vec::new();
    DeflateDecoder::new(encoded)
        .read_to_end(&mut raw_decoded)
        .map_err(|error| fetch_error(format!("解压 deflate 响应失败: {error}")))?;
    Ok(raw_decoded)
}

async fn segmented_resume_is_valid(
    temp_path: &Path,
    metadata: &RemoteResourceMetadata,
    ranges: &[(u64, u64)],
) -> BdlResult<bool> {
    let state_path = state_path_for(temp_path);
    if !state_path.exists() {
        return Ok(false);
    }

    let state = read_fetch_state(&state_path).await?;
    Ok(state.segment_ranges.as_deref() == Some(ranges)
        && state.total_bytes == metadata.total_bytes
        && validator_matches(&state.etag, &metadata.etag)
        && validator_matches(&state.last_modified, &metadata.last_modified))
}

fn segment_ranges(total_bytes: u64, segment_count: usize) -> Vec<(u64, u64)> {
    if total_bytes == 0 {
        return Vec::new();
    }

    let max_segments = usize::try_from(total_bytes).unwrap_or(usize::MAX);
    let count = segment_count.clamp(1, 8).min(max_segments);
    let count_u64 = count as u64;
    let base_size = total_bytes / count_u64;
    let remainder = total_bytes % count_u64;
    let mut ranges = Vec::with_capacity(count);
    let mut start = 0_u64;

    for index in 0..count {
        let size = base_size + if (index as u64) < remainder { 1 } else { 0 };
        let end = start + size - 1;
        ranges.push((start, end));
        start = end + 1;
    }

    ranges
}

pub fn segment_path_for(temp_path: &Path, index: usize) -> PathBuf {
    auxiliary_path_for(temp_path, &format!(".seg{index}"))
}

async fn resume_offset(temp_path: &Path, metadata: &RemoteResourceMetadata) -> BdlResult<u64> {
    let state_path = state_path_for(temp_path);
    if !temp_path.exists() || !state_path.exists() {
        return Ok(0);
    }

    let temp_len = fs::metadata(temp_path).await?.len();
    let state = read_fetch_state(&state_path).await?;
    if state.total_bytes == metadata.total_bytes
        && state.downloaded_bytes == temp_len
        && temp_len > 0
        && validator_matches(&state.etag, &metadata.etag)
        && validator_matches(&state.last_modified, &metadata.last_modified)
    {
        return Ok(temp_len);
    }

    remove_if_exists(temp_path).await?;
    remove_if_exists(&state_path).await?;
    Ok(0)
}

fn validator_matches(previous: &Option<String>, current: &Option<String>) -> bool {
    match (previous, current) {
        (Some(previous), Some(current)) => previous == current,
        _ => true,
    }
}

async fn read_fetch_state(path: &Path) -> BdlResult<FetchState> {
    let bytes = fs::read(path).await?;
    serde_json::from_slice(&bytes)
        .map_err(|error| fetch_error(format!("读取下载状态失败，需要重新下载: {error}")))
}

fn request_headers(resource: &DownloadResource) -> BdlResult<HeaderMap> {
    let mut headers = HeaderMap::new();
    for header in &resource.headers {
        let name = HeaderName::from_bytes(header.name.as_bytes())
            .map_err(|error| fetch_error(format!("无效请求头 `{}`: {error}", header.name)))?;
        let value = HeaderValue::from_str(&header.value)
            .map_err(|error| fetch_error(format!("无效请求头 `{}`: {error}", header.name)))?;
        headers.insert(name, value);
    }
    Ok(headers)
}

fn header_to_string(headers: &HeaderMap, name: HeaderName) -> BdlResult<Option<String>> {
    headers
        .get(name)
        .map(|value| {
            value
                .to_str()
                .map(str::to_owned)
                .map_err(|error| fetch_error(format!("无效响应头: {error}")))
        })
        .transpose()
}

fn validate_get_status(status: StatusCode, resume_from: u64) -> BdlResult<()> {
    if resume_from > 0 && status != StatusCode::PARTIAL_CONTENT {
        return Err(fetch_error(format!(
            "服务器未按 Range 返回分段响应: HTTP {status}"
        )));
    }

    if resume_from == 0 && !status.is_success() {
        return Err(fetch_error(format!("下载请求失败: HTTP {status}")));
    }

    if resume_from > 0 && !status.is_success() {
        return Err(fetch_error(format!("续传请求失败: HTTP {status}")));
    }

    Ok(())
}

async fn ensure_parent_dir(path: &Path) -> BdlResult<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).await.map_err(|error| {
            fetch_error(format!(
                "无法创建下载目录 `{}`：{error}。请检查路径和写入权限，或更换保存目录后重试。",
                parent.display()
            ))
        })?;
    }
    Ok(())
}

async fn remove_if_exists(path: &Path) -> BdlResult<()> {
    if path.exists() {
        fs::remove_file(path).await?;
    }
    Ok(())
}

fn fetch_error(message: impl Into<String>) -> BdlError {
    BdlError::Fetch {
        message: message.into(),
    }
}

#[cfg(test)]
mod directory_tests {
    use super::*;

    #[tokio::test]
    async fn download_directory_is_created_when_fetching_needs_it() {
        let root = std::env::temp_dir().join(format!("bdl-dir-{}", uuid::Uuid::new_v4()));
        let target = root.join("new/nested/video.m4s");
        assert!(!root.exists());
        ensure_parent_dir(&target).await.unwrap();
        assert!(target.parent().unwrap().is_dir());
        fs::remove_dir_all(&root).await.unwrap();
    }

    #[tokio::test]
    async fn download_directory_creation_failure_names_path_and_recovery() {
        let root = std::env::temp_dir().join(format!("bdl-dir-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&root).await.unwrap();
        let blocker = root.join("file-instead-of-directory");
        fs::write(&blocker, b"preserve").await.unwrap();
        let error = ensure_parent_dir(&blocker.join("video.m4s"))
            .await
            .unwrap_err()
            .to_string();
        assert_eq!(fs::read(&blocker).await.unwrap(), b"preserve");
        fs::remove_dir_all(&root).await.unwrap();
        assert!(error.contains("无法创建下载目录"), "{error}");
        assert!(
            error.contains(&blocker.to_string_lossy().to_string()),
            "{error}"
        );
        assert!(error.contains("更换保存目录"), "{error}");
    }
}
