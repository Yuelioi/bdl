pub type BdlResult<T> = Result<T, BdlError>;

#[derive(Debug, thiserror::Error)]
pub enum BdlError {
    #[error("invalid input: {message}")]
    InvalidInput { message: String },

    #[error("unsupported source: {kind}")]
    UnsupportedSource { kind: String },

    #[error("bpi error: {0}")]
    Bpi(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
