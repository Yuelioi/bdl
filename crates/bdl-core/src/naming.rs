use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{BdlError, BdlResult};

pub const DEFAULT_NAMING_TEMPLATE: &str = "{title}/P{part_index} - {part_title}";
pub const MAX_FILE_NAME_BYTES: usize = 255;

pub(crate) fn utf8_prefix(value: &str, max_bytes: usize) -> &str {
    let mut end = value.len().min(max_bytes);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    &value[..end]
}

/// Output reservations use conservative case-insensitive keys on Windows,
/// including before the output files exist. Other platforms retain path case.
#[derive(Default)]
pub struct ReservedPaths(HashSet<PathBuf>);

impl ReservedPaths {
    pub fn new() -> Self {
        Self::default()
    }
    fn key(path: &Path) -> PathBuf {
        let path: PathBuf = path
            .components()
            .filter(|part| !matches!(part, std::path::Component::CurDir))
            .collect();
        #[cfg(windows)]
        {
            PathBuf::from(path.to_string_lossy().to_uppercase())
        }
        #[cfg(not(windows))]
        {
            path
        }
    }
    pub fn contains(&self, path: &Path) -> bool {
        self.0.contains(&Self::key(path))
    }
    pub fn insert(&mut self, path: PathBuf) -> bool {
        self.0.insert(Self::key(&path))
    }
    pub fn remove(&mut self, path: &Path) -> bool {
        self.0.remove(&Self::key(path))
    }
}

impl Extend<PathBuf> for ReservedPaths {
    fn extend<T: IntoIterator<Item = PathBuf>>(&mut self, paths: T) {
        for path in paths {
            self.insert(path);
        }
    }
}

impl FromIterator<PathBuf> for ReservedPaths {
    fn from_iter<T: IntoIterator<Item = PathBuf>>(paths: T) -> Self {
        let mut reserved = Self::new();
        reserved.extend(paths);
        reserved
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DuplicateNamingStrategy {
    #[default]
    SkipExisting,
    AppendSuffix,
    OverwriteExisting,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NamingContext<'a> {
    pub title: &'a str,
    pub part_title: &'a str,
    pub part_index: usize,
    pub bvid: Option<&'a str>,
    pub aid: Option<u64>,
    pub cid: Option<u64>,
    pub owner_name: Option<&'a str>,
    pub owner_mid: Option<u64>,
    pub series_title: Option<&'a str>,
    pub season_index: Option<usize>,
    pub episode_index: Option<usize>,
    pub collection_title: Option<&'a str>,
    pub index: Option<usize>,
    pub quality: Option<&'a str>,
    pub codec: Option<&'a str>,
    pub date: Option<&'a str>,
    pub publish_date: Option<&'a str>,
    pub ext: &'a str,
}

pub fn render_output_path(template: &str, context: &NamingContext<'_>) -> BdlResult<PathBuf> {
    let rendered = render_template(template, context)?;
    let mut path = PathBuf::new();

    for component in rendered
        .split(['/', '\\'])
        .map(sanitize_path_component)
        .filter(|component| !component.is_empty())
    {
        path.push(component);
    }

    if path.as_os_str().is_empty() {
        return Err(BdlError::Planning {
            message: "命名模板渲染结果为空。".to_owned(),
        });
    }

    let extension = sanitize_variable_value(context.ext);
    if extension.len() + 1 >= MAX_FILE_NAME_BYTES {
        return Err(BdlError::Planning {
            message: "文件扩展名过长。".into(),
        });
    }
    if !extension.is_empty()
        && !path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case(&extension))
    {
        // Append rather than replace: dots may be part of the custom name.
        let mut file_name = path.file_name().unwrap_or_default().to_os_string();
        file_name.push(".");
        file_name.push(extension);
        path.set_file_name(file_name);
    }

    path = with_counter_suffix(&path, 0, MAX_FILE_NAME_BYTES);

    Ok(path)
}

pub fn validate_template(template: &str) -> BdlResult<()> {
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '}' {
            return Err(BdlError::Planning {
                message: "命名模板存在多余的 `}`。".to_owned(),
            });
        }
        if ch != '{' {
            continue;
        }

        let mut name = String::new();
        let mut closed = false;
        for next in chars.by_ref() {
            if next == '}' {
                closed = true;
                break;
            }
            name.push(next);
        }

        if !closed {
            return Err(BdlError::Planning {
                message: "命名模板存在未闭合变量。".to_owned(),
            });
        }

        validate_variable_name(name.trim())?;
    }

    Ok(())
}

pub fn unique_path(path: PathBuf, reserved: &mut ReservedPaths) -> PathBuf {
    unique_path_with_stem_limit(path, reserved, MAX_FILE_NAME_BYTES)
}

pub fn unique_path_with_stem_limit(
    path: PathBuf,
    reserved: &mut ReservedPaths,
    max_stem_bytes: usize,
) -> PathBuf {
    let mut candidate = with_counter_suffix(&path, 0, max_stem_bytes);
    let mut suffix = 1;

    while candidate.exists() || reserved.contains(&candidate) {
        candidate = with_counter_suffix(&path, suffix, max_stem_bytes);
        suffix += 1;
    }

    reserved.insert(candidate.clone());
    candidate
}

pub fn resolve_duplicate_path(
    path: PathBuf,
    reserved: &mut ReservedPaths,
    strategy: DuplicateNamingStrategy,
) -> Option<PathBuf> {
    resolve_duplicate_path_with_stem_limit(path, reserved, strategy, MAX_FILE_NAME_BYTES)
}

