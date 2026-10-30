//! Saved download recipes. Tasks own a snapshot, independent of later preset edits.
use serde::{Deserialize, Serialize};

use crate::planner::{
    ArchiveAssetSelection, ArchiveMode, DownloadMediaMode, DownloadOptions, MissingQualityPolicy,
    StreamPreference, parse_stream_codec,
};
use crate::queue::{DownloadResourceIntent, TaskProcessingOptions};
use crate::{BdlError, BdlResult};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MediaInput {
    pub enabled: bool,
    pub save: bool,
    pub format: String,
    pub retain_original: bool,
}

impl Default for MediaInput {
    fn default() -> Self {
        Self {
            enabled: false,
            save: false,
            format: "m4s".into(),
            retain_original: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AssetOutput {
    pub enabled: bool,
    pub save: bool,
    pub embed: bool,
    pub format: String,
    pub retain_original: bool,
}

impl Default for AssetOutput {
    fn default() -> Self {
        Self {
            enabled: false,
            save: true,
            embed: false,
            format: "original".into(),
            retain_original: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DownloadWorkflow {
    pub video: MediaInput,
    pub audio: MediaInput,
    pub cover: AssetOutput,
    pub subtitles: AssetOutput,
    pub danmaku: AssetOutput,
    pub nfo: bool,
    pub nfo_options: crate::nfo::NfoOptions,
    pub merge_media: bool,
    pub container: String,
    pub quality: String,
    pub audio_quality: String,
    pub codec: String,
    pub media_preferences: crate::media_preferences::MediaPreferences,
    pub missing_quality_policy: String,
}

impl Default for DownloadWorkflow {
    fn default() -> Self {
        Self {
            video: MediaInput {
                enabled: true,
                ..Default::default()
            },
            audio: MediaInput {
                enabled: true,
                ..Default::default()
            },
            cover: Default::default(),
            subtitles: AssetOutput {
                format: "srt".into(),
                ..Default::default()
            },
            danmaku: AssetOutput {
                format: "xml".into(),
                ..Default::default()
            },
            nfo: false,
            nfo_options: Default::default(),
            merge_media: true,
            container: "mp4".into(),
            quality: "best".into(),
            audio_quality: "best".into(),
            codec: "auto".into(),
            media_preferences: Default::default(),
            missing_quality_policy: "lower".into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadPreset {
    pub id: String,
    pub name: String,
    pub workflow: DownloadWorkflow,
}

/// Planned retained files, including converted outputs and Android document URIs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadArtifact {
    pub path: std::path::PathBuf,
    pub intent: Option<DownloadResourceIntent>,
    pub resource_id: Option<String>,
    pub original: bool,
    #[serde(default)]
    pub document_uri: Option<String>,
}

impl DownloadWorkflow {
    pub fn validate(&self) -> BdlResult<()> {
        if self.nfo {
            self.nfo_options.validate()?;
        }
        let invalid = |message: &str| BdlError::Planning {
            message: message.into(),
        };
        if !matches!(self.container.as_str(), "mp4" | "mkv")
            || self.video.format != "m4s"
            || !matches!(self.audio.format.as_str(), "m4s" | "mp3")
        {
            return Err(invalid(
                "媒体格式无效：视频成品支持 MP4/MKV，独立轨道支持 m4s，音频还支持 MP3。",
            ));
        }
        if !matches!(self.subtitles.format.as_str(), "original" | "srt" | "ass")
            || !matches!(self.danmaku.format.as_str(), "xml" | "html" | "srt" | "ass")
            || self.cover.format != "original"
            || self.danmaku.embed
        {
            return Err(invalid("字幕、封面或弹幕输出格式无效。"));
        }
        let merge = self.merge_media && self.video.enabled;
        for rule in [&self.video, &self.audio] {
            if rule.enabled && !rule.save && !rule.retain_original && !merge {
                return Err(invalid("未生成媒体成品时，所选音视频必须保存为独立文件。"));
            }
        }
        for rule in [&self.cover, &self.subtitles, &self.danmaku] {
            if rule.enabled && !rule.save && !rule.embed && !rule.retain_original {
                return Err(invalid("所选内容需要独立保存、嵌入或保留原始文件。"));
            }
            if rule.enabled && rule.embed && (!merge || self.container != "mkv") {
                return Err(invalid("嵌入封面或字幕需要生成 MKV 视频成品。"));
            }
        }
        if self.subtitles.enabled && self.subtitles.embed && self.subtitles.format == "original" {
            return Err(invalid("嵌入字幕请选择 SRT 或 ASS。"));
        }
        if !self.video.enabled
            && !self.audio.enabled
            && !self.cover.enabled
            && !self.subtitles.enabled
            && !self.danmaku.enabled
            && !self.nfo
        {
            return Err(invalid("预设至少需要选择一种内容。"));
        }
        StreamPreference::parse_video(&self.quality)?;
        StreamPreference::parse(&self.audio_quality, "音频质量")?;
        parse_stream_codec(&self.codec)?;
        self.media_preferences.validate()?;
        MissingQualityPolicy::parse(&self.missing_quality_policy)?;
        Ok(())
    }

    pub fn with_quality_settings(&self, settings: &crate::settings::AppSettings) -> Self {
        Self {
            quality: settings.quality.clone(),
            audio_quality: settings.audio_quality.clone(),
            codec: settings.codec.clone(),
            media_preferences: settings.media_preferences.clone(),
            missing_quality_policy: settings.missing_quality_policy.clone(),
            ..self.clone()
        }
    }

    pub fn has_media_output(&self) -> bool {
        self.merge_media && self.video.enabled
    }

    pub fn apply(&self, options: &mut DownloadOptions) -> BdlResult<()> {
        self.validate()?;
        options.media_mode = match (self.video.enabled, self.audio.enabled) {
            (true, true) => DownloadMediaMode::AudioVideo,
            (true, false) => DownloadMediaMode::VideoOnly,
            (false, true) => DownloadMediaMode::AudioOnly,
            (false, false) => DownloadMediaMode::AssetsOnly,
        };
        options.video_quality = StreamPreference::parse_video(&self.quality)?;
        options.audio_quality = StreamPreference::parse(&self.audio_quality, "音频质量")?;
        options.video_codec = parse_stream_codec(&self.codec)?;
        options.media_preferences = self.media_preferences.clone();
        options.missing_quality_policy = MissingQualityPolicy::parse(&self.missing_quality_policy)?;
        options.archive_mode = ArchiveMode::Custom;
        options.archive_assets = ArchiveAssetSelection {
            cover: self.cover.enabled,
            subtitles: self.subtitles.enabled,
            danmaku: self.danmaku.enabled,
            nfo: self.nfo,
        };
        options.output_extension = if self.has_media_output() {
            self.container.clone()
        } else if self.video.enabled {
            "m4s".into()
        } else if self.audio.enabled {
            self.audio.format.clone()
        } else if self.subtitles.enabled {
            if self.subtitles.format == "original" {
                "json".into()
            } else {
                self.subtitles.format.clone()
            }
        } else if self.cover.enabled {
            "jpg".into()
        } else if self.danmaku.enabled {
            self.danmaku.format.clone()
        } else {
            "nfo".into()
        };
        options.processing = Some(TaskProcessingOptions {
            retain_raw_streams: self.video.retain_original || self.audio.retain_original,
            embed_cover: self.cover.enabled && self.cover.embed,
            embed_subtitles: self.subtitles.enabled && self.subtitles.embed,
            archive_assets: Some(ArchiveAssetSelection {
                cover: self.cover.enabled && self.cover.save,
                subtitles: self.subtitles.enabled && self.subtitles.save,
                danmaku: self.danmaku.enabled && self.danmaku.save,
                nfo: self.nfo,
            }),
            subtitle_format: match self.subtitles.format.as_str() {
                "srt" => Some(crate::subtitles::SubtitleFormat::Srt),
                "ass" => Some(crate::subtitles::SubtitleFormat::Ass),
                _ => None,
            },
            danmaku_format: Some(match self.danmaku.format.as_str() {
                "html" => crate::danmaku::DanmakuFormat::Html,
                "srt" => crate::danmaku::DanmakuFormat::Srt,
                "ass" => crate::danmaku::DanmakuFormat::Ass,
                _ => crate::danmaku::DanmakuFormat::Xml,
            }),
        });
        options.workflow = Some(self.clone());
        Ok(())
    }

    pub fn fingerprint(&self) -> String {
        // Stable FNV-1a over the configuration, not the preset's display name/id.
        let bytes = serde_json::to_vec(self).expect("workflow serialization is infallible");
        let hash = bytes.iter().fold(0xcbf29ce484222325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        });
        format!("{hash:016x}")
    }

    pub fn from_legacy(settings: &crate::settings::AppSettings) -> Self {
        let selected = match settings.archive_mode.as_str() {
            "complete_archive" => ArchiveAssetSelection::all(),
            "custom" => settings.archive_assets,
            _ => ArchiveAssetSelection::none(),
        };
        let video = settings.media_mode != DownloadMediaMode::AudioOnly;
        let audio = settings.media_mode != DownloadMediaMode::VideoOnly;
        let asset = |enabled, save, embed, format: String| AssetOutput {
            enabled,
            save: !enabled || save,
            embed,
            format,
            retain_original: false,
        };
        Self {
            video: MediaInput {
                enabled: video,
                save: false,
                retain_original: settings.retain_raw_streams,
                format: "m4s".into(),
            },
            audio: MediaInput {
                enabled: audio,
                save: !video,
                retain_original: settings.retain_raw_streams,
                format: settings.audio_output_format.clone(),
            },
            cover: asset(
                selected.cover || (video && settings.embed_cover),
                selected.cover,
                video && settings.embed_cover,
                "original".into(),
            ),
            subtitles: asset(
                selected.subtitles || (video && settings.embed_subtitles),
                selected.subtitles,
                video && settings.embed_subtitles,
                settings.subtitle_format.extension().into(),
            ),
            danmaku: asset(
                selected.danmaku,
                selected.danmaku,
                false,
                settings.danmaku_format.extension().into(),
            ),
            nfo: selected.nfo,
            nfo_options: Default::default(),
            merge_media: video,
            container: settings.output_extension.clone(),
            quality: settings.quality.clone(),
            audio_quality: settings.audio_quality.clone(),
            codec: settings.codec.clone(),
            media_preferences: settings.media_preferences.clone(),
            missing_quality_policy: settings.missing_quality_policy.clone(),
        }
    }
}

/// Templates from the first local preset implementation, retained for migration and fixtures.
pub fn legacy_builtin_presets() -> Vec<DownloadPreset> {
    let base = DownloadWorkflow {
        subtitles: AssetOutput {
            format: "srt".into(),
            ..Default::default()
        },
        danmaku: AssetOutput {
            format: "xml".into(),
            ..Default::default()
        },
        ..Default::default()
    };
    let assets = DownloadWorkflow {
        video: Default::default(),
        audio: Default::default(),
        merge_media: false,
        ..base.clone()
    };
    let audio = DownloadWorkflow {
        video: Default::default(),
        audio: MediaInput {
            enabled: true,
            save: true,
            format: "mp3".into(),
            retain_original: false,
        },
        merge_media: false,
        ..base.clone()
    };
    let subtitle = DownloadWorkflow {
        subtitles: AssetOutput {
            enabled: true,
            format: "srt".into(),
            ..Default::default()
        },
        ..assets.clone()
    };
    let danmaku = DownloadWorkflow {
        danmaku: AssetOutput {
            enabled: true,
            format: "html".into(),
            ..Default::default()
        },
        ..assets.clone()
    };
    let cover = DownloadWorkflow {
        cover: AssetOutput {
            enabled: true,
            ..Default::default()
        },
        ..assets
    };
    let archive = DownloadWorkflow {
        container: "mkv".into(),
        cover: AssetOutput {
            enabled: true,
            embed: true,
            ..Default::default()
        },
        subtitles: AssetOutput {
            enabled: true,
            embed: true,
            ..base.subtitles.clone()
        },
        danmaku: AssetOutput {
            enabled: true,
            ..base.danmaku.clone()
        },
        nfo: true,
        ..base.clone()
    };
    [
        ("video", "音频+视频", base),
        ("audio", "仅音频（MP3）", audio),
        ("subtitles", "仅字幕（SRT）", subtitle),
        ("danmaku", "仅弹幕（HTML）", danmaku),
        ("cover", "仅封面", cover),
        ("archive", "完整归档（MKV）", archive),
    ]
    .into_iter()
    .map(|(id, name, workflow)| DownloadPreset {
        id: id.into(),
        name: name.into(),
        workflow,
    })
    .collect()
}

pub fn builtin_presets() -> Vec<DownloadPreset> {
    let base = DownloadWorkflow::default();
    let all = DownloadWorkflow {
        cover: AssetOutput {
            enabled: true,
            ..base.cover.clone()
        },
        subtitles: AssetOutput {
            enabled: true,
            ..base.subtitles.clone()
        },
        danmaku: AssetOutput {
            enabled: true,
            ..base.danmaku.clone()
        },
        nfo: true,
        ..base.clone()
    };
    let mkv = DownloadWorkflow {
        container: "mkv".into(),
        subtitles: AssetOutput {
            enabled: true,
            save: false,
            embed: true,
            ..base.subtitles.clone()
        },
        ..base.clone()
    };
    [
        ("video", "快速下载", base),
        ("all", "下载全部资源", all),
        ("mkv", "封装 MKV（视频+字幕）", mkv),
    ]
    .into_iter()
    .map(|(id, name, workflow)| DownloadPreset {
        id: id.into(),
        name: name.into(),
        workflow,
    })
    .collect()
}

pub fn validate_presets(presets: &[DownloadPreset], selected: &str) -> BdlResult<()> {
    let mut ids = std::collections::HashSet::new();
    let mut names = std::collections::HashSet::new();
    if presets.is_empty() || presets.len() > 32 {
        return Err(BdlError::Planning {
            message: "下载预设需保留 1～32 个。".into(),
        });
    }
    for preset in presets {
        if preset.id.trim().is_empty()
            || preset.name.trim().is_empty()
            || preset.name.chars().count() > 40
            || !ids.insert(preset.id.as_str())
            || !names.insert(preset.name.trim())
        {
            return Err(BdlError::Planning {
                message: "下载预设需要唯一标识和不重复的名称，名称最多 40 个字符。".into(),
            });
        }
        preset.workflow.validate()?;
    }
    if !ids.contains(selected) {
        return Err(BdlError::Planning {
            message: "默认下载预设不存在，请重新选择。".into(),
        });
    }
    Ok(())
}

pub fn reserve_task_paths(
    task: crate::queue::DownloadTask,
    reserved: &mut crate::naming::ReservedPaths,
    strategy: crate::naming::DuplicateNamingStrategy,
) -> BdlResult<Option<crate::queue::DownloadTask>> {
    use crate::naming::DuplicateNamingStrategy;
    let base = task.output_path.clone();
    let stem_limit = task.file_stem_byte_limit()?;
    let mut candidate = task;
    for index in 0..10000 {
        if index > 0 {
            candidate = candidate
                .with_output_path(crate::naming::with_counter_suffix(&base, index, stem_limit))?;
        }
        let live_collision = candidate
            .file_paths()
            .iter()
            .any(|path| reserved.contains(path));
        let disk_collision = candidate
            .media_selection
            .artifacts
            .iter()
            .any(|artifact| artifact.path.exists());
        if !live_collision && disk_collision && strategy == DuplicateNamingStrategy::SkipExisting {
            return Ok(None);
        }
        if !live_collision
            && (!disk_collision || strategy == DuplicateNamingStrategy::OverwriteExisting)
        {
            reserved.extend(candidate.file_paths());
            return Ok(Some(candidate));
        }
    }
    Err(BdlError::Planning {
        message: "无法分配独立的下载文件名。".into(),
    })
}
