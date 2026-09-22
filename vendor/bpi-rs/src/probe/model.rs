use crate::BpiResult;

pub fn parse_registered_response_model(model: &str, body: &serde_json::Value) -> BpiResult<bool> {
    let bytes = serde_json::to_vec(body)?;
    parse_registered_model(model, &bytes)
}

pub fn parse_registered_model(model: &str, bytes: &[u8]) -> BpiResult<bool> {
    let _ = bytes;
    match model {
        #[cfg(feature = "historytoview")]
        "HistoryListData" => {
            parse_envelope::<crate::historytoview::history::HistoryListData>(bytes)
        }
        #[cfg(feature = "login")]
        "LoginVipInfo" => parse_envelope::<crate::login::LoginVipInfo>(bytes),
        #[cfg(feature = "wallet")]
        "UserWallet" => parse_envelope::<crate::wallet::UserWallet>(bytes),
        #[cfg(feature = "video")]
        "VideoView" => parse_envelope::<crate::video::VideoView>(bytes),
        #[cfg(feature = "vip")]
        "VipCenterData" => parse_envelope::<crate::vip::center::VipCenterData>(bytes),
        _ => Ok(false),
    }
}

#[cfg(any(
    feature = "historytoview",
    feature = "login",
    feature = "wallet",
    feature = "video",
    feature = "vip"
))]
fn parse_envelope<T>(bytes: &[u8]) -> BpiResult<bool>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let envelope = crate::ApiEnvelope::<T>::from_slice(bytes)?;
    if envelope.code == 0 {
        envelope.into_payload()?;
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "historytoview")]
    #[test]
    fn parses_registered_history_model() -> BpiResult<()> {
        let bytes = include_bytes!(
            "../../tests/contracts/historytoview/read/history-list/responses/authenticated.success.json"
        );

        assert!(parse_registered_model("HistoryListData", bytes)?);
        Ok(())
    }

    #[test]
    fn skips_unregistered_model() -> BpiResult<()> {
        assert!(!parse_registered_model("UnknownModel", br#"{}"#)?);
        Ok(())
    }
}
