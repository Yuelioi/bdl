use serde::{Deserialize, Serialize};

use crate::error::BdlResult;
use crate::naming::{DEFAULT_NAMING_TEMPLATE, DuplicateNamingStrategy, validate_template};
use crate::planner::{
    ArchiveAssetSelection, MissingQualityPolicy, StreamPreference, parse_stream_codec,
};

const MAX_SPEED_LIMIT_BYTES_PER_SECOND: u64 = 10 * 1024 * 1024 * 1024;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyMode {
    #[default]
    System,
    Direct,
    Custom,
}

impl ProxyMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Direct => "direct",
            Self::Custom => "custom",
        }
    }

    pub fn configure_client(
        self,
        builder: reqwest::ClientBuilder,
        proxy_url: Option<&str>,
    ) -> BdlResult<reqwest::ClientBuilder> {
        match self {
            Self::System => Ok(builder),
            Self::Direct => Ok(builder.no_proxy()),
            Self::Custom => {
                let address = required_proxy_url(proxy_url)?;
                validate_proxy_url(Some(address))?;
                let proxy =
                    reqwest::Proxy::all(address).map_err(|_| crate::error::BdlError::Planning {
                        message: "代理地址无效。".into(),
                    })?;
                Ok(builder.proxy(proxy))
            }
        }
    }
}

fn legacy_settings_schema_version() -> u16 {
    1
}

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
pub struct DocumentTreeDirectory {
    pub tree_uri: String,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    #[serde(default = "legacy_settings_schema_version")]
    pub settings_schema_version: u16,
    #[serde(default)]
    pub clipboard_monitor_enabled: bool,
    pub usage_notice_acknowledged: bool,
    pub auto_check_updates: bool,
    pub theme_preference: String,
    pub download_dir: Option<String>,
    pub document_tree_output: Option<DocumentTreeDirectory>,
    pub parse_rules: ParseRules,
    pub naming_template: String,
    pub naming_presets: Vec<NamingPreset>,
    pub download_presets: Vec<crate::workflow::DownloadPreset>,
    pub selected_download_preset: String,
    pub quality: String,
    pub archive_mode: String,
    pub archive_assets: ArchiveAssetSelection,
    pub output_extension: String,
    pub media_mode: crate::planner::DownloadMediaMode,
    pub audio_output_format: String,
    pub subtitle_format: crate::subtitles::SubtitleFormat,
    pub danmaku_format: crate::danmaku::DanmakuFormat,
    pub duplicate_naming_strategy: DuplicateNamingStrategy,
    pub audio_quality: String,
    pub codec: String,
    pub media_preferences: crate::media_preferences::MediaPreferences,
    pub missing_quality_policy: String,
    pub ffmpeg_path: Option<String>,
    pub retain_raw_streams: bool,
    pub embed_cover: bool,
    pub embed_subtitles: bool,
    pub proxy_mode: ProxyMode,
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
            settings_schema_version: 2,
            clipboard_monitor_enabled: false,
            usage_notice_acknowledged: false,
            auto_check_updates: false,
            theme_preference: "system".to_owned(),
            download_dir: None,
            document_tree_output: None,
            parse_rules: ParseRules::default(),
            naming_template: DEFAULT_NAMING_TEMPLATE.to_owned(),
            naming_presets: Vec::new(),
            download_presets: Vec::new(),
            selected_download_preset: String::new(),
            quality: "best".to_owned(),
            archive_mode: "fast".to_owned(),
            archive_assets: ArchiveAssetSelection::all(),
            output_extension: "mp4".to_owned(),
            media_mode: crate::planner::DownloadMediaMode::default(),
            audio_output_format: "m4s".to_owned(),
            subtitle_format: crate::subtitles::SubtitleFormat::default(),
            danmaku_format: crate::danmaku::DanmakuFormat::default(),
            duplicate_naming_strategy: DuplicateNamingStrategy::default(),
            audio_quality: "best".to_owned(),
            codec: "auto".to_owned(),
            media_preferences: Default::default(),
            missing_quality_policy: "lower".to_owned(),
            ffmpeg_path: None,
            retain_raw_streams: false,
            embed_cover: false,
            embed_subtitles: false,
            proxy_mode: ProxyMode::System,
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
        // Missing nested fields retain defaults. The v1 proxy setting is migrated
        // below because an empty custom address never meant explicit direct mode.
        if self.naming_template.trim().is_empty() {
            self.naming_template = DEFAULT_NAMING_TEMPLATE.to_owned();
        }
        self.ffmpeg_path = self
            .ffmpeg_path
            .as_deref()
            .map(str::trim)
            .filter(|path| !path.is_empty())
            .map(ToOwned::to_owned);
        self.document_tree_output = self.document_tree_output.and_then(|directory| {
            let tree_uri = directory.tree_uri.trim().to_owned();
            if tree_uri.is_empty() {
                return None;
            }
            let display_name = directory.display_name.trim().to_owned();
            Some(DocumentTreeDirectory {
                tree_uri,
                display_name: if display_name.is_empty() {
                    "已选择目录".to_owned()
                } else {
                    display_name
                },
            })
        });
        self.proxy_url = self
            .proxy_url
            .as_deref()
            .map(str::trim)
            .filter(|url| !url.is_empty())
            .map(ToOwned::to_owned);
        if self.settings_schema_version < 2 {
            self.proxy_mode = if self.proxy_url.is_some() {
                ProxyMode::Custom
            } else {
                ProxyMode::System
            };
            self.settings_schema_version = 2;
        }
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

