use crate::err::error::BpiError;
use serde::{Deserialize, Deserializer, Serialize, de::DeserializeOwned};

/// bpi-rs 操作的 crate 级结果类型。
pub type BpiResult<T> = Result<T, BpiError>;

/// 大多数 Web API endpoint 使用的标准 Bilibili JSON envelope。
#[derive(Debug, Serialize, Clone)]
pub struct ApiEnvelope<T> {
    /// API 返回码。`0` 表示成功。
    pub code: i32,

    /// 成功 endpoint 返回的 payload。
    pub data: Option<T>,

    /// API 消息。Bilibili 成功时常返回 `"0"`。
    pub message: String,

    /// 部分 endpoint 返回的可选状态标记。
    pub status: bool,
}

impl<'de, T> Deserialize<'de> for ApiEnvelope<T>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawEnvelope::<T>::deserialize(deserializer)?;
        Ok(Self {
            code: raw.code.or(raw.errno).unwrap_or_default(),
            data: raw.data,
            message: raw.message.or(raw.msg).or(raw.show_msg).unwrap_or_default(),
            status: raw.status,
        })
    }
}

impl<T> ApiEnvelope<T> {
    /// 从字节解析 JSON envelope。
    pub fn from_slice(bytes: &[u8]) -> BpiResult<Self>
    where
        T: DeserializeOwned,
    {
        serde_json::from_slice(bytes).map_err(BpiError::from)
    }

    /// 如果此 envelope 表示成功的 API 响应，则返回它。
    pub fn ensure_success(self) -> BpiResult<Self> {
        if self.code == 0 {
            return Ok(self);
        }

        if self.message.is_empty() || self.message == "0" {
            Err(BpiError::from_code(self.code))
        } else {
            Err(BpiError::from_code_message(self.code, self.message))
        }
    }

    /// 从成功响应中提取必需 payload。
    pub fn into_payload(self) -> BpiResult<T> {
        self.ensure_success()?.data.ok_or(BpiError::MissingData)
    }

    /// 从成功响应中提取可选 payload。
    pub fn into_optional_payload(self) -> BpiResult<Option<T>> {
        Ok(self.ensure_success()?.data)
    }

    /// 不检查响应码，直接提取必需 payload。
    ///
    /// 这是为 payload 包含业务状态的 endpoint 保留的，例如 QR 轮询。
    pub fn into_data(self) -> Result<T, BpiError> {
        self.data.ok_or(BpiError::missing_data())
    }
}

/// 仍暴露完整 Bilibili envelope 的模块使用的兼容别名。
pub type BpiResponse<T> = ApiEnvelope<T>;

#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
struct RawEnvelope<T> {
    #[serde(default)]
    code: Option<i32>,
    #[serde(default)]
    errno: Option<i32>,
    #[serde(default, alias = "result")]
    data: Option<T>,
    #[serde(default)]
    message: Option<String>,
    #[serde(default)]
    msg: Option<String>,
    #[serde(default, rename = "showMsg")]
    show_msg: Option<String>,
    #[serde(default)]
    status: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct FixturePayload {
        title: String,
        aid: u64,
    }

    fn fixture(name: &str) -> &'static [u8] {
        match name {
            "success" => include_bytes!("../tests/fixtures/envelope/success.json"),
            "result-alias" => include_bytes!("../tests/fixtures/envelope/result-alias.json"),
            "api-error" => include_bytes!("../tests/fixtures/envelope/api-error.json"),
            "missing-data" => include_bytes!("../tests/fixtures/envelope/missing-data.json"),
            "no-payload" => include_bytes!("../tests/fixtures/envelope/no-payload.json"),
            _ => unreachable!("unknown fixture"),
        }
    }

    #[test]
    fn api_envelope_extracts_data_payload() -> Result<(), BpiError> {
        let payload =
            ApiEnvelope::<FixturePayload>::from_slice(fixture("success"))?.into_payload()?;

        assert_eq!(payload.title, "fixture video");
        Ok(())
    }

    #[test]
    fn api_envelope_extracts_result_alias_payload() -> Result<(), BpiError> {
        let payload =
            ApiEnvelope::<FixturePayload>::from_slice(fixture("result-alias"))?.into_payload()?;

        assert_eq!(payload.aid, 170002);
        Ok(())
    }

    #[test]
    fn api_envelope_returns_missing_data_for_required_payload() {
        let err = ApiEnvelope::<FixturePayload>::from_slice(fixture("missing-data"))
            .and_then(ApiEnvelope::into_payload)
            .unwrap_err();

        assert!(matches!(err, BpiError::MissingData));
    }

    #[test]
    fn api_envelope_allows_optional_payload() -> Result<(), BpiError> {
        let payload = ApiEnvelope::<FixturePayload>::from_slice(fixture("no-payload"))?
            .into_optional_payload()?;

        assert!(payload.is_none());
        Ok(())
    }

    #[test]
    fn api_envelope_converts_api_error() {
        let err = ApiEnvelope::<FixturePayload>::from_slice(fixture("api-error"))
            .and_then(ApiEnvelope::ensure_success)
            .unwrap_err();

        assert!(matches!(err, BpiError::Api { code: -101, .. }));
        assert_eq!(err.code(), Some(-101));
    }

    #[test]
    fn api_envelope_treats_null_message_as_empty() -> Result<(), BpiError> {
        let payload = ApiEnvelope::<LoginCoinFixture>::from_slice(
            br#"{ "code": 0, "message": null, "data": { "money": 0.0 } }"#,
        )?
        .into_payload()?;

        assert_eq!(payload.money, 0.0);
        Ok(())
    }

    #[test]
    fn api_envelope_maps_errno_login_error() {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(
            br#"{ "errno": 800501007, "msg": "user not login", "showMsg": "user not login" }"#,
        )
        .and_then(ApiEnvelope::ensure_success)
        .unwrap_err();

        assert!(err.requires_login());
        assert_eq!(err.code(), Some(800501007));
    }

    #[test]
    fn api_envelope_returns_decode_error_for_invalid_json() {
        let err = ApiEnvelope::<FixturePayload>::from_slice(b"{not json").unwrap_err();

        assert!(matches!(err, BpiError::Decode { .. }));
    }

    #[derive(Debug, Deserialize)]
    struct LoginCoinFixture {
        money: f64,
    }
}
