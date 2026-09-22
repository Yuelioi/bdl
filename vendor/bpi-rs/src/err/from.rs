use super::error::BpiError;

// 从reqwest错误转换
impl From<reqwest::Error> for BpiError {
    fn from(err: reqwest::Error) -> Self {
        BpiError::Transport { source: err }
    }
}

// 从API响应转换
impl<T> From<crate::response::ApiEnvelope<T>> for BpiError {
    fn from(resp: crate::response::ApiEnvelope<T>) -> Self {
        BpiError::from_api_response(resp)
    }
}

// 从JSON序列化错误转换
impl From<serde_json::Error> for BpiError {
    fn from(err: serde_json::Error) -> Self {
        BpiError::Decode { source: err }
    }
}
