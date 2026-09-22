use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::{BpiError, BpiResult};

const KEY_ID: &str = "ec02";
const HMAC_KEY: &str = "XgwSnGZ1p";

type HmacSha256 = Hmac<Sha256>;

pub fn hexsign(key: &str, timestamp: u64) -> BpiResult<String> {
    if key.is_empty() {
        return Err(BpiError::invalid_parameter(
            "key",
            "HMAC key cannot be empty",
        ));
    }

    let message = format!("ts{timestamp}");
    let mut mac = HmacSha256::new_from_slice(key.as_bytes())
        .map_err(|error| BpiError::parse(format!("HMAC key error: {error}")))?;

    mac.update(message.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

pub fn ticket_hexsign(timestamp: u64) -> BpiResult<String> {
    hexsign(HMAC_KEY, timestamp)
}

pub fn ticket_request_params(timestamp: u64, csrf: &str) -> BpiResult<Vec<(String, String)>> {
    Ok(vec![
        ("key_id".to_string(), KEY_ID.to_string()),
        ("hexsign".to_string(), hexsign(HMAC_KEY, timestamp)?),
        ("context[ts]".to_string(), timestamp.to_string()),
        ("csrf".to_string(), csrf.to_string()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BpiError;

    #[test]
    fn hexsign_returns_stable_lowercase_digest() -> Result<(), BpiError> {
        let digest = hexsign("XgwSnGZ1p", 1_234_567_890)?;

        assert_eq!(
            digest,
            "a7da9d971f117aa2b439c4b6cc46c7afbba8ade9f3ca959578af1bcfb37ebd2f"
        );
        assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));
        Ok(())
    }

    #[test]
    fn ticket_hexsign_uses_web_ticket_hmac_key() -> Result<(), BpiError> {
        assert_eq!(
            ticket_hexsign(1_234_567_890)?,
            "a7da9d971f117aa2b439c4b6cc46c7afbba8ade9f3ca959578af1bcfb37ebd2f"
        );
        Ok(())
    }

    #[test]
    fn hexsign_rejects_empty_key() {
        let err = hexsign("", 1_234_567_890).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "key", .. }
        ));
    }

    #[test]
    fn ticket_request_params_include_required_fields() -> Result<(), BpiError> {
        let params = ticket_request_params(1_234_567_890, "csrf-token")?;

        assert_eq!(
            params,
            vec![
                ("key_id".to_string(), "ec02".to_string()),
                ("hexsign".to_string(), ticket_hexsign(1_234_567_890)?),
                ("context[ts]".to_string(), "1234567890".to_string()),
                ("csrf".to_string(), "csrf-token".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn ticket_request_params_allows_empty_csrf_for_guest_ticket() -> Result<(), BpiError> {
        let params = ticket_request_params(1_234_567_890, "")?;

        assert_eq!(params[3], ("csrf".to_string(), "".to_string()));
        Ok(())
    }
}