pub fn resolve_duplicate_path_with_stem_limit(
    path: PathBuf,
    reserved: &mut ReservedPaths,
    strategy: DuplicateNamingStrategy,
    max_stem_bytes: usize,
) -> Option<PathBuf> {
    match strategy {
        DuplicateNamingStrategy::SkipExisting if path.exists() => None,
        DuplicateNamingStrategy::SkipExisting | DuplicateNamingStrategy::OverwriteExisting => {
            Some(if reserved.contains(&path) {
                unique_path_with_stem_limit(path, reserved, max_stem_bytes)
            } else {
                reserved.insert(path.clone());
                path
            })
        }
        DuplicateNamingStrategy::AppendSuffix => {
            Some(unique_path_with_stem_limit(path, reserved, max_stem_bytes))
        }
    }
}

pub fn sanitize_path_component(value: &str) -> String {
    let trimmed = sanitize_variable_value(value);
    let trimmed = utf8_prefix(&trimmed, MAX_FILE_NAME_BYTES).trim_end_matches([' ', '.']);

    if trimmed.is_empty() {
        "untitled".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn sanitize_variable_value(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*')
            {
                '_'
            } else {
                ch
            }
        })
        .collect();
    sanitized.trim().trim_matches('.').to_owned()
}

fn render_template(template: &str, context: &NamingContext<'_>) -> BdlResult<String> {
    let mut rendered = String::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '{' {
            rendered.push(ch);
            continue;
        }

        let mut name = String::new();
        let mut closed = false;
        for next in chars.by_ref() {
            if next == '}' {
                closed = true;
                break;
            }
            name.push(next);
        }

        if !closed {
            return Err(BdlError::Planning {
                message: "命名模板存在未闭合变量。".to_owned(),
            });
        }

        rendered.push_str(&sanitize_variable_value(&variable_value(
            name.trim(),
            context,
        )?));
    }

    Ok(rendered)
}

fn variable_value(name: &str, context: &NamingContext<'_>) -> BdlResult<String> {
    validate_variable_name(name)?;
    if let Some((field, format)) = name.split_once(':') {
        let raw = if field == "date" {
            context.date
        } else {
            context.publish_date
        };
        return Ok(raw
            .and_then(|v| chrono::NaiveDate::parse_from_str(v, "%Y-%m-%d").ok())
            .map(|v| v.format(format).to_string())
            .unwrap_or_default());
    }
    let value = match name {
        "title" => context.title.to_owned(),
        "part_title" => context.part_title.to_owned(),
        "part_index" => context.part_index.to_string(),
        "bvid" => context.bvid.unwrap_or_default().to_owned(),
        "aid" => optional_u64(context.aid),
        "cid" => optional_u64(context.cid),
        "owner_name" => context.owner_name.unwrap_or_default().to_owned(),
        "owner_mid" => optional_u64(context.owner_mid),
        "series_title" => context.series_title.unwrap_or_default().to_owned(),
        "season_index" => optional_usize(context.season_index),
        "episode_index" => optional_usize(context.episode_index),
        "episode_title" => context.part_title.to_owned(),
        "collection_title" => context.collection_title.unwrap_or_default().to_owned(),
        "index" => optional_usize(context.index),
        "quality" => context.quality.unwrap_or_default().to_owned(),
        "quality_label" => quality_label(context.quality.unwrap_or_default()).to_owned(),
        "codec" => context.codec.unwrap_or_default().to_owned(),
        "date" => context.date.unwrap_or_default().to_owned(),
        "publish_date" => context.publish_date.unwrap_or_default().to_owned(),
        "ext" => context.ext.to_owned(),
        _ => unreachable!("naming variable was validated before rendering"),
    };

    Ok(value)
}

fn validate_variable_name(name: &str) -> BdlResult<()> {
    if let Some((field, format)) = name.split_once(':')
        && matches!(field, "date" | "publish_date")
        && matches!(format, "%Y-%m-%d" | "%Y.%m.%d" | "%Y%m%d")
    {
        return Ok(());
    }
    match name {
        "title" | "part_title" | "part_index" | "bvid" | "aid" | "cid" | "owner_name"
        | "owner_mid" | "series_title" | "season_index" | "episode_index" | "episode_title"
        | "collection_title" | "index" | "quality" | "quality_label" | "codec" | "date"
        | "publish_date" | "ext" => Ok(()),
        "" => Err(BdlError::Planning {
            message: "命名模板存在空变量。".to_owned(),
        }),
        other => Err(BdlError::Planning {
            message: format!("命名模板存在未知变量 `{other}`。"),
        }),
    }
}

fn optional_u64(value: Option<u64>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

fn optional_usize(value: Option<usize>) -> String {
    value.map(|value| value.to_string()).unwrap_or_default()
}

pub(crate) fn with_counter_suffix(path: &Path, suffix: usize, max_stem_bytes: usize) -> PathBuf {
    let parent = path.parent();
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("untitled");
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty())
        .map(|value| format!(".{value}"))
        .unwrap_or_default();
    let counter = if suffix == 0 {
        String::new()
    } else {
        format!(" ({suffix})")
    };
    let budget = max_stem_bytes.min(MAX_FILE_NAME_BYTES.saturating_sub(extension.len()));
    let stem = utf8_prefix(stem, budget.saturating_sub(counter.len()));
    let file_name = format!("{stem}{counter}{extension}");

    parent
        .map(|parent| parent.join(&file_name))
        .unwrap_or_else(|| PathBuf::from(file_name))
}

/// Bilibili quality tiers, not inferred pixel dimensions.
pub fn quality_label(value: &str) -> &str {
    match value {
        "6" => "240P",
        "16" => "360P",
        "32" => "480P",
        "64" => "720P",
        "74" => "720P60",
        "80" => "1080P",
        "112" => "1080P+",
        "116" => "1080P60",
        "120" => "4K",
        "125" => "HDR",
        "126" => "Dolby Vision",
        "127" => "8K",
        other => other,
    }
}
