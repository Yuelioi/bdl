use serde::{Deserialize, Serialize};

use crate::error::BdlResult;
use crate::naming::{DEFAULT_NAMING_TEMPLATE, DuplicateNamingStrategy, validate_template};
use crate::planner::{
    ArchiveAssetSelection, MissingQualityPolicy, StreamPreference, parse_stream_codec,
};

const LEGACY_DUPLICATE_TITLE_TEMPLATE: &str =
    "{title}/{title} - P{part_index} - {part_title}.{ext}";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub download_dir: Option<String>,
    pub naming_template: String,
    pub quality: String,
    pub archive_mode: String,
    pub archive_assets: ArchiveAssetSelection,
    pub output_extension: String,
    pub duplicate_naming_strategy: DuplicateNamingStrategy,
    pub audio_quality: String,
    pub codec: String,
    pub missing_quality_policy: String,
    pub ffmpeg_path: Option<String>,
    pub retain_raw_streams: bool,
    pub proxy_url: Option<String>,
    pub log_level: String,
    pub data_dir: Option<String>,
    pub concurrent_tasks: usize,
    pub retry_count: usize,
    pub auto_refresh_expired_urls: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            download_dir: None,
            naming_template: DEFAULT_NAMING_TEMPLATE.to_owned(),
            quality: "best".to_owned(),
            archive_mode: "fast".to_owned(),
            archive_assets: ArchiveAssetSelection::all(),
            output_extension: "mp4".to_owned(),
            duplicate_naming_strategy: DuplicateNamingStrategy::default(),
            audio_quality: "best".to_owned(),
            codec: "auto".to_owned(),
            missing_quality_policy: "lower".to_owned(),
            ffmpeg_path: None,
            retain_raw_streams: false,
            proxy_url: None,
            log_level: "info".to_owned(),
            data_dir: None,
            concurrent_tasks: 1,
            retry_count: 3,
            auto_refresh_expired_urls: true,
        }
    }
}

impl AppSettings {
    pub fn normalized(mut self) -> Self {
        if self.naming_template.trim().is_empty()
            || self.naming_template.trim() == LEGACY_DUPLICATE_TITLE_TEMPLATE
        {
            self.naming_template = DEFAULT_NAMING_TEMPLATE.to_owned();
        }
        self.ffmpeg_path = self
            .ffmpeg_path
            .as_deref()
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(ToOwned::to_owned);
        self.proxy_url = self
            .proxy_url
            .as_deref()
            .map(str::trim)
            .filter(|url| !url.is_empty())
            .map(ToOwned::to_owned);
        self.data_dir = self
            .data_dir
            .as_deref()
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(ToOwned::to_owned);

        self
    }

    pub fn validate(&self) -> BdlResult<()> {
        validate_template(&self.naming_template)?;
        validate_archive_mode(&self.archive_mode)?;
        validate_output_extension(&self.output_extension)?;
        StreamPreference::parse(&self.quality, "视频清晰度")?;
        StreamPreference::parse(&self.audio_quality, "音频质量")?;
        parse_stream_codec(&self.codec)?;
        MissingQualityPolicy::parse(&self.missing_quality_policy)?;
        validate_log_level(&self.log_level)?;
        validate_proxy_url(self.proxy_url.as_deref())?;
        Ok(())
    }
}

fn validate_archive_mode(value: &str) -> BdlResult<()> {
    match value {
        "fast" | "complete_archive" | "custom" => Ok(()),
        other => Err(crate::error::BdlError::Planning {
            message: format!("归档模式设置无效：`{other}`。"),
        }),
    }
}

fn validate_output_extension(value: &str) -> BdlResult<()> {
    match value {
        "mp4" | "mkv" => Ok(()),
        other => Err(crate::error::BdlError::Planning {
            message: format!("封装格式设置无效：`{other}`。"),
        }),
    }
}

fn validate_log_level(value: &str) -> BdlResult<()> {
    match value {
        "debug" | "info" | "warning" | "error" => Ok(()),
        other => Err(crate::error::BdlError::Planning {
            message: format!("日志级别设置无效：`{other}`。"),
        }),
    }
}

fn validate_proxy_url(value: Option<&str>) -> BdlResult<()> {
    let Some(value) = value else {
        return Ok(());
    };
    let parsed = url::Url::parse(value).map_err(|error| crate::error::BdlError::Planning {
        message: format!("代理地址无效：{error}"),
    })?;
    match parsed.scheme() {
        "http" | "https" | "socks5" | "socks5h" => Ok(()),
        scheme => Err(crate::error::BdlError::Planning {
            message: format!("代理地址协议不支持：`{scheme}`。"),
        }),
    }
}
