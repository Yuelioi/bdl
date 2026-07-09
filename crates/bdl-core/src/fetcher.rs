use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use futures::{StreamExt, future::try_join_all};
use reqwest::header::{
    CONTENT_LENGTH, ETAG, HeaderMap, HeaderName, HeaderValue, LAST_MODIFIED, RANGE,
};
use reqwest::{Client, Proxy, StatusCode};
use serde::{Deserialize, Serialize};
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::sync::mpsc::UnboundedSender;

use crate::error::{BdlError, BdlResult};
use crate::queue::DownloadResource;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchConfig {
    pub max_retries: usize,
    pub proxy_url: Option<String>,
    pub segment_count: usize,
}

impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            proxy_url: None,
            segment_count: 1,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct RemoteResourceMetadata {
    total_bytes: Option<u64>,
    etag: Option<String>,
    last_modified: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SegmentRequest {
    index: usize,
    start: u64,
    end: u64,
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
}

impl ReqwestFetcher {
    pub fn new() -> BdlResult<Self> {
        Self::with_config(FetchConfig::default())
    }

    pub fn with_config(config: FetchConfig) -> BdlResult<Self> {
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
            .build()
            .map_err(|error| fetch_error(format!("创建下载客户端失败: {error}")))?;

        Ok(Self { client, config })
    }
}

#[async_trait]
impl Fetcher for ReqwestFetcher {
    async fn fetch(
        &self,
        resource: &DownloadResource,
        progress: Option<ProgressSender>,
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
        let attempts = self.config.max_retries + 1;
        let mut attempted = 0;
        for _ in 0..attempts {
            for url in &urls {
                attempted += 1;
                match self.fetch_once(resource, url, progress.clone()).await {
                    Ok(outcome) => return Ok(outcome),
                    Err(error) => {
                        last_error = Some(error.to_string());
                    }
                }
            }
        }

        Err(fetch_error(format!(
            "下载 `{}` 失败，已尝试 {attempted} attempts across {} URLs: {}",
            resource.id,
            urls.len(),
            last_error.unwrap_or_else(|| "unknown error".to_owned())
        )))
    }
}

impl ReqwestFetcher {
    async fn fetch_once(
        &self,
        resource: &DownloadResource,
        url: &str,
        progress: Option<ProgressSender>,
    ) -> BdlResult<FetchOutcome> {
        ensure_parent_dir(&resource.target_path).await?;
        ensure_parent_dir(&resource.temp_path).await?;

        let headers = request_headers(resource)?;
        let metadata = self.resource_metadata(url, headers.clone()).await?;
        let resume_from = resume_offset(&resource.temp_path, &metadata).await?;
        if should_fetch_segmented(&metadata, resume_from, self.config.segment_count) {
            return self
                .fetch_segmented(resource, url, headers, metadata, progress)
                .await;
        }

        let mut request = self.client.get(url).headers(headers);
        if resume_from > 0 {
            request = request.header(RANGE, format!("bytes={resume_from}-"));
        }

        let response = request
            .send()
            .await
            .map_err(|error| fetch_error(format!("请求下载地址失败: {error}")))?;

        validate_get_status(response.status(), resume_from)?;

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(resume_from > 0)
            .truncate(resume_from == 0)
            .open(&resource.temp_path)
            .await?;

        let mut downloaded_bytes = resume_from;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|error| fetch_error(format!("读取响应失败: {error}")))?;
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

        if let Some(total_bytes) = metadata.total_bytes {
            if downloaded_bytes != total_bytes {
                return Err(fetch_error(format!(
                    "下载长度不完整: expected {total_bytes}, got {downloaded_bytes}"
                )));
            }
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

    async fn fetch_segmented(
        &self,
        resource: &DownloadResource,
        url: &str,
        headers: HeaderMap,
        metadata: RemoteResourceMetadata,
        progress: Option<ProgressSender>,
    ) -> BdlResult<FetchOutcome> {
        let total_bytes = metadata
            .total_bytes
            .expect("segmented fetch requires content length");
        let ranges = segment_ranges(total_bytes, self.config.segment_count);
        let progress_by_segment = Arc::new(Mutex::new(vec![0_u64; ranges.len()]));

        remove_if_exists(&resource.temp_path).await?;
        remove_if_exists(&state_path_for(&resource.temp_path)).await?;
        for index in 0..ranges.len() {
            remove_if_exists(&segment_path_for(&resource.temp_path, index)).await?;
        }

        try_join_all(ranges.iter().enumerate().map(|(index, (start, end))| {
            self.fetch_segment(
                resource,
                url,
                headers.clone(),
                &metadata,
                progress.clone(),
                progress_by_segment.clone(),
                SegmentRequest {
                    index,
                    start: *start,
                    end: *end,
                },
            )
        }))
        .await?;

        let mut temp_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&resource.temp_path)
            .await?;
        for index in 0..ranges.len() {
            let segment_path = segment_path_for(&resource.temp_path, index);
            let mut segment_file = fs::File::open(&segment_path).await?;
            tokio::io::copy(&mut segment_file, &mut temp_file).await?;
            remove_if_exists(&segment_path).await?;
        }
        temp_file.flush().await?;
        drop(temp_file);

        if resource.target_path.exists() {
            fs::remove_file(&resource.target_path).await?;
        }
        fs::rename(&resource.temp_path, &resource.target_path).await?;
        remove_if_exists(&state_path_for(&resource.temp_path)).await?;

        Ok(FetchOutcome {
            bytes_written: total_bytes,
            target_path: resource.target_path.clone(),
        })
    }

    async fn fetch_segment(
        &self,
        resource: &DownloadResource,
        url: &str,
        headers: HeaderMap,
        metadata: &RemoteResourceMetadata,
        progress: Option<ProgressSender>,
        progress_by_segment: Arc<Mutex<Vec<u64>>>,
        segment: SegmentRequest,
    ) -> BdlResult<()> {
        let segment_path = segment_path_for(&resource.temp_path, segment.index);
        let expected_len = segment.end - segment.start + 1;
        let response = self
            .client
            .get(url)
            .headers(headers)
            .header(RANGE, format!("bytes={}-{}", segment.start, segment.end))
            .send()
            .await
            .map_err(|error| fetch_error(format!("请求分段下载地址失败: {error}")))?;

        if response.status() != StatusCode::PARTIAL_CONTENT {
            return Err(fetch_error(format!(
                "分段下载请求失败: HTTP {}",
                response.status()
            )));
        }

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&segment_path)
            .await?;
        let mut downloaded_bytes = 0_u64;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|error| fetch_error(format!("读取分段响应失败: {error}")))?;
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

    async fn resource_metadata(
        &self,
        url: &str,
        headers: HeaderMap,
    ) -> BdlResult<RemoteResourceMetadata> {
        let response = self
            .client
            .head(url)
            .headers(headers)
            .send()
            .await
            .map_err(|error| fetch_error(format!("请求资源长度失败: {error}")))?;

        if !response.status().is_success() {
            return Err(fetch_error(format!(
                "请求资源长度失败: HTTP {}",
                response.status()
            )));
        }

        let headers = response.headers();
        let total_bytes = headers
            .get(CONTENT_LENGTH)
            .map(|value| {
                value
                    .to_str()
                    .map_err(|error| fetch_error(format!("无效 Content-Length: {error}")))?
                    .parse::<u64>()
                    .map_err(|error| fetch_error(format!("无效 Content-Length: {error}")))
            })
            .transpose()?;

        Ok(RemoteResourceMetadata {
            total_bytes,
            etag: header_to_string(headers, ETAG)?,
            last_modified: header_to_string(headers, LAST_MODIFIED)?,
        })
    }
}

pub async fn write_fetch_state(temp_path: &Path, state: &FetchState) -> BdlResult<()> {
    let path = state_path_for(temp_path);
    ensure_parent_dir(&path).await?;
    let bytes = serde_json::to_vec(state)
        .map_err(|error| fetch_error(format!("序列化下载状态失败: {error}")))?;
    fs::write(path, bytes).await?;
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

pub fn state_path_for(temp_path: &Path) -> PathBuf {
    let mut value = OsString::from(temp_path.as_os_str());
    value.push(".state");
    PathBuf::from(value)
}

fn should_fetch_segmented(
    metadata: &RemoteResourceMetadata,
    resume_from: u64,
    segment_count: usize,
) -> bool {
    resume_from == 0
        && segment_count > 1
        && metadata
            .total_bytes
            .is_some_and(|total_bytes| total_bytes > 1)
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

fn segment_path_for(temp_path: &Path, index: usize) -> PathBuf {
    let mut value = OsString::from(temp_path.as_os_str());
    value.push(format!(".seg{index}"));
    PathBuf::from(value)
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
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
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
