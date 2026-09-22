use std::fmt;

use bytes::Bytes;
use serde::Serialize;
use thiserror::Error;

/// HTTP 响应模型解码失败时保留的可恢复上下文。
///
/// 调试和错误格式化只显示响应长度，不会输出原始响应内容。调用方可以通过
/// [`BpiError::response_body`] 显式取得响应字节并使用临时模型重新解析。
pub struct ResponseDecodeError {
    source: serde_json::Error,
    body: Bytes,
}

impl ResponseDecodeError {
    pub(crate) fn new(source: serde_json::Error, body: Bytes) -> Self {
        Self { source, body }
    }

    pub(crate) fn source_error(&self) -> &serde_json::Error {
        &self.source
    }

    fn response_body(&self) -> &[u8] {
        &self.body
    }
}

impl fmt::Debug for ResponseDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResponseDecodeError")
            .field("category", &self.source.classify())
            .field("line", &self.source.line())
            .field("column", &self.source.column())
            .field(
                "response_body",
                &format_args!("<redacted: {} bytes>", self.body.len()),
            )
            .finish()
    }
}

impl fmt::Display for ResponseDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "failed to decode response ({:?}) at line {} column {}",
            self.source.classify(),
            self.source.line(),
            self.source.column()
        )
    }
}

impl std::error::Error for ResponseDecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// 错误类型分类
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ErrorCategory {
    /// 权限认证类错误
    Auth,
    /// 请求参数类错误
    Request,
    /// 服务器类错误
    Server,
    /// 业务逻辑类错误
    Business,
    /// 网络类错误
    Network,
    /// 未知错误
    Unknown,
}

#[derive(Debug, Error, Serialize)]
pub enum BpiError {
    /// 网络请求失败
    #[error("网络请求失败: {message}")]
    Network { message: String },

    /// transport 层请求失败。
    #[error("transport request failed: {source}")]
    Transport {
        #[serde(skip)]
        source: reqwest::Error,
    },

    /// HTTP状态码错误
    #[error("HTTP请求失败，状态码: {status}")]
    Http { status: u16 },

    /// HTTP 状态错误。
    #[error("HTTP request failed with status {status}")]
    HttpStatus { status: u16 },

    /// JSON解析失败
    #[error("数据解析失败: {message}")]
    Parse { message: String },

    /// 响应解码失败。
    #[error("failed to decode response: {source}")]
    Decode {
        #[serde(skip)]
        source: serde_json::Error,
    },

    /// HTTP 响应模型解码失败；原始响应可供调用方使用临时模型恢复。
    #[error(transparent)]
    ResponseDecode {
        #[serde(skip)]
        error: ResponseDecodeError,
    },

    /// API返回的业务错误
    #[error("API错误 [{code}]: {message}")]
    Api {
        code: i32,
        message: String,
        category: ErrorCategory,
    },

    /// 验证错误
    #[error("验证失败: {message}")]
    Authentication { message: String },

    /// 认证或授权错误。
    #[error("authentication failed: {message}")]
    Auth { message: String },

    /// # 参数错误
    #[error("参数错误 [{field}]: {message}")]
    InvalidParameter {
        field: &'static str,
        message: &'static str,
    },

    /// API 响应成功，但未包含必需的 payload 数据。
    #[error("missing response data")]
    MissingData,

    /// 当前解析器不支持该响应格式。
    #[error("unsupported response: {message}")]
    UnsupportedResponse { message: String },
}

impl BpiError {
    pub fn missing_csrf() -> Self {
        BpiError::InvalidParameter {
            field: "csrf",
            message: "缺少CSRF",
        }
    }

    pub fn missing_data() -> Self {
        BpiError::MissingData
    }

    pub fn auth_required() -> Self {
        BpiError::Auth {
            message: "需要登录".to_string(),
        }
    }

    pub(crate) fn response_decode(source: serde_json::Error, body: Bytes) -> Self {
        Self::ResponseDecode {
            error: ResponseDecodeError::new(source, body),
        }
    }
}

