use std::collections::BTreeMap;

use bpi_rs::BpiClient;
use bpi_rs::login::LoginQrPollParams;
use qrcode::QrCode;
use qrcode::render::svg;
use serde::{Deserialize, Serialize};

use crate::error::{BdlError, BdlResult};

const MAX_REDACTED_TEXT_CHARS: usize = 4000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedCookie {
    raw: String,
    pairs: BTreeMap<String, String>,
}

impl ImportedCookie {
    pub fn parse(raw: impl Into<String>) -> BdlResult<Self> {
        let raw = raw.into();
        let pairs = parse_cookie_pairs(&raw);
        require_cookie(&pairs, "SESSDATA")?;
        require_cookie(&pairs, "bili_jct")?;

        Ok(Self { raw, pairs })
    }

    pub fn as_header(&self) -> &str {
        &self.raw
    }

    pub fn dede_user_id(&self) -> Option<&str> {
        self.pairs.get("dedeuserid").map(String::as_str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AccountSummary {
    pub logged_in: bool,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub mid: Option<String>,
    pub vip_label: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QrLoginSession {
    pub qr_url: String,
    pub qrcode_key: String,
    pub qr_image_svg: String,
    pub expires_in_seconds: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QrLoginStatus {
    Waiting,
    Scanned,
    Confirmed,
    Expired,
    Unknown,
}

impl QrLoginStatus {
    pub fn from_bilibili_code(code: i32) -> Self {
        match code {
            0 => Self::Confirmed,
            86038 => Self::Expired,
            86090 => Self::Scanned,
            86101 => Self::Waiting,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QrLoginPollOutcome {
    pub status: QrLoginStatus,
    pub message: String,
    pub cookie_header: Option<String>,
}

impl AccountSummary {
    pub fn from_imported_cookie(cookie: &ImportedCookie) -> Self {
        Self {
            logged_in: true,
            name: None,
            avatar_url: None,
            mid: cookie.dede_user_id().map(str::to_owned),
            vip_label: None,
        }
    }
}

pub async fn start_qr_login() -> BdlResult<QrLoginSession> {
    let client = BpiClient::new().map_err(|error| BdlError::Bpi(error.to_string()))?;
    let data = client
        .login()
        .qr_generate()
        .await
        .map_err(|error| BdlError::Bpi(error.to_string()))?;

    Ok(QrLoginSession {
        qr_image_svg: render_qr_svg(&data.url)?,
        qr_url: data.url,
        qrcode_key: data.qrcode_key,
        expires_in_seconds: 180,
    })
}

pub async fn poll_qr_login(qrcode_key: &str) -> BdlResult<QrLoginPollOutcome> {
    let client = BpiClient::new().map_err(|error| BdlError::Bpi(error.to_string()))?;
    let params =
        LoginQrPollParams::new(qrcode_key).map_err(|error| BdlError::Bpi(error.to_string()))?;
    let data = client
        .login()
        .qr_poll(params)
        .await
        .map_err(|error| BdlError::Bpi(error.to_string()))?;
    let status = QrLoginStatus::from_bilibili_code(data.code);
    let cookie_header = if status == QrLoginStatus::Confirmed {
        Some(cookie_header_from_pairs(&data.cookies)?)
    } else {
        None
    };

    Ok(QrLoginPollOutcome {
        status,
        message: data.message,
        cookie_header,
    })
}

pub fn redact_sensitive(input: &str) -> String {
    let value = ["Cookie", "Authorization", "Proxy-Authorization"]
        .into_iter()
        .fold(input.to_owned(), |value, key| {
            redact_header_value(&value, key)
        });
    let value = redact_signed_urls(&value);
    let value = [
        "SESSDATA",
        "bili_jct",
        "DedeUserID",
        "DedeUserID__ckMd5",
        "buvid3",
        "sid",
        "access_key",
        "token",
        "deadline",
        "expires",
        "bili_ticket",
        "bili_ticket_expires",
        "sign",
    ]
    .into_iter()
    .fold(value, |value, key| redact_key_value(&value, key));
    truncate_chars(&value, MAX_REDACTED_TEXT_CHARS)
}

fn render_qr_svg(url: &str) -> BdlResult<String> {
    QrCode::new(url.as_bytes())
        .map_err(|error| BdlError::Account {
            message: format!("二维码生成失败: {error}"),
        })
        .map(|code| {
            code.render::<svg::Color<'_>>()
                .min_dimensions(180, 180)
                .dark_color(svg::Color("#17211d"))
                .light_color(svg::Color("#ffffff"))
                .build()
        })
}

fn cookie_header_from_pairs(pairs: &[(String, String)]) -> BdlResult<String> {
    if pairs.is_empty() {
        return Err(BdlError::Account {
            message: "扫码登录成功但未返回 Cookie，请重新扫码。".to_owned(),
        });
    }

    Ok(pairs
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("; "))
}

fn parse_cookie_pairs(raw: &str) -> BTreeMap<String, String> {
    raw.split(';')
        .filter_map(|part| {
            let (name, value) = part.trim().split_once('=')?;
            Some((name.trim().to_ascii_lowercase(), value.trim().to_owned()))
        })
        .collect()
}

fn require_cookie(pairs: &BTreeMap<String, String>, name: &str) -> BdlResult<()> {
    let normalized = name.to_ascii_lowercase();
    if pairs
        .get(&normalized)
        .is_some_and(|value| !value.is_empty())
    {
        return Ok(());
    }

    Err(BdlError::Account {
        message: format!("Cookie 缺少 `{name}`，请重新导入完整登录 Cookie。"),
    })
}

fn redact_key_value(input: &str, key: &str) -> String {
    let lower = input.to_ascii_lowercase();
    let key_lower = key.to_ascii_lowercase();
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;

    while let Some(relative_start) = lower[cursor..].find(&key_lower) {
        let start = cursor + relative_start;
        let key_end = start + key_lower.len();
        let Some((separator_start, separator)) = next_separator(input, key_end) else {
            output.push_str(&input[cursor..key_end]);
            cursor = key_end;
            continue;
        };

        output.push_str(&input[cursor..separator_start]);
        output.push(separator);
        output.push_str("<redacted>");
        cursor = consume_sensitive_value(input, separator_start + separator.len_utf8());
    }

    output.push_str(&input[cursor..]);
    output
}

fn redact_header_value(input: &str, key: &str) -> String {
    let lower = input.to_ascii_lowercase();
    let key_lower = key.to_ascii_lowercase();
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;

    while let Some(relative_start) = lower[cursor..].find(&key_lower) {
        let start = cursor + relative_start;
        let key_end = start + key_lower.len();
        let Some((separator_start, separator)) = next_separator(input, key_end) else {
            output.push_str(&input[cursor..key_end]);
            cursor = key_end;
            continue;
        };
        if separator != ':' {
            output.push_str(&input[cursor..=separator_start]);
            cursor = separator_start + separator.len_utf8();
            continue;
        }

        output.push_str(&input[cursor..separator_start]);
        output.push_str(": <redacted>");
        cursor = consume_header_value(input, separator_start + separator.len_utf8());
    }

    output.push_str(&input[cursor..]);
    output
}

fn redact_signed_urls(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;

    while let Some(relative_start) = find_next_url(&input[cursor..]) {
        let start = cursor + relative_start;
        output.push_str(&input[cursor..start]);
        let end = url_end(input, start);
        let raw_url = &input[start..end];
        output.push_str(&redact_url(raw_url));
        cursor = end;
    }

    output.push_str(&input[cursor..]);
    output
}

fn find_next_url(input: &str) -> Option<usize> {
    match (input.find("http://"), input.find("https://")) {
        (Some(http), Some(https)) => Some(http.min(https)),
        (Some(http), None) => Some(http),
        (None, Some(https)) => Some(https),
        (None, None) => None,
    }
}

fn url_end(input: &str, start: usize) -> usize {
    let mut end = start;
    for ch in input[start..].chars() {
        if ch.is_whitespace() || matches!(ch, '"' | '\'' | ')' | ']' | '}' | '<' | '>') {
            break;
        }
        end += ch.len_utf8();
    }

    end
}

fn redact_url(raw_url: &str) -> String {
    let Ok(parsed) = url::Url::parse(raw_url) else {
        return raw_url.to_owned();
    };
    let Some(query) = parsed.query() else {
        return raw_url.to_owned();
    };
    if query.len() < 32 && !signed_query_like(query) {
        return raw_url.to_owned();
    }

    raw_url
        .find('?')
        .map(|query_start| format!("{}?<redacted>", &raw_url[..query_start]))
        .unwrap_or_else(|| raw_url.to_owned())
}

fn signed_query_like(query: &str) -> bool {
    let lower = query.to_ascii_lowercase();
    [
        "token",
        "sign",
        "deadline",
        "expires",
        "bcdn_token",
        "access_key",
    ]
    .iter()
    .any(|key| lower.contains(key))
}

fn next_separator(input: &str, offset: usize) -> Option<(usize, char)> {
    let mut index = offset;
    for ch in input[offset..].chars() {
        if ch.is_whitespace() {
            index += ch.len_utf8();
            continue;
        }

        return matches!(ch, '=' | ':').then_some((index, ch));
    }

    None
}

fn consume_header_value(input: &str, offset: usize) -> usize {
    let mut index = offset;
    for ch in input[offset..].chars() {
        if matches!(ch, '\n' | '\r') {
            return index;
        }
        index += ch.len_utf8();
    }

    index
}

fn consume_sensitive_value(input: &str, offset: usize) -> usize {
    let mut index = offset;
    let mut saw_non_space = false;

    for ch in input[offset..].chars() {
        if !saw_non_space && ch.is_whitespace() {
            index += ch.len_utf8();
            continue;
        }

        saw_non_space = true;
        if matches!(ch, ';' | '&') || ch.is_whitespace() {
            return index;
        }

        index += ch.len_utf8();
    }

    index
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_owned();
    }

    let keep_chars = max_chars.saturating_sub(20);
    let mut truncated = value.chars().take(keep_chars).collect::<String>();
    truncated.push_str("...<truncated>");
    truncated
}
