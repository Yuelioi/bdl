use serde::{Deserialize, Serialize};

use crate::probe::endpoint_contract::ApiRisk;

pub const SANITIZED_MID: u64 = 1_000_001;
pub const SANITIZED_USER: &str = "sanitized-user";
pub const REDACTED: &str = "<redacted>";
pub const SANITIZED_AVATAR_URL: &str = "https://example.invalid/avatar.png";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SanitizeFinding {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SanitizeAction {
    ReplaceString(String),
    ReplaceNumber(u64),
    Drop,
    Keep,
}

pub fn sanitize_value(value: &mut serde_json::Value, risk: Option<ApiRisk>) {
    sanitize_at_path(value, "", risk);
}

pub fn audit_value(value: &serde_json::Value, risk: Option<ApiRisk>) -> Vec<SanitizeFinding> {
    let mut findings = Vec::new();
    audit_at_path(value, "", risk, &mut findings);
    findings
}

fn sanitize_at_path(value: &mut serde_json::Value, path: &str, risk: Option<ApiRisk>) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map.iter_mut() {
                let child_path = child_path(path, key);
                let normalized = normalize_key(key);
                if should_sanitize_account_fields(risk)
                    && let Some(action) = path_sanitize_action(&child_path)
                {
                    apply_action(child, action);
                    continue;
                }
                if always_redact_key(&normalized) {
                    *child = serde_json::Value::String(REDACTED.to_string());
                    continue;
                }
                if account_id_key(&normalized) && should_sanitize_account_fields(risk) {
                    *child = serde_json::Value::Number(SANITIZED_MID.into());
                    continue;
                }
                if account_name_key(&normalized) && should_sanitize_account_fields(risk) {
                    *child = serde_json::Value::String(SANITIZED_USER.to_string());
                    continue;
                }
                if avatar_key(&normalized) && should_sanitize_account_fields(risk) {
                    *child = serde_json::Value::String(SANITIZED_AVATAR_URL.to_string());
                    continue;
                }

                sanitize_at_path(child, &child_path, risk);
            }
        }
        serde_json::Value::Array(items) => {
            let item_path = format!("{path}[]");
            for item in items {
                sanitize_at_path(item, &item_path, risk);
            }
        }
        _ => {}
    }
}

fn audit_at_path(
    value: &serde_json::Value,
    path: &str,
    risk: Option<ApiRisk>,
    findings: &mut Vec<SanitizeFinding>,
) {
    match value {
        serde_json::Value::Object(map) => {
            for (key, child) in map {
                let child_path = child_path(path, key);
                let normalized = normalize_key(key);
                if always_redact_key(&normalized) && !is_redacted(child) {
                    findings.push(SanitizeFinding {
                        path: child_path.clone(),
                        reason: "敏感认证字段未脱敏".to_string(),
                    });
                } else if should_sanitize_account_fields(risk)
                    && let Some(action) = path_sanitize_action(&child_path)
                    && !is_sanitized_for_action(child, &action)
                {
                    findings.push(SanitizeFinding {
                        path: child_path.clone(),
                        reason: "路径覆盖字段未脱敏".to_string(),
                    });
                } else if should_sanitize_account_fields(risk)
                    && (account_id_key(&normalized)
                        || account_name_key(&normalized)
                        || avatar_key(&normalized))
                    && !is_sanitized_account_value(child)
                {
                    findings.push(SanitizeFinding {
                        path: child_path.clone(),
                        reason: "账号可识别字段未脱敏".to_string(),
                    });
                }

                audit_at_path(child, &child_path, risk, findings);
            }
        }
        serde_json::Value::Array(items) => {
            let item_path = format!("{path}[]");
            for item in items {
                audit_at_path(item, &item_path, risk, findings);
            }
        }
        _ => audit_scalar_value(value, path, findings),
    }
}

fn audit_scalar_value(value: &serde_json::Value, path: &str, findings: &mut Vec<SanitizeFinding>) {
    let Some(value) = value.as_str() else {
        return;
    };
    let lower = value.to_ascii_lowercase();
    if lower.contains("sessdata=")
        || lower.contains("bili_jct=")
        || lower.contains("dedeuserid=")
        || lower.contains("buvid3=")
        || looks_like_email(value)
    {
        findings.push(SanitizeFinding {
            path: path.to_string(),
            reason: "字符串值疑似包含账号凭据或联系方式".to_string(),
        });
    }
}

fn child_path(path: &str, key: &str) -> String {
    if path.is_empty() {
        format!("$.{key}")
    } else {
        format!("{path}.{key}")
    }
}

fn normalize_key(key: &str) -> String {
    key.chars()
        .filter(|ch| *ch != '_' && *ch != '-')
        .flat_map(char::to_lowercase)
        .collect()
}

fn always_redact_key(key: &str) -> bool {
    matches!(
        key,
        "sessdata"
            | "bilijct"
            | "csrf"
            | "cookie"
            | "authorization"
            | "setcookie"
            | "buvid3"
            | "qrcodekey"
    )
}

fn path_sanitize_action(path: &str) -> Option<SanitizeAction> {
    match path {
        "$.data.mid"
        | "$.data.owner.mid"
        | "$.data.list[].owner.mid"
        | "$.data.cooperators[].mid" => Some(SanitizeAction::ReplaceNumber(SANITIZED_MID)),
        "$.data.name" | "$.data.owner.name" | "$.data.list[].owner.name" => {
            Some(SanitizeAction::ReplaceString(SANITIZED_USER.to_string()))
        }
        "$.data.cooperators[].nick_name" => {
            Some(SanitizeAction::ReplaceString(SANITIZED_USER.to_string()))
        }
        "$.data.list[].owner.face" => Some(SanitizeAction::ReplaceString(
            SANITIZED_AVATAR_URL.to_string(),
        )),
        _ => None,
    }
}