/// 生成Error的From实现
impl BpiError {
    /// 根据API错误码创建BpiError
    pub fn from_code(code: i32) -> Self {
        let message = super::code::get_error_message(code);
        let category = super::code::categorize_error(code);

        BpiError::Api {
            code,
            message,
            category,
        }
    }

    // 不在错误码表中的API错误
    pub fn from_code_message(code: i32, message: String) -> Self {
        let category = super::code::categorize_error(code);
        BpiError::Api {
            code,
            message,
            category,
        }
    }

    /// 从API响应创建BpiError
    pub fn from_api_response<T>(resp: crate::response::ApiEnvelope<T>) -> Self {
        if resp.code == 0 {
            return BpiError::Api {
                code: 0,
                message: "API返回成功状态但被当作错误处理".to_string(),
                category: ErrorCategory::Unknown,
            };
        }

        if resp.message.is_empty() || resp.message == "0" {
            Self::from_code(resp.code)
        } else {
            Self::from_code_message(resp.code, resp.message)
        }
    }
}

/// 获取错误属性
impl BpiError {
    /// 获取错误码
    pub fn code(&self) -> Option<i32> {
        match self {
            BpiError::Api { code, .. } => Some(*code),
            _ => None,
        }
    }

    /// 获取 HTTP 状态码
    pub fn http_status(&self) -> Option<u16> {
        match self {
            BpiError::Http { status } | BpiError::HttpStatus { status } => Some(*status),
            _ => None,
        }
    }

    /// 返回导致模型解码失败的原始 HTTP 响应。
    ///
    /// 只有从 transport 响应反序列化产生的 [`BpiError::ResponseDecode`] 才会返回
    /// `Some`。普通 JSON 解析错误不会伪装成可恢复的 HTTP 响应错误。
    pub fn response_body(&self) -> Option<&[u8]> {
        match self {
            Self::ResponseDecode { error } => Some(error.response_body()),
            _ => None,
        }
    }

    /// 获取错误分类
    pub fn category(&self) -> ErrorCategory {
        match self {
            BpiError::Api { category, .. } => category.clone(),
            BpiError::Network { .. } => ErrorCategory::Network,
            BpiError::Transport { .. } => ErrorCategory::Network,
            BpiError::Http { .. } => ErrorCategory::Network,
            BpiError::HttpStatus { .. } => ErrorCategory::Network,
            BpiError::Parse { .. } => ErrorCategory::Request,
            BpiError::Decode { .. } => ErrorCategory::Request,
            BpiError::ResponseDecode { .. } => ErrorCategory::Request,
            BpiError::InvalidParameter { .. } => ErrorCategory::Request,
            BpiError::Authentication { .. } => ErrorCategory::Auth,
            BpiError::Auth { .. } => ErrorCategory::Auth,
            BpiError::MissingData => ErrorCategory::Request,
            BpiError::UnsupportedResponse { .. } => ErrorCategory::Request,
        }
    }
}

/// 错误创建函数
impl BpiError {
    /// 创建网络错误
    pub fn network(message: impl Into<String>) -> Self {
        BpiError::Network {
            message: message.into(),
        }
    }

    /// 创建HTTP错误
    pub fn http(status: u16) -> Self {
        BpiError::HttpStatus { status }
    }

    /// 创建解析错误
    pub fn parse(message: impl Into<String>) -> Self {
        BpiError::Parse {
            message: message.into(),
        }
    }

    /// 创建参数错误
    pub fn invalid_parameter(field: &'static str, message: &'static str) -> Self {
        BpiError::InvalidParameter { field, message }
    }

    pub fn auth(message: impl Into<String>) -> Self {
        BpiError::Auth {
            message: message.into(),
        }
    }

    /// 创建不支持响应错误。
    pub fn unsupported_response(message: impl Into<String>) -> Self {
        BpiError::UnsupportedResponse {
            message: message.into(),
        }
    }
}

