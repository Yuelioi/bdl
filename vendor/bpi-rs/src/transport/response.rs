use std::time::Duration;

use bytes::Bytes;
use serde::de::{DeserializeOwned, IgnoredAny};

use crate::{BpiError, BpiResult, response::ApiEnvelope};

/// 可安全写入响应日志的元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseMetadata {
    pub status: u16,
    pub duration: Duration,
    pub api_code: Option<i32>,
}

/// 原始 HTTP 响应字节和安全响应元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportResponse {
    pub metadata: ResponseMetadata,
    pub body: Bytes,
}

/// 已解码的 JSON API envelope，以及解析前观察到的响应元数据。
#[derive(Debug, Clone)]
pub struct TransportEnvelope<T> {
    pub metadata: ResponseMetadata,
    pub envelope: ApiEnvelope<T>,
}

impl TransportResponse {
    /// 将此响应解码为 Bilibili JSON API envelope。
    pub fn decode_api_envelope<T>(&self) -> BpiResult<TransportEnvelope<T>>
    where
        T: DeserializeOwned,
    {
        let header: ApiEnvelope<IgnoredAny> = serde_json::from_slice(&self.body)
            .map_err(|source| BpiError::response_decode(source, self.body.clone()))?;
        let envelope = if header.code == 0 {
            serde_json::from_slice(&self.body)
                .map_err(|source| BpiError::response_decode(source, self.body.clone()))?
        } else {
            ApiEnvelope {
                code: header.code,
                data: None,
                message: header.message,
                status: header.status,
            }
        };
        let mut metadata = self.metadata.clone();
        metadata.api_code = Some(envelope.code);

        Ok(TransportEnvelope { metadata, envelope })
    }
}

impl<T> TransportEnvelope<T> {
    /// 如果此 envelope 表示成功的 API 响应，则返回它。
    pub fn ensure_success(self) -> BpiResult<Self> {
        Ok(Self {
            metadata: self.metadata,
            envelope: self.envelope.ensure_success()?,
        })
    }

    /// 从成功的 API 响应中提取必需 payload。
    pub fn into_payload(self) -> BpiResult<TransportPayload<T>> {
        let Self { metadata, envelope } = self.ensure_success()?;
        let payload = envelope.data.ok_or(crate::BpiError::MissingData)?;

        Ok(TransportPayload { metadata, payload })
    }

    /// 从成功的 API 响应中提取可选 payload。
    pub fn into_optional_payload(self) -> BpiResult<TransportOptionalPayload<T>> {
        let Self { metadata, envelope } = self.ensure_success()?;

        Ok(TransportOptionalPayload {
            metadata,
            payload: envelope.data,
        })
    }

    /// 成功校验后返回已解码的 API envelope。
    pub fn into_api_envelope(self) -> BpiResult<crate::response::ApiEnvelope<T>> {
        Ok(self.ensure_success()?.envelope)
    }
}

/// 必需 API payload，以及提取前观察到的响应元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportPayload<T> {
    pub metadata: ResponseMetadata,
    pub payload: T,
}

/// 可选 API payload，以及提取前观察到的响应元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportOptionalPayload<T> {
    pub metadata: ResponseMetadata,
    pub payload: Option<T>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct Payload {
        value: u64,
    }

    #[test]
    fn decode_api_envelope_preserves_response_metadata_and_api_code() -> BpiResult<()> {
        let response = TransportResponse {
            metadata: ResponseMetadata {
                status: 200,
                duration: Duration::from_millis(12),
                api_code: None,
            },
            body: Bytes::from_static(br#"{ "code": 0, "data": { "value": 42 } }"#),
        };

        let decoded = response.decode_api_envelope::<Payload>()?;

        assert_eq!(decoded.metadata.status, 200);
        assert_eq!(decoded.metadata.api_code, Some(0));
        assert_eq!(decoded.envelope.data.unwrap().value, 42);
        Ok(())
    }

    #[test]
    fn transport_envelope_into_payload_returns_payload_with_metadata() -> BpiResult<()> {
        let decoded = success_response().decode_api_envelope::<Payload>()?;

        let payload = decoded.into_payload()?;

        assert_eq!(payload.metadata.api_code, Some(0));
        assert_eq!(payload.payload.value, 42);
        Ok(())
    }

    #[test]
    fn transport_envelope_into_payload_returns_api_error() {
        let response = TransportResponse {
            metadata: ResponseMetadata {
                status: 200,
                duration: Duration::from_millis(12),
                api_code: None,
            },
            body: Bytes::from_static(br#"{ "code": -101, "message": "not logged in" }"#),
        };

        let err = response
            .decode_api_envelope::<Payload>()
            .and_then(TransportEnvelope::into_payload)
            .unwrap_err();

        assert!(matches!(err, BpiError::Api { code: -101, .. }));
    }

    #[test]
    fn transport_envelope_prioritizes_api_error_over_incompatible_error_payload() {
        let response = TransportResponse {
            metadata: ResponseMetadata {
                status: 200,
                duration: Duration::from_millis(12),
                api_code: None,
            },
            body: Bytes::from_static(
                br#"{ "code": -352, "message": "risk control", "data": { "ga_data": null } }"#,
            ),
        };

        let err = response
            .decode_api_envelope::<Payload>()
            .and_then(TransportEnvelope::into_payload)
            .unwrap_err();

        assert!(matches!(err, BpiError::Api { code: -352, .. }));
        assert!(err.is_risk_control());
    }

    #[test]
    fn transport_envelope_into_optional_payload_allows_empty_success() -> BpiResult<()> {
        let response = TransportResponse {
            metadata: ResponseMetadata {
                status: 200,
                duration: Duration::from_millis(12),
                api_code: None,
            },
            body: Bytes::from_static(br#"{ "code": 0, "message": "0" }"#),
        };

        let payload = response
            .decode_api_envelope::<Payload>()?
            .into_optional_payload()?;

        assert_eq!(payload.metadata.api_code, Some(0));
        assert!(payload.payload.is_none());
        Ok(())
    }

    #[test]
    fn decode_api_envelope_preserves_body_when_payload_model_does_not_match() {
        let body = Bytes::from_static(br#"{ "code": 0, "data": { "value": -1000 } }"#);
        let response = TransportResponse {
            metadata: ResponseMetadata {
                status: 200,
                duration: Duration::from_millis(12),
                api_code: None,
            },
            body: body.clone(),
        };

        let err = response.decode_api_envelope::<Payload>().unwrap_err();

        assert_eq!(err.response_body(), Some(body.as_ref()));
    }

    #[test]
    fn ordinary_json_decode_error_does_not_expose_response_body() {
        let err = BpiError::from(serde_json::from_slice::<serde_json::Value>(b"{").unwrap_err());

        assert!(err.response_body().is_none());
    }

    fn success_response() -> TransportResponse {
        TransportResponse {
            metadata: ResponseMetadata {
                status: 200,
                duration: Duration::from_millis(12),
                api_code: None,
            },
            body: Bytes::from_static(br#"{ "code": 0, "data": { "value": 42 } }"#),
        }
    }
}
