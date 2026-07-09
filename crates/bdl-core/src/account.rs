use std::collections::BTreeMap;

use bpi_rs::BpiClient;
use bpi_rs::login::LoginQrPollParams;
use qrcode::QrCode;
use qrcode::render::svg;
use serde::{Deserialize, Serialize};

use crate::error::{BdlError, BdlResult};

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
    [
        "Cookie",
        "SESSDATA",
        "bili_jct",
        "DedeUserID",
        "token",
        "deadline",
        "expires",
        "sign",
    ]
    .into_iter()
    .fold(input.to_owned(), |value, key| redact_key_value(&value, key))
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
