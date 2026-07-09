use crate::error::{BdlError, BdlResult};
use crate::model::SourceKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassifiedInput {
    VideoBvid(String),
    VideoAid(u64),
    ShortUrl(String),
    Uploader { mid: u64 },
    Favorite { raw_url: String },
    Collection { raw_url: String },
    Series { raw_url: String },
    Bangumi { raw_url: String },
    Cheese { raw_url: String },
}

impl ClassifiedInput {
    pub fn source_kind(&self) -> SourceKind {
        match self {
            Self::VideoBvid(_) | Self::VideoAid(_) | Self::ShortUrl(_) => SourceKind::Video,
            Self::Uploader { .. } => SourceKind::Uploader,
            Self::Favorite { .. } => SourceKind::Favorite,
            Self::Collection { .. } => SourceKind::Collection,
            Self::Series { .. } => SourceKind::Series,
            Self::Bangumi { .. } => SourceKind::Bangumi,
            Self::Cheese { .. } => SourceKind::Cheese,
        }
    }
}

pub fn classify_input(input: &str) -> BdlResult<ClassifiedInput> {
    let trimmed = input.trim();

    if let Some(bvid) = extract_bvid(trimmed) {
        return Ok(ClassifiedInput::VideoBvid(bvid));
    }

    if let Some(aid) = extract_aid(trimmed) {
        return Ok(ClassifiedInput::VideoAid(aid));
    }

    if trimmed.starts_with("https://b23.tv/") || trimmed.starts_with("http://b23.tv/") {
        return Ok(ClassifiedInput::ShortUrl(trimmed.to_string()));
    }

    if trimmed.contains("space.bilibili.com/") && trimmed.contains("favlist") {
        return Ok(ClassifiedInput::Favorite {
            raw_url: trimmed.to_string(),
        });
    }

    if trimmed.contains("space.bilibili.com/") && trimmed.contains("/video") {
        let mid = trimmed
            .split("space.bilibili.com/")
            .nth(1)
            .and_then(|tail| tail.split(['/', '?']).next())
            .and_then(|value| value.parse::<u64>().ok())
            .ok_or_else(unrecognized)?;
        return Ok(ClassifiedInput::Uploader { mid });
    }

    if trimmed.contains("/bangumi/play/") {
        return Ok(ClassifiedInput::Bangumi {
            raw_url: trimmed.to_string(),
        });
    }

    if trimmed.contains("/cheese/play/") {
        return Ok(ClassifiedInput::Cheese {
            raw_url: trimmed.to_string(),
        });
    }

    if trimmed.contains("/medialist/play/") || trimmed.contains("season_id=") {
        return Ok(ClassifiedInput::Collection {
            raw_url: trimmed.to_string(),
        });
    }

    if trimmed.contains("/series/") || trimmed.contains("series_id=") {
        return Ok(ClassifiedInput::Series {
            raw_url: trimmed.to_string(),
        });
    }

    Err(unrecognized())
}

fn extract_bvid(input: &str) -> Option<String> {
    input
        .split(|ch: char| !(ch.is_ascii_alphanumeric()))
        .find(|part| part.starts_with("BV") && part.len() >= 10)
        .map(ToOwned::to_owned)
}

fn extract_aid(input: &str) -> Option<u64> {
    let lower = input.to_ascii_lowercase();
    lower
        .strip_prefix("av")
        .and_then(|digits| digits.parse::<u64>().ok())
}

fn unrecognized() -> BdlError {
    BdlError::InvalidInput {
        message: "无法识别这个输入。请粘贴 BV/AV、视频、番剧、课程、收藏夹、合集或 UP 主空间链接。"
            .into(),
    }
}