        if self.download_presets.is_empty() {
            self.download_presets = crate::workflow::builtin_presets();
            let legacy = crate::workflow::DownloadWorkflow::from_legacy(&self);
            let matching = self
                .download_presets
                .iter()
                .find(|preset| preset.workflow.with_quality_settings(&self) == legacy);
            self.selected_download_preset = if let Some(preset) = matching {
                preset.id.clone()
            } else {
                self.download_presets.push(crate::workflow::DownloadPreset {
                    id: "migrated".into(),
                    name: "历史配置".into(),
                    workflow: legacy,
                });
                "migrated".into()
            };
        } else if self.selected_download_preset.is_empty() {
            self.selected_download_preset = self.download_presets[0].id.clone();
        }

        for preset in &mut self.download_presets {
            if preset.id == "migrated" && preset.name == "原下载配置" {
                preset.name = "历史配置".into();
            }
        }

        self
    }

    pub fn validate(&self) -> BdlResult<()> {
        self.parse_rules.validate()?;
        validate_template(&self.naming_template)?;
        validate_naming_presets(&self.naming_presets)?;
        if !self.download_presets.is_empty() {
            crate::workflow::validate_presets(
                &self.download_presets,
                &self.selected_download_preset,
            )?;
        }
        validate_archive_mode(&self.archive_mode)?;
        validate_output_extension(&self.output_extension)?;
        if !matches!(self.audio_output_format.as_str(), "m4s" | "mp3") {
            return Err(crate::error::BdlError::Planning {
                message: "音频输出格式仅支持 m4s 或 MP3。".to_owned(),
            });
        }
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
        if self.proxy_mode == ProxyMode::Custom {
            validate_proxy_url(Some(required_proxy_url(self.proxy_url.as_deref())?))?;
        }
        validate_segment_count(self.segment_count)?;
        validate_speed_limit(self.global_speed_limit_bytes_per_second, "全局下载限速")?;
        if let Some(directory) = &self.document_tree_output
            && !directory.tree_uri.starts_with("content://")
        {
            return Err(crate::error::BdlError::Planning {
                message: "Android 导出目录无效，请重新选择保存位置。".to_owned(),
            });
        }
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

fn required_proxy_url(value: Option<&str>) -> BdlResult<&str> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| crate::error::BdlError::Planning {
            message: "手动设置代理时，请填写代理地址。".into(),
        })
}

fn validate_proxy_url(value: Option<&str>) -> BdlResult<()> {
    let Some(value) = value else {
        return Ok(());
    };
    let parsed = url::Url::parse(value).map_err(|error| crate::error::BdlError::Planning {
        message: format!("代理地址无效：{error}"),
    })?;
    if matches!(parsed.scheme(), "http" | "https" | "socks5" | "socks5h")
        && (parsed.host_str().is_none() || parsed.fragment().is_some() || parsed.query().is_some())
    {
        return Err(crate::error::BdlError::Planning {
            message: "代理地址须包含主机，且不能包含查询参数或片段。".into(),
        });
    }
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