/// 错误判断
impl BpiError {
    /// 判断是否需要用户登录
    pub fn requires_login(&self) -> bool {
        matches!(self.code(), Some(-101) | Some(-401) | Some(800501007))
            || matches!(self.http_status(), Some(401))
    }

    /// 判断是否为权限问题
    pub fn is_permission_error(&self) -> bool {
        matches!(self.category(), ErrorCategory::Auth)
            || matches!(self.code(), Some(-403) | Some(-4))
            || matches!(self.http_status(), Some(403))
    }

    /// 判断是否需要VIP权限
    pub fn requires_vip(&self) -> bool {
        matches!(self.code(), Some(-106) | Some(-650))
    }

    /// 判断是否为风控拦截
    pub fn is_risk_control(&self) -> bool {
        matches!(self.code(), Some(-352) | Some(-412)) || matches!(self.http_status(), Some(412))
    }

    /// 判断是否为业务逻辑错误
    pub fn is_business_error(&self) -> bool {
        matches!(self.category(), ErrorCategory::Business)
    }

    /// 获取可写入契约的稳定语义错误标签
    pub fn semantic_error(&self) -> Option<&'static str> {
        if self.requires_login() {
            Some("requires_login")
        } else if self.requires_vip() {
            Some("requires_vip")
        } else if self.is_risk_control() {
            Some("risk_control")
        } else if self.is_permission_error() {
            Some("permission_denied")
        } else if self.is_business_error() {
            Some("business_error")
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;

    #[test]
    fn http_status_returns_status_for_legacy_and_current_http_variants() {
        assert_eq!(BpiError::Http { status: 412 }.http_status(), Some(412));
        assert_eq!(BpiError::http(403).http_status(), Some(403));
    }

    #[test]
    fn requires_login_recognizes_api_and_http_unauthorized_errors() {
        assert!(BpiError::from_code(-101).requires_login());
        assert!(BpiError::from_code(800501007).requires_login());
        assert!(BpiError::http(401).requires_login());
    }

    #[test]
    fn is_permission_error_recognizes_api_and_http_forbidden_errors() {
        assert!(BpiError::from_code(-403).is_permission_error());
        assert!(BpiError::http(403).is_permission_error());
    }

    #[test]
    fn is_risk_control_recognizes_api_and_http_risk_blocks() {
        assert!(BpiError::from_code(-352).is_risk_control());
        assert!(BpiError::from_code(-412).is_risk_control());
        assert!(BpiError::http(412).is_risk_control());
    }

    #[test]
    fn semantic_error_returns_stable_contract_labels() {
        assert_eq!(
            BpiError::from_code(-101).semantic_error(),
            Some("requires_login")
        );
        assert_eq!(
            BpiError::from_code(-106).semantic_error(),
            Some("requires_vip")
        );
        assert_eq!(
            BpiError::from_code(-352).semantic_error(),
            Some("risk_control")
        );
        assert_eq!(
            BpiError::from_code(-403).semantic_error(),
            Some("permission_denied")
        );
    }

    #[test]
    fn response_decode_debug_redacts_response_body() {
        let err = response_decode_error();

        assert!(!format!("{err:?}").contains("private-response-marker"));
    }

    #[test]
    fn response_decode_display_redacts_response_body() {
        let err = response_decode_error();

        assert!(!err.to_string().contains("private-response-marker"));
    }

    #[test]
    fn response_decode_serialization_redacts_response_body() -> Result<(), serde_json::Error> {
        let err = response_decode_error();
        let serialized = serde_json::to_string(&err)?;

        assert!(!serialized.contains("private-response-marker"));
        Ok(())
    }

    fn response_decode_error() -> BpiError {
        let source = serde_json::from_slice::<u64>(br#""private-response-marker""#).unwrap_err();
        BpiError::response_decode(
            source,
            Bytes::from_static(br#"{"secret":"private-response-marker"}"#),
        )
    }
}
