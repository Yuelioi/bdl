use std::collections::HashSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{BdlError, BdlResult};

pub const DEFAULT_NAMING_TEMPLATE: &str = "{title}/P{part_index} - {part_title}.{ext}";

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

pub fn unique_path(path: PathBuf, reserved: &mut HashSet<PathBuf>) -> PathBuf {
    let mut candidate = path.clone();
    let mut suffix = 1;

    while candidate.exists() || reserved.contains(&candidate) {
        candidate = with_counter_suffix(&path, suffix);
        suffix += 1;
    }

    reserved.insert(candidate.clone());
    candidate
}

pub fn resolve_duplicate_path(
    path: PathBuf,
    reserved: &mut HashSet<PathBuf>,
    strategy: DuplicateNamingStrategy,
) -> Option<PathBuf> {
    match strategy {
        DuplicateNamingStrategy::SkipExisting if path.exists() => None,
        DuplicateNamingStrategy::SkipExisting | DuplicateNamingStrategy::OverwriteExisting => {
            Some(overwrite_existing_path(path, reserved))
        }
        DuplicateNamingStrategy::AppendSuffix => Some(unique_path(path, reserved)),
    }
}

pub fn sanitize_path_component(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|ch| {
            if ch.is_control() || matches!(ch, '<' | '>' | ':' | '"' | '|' | '?' | '*') {
                '_'
            } else {
                ch
            }
        })
        .collect();
    let trimmed = sanitized.trim().trim_matches('.').to_owned();

    if trimmed.is_empty() {
        "untitled".to_owned()
    } else {
        trimmed
    }
}

fn overwrite_existing_path(path: PathBuf, reserved: &mut HashSet<PathBuf>) -> PathBuf {
    if reserved.contains(&path) {
        return unique_path(path, reserved);
    }

    reserved.insert(path.clone());
    path
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

        rendered.push_str(&variable_value(name.trim(), context)?);
    }

    Ok(rendered)
}

fn variable_value(name: &str, context: &NamingContext<'_>) -> BdlResult<String> {
    validate_variable_name(name)?;
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
        "codec" => context.codec.unwrap_or_default().to_owned(),
        "date" => context.date.unwrap_or_default().to_owned(),
        "ext" => context.ext.to_owned(),
        _ => unreachable!("naming variable was validated before rendering"),
    };

    Ok(value)
}

fn validate_variable_name(name: &str) -> BdlResult<()> {
    match name {
        "title" | "part_title" | "part_index" | "bvid" | "aid" | "cid" | "owner_name"
        | "owner_mid" | "series_title" | "season_index" | "episode_index" | "episode_title"
        | "collection_title" | "index" | "quality" | "codec" | "date" | "ext" => Ok(()),
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

fn with_counter_suffix(path: &Path, suffix: usize) -> PathBuf {
    let parent = path.parent();
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("untitled");
    let extension = path.extension().and_then(|value| value.to_str());
    let file_name = match extension {
        Some(extension) if !extension.is_empty() => format!("{stem} ({suffix}).{extension}"),
        _ => format!("{stem} ({suffix})"),
    };

    parent
        .map(|parent| parent.join(&file_name))
        .unwrap_or_else(|| PathBuf::from(file_name))
}