fn apply_action(value: &mut serde_json::Value, action: SanitizeAction) {
    match action {
        SanitizeAction::ReplaceString(replacement) => {
            *value = serde_json::Value::String(replacement);
        }
        SanitizeAction::ReplaceNumber(replacement) => {
            *value = serde_json::Value::Number(replacement.into());
        }
        SanitizeAction::Drop => {
            *value = serde_json::Value::Null;
        }
        SanitizeAction::Keep => {}
    }
}

fn is_sanitized_for_action(value: &serde_json::Value, action: &SanitizeAction) -> bool {
    match action {
        SanitizeAction::ReplaceString(replacement) => value.as_str() == Some(replacement.as_str()),
        SanitizeAction::ReplaceNumber(replacement) => value.as_u64() == Some(*replacement),
        SanitizeAction::Drop => value.is_null(),
        SanitizeAction::Keep => true,
    }
}

fn account_id_key(key: &str) -> bool {
    matches!(
        key,
        "mid" | "uid" | "ownermid" | "authoruid" | "memberid" | "targetid"
    )
}

fn account_name_key(key: &str) -> bool {
    matches!(
        key,
        "uname" | "name" | "nickname" | "nick" | "authorname" | "ownername"
    )
}

fn avatar_key(key: &str) -> bool {
    matches!(key, "face" | "avatar" | "authorface" | "ownerface")
}

fn should_sanitize_account_fields(risk: Option<ApiRisk>) -> bool {
    !matches!(risk, Some(ApiRisk::PublicRead))
}

fn is_redacted(value: &serde_json::Value) -> bool {
    value.as_str() == Some(REDACTED)
        || value
            .as_str()
            .is_some_and(|value| value.starts_with("sanitized-"))
}

fn is_sanitized_account_value(value: &serde_json::Value) -> bool {
    value.as_u64() == Some(SANITIZED_MID)
        || value.as_str() == Some(SANITIZED_USER)
        || value.as_str() == Some(SANITIZED_AVATAR_URL)
        || value.as_str() == Some(REDACTED)
}

fn looks_like_email(value: &str) -> bool {
    if value.contains('/') || value.contains('\\') || value.contains(' ') {
        return false;
    }
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    !local.is_empty() && domain.contains('.') && !domain.ends_with('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_redacts_credentials_and_private_account_fields() {
        let mut value = serde_json::json!({
            "data": {
                "mid": 42,
                "name": "real-user",
                "face": "https://example.com/face.jpg",
                "csrf": "secret",
                "title": "public title"
            }
        });

        sanitize_value(&mut value, Some(ApiRisk::PrivateRead));

        assert_eq!(value["data"]["mid"], SANITIZED_MID);
        assert_eq!(value["data"]["name"], SANITIZED_USER);
        assert_eq!(value["data"]["face"], SANITIZED_AVATAR_URL);
        assert_eq!(value["data"]["csrf"], REDACTED);
        assert_eq!(value["data"]["title"], "public title");
    }

    #[test]
    fn public_read_keeps_account_like_public_fields() {
        let mut value = serde_json::json!({
            "data": {
                "owner": {
                    "mid": 42,
                    "name": "public-up"
                }
            }
        });

        sanitize_value(&mut value, Some(ApiRisk::PublicRead));

        assert_eq!(value["data"]["owner"]["mid"], 42);
        assert_eq!(value["data"]["owner"]["name"], "public-up");
    }

    #[test]
    fn audit_reports_unsanitized_private_fields() {
        let value = serde_json::json!({
            "data": {
                "mid": 42,
                "name": "real-user",
                "cookie": "SESSDATA=secret"
            }
        });

        let findings = audit_value(&value, Some(ApiRisk::PrivateRead));

        assert!(findings.iter().any(|finding| finding.path == "$.data.mid"));
        assert!(
            findings
                .iter()
                .any(|finding| finding.path == "$.data.cookie")
        );
    }

    #[test]
    fn sanitize_applies_private_path_overrides() {
        let mut value = serde_json::json!({
            "data": {
                "list": [
                    {
                        "owner": {
                            "mid": 42,
                            "name": "real-user",
                            "face": "https://example.com/face.jpg"
                        }
                    }
                ],
                "cooperators": [
                    {
                        "mid": 43,
                        "nick_name": "real-cooperator"
                    }
                ]
            }
        });

        sanitize_value(&mut value, Some(ApiRisk::PrivateRead));

        assert_eq!(value["data"]["list"][0]["owner"]["mid"], SANITIZED_MID);
        assert_eq!(value["data"]["list"][0]["owner"]["name"], SANITIZED_USER);
        assert_eq!(
            value["data"]["list"][0]["owner"]["face"],
            SANITIZED_AVATAR_URL
        );
        assert_eq!(value["data"]["cooperators"][0]["mid"], SANITIZED_MID);
        assert_eq!(value["data"]["cooperators"][0]["nick_name"], SANITIZED_USER);
    }

    #[test]
    fn audit_reports_unsanitized_path_overrides() {
        let value = serde_json::json!({
            "data": {
                "cooperators": [
                    {
                        "mid": 43,
                        "nick_name": "real-cooperator"
                    }
                ]
            }
        });

        let findings = audit_value(&value, Some(ApiRisk::PrivateRead));

        assert!(
            findings
                .iter()
                .any(|finding| finding.path == "$.data.cooperators[].mid")
        );
        assert!(
            findings
                .iter()
                .any(|finding| finding.path == "$.data.cooperators[].nick_name")
        );
    }
}
