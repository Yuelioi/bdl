pub type BdlResult<T> = Result<T, BdlError>;

#[derive(Debug, thiserror::Error)]
pub enum BdlError {
    #[error("invalid input: {message}")]
    InvalidInput { message: String },

    #[error("unsupported source: {kind}")]
    UnsupportedSource { kind: String },

    #[error("bpi error: {0}")]
    Bpi(String),

    #[error("该分集使用 DRM 加密，BDL 暂不支持下载。请前往哔哩哔哩观看。")]
    DrmProtected,

    #[error("当前账号只能试看此分集，请登录已购买该课程的账号后重试。")]
    CoursePreviewOnly,

    #[error("planning error: {message}")]
    Planning { message: String },

    #[error("fetch error: {message}")]
    Fetch { message: String },

    #[error("account error: {message}")]
    Account { message: String },

    #[error("platform error: {message}")]
    Platform { message: String },

    #[error("storage error: {0}")]
    Storage(#[from] rusqlite::Error),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Mux(#[from] crate::muxer::MuxError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
