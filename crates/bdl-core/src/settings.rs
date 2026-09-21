use serde::{Deserialize, Serialize};

use crate::error::BdlResult;
use crate::naming::{DEFAULT_NAMING_TEMPLATE, DuplicateNamingStrategy, validate_template};
use crate::planner::{
    ArchiveAssetSelection, MissingQualityPolicy, StreamPreference, parse_stream_codec,
};

const MAX_SPEED_LIMIT_BYTES_PER_SECOND: u64 = 10 * 1024 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ParseRules {
    #[serde(alias = "cooldown_min_seconds")]
    pub interval_seconds: u64,
    #[serde(alias = "cooldown_max_seconds")]
    pub rest_seconds: u64,
    pub pages_per_round: usize,
}

impl Default for ParseRules {
    fn default() -> Self {
        Self {
            interval_seconds: 1,
            rest_seconds: 3,
            pages_per_round: 3,
        }
    }
}

impl ParseRules {
    pub fn validate(&self) -> BdlResult<()> {
        if !(1..=120).contains(&self.interval_seconds)
            || !(self.interval_seconds..=120).contains(&self.rest_seconds)
            || !(1..=10).contains(&self.pages_per_round)
        {
            return Err(crate::error::BdlError::Planning {
                message: "每批解析需为20～200条，等待时间需为1～120秒，长休息不能短于批间等待。"
                    .into(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NamingPreset {
    pub id: String,
    pub name: String,
    pub template: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub settings_schema_version: u16,
    pub download_dir: Option<String>,
    pub parse_rules: ParseRules,
    pub naming_template: String,
    pub naming_presets: Vec<NamingPreset>,
    pub quality: String,
    pub archive_mode: String,
    pub archive_assets: ArchiveAssetSelection,
    pub output_extension: String,
    pub duplicate_naming_strategy: DuplicateNamingStrategy,
    pub audio_quality: String,
    pub codec: String,
    pub media_preferences: crate::media_preferences::MediaPreferences,
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
            parse_rules: ParseRules::default(),
            naming_template: DEFAULT_NAMING_TEMPLATE.to_owned(),
            naming_presets: Vec::new(),
            quality: "best".to_owned(),
            archive_mode: "fast".to_owned(),
            archive_assets: ArchiveAssetSelection::all(),
            output_extension: "mp4".to_owned(),
            duplicate_naming_strategy: DuplicateNamingStrategy::default(),
            audio_quality: "best".to_owned(),
            codec: "auto".to_owned(),
            media_preferences: Default::default(),
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
        // New fields (including nested settings) use serde(default). Existing valid
        // choices are not rewritten based on a release/schema version.
        if self.naming_template.trim().is_empty() {
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
        self.parse_rules.validate()?;
        validate_template(&self.naming_template)?;
        validate_naming_presets(&self.naming_presets)?;
        validate_archive_mode(&self.archive_mode)?;
        validate_output_extension(&self.output_extension)?;
        validate_embedding_container(
            &self.output_extension,
            self.embed_cover,
            self.embed_subtitles,
        )?;
        StreamPreference::parse_video(&self.quality)?;
        StreamPreference::parse(&self.audio_quality, "音频质量")?;
        parse_stream_codec(&self.codec)?;
        self.media_preferences.validate()?;
        MissingQualityPolicy::parse(&self.missing_quality_policy)?;
        validate_log_level(&self.log_level)?;
        validate_proxy_url(self.proxy_url.as_deref())?;
        validate_segment_count(self.segment_count)?;
        validate_speed_limit(self.global_speed_limit_bytes_per_second, "全局下载限速")?;
        Ok(())
    }
}

fn validate_naming_presets(presets: &[NamingPreset]) -> BdlResult<()> {
    let mut ids = std::collections::HashSet::new();
    let mut names = std::collections::HashSet::new();
    if presets.len() > 32 {
        return Err(crate::error::BdlError::Planning {
            message: "命名预设最多保存 32 个。".into(),
        });
    }
    for preset in presets {
        if preset.id.is_empty()
            || preset.name.trim().is_empty()
            || preset.name.chars().count() > 40
            || !ids.insert(&preset.id)
            || !names.insert(preset.name.trim())
        {
            return Err(crate::error::BdlError::Planning {
                message: "命名预设需要唯一标识和不重复的名称，名称最多 40 个字符。".into(),
            });
        }
        validate_template(&preset.template)?;
    }
    Ok(())
}

pub fn validate_embedding_container(
    output_extension: &str,
    embed_cover: bool,
    embed_subtitles: bool,
) -> BdlResult<()> {
    if output_extension != "mkv" && (embed_cover || embed_subtitles) {
        return Err(crate::error::BdlError::Planning {
            message: "嵌入封面和字幕仅支持 MKV 封装，请改用 MKV 或关闭嵌入选项。".to_owned(),
        });
    }

    Ok(())
}

fn validate_archive_mode(value: &str) -> BdlResult<()> {
    match value {
        "fast" | "complete_archive" | "custom" => Ok(()),
        other => Err(crate::error::BdlError::Planning {
            message: format!("下载内容模式设置无效：`{other}`。"),
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

#[cfg(test)]
mod naming_preset_tests {
    use super::*;

    #[test]
    fn naming_presets_roundtrip_and_old_settings_default() {
        let mut settings: AppSettings = serde_json::from_str("{}").unwrap();
        assert!(settings.naming_presets.is_empty());
        settings.naming_presets.push(NamingPreset {
            id: "favorite".into(),
            name: "收藏".into(),
            template: "{title}.{ext}".into(),
        });
        settings.validate().unwrap();
        let restored: AppSettings =
            serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
        assert_eq!(restored.naming_presets, settings.naming_presets);
        settings
            .naming_presets
            .push(settings.naming_presets[0].clone());
        assert!(settings.validate().is_err());
        settings.naming_presets.pop();
        settings.naming_presets[0].template = "{unknown}.{ext}".into();
        assert!(settings.validate().is_err());
    }
}

#[cfg(test)]
mod parse_rules_tests {
    use super::*;
    #[test]
    fn pagination_rules_default_and_validate() {
        let settings: AppSettings = serde_json::from_str("{}").unwrap();
        assert_eq!(settings.parse_rules.pages_per_round, 3);
        assert_eq!(settings.parse_rules.interval_seconds, 1);
        assert_eq!(settings.parse_rules.rest_seconds, 3);
        settings.validate().unwrap();
        for rules in [
            ParseRules {
                pages_per_round: 0,
                ..Default::default()
            },
            ParseRules {
                interval_seconds: 8,
                rest_seconds: 3,
                ..Default::default()
            },
            ParseRules {
                rest_seconds: 121,
                ..Default::default()
            },
        ] {
            assert!(rules.validate().is_err());
        }
    }
}
