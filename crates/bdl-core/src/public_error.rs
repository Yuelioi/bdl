//! Public failure data shared by process boundaries. Native errors retain the private cause.
use std::collections::BTreeMap;
use std::io::ErrorKind;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::{BdlError, muxer::MuxError};

include!(concat!(env!("OUT_DIR"), "/error_codes.rs"));

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[error("{code}")]
pub struct ErrorCondition {
    pub code: ErrorCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<ErrorParams>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged, rename_all_fields = "camelCase")]
pub enum ErrorParams {
    Cooldown { retry_after_seconds: u64 },
    MissingArguments { arguments: Vec<String> },
}

impl ErrorCondition {
    pub const fn new(code: ErrorCode) -> Self {
        Self { code, params: None }
    }

    pub fn cooldown(seconds: u64) -> Self {
        Self {
            code: ErrorCode::ParseCooldown,
            params: Some(ErrorParams::Cooldown {
                retry_after_seconds: seconds,
            }),
        }
    }

    pub fn from_io(error: &std::io::Error) -> Self {
        Self::new(match error.kind() {
            ErrorKind::StorageFull => ErrorCode::FileDiskFull,
            ErrorKind::NotFound => ErrorCode::FileNotFound,
            ErrorKind::PermissionDenied => ErrorCode::FileAccessDenied,
            ErrorKind::ReadOnlyFilesystem => ErrorCode::FileReadOnly,
            _ => ErrorCode::FileIoFailed,
        })
    }

    pub fn from_mux(error: &MuxError) -> Self {
        match error {
            MuxError::FfmpegNotFound { .. } => Self::new(ErrorCode::MediaFfmpegMissing),
            MuxError::MissingMediaInput => Self::new(ErrorCode::MediaInputMissing),
            MuxError::CommandFailed { .. } => Self::new(ErrorCode::MediaMuxFailed),
            MuxError::Io(source) => Self::from_io(source),
        }
    }
}

impl BdlError {
    pub fn public_problem(&self) -> PublicError {
        match self {
            Self::Traced { trace_id, source } => PublicError {
                condition: source.public_condition(),
                trace_id: *trace_id,
            },
            Self::Context { source, .. } => source.public_problem(),
            _ => PublicError::new(self.public_condition()),
        }
    }

    pub fn public_condition(&self) -> ErrorCondition {
        match self {
            Self::Context { source, .. } | Self::Traced { source, .. } => source.public_condition(),
            Self::DownloadHttp { status, .. } => ErrorCondition::new(match status.as_u16() {
                404 | 410 => ErrorCode::DownloadUrlExpired,
                403 => ErrorCode::DownloadForbidden,
                429 => ErrorCode::SourceRateLimited,
                412 => ErrorCode::SourceVerificationRequired,
                200..=299 => ErrorCode::DownloadRangeRejected,
                _ => ErrorCode::DownloadHttpFailed,
            }),
            Self::Classified { code, .. } => ErrorCondition::new(*code),
            Self::FileIo { source, .. } | Self::Io(source) => ErrorCondition::from_io(source),
            Self::Mux(source) => ErrorCondition::from_mux(source),
            Self::ParseRestricted { seconds, .. } => ErrorCondition::cooldown(*seconds),
            Self::DrmProtected => ErrorCondition::new(ErrorCode::MediaDrmProtected),
            Self::CoursePreviewOnly => ErrorCondition::new(ErrorCode::AccountPreviewOnly),
            Self::QueueInUse => ErrorCondition::new(ErrorCode::QueueInUse),
            Self::UnsupportedSource { .. } => {
                ErrorCondition::new(ErrorCode::ParseUnsupportedSource)
            }
            Self::InvalidInput { .. } => ErrorCondition::new(ErrorCode::InputInvalid),
            Self::ParseCancelled { .. } => ErrorCondition::new(ErrorCode::ParseCancelled),
            Self::BpiTransport { source, .. } if source.is_timeout() => {
                ErrorCondition::new(ErrorCode::NetworkTimeout)
            }
            Self::BpiTransport { .. } => ErrorCondition::new(ErrorCode::NetworkRequestFailed),
            Self::Bpi(_) => ErrorCondition::new(ErrorCode::ParseUpstreamFailed),
            Self::BpiHttp { status, .. } => ErrorCondition::new(match status {
                401 => ErrorCode::AccountLoginRequired,
                403 => ErrorCode::SourceForbidden,
                412 => ErrorCode::SourceVerificationRequired,
                429 => ErrorCode::SourceRateLimited,
                _ => ErrorCode::SourceHttpFailed,
            }),
            Self::Planning { .. } => ErrorCondition::new(ErrorCode::DownloadPlanFailed),
            Self::Fetch { .. } => ErrorCondition::new(ErrorCode::DownloadFetchFailed),
            Self::Account { .. } => ErrorCondition::new(ErrorCode::AccountFailed),
            Self::Platform { .. } => ErrorCondition::new(ErrorCode::PlatformFailed),
            Self::Storage(_) => ErrorCondition::new(ErrorCode::StorageDatabaseFailed),
            Self::Serialization(_) => ErrorCondition::new(ErrorCode::StorageInvalidData),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicError {
    #[serde(flatten)]
    pub condition: ErrorCondition,
    #[serde(with = "uuid::serde::simple")]
    pub trace_id: uuid::Uuid,
}

impl From<ErrorCode> for PublicError {
    fn from(code: ErrorCode) -> Self {
        Self::new(ErrorCondition::new(code))
    }
}

impl PublicError {
    pub fn new(condition: ErrorCondition) -> Self {
        Self {
            condition,
            trace_id: uuid::Uuid::new_v4(),
        }
    }
}

#[derive(Deserialize)]
pub struct LocalizedText {
    pub en: String,
    #[serde(rename = "zh-CN")]
    pub zh_cn: String,
}

impl LocalizedText {
    pub fn select(&self, chinese: bool) -> &str {
        if chinese { &self.zh_cn } else { &self.en }
    }
}

#[derive(Deserialize)]
struct Entry {
    message: Option<LocalizedText>,
    advice: Option<LocalizedText>,
}

fn catalog() -> &'static BTreeMap<String, Entry> {
    static CATALOG: OnceLock<BTreeMap<String, Entry>> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(include_str!("error-catalog.json"))
            .expect("build-checked error catalog")
    })
}

impl ErrorCondition {
    pub fn message(&self, chinese: bool) -> String {
        let text = catalog()[self.code.as_str()]
            .message
            .as_ref()
            .expect("registered error")
            .select(chinese);
        match &self.params {
            Some(ErrorParams::Cooldown {
                retry_after_seconds,
            }) => text.replace("{retryAfterSeconds}", &retry_after_seconds.to_string()),
            Some(ErrorParams::MissingArguments { arguments }) => {
                text.replace("{arguments}", &arguments.join(", "))
            }
            None => text.to_owned(),
        }
    }
}

/// Return the closest registered solution. The original error code stays intact.
pub fn find_solution(mut code: &str) -> Option<(&'static str, &'static LocalizedText)> {
    if !code.split('.').all(|s| {
        !s.is_empty()
            && s.bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_')
    }) {
        return None;
    }
    loop {
        if let Some((key, entry)) = catalog().get_key_value(code)
            && let Some(advice) = &entry.advice
        {
            return Some((key, advice));
        }
        code = code.rsplit_once('.')?.0;
    }
}
