use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{BpiError, BpiResult};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldValueKind {
    Object,
    Array,
    String,
    Number,
    Bool,
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldStat {
    pub path: String,
    pub count: usize,
    pub value_kinds: BTreeSet<FieldValueKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FieldAuditReport {
    pub files: usize,
    pub fields: Vec<FieldStat>,
}

pub fn audit_contract_fields(root: impl AsRef<Path>) -> BpiResult<FieldAuditReport> {
    let mut stats = BTreeMap::<String, FieldStat>::new();
    let mut files = 0;

    for path in json_files(root.as_ref())? {
        let bytes = fs::read(&path)
            .map_err(|err| BpiError::parse(format!("failed to read {}: {err}", path.display())))?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .map_err(|err| BpiError::parse(format!("failed to parse {}: {err}", path.display())))?;

        files += 1;
        collect_fields(&value, "", &mut stats);
    }

    let mut fields = stats.into_values().collect::<Vec<_>>();
    fields.sort_by(|left, right| {
        right
            .count
            .cmp(&left.count)
            .then_with(|| left.path.cmp(&right.path))
    });

    Ok(FieldAuditReport { files, fields })
}

fn json_files(root: &Path) -> BpiResult<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_json_files(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_json_files(path: &Path, files: &mut Vec<PathBuf>) -> BpiResult<()> {
    if path.is_file() {
        if path.extension().and_then(|value| value.to_str()) == Some("json") {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }

    if !path.is_dir() {
        return Err(BpiError::invalid_parameter(
            "path",
            "field audit path must be a JSON file or directory",
        ));
    }

    let entries = fs::read_dir(path)
        .map_err(|err| BpiError::parse(format!("failed to read {}: {err}", path.display())))?;
    for entry in entries {
        let entry = entry.map_err(|err| BpiError::parse(err.to_string()))?;
        collect_json_files(&entry.path(), files)?;
    }

    Ok(())
}

fn collect_fields(value: &serde_json::Value, path: &str, stats: &mut BTreeMap<String, FieldStat>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                let child_path = if path.is_empty() {
                    format!("$.{key}")
                } else {
                    format!("{path}.{key}")
                };
                record_field(&child_path, child, stats);
                collect_fields(child, &child_path, stats);
            }
        }
        serde_json::Value::Array(items) => {
            let child_path = format!("{path}[]");
            record_kind(&child_path, FieldValueKind::Array, stats);
            for item in items {
                collect_fields(item, &child_path, stats);
            }
        }
        _ => {}
    }
}

fn record_field(path: &str, value: &serde_json::Value, stats: &mut BTreeMap<String, FieldStat>) {
    record_kind(path, value_kind(value), stats);
}

fn record_kind(path: &str, kind: FieldValueKind, stats: &mut BTreeMap<String, FieldStat>) {
    let stat = stats.entry(path.to_string()).or_insert_with(|| FieldStat {
        path: path.to_string(),
        count: 0,
        value_kinds: BTreeSet::new(),
    });
    stat.count += 1;
    stat.value_kinds.insert(kind);
}

fn value_kind(value: &serde_json::Value) -> FieldValueKind {
    match value {
        serde_json::Value::Object(_) => FieldValueKind::Object,
        serde_json::Value::Array(_) => FieldValueKind::Array,
        serde_json::Value::String(_) => FieldValueKind::String,
        serde_json::Value::Number(_) => FieldValueKind::Number,
        serde_json::Value::Bool(_) => FieldValueKind::Bool,
        serde_json::Value::Null => FieldValueKind::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn field_audit_collapses_array_indices() {
        let value = serde_json::json!({
            "data": {
                "list": [
                    { "owner": { "mid": 1, "name": "a" } },
                    { "owner": { "mid": 2, "name": "b" } }
                ]
            }
        });
        let mut stats = BTreeMap::new();

        collect_fields(&value, "", &mut stats);

        assert_eq!(stats["$.data.list[].owner.mid"].count, 2);
        assert_eq!(stats["$.data.list[].owner.name"].count, 2);
        assert!(
            stats["$.data.list[].owner.mid"]
                .value_kinds
                .contains(&FieldValueKind::Number)
        );
    }
}
