pub type BdlResult<T> = Result<T, BdlError>;

/// Attach filesystem context where the operation is known, before errors cross
/// task/IPC boundaries. A local failure must never be retried as a CDN failure.
pub trait IoResultExt<T> {
    fn path_context(self, operation: &str, path: &std::path::Path) -> BdlResult<T>;
}

impl<T> IoResultExt<T> for std::io::Result<T> {
    fn path_context(self, operation: &str, path: &std::path::Path) -> BdlResult<T> {
        self.map_err(|source| file_io_error(operation, path, source))
    }
}

pub fn file_io_error(operation: &str, path: &std::path::Path, source: std::io::Error) -> BdlError {
    let code = match source.kind() {
        std::io::ErrorKind::NotFound => "file_not_found",
        std::io::ErrorKind::PermissionDenied => "file_access_denied",
        std::io::ErrorKind::StorageFull => "disk_full",
        std::io::ErrorKind::ReadOnlyFilesystem => "file_read_only",
        std::io::ErrorKind::IsADirectory
        | std::io::ErrorKind::NotADirectory
        | std::io::ErrorKind::AlreadyExists
        | std::io::ErrorKind::InvalidInput => "file_path_invalid",
        _ => "local_io_failed",
    };
    BdlError::FileIo {
        context: format!(
            "[stage=file_io code={code}] operation={operation} path={} filename_bytes={} path_bytes={} os_code={:?} io_kind={:?}",
            path.display(),
            path.file_name()
                .map_or(0, |name| name.as_encoded_bytes().len()),
            path.as_os_str().as_encoded_bytes().len(),
            source.raw_os_error(),
            source.kind(),
        ),
        source,
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BdlError {
    #[error("invalid input: {message}")]
    InvalidInput { message: String },

    #[error("unsupported source: {kind}")]
    UnsupportedSource { kind: String },

    #[error("bpi error: {0}")]
    Bpi(String),

    #[error("{context}: {source}")]
    BpiTransport {
        context: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("该分集使用 DRM 加密，BDL 暂不支持下载。请前往哔哩哔哩观看。")]
    DrmProtected,

    #[error("当前账号只能试看此分集，请登录已购买该课程的账号后重试。")]
    CoursePreviewOnly,

    #[error("planning error: {message}")]
    Planning { message: String },

    #[error("解析已停止。\n{context}")]
    ParseCancelled { context: String },

    #[error("{context}\n原始错误：{source}")]
    Context {
        context: String,
        #[source]
        source: Box<BdlError>,
    },

    #[error("fetch error: {message}")]
    Fetch { message: String },

    #[error("account error: {message}")]
    Account { message: String },

    #[error("platform error: {message}")]
    Platform { message: String },

    #[error("storage error: {0}{sqlite}", sqlite = sqlite_error_context(.0))]
    Storage(#[from] rusqlite::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Mux(#[from] crate::muxer::MuxError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{context}: {source}")]
    FileIo {
        context: String,
        #[source]
        source: std::io::Error,
    },
}

fn sqlite_error_context(error: &rusqlite::Error) -> String {
    error.sqlite_error().map_or_else(String::new, |code| {
        format!(
            "; sqlite_code={:?} sqlite_extended_code={}",
            code.code, code.extended_code
        )
    })
}

impl BdlError {
    pub fn is_local_io(&self) -> bool {
        matches!(self.root_cause(), Self::FileIo { .. } | Self::Io(_))
    }
    pub fn with_context(self, context: impl Into<String>) -> Self {
        Self::Context {
            context: context.into(),
            source: Box::new(self),
        }
    }

    pub fn root_cause(&self) -> &Self {
        match self {
            Self::Context { source, .. } => source.root_cause(),
            _ => self,
        }
    }
}

/// Preserve transport causes and the exact BDL call site. Response bodies are
/// never logged; serde's quoted input values are removed from decode causes.
#[track_caller]
pub fn bpi_error(error: impl Into<bpi_rs::BpiError>) -> BdlError {
    let error = error.into();
    let location = std::panic::Location::caller();
    let mut message = if matches!(error, bpi_rs::BpiError::Decode { .. }) {
        safe_decode_cause(&error.to_string())
    } else {
        error.to_string()
    };
    if let bpi_rs::BpiError::Transport { source } = error {
        let endpoint = source
            .url()
            .map(|url| format!("{}{}", url.host_str().unwrap_or_default(), url.path()));
        let source = source.without_url();
        let code = if source.is_timeout() {
            "network_timeout"
        } else {
            "bilibili_request_failed"
        };
        let context = crate::account::redact_diagnostic(&format!(
            "[stage=bilibili_transport code={code}] origin={location} endpoint={endpoint:?} is_connect={} is_timeout={} is_body={} is_decode={} cause={}",
            source.is_connect(),
            source.is_timeout(),
            source.is_body(),
            source.is_decode(),
            error_causes(&source)
        ));
        return BdlError::BpiTransport { context, source };
    } else if let Some(source) = std::error::Error::source(&error) {
        message.push_str(&format!(
            "; cause={}",
            safe_decode_cause(&source.to_string())
        ));
    }
    BdlError::Bpi(crate::account::redact_diagnostic(&format!(
        "[stage=bilibili_api origin={location}] {message}"
    )))
}

pub(crate) fn error_causes(error: &(dyn std::error::Error + 'static)) -> String {
    let mut causes = Vec::new();
    let mut current = error.source();
    while let Some(cause) = current {
        if causes.len() == 8 {
            causes.push("<cause_chain_truncated>".into());
            break;
        }
        causes.push(cause.to_string());
        current = cause.source();
    }
    crate::account::redact_diagnostic(&causes.join(" <- "))
}

fn safe_decode_cause(message: &str) -> String {
    let mut result = String::new();
    let mut quote = None;
    let mut escaped = false;
    for ch in message.chars() {
        if let Some(end) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == end {
                quote = None;
            }
        } else if matches!(ch, '"' | '`') {
            result.push_str("<redacted>");
            quote = Some(ch);
        } else {
            result.push(ch);
        }
    }
    result
}
