use serde_json::{Value, json};

use crate::{BpiError, BpiResult};

/// `/paywallet/wallet/getUserWallet` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletInfoParams {
    platform_type: u32,
    timestamp_ms: i64,
    trace_id: i64,
    version: String,
}

impl WalletInfoParams {
    /// 使用当前 UTC 毫秒时间戳创建钱包信息参数。
    pub fn new() -> Self {
        Self::at_timestamp(chrono::Utc::now().timestamp_millis())
    }

    /// 使用固定时间戳创建钱包信息参数，方便确定性调用方/测试。
    pub fn at_timestamp(timestamp_ms: i64) -> Self {
        Self {
            platform_type: 3,
            timestamp_ms,
            trace_id: timestamp_ms,
            version: "1.0".to_string(),
        }
    }

    pub fn with_platform_type(mut self, platform_type: u32) -> BpiResult<Self> {
        if platform_type == 0 {
            return Err(BpiError::invalid_parameter(
                "platformType",
                "platform type must be non-zero",
            ));
        }

        self.platform_type = platform_type;
        Ok(self)
    }

    pub fn with_trace_id(mut self, trace_id: i64) -> BpiResult<Self> {
        if trace_id <= 0 {
            return Err(BpiError::invalid_parameter(
                "traceId",
                "trace id must be positive",
            ));
        }

        self.trace_id = trace_id;
        Ok(self)
    }

    pub fn with_version(mut self, version: impl Into<String>) -> BpiResult<Self> {
        let version = version.into();
        let version = version.trim();
        if version.is_empty() {
            return Err(BpiError::invalid_parameter(
                "version",
                "version cannot be blank",
            ));
        }

        self.version = version.to_string();
        Ok(self)
    }

    pub(crate) fn body(&self, csrf: &str) -> Value {
        json!({
            "csrf": csrf,
            "platformType": self.platform_type,
            "timestamp": self.timestamp_ms,
            "traceId": self.trace_id,
            "version": self.version,
        })
    }
}

impl Default for WalletInfoParams {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wallet_info_params_serializes_default_body() {
        let params = WalletInfoParams::at_timestamp(1_700_000_000_000);

        assert_eq!(
            params.body("csrf-token"),
            json!({
                "csrf": "csrf-token",
                "platformType": 3,
                "timestamp": 1_700_000_000_000_i64,
                "traceId": 1_700_000_000_000_i64,
                "version": "1.0",
            })
        );
    }

    #[test]
    fn wallet_info_params_serializes_custom_values() -> BpiResult<()> {
        let params = WalletInfoParams::at_timestamp(1_700_000_000_000)
            .with_platform_type(4)?
            .with_trace_id(1_700_000_000_001)?
            .with_version("2.0")?;

        assert_eq!(
            params.body("csrf-token"),
            json!({
                "csrf": "csrf-token",
                "platformType": 4,
                "timestamp": 1_700_000_000_000_i64,
                "traceId": 1_700_000_000_001_i64,
                "version": "2.0",
            })
        );
        Ok(())
    }

    #[test]
    fn wallet_info_params_rejects_zero_platform_type() {
        let err = WalletInfoParams::at_timestamp(1)
            .with_platform_type(0)
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "platformType",
                ..
            }
        ));
    }

    #[test]
    fn wallet_info_params_rejects_blank_version() {
        let err = WalletInfoParams::at_timestamp(1)
            .with_version("  ")
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "version",
                ..
            }
        ));
    }

    #[test]
    fn wallet_info_params_trims_version() -> BpiResult<()> {
        let params = WalletInfoParams::at_timestamp(1).with_version(" 2.0 ")?;

        assert_eq!(params.body("csrf-token")["version"], "2.0");
        Ok(())
    }
}
