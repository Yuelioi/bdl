use crate::ids::Mid;
use crate::{BpiError, BpiResult};

/// `/x/safecenter/login_notice` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginNoticeParams {
    mid: Mid,
    buvid: Option<String>,
}

impl LoginNoticeParams {
    pub fn new(mid: Mid) -> Self {
        Self { mid, buvid: None }
    }

    pub fn with_buvid(mut self, buvid: impl Into<String>) -> BpiResult<Self> {
        self.buvid = Some(normalize_non_blank("buvid", buvid.into())?);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut query = vec![("mid", self.mid.to_string())];
        if let Some(buvid) = &self.buvid {
            query.push(("buvid", buvid.clone()));
        }

        query
    }
}

/// `/x/member/web/login/log` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginLogParams {
    jsonp: String,
    web_location: String,
}

impl Default for LoginLogParams {
    fn default() -> Self {
        Self {
            jsonp: "jsonp".to_string(),
            web_location: "333.33".to_string(),
        }
    }
}

impl LoginLogParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_jsonp(mut self, jsonp: impl Into<String>) -> BpiResult<Self> {
        self.jsonp = normalize_non_blank("jsonp", jsonp.into())?;
        Ok(self)
    }

    pub fn with_web_location(mut self, web_location: impl Into<String>) -> BpiResult<Self> {
        self.web_location = normalize_non_blank("web_location", web_location.into())?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("jsonp", self.jsonp.clone()),
            ("web_location", self.web_location.clone()),
        ]
    }
}

/// `/x/passport-login/web/qrcode/poll` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginQrPollParams {
    qrcode_key: String,
}

impl LoginQrPollParams {
    pub fn new(qrcode_key: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            qrcode_key: normalize_non_blank("qrcode_key", qrcode_key.into())?,
        })
    }

    pub(crate) fn query_pairs(&self) -> [(&'static str, String); 1] {
        [("qrcode_key", self.qrcode_key.clone())]
    }
}

fn normalize_non_blank(field: &'static str, value: String) -> BpiResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(BpiError::invalid_parameter(field, "value cannot be blank"));
    }

    Ok(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_notice_params_serializes_mid() -> BpiResult<()> {
        let params = LoginNoticeParams::new(Mid::new(1000001)?);

        assert_eq!(params.query_pairs(), vec![("mid", "1000001".to_string())]);
        Ok(())
    }

    #[test]
    fn login_notice_params_serializes_buvid() -> BpiResult<()> {
        let params = LoginNoticeParams::new(Mid::new(1000001)?).with_buvid("BUVID3")?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("mid", "1000001".to_string()),
                ("buvid", "BUVID3".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn login_notice_params_rejects_blank_buvid() {
        let err = LoginNoticeParams::new(Mid::new(1000001).unwrap())
            .with_buvid("   ")
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "buvid", .. }
        ));
    }

    #[test]
    fn login_log_params_serializes_defaults() {
        let params = LoginLogParams::new();

        assert_eq!(
            params.query_pairs(),
            vec![
                ("jsonp", "jsonp".to_string()),
                ("web_location", "333.33".to_string()),
            ]
        );
    }

    #[test]
    fn login_log_params_serializes_custom_location() -> BpiResult<()> {
        let params = LoginLogParams::new().with_web_location("1550101")?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("jsonp", "jsonp".to_string()),
                ("web_location", "1550101".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn login_qr_poll_params_serializes_qrcode_key() -> BpiResult<()> {
        let params = LoginQrPollParams::new("qr-key-123")?;

        assert_eq!(
            params.query_pairs(),
            [("qrcode_key", "qr-key-123".to_string())]
        );
        Ok(())
    }

    #[test]
    fn login_qr_poll_params_rejects_blank_key() {
        let err = LoginQrPollParams::new("   ").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "qrcode_key",
                ..
            }
        ));
    }
}
