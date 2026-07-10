use serde::{Deserialize, Serialize};

use crate::error::BdlResult;
use crate::naming::{DEFAULT_NAMING_TEMPLATE, DuplicateNamingStrategy, validate_template};
use crate::planner::{
    ArchiveAssetSelection, MissingQualityPolicy, StreamPreference, parse_stream_codec,
};

const LEGACY_DUPLICATE_TITLE_TEMPLATE: &str =
    "{title}/{title} - P{part_index} - {part_title}.{ext}";
const MAX_SPEED_LIMIT_BYTES_PER_SECOND: u64 = 10 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    #[serde(default)]
    pub settings_schema_version: u16,
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
    pub embed_cover: bool,
    pub embed_subtitles: bool,
    pub proxy_url: Option<String>,
    pub log_level: String,
    pub data_dir: Option<String>,
    pub concurrent_tasks: usize,
    pub retry_count: usize,
    pub segment_count: usize,
    pub global_speed_limit_bytes_per_second: Option<u64>,
    pub startup_auto_recovery: bool,
    pub auto_refresh_expired_urls: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            settings_schema_version: 1,
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
            embed_cover: false,
            embed_subtitles: false,
            proxy_url: None,
            log_level: "info".to_owned(),
            data_dir: None,
            concurrent_tasks: 1,
            retry_count: 3,
            segment_count: 4,
            global_speed_limit_bytes_per_second: None,
            startup_auto_recovery: false,
            auto_refresh_expired_urls: true,
        }
    }
}

impl AppSettings {
    pub fn normalized(mut self) -> Self {
        if self.settings_schema_version == 0 {
            if self.segment_count == 1 {
                self.segment_count = 4;
            }
            self.settings_schema_version = 1;
        }
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
        self.segment_count = normalize_segment_count(self.segment_count);
        self.global_speed_limit_bytes_per_second = self
            .global_speed_limit_bytes_per_second
            .filter(|limit| *limit > 0);

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
        validate_segment_count(self.segment_count)?;
        validate_speed_limit(self.global_speed_limit_bytes_per_second, "全局下载限速")?;
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

fn normalize_segment_count(value: usize) -> usize {
    match value {
        1 | 2 | 4 | 8 => value,
        _ => 1,
    }
}

fn validate_segment_count(value: usize) -> BdlResult<()> {
    match value {
        1 | 2 | 4 | 8 => Ok(()),
        other => Err(crate::error::BdlError::Planning {
            message: format!("单任务分段数设置无效：`{other}`。"),
        }),
    }
}

pub fn validate_speed_limit(value: Option<u64>, label: &str) -> BdlResult<()> {
    let Some(value) = value else {
        return Ok(());
    };
    if value == 0 || value > MAX_SPEED_LIMIT_BYTES_PER_SECOND {
        return Err(crate::error::BdlError::Planning {
            message: format!("{label}必须大于 0 且不超过 10 GiB/s。"),
        });
    }
    Ok(())
}
