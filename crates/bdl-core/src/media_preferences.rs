//! Ordered source-track preferences. These select existing tracks; they never transcode.
use serde::{Deserialize, Serialize};

use crate::error::{BdlError, BdlResult};
use crate::model::{MediaStream, StreamQuality};
use crate::planner::parse_stream_codec;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VideoPreference {
    pub quality: String,
    pub codec: String,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreferenceFallback {
    Error,
    #[default]
    Best,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct MediaPreferences {
    pub video: Vec<VideoPreference>,
    pub audio: Vec<String>,
    pub fallback: PreferenceFallback,
}

impl MediaPreferences {
    pub fn validate(&self) -> BdlResult<()> {
        if self.video.len() > 32 || self.audio.len() > 32 {
            return Err(BdlError::Planning {
                message: "媒体偏好最多支持 32 条规则。".to_owned(),
            });
        }
        for rule in &self.video {
            if !matches!(
                rule.quality.as_str(),
                "best"
                    | "sdr"
                    | "127"
                    | "126"
                    | "125"
                    | "120"
                    | "116"
                    | "112"
                    | "80"
                    | "74"
                    | "64"
                    | "32"
                    | "16"
            ) {
                return Err(BdlError::Planning {
                    message: format!("视频偏好画质无效：`{}`。", rule.quality),
                });
            }
            parse_stream_codec(&rule.codec)?;
        }
        for quality in &self.audio {
            if !matches!(
                quality.as_str(),
                "best" | "30251" | "30250" | "30255" | "30280" | "30232" | "30216"
            ) {
                return Err(BdlError::Planning {
                    message: format!("音频偏好无效：`{quality}`。"),
                });
            }
        }
        Ok(())
    }
}

impl VideoPreference {
    pub(crate) fn matches(&self, stream: &MediaStream) -> bool {
        let quality_matches = match self.quality.as_str() {
            "best" => true,
            "sdr" => is_sdr(stream.quality),
            value => value
                .parse::<u32>()
                .is_ok_and(|value| stream.quality == StreamQuality::Quality(value)),
        };
        quality_matches
            && (self.codec == "auto"
                || parse_stream_codec(&self.codec).is_ok_and(|codec| codec == stream.codec))
    }
}

pub(crate) fn is_sdr(quality: StreamQuality) -> bool {
    matches!(
        quality,
        StreamQuality::Quality(16 | 32 | 64 | 74 | 80 | 112 | 116 | 120 | 127)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_preferences_settings_roundtrip_and_legacy_defaults() {
        let old: crate::settings::AppSettings =
            serde_json::from_str(r#"{"quality":"80","codec":"avc"}"#).unwrap();
        assert_eq!(old.media_preferences, MediaPreferences::default());
        assert_eq!(old.quality, "80");
        let mut settings = old;
        settings.media_preferences.video.push(VideoPreference {
            quality: "125".into(),
            codec: "hevc".into(),
        });
        settings.media_preferences.audio = vec!["30251".into(), "30280".into()];
        settings.validate().unwrap();
        let restored: crate::settings::AppSettings =
            serde_json::from_str(&serde_json::to_string(&settings).unwrap()).unwrap();
        assert_eq!(restored, settings);
        settings.media_preferences.video[0].codec = "invalid".into();
        assert!(settings.validate().is_err());
    }
}
