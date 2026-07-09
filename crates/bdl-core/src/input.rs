use url::Url;

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
            Self::VideoBvid(_) | Self::VideoAid(_) => SourceKind::Video,
            Self::ShortUrl(_) => SourceKind::Unknown,
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

    if let Ok(url) = Url::parse(trimmed) {
        return classify_url(trimmed, &url).ok_or_else(unrecognized);
    }

    if let Some(bvid) = extract_bvid(trimmed) {
        return Ok(ClassifiedInput::VideoBvid(bvid));
    }

    if let Some(aid) = extract_aid(trimmed) {
        return Ok(ClassifiedInput::VideoAid(aid));
    }

    Err(unrecognized())
}

fn classify_url(raw_url: &str, url: &Url) -> Option<ClassifiedInput> {
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }

    let host = url.host_str()?.to_ascii_lowercase();
    if is_b23_host(&host) {
        return Some(ClassifiedInput::ShortUrl(raw_url.to_string()));
    }

    if !is_bilibili_host(&host) {
        return None;
    }

    let segments = path_segments(url);

    if let Some(video) = classify_video_path(&segments) {
        return Some(video);
    }

    if is_space_host(&host) {
        if is_favorite_path(&segments) {
            return Some(ClassifiedInput::Favorite {
                raw_url: raw_url.to_string(),
            });
        }

        if is_uploader_path(&segments) {
            if let Some(mid) = space_mid(&segments) {
                return Some(ClassifiedInput::Uploader { mid });
            }
        }
    }

    if path_starts_with(&segments, &["bangumi", "play"]) {
        return Some(ClassifiedInput::Bangumi {
            raw_url: raw_url.to_string(),
        });
    }

    if path_starts_with(&segments, &["cheese", "play"]) {
        return Some(ClassifiedInput::Cheese {
            raw_url: raw_url.to_string(),
        });
    }

    if path_starts_with(&segments, &["medialist", "play"]) || has_query_param(url, "season_id") {
        return Some(ClassifiedInput::Collection {
            raw_url: raw_url.to_string(),
        });
    }

    if segments
        .iter()
        .any(|segment| segment.eq_ignore_ascii_case("series"))
        || has_query_param(url, "series_id")
    {
        return Some(ClassifiedInput::Series {
            raw_url: raw_url.to_string(),
        });
    }

    None
}

fn classify_video_path(segments: &[&str]) -> Option<ClassifiedInput> {
    if !path_starts_with(segments, &["video"]) {
        return None;
    }

    let video_id = segments.get(1)?;
    if let Some(bvid) = parse_bvid_token(video_id) {
        return Some(ClassifiedInput::VideoBvid(bvid.to_owned()));
    }

    parse_aid_token(video_id).map(ClassifiedInput::VideoAid)
}

fn extract_bvid(input: &str) -> Option<String> {
    parse_bvid_token(input).map(ToOwned::to_owned)
}

fn extract_aid(input: &str) -> Option<u64> {
    parse_aid_token(input)
}

fn parse_bvid_token(token: &str) -> Option<&str> {
    let suffix = token.strip_prefix("BV")?;
    (suffix.len() == 10 && suffix.chars().all(|ch| ch.is_ascii_alphanumeric())).then_some(token)
}

fn parse_aid_token(token: &str) -> Option<u64> {
    let lower = token.to_ascii_lowercase();
    let digits = lower.strip_prefix("av")?;
    (!digits.is_empty() && digits.chars().all(|ch| ch.is_ascii_digit()))
        .then(|| digits.parse::<u64>().ok())
        .flatten()
}

fn path_segments(url: &Url) -> Vec<&str> {
    url.path_segments()
        .map(|segments| segments.filter(|segment| !segment.is_empty()).collect())
        .unwrap_or_default()
}

fn path_starts_with(segments: &[&str], prefix: &[&str]) -> bool {
    segments
        .iter()
        .zip(prefix.iter())
        .all(|(segment, expected)| segment.eq_ignore_ascii_case(expected))
        && segments.len() >= prefix.len()
}

fn is_favorite_path(segments: &[&str]) -> bool {
    segments
        .get(1)
        .is_some_and(|segment| segment.eq_ignore_ascii_case("favlist"))
}

fn is_uploader_path(segments: &[&str]) -> bool {
    segments.len() == 1
        || segments
            .get(1)
            .is_some_and(|segment| segment.eq_ignore_ascii_case("video"))
}

fn space_mid(segments: &[&str]) -> Option<u64> {
    segments.first()?.parse::<u64>().ok()
}

fn has_query_param(url: &Url, key: &str) -> bool {
    url.query_pairs().any(|(name, _)| name == key)
}

fn is_b23_host(host: &str) -> bool {
    host == "b23.tv" || host.ends_with(".b23.tv")
}

fn is_bilibili_host(host: &str) -> bool {
    host == "bilibili.com" || host.ends_with(".bilibili.com")
}

fn is_space_host(host: &str) -> bool {
    host == "space.bilibili.com"
}

fn unrecognized() -> BdlError {
    BdlError::InvalidInput {
        message: "无法识别这个输入。请粘贴 BV/AV、视频、番剧、课程、收藏夹、合集或 UP 主空间链接。"
            .into(),
    }
}
