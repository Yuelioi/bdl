use reqwest::{Method, RequestBuilder, Url};

/// 可安全写入请求日志的元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestMetadata {
    pub method: Method,
    pub endpoint: String,
    pub sanitized_url: String,
}

impl RequestMetadata {
    /// 从可克隆的 reqwest request builder 构建安全请求元数据。
    pub fn from_builder(builder: &RequestBuilder, endpoint: impl Into<String>) -> Option<Self> {
        let request = builder.try_clone()?.build().ok()?;

        Some(Self {
            method: request.method().clone(),
            endpoint: endpoint.into(),
            sanitized_url: sanitize_url_for_logging(request.url().as_str()),
        })
    }
}

/// 返回可安全用于日志的 URL 字符串。
pub fn sanitize_url_for_logging(url: &str) -> String {
    let Ok(mut parsed) = Url::parse(url) else {
        return "<invalid-url>".to_string();
    };

    let safe_pairs = parsed
        .query_pairs()
        .filter(|(key, _)| !is_sensitive_query_key(key))
        .map(|(key, value)| (key.into_owned(), value.into_owned()))
        .collect::<Vec<_>>();

    parsed.set_query(None);

    if !safe_pairs.is_empty() {
        let query = safe_pairs
            .into_iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join("&");
        parsed.set_query(Some(&query));
    }

    parsed.to_string()
}

/// 返回可安全用于日志的请求头键值对；请求头敏感时返回 `None`。
pub fn sanitize_header_for_logging(name: &str, value: &str) -> Option<(String, String)> {
    if is_sensitive_header_name(name) {
        return None;
    }

    Some((name.to_string(), value.to_string()))
}

fn is_sensitive_query_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "sessdata"
            | "dedeuserid"
            | "dedeuserid__ckmd5"
            | "bili_jct"
            | "csrf"
            | "csrf_token"
            | "w_rid"
            | "access_key"
            | "token"
            | "cookie"
    )
}

fn is_sensitive_header_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "cookie" | "authorization" | "proxy-authorization" | "set-cookie"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_url_for_logging_removes_sensitive_query_values() {
        let sanitized = sanitize_url_for_logging(
            "https://api.bilibili.com/x/test?mid=1&SESSDATA=secret&csrf=token&bili_jct=token&w_rid=signed",
        );

        assert_eq!(sanitized, "https://api.bilibili.com/x/test?mid=1");
    }

    #[test]
    fn sanitize_url_for_logging_keeps_path_when_all_query_values_are_sensitive() {
        let sanitized =
            sanitize_url_for_logging("https://api.bilibili.com/x/test?SESSDATA=secret&csrf=token");

        assert_eq!(sanitized, "https://api.bilibili.com/x/test");
    }

    #[test]
    fn sanitize_url_for_logging_handles_invalid_url_without_panicking() {
        let sanitized = sanitize_url_for_logging("not a url with SESSDATA=secret");

        assert_eq!(sanitized, "<invalid-url>");
    }

    #[test]
    fn sanitize_header_for_logging_removes_raw_cookie_header() {
        let sanitized = sanitize_header_for_logging("Cookie", "SESSDATA=secret; bili_jct=token");

        assert!(sanitized.is_none());
    }

    #[test]
    fn request_metadata_from_builder_sanitizes_url_and_preserves_method() {
        let client = reqwest::Client::new();
        let builder = client.post("https://api.bilibili.com/x/test?mid=1&csrf=secret&w_rid=signed");

        let metadata = RequestMetadata::from_builder(&builder, "test.endpoint").unwrap();

        assert_eq!(metadata.method, Method::POST);
        assert_eq!(metadata.endpoint, "test.endpoint");
        assert_eq!(
            metadata.sanitized_url,
            "https://api.bilibili.com/x/test?mid=1"
        );
    }
}
