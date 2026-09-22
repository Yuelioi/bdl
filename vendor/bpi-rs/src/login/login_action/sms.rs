use crate::ApiEnvelope;
use crate::BilibiliRequest;
use crate::BpiError;
use crate::BpiResult;
use crate::login::LoginClient;
use reqwest::header::SET_COOKIE;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

#[derive(Debug, Deserialize, Serialize)]
pub struct SMSSendData {
    captcha_key: String, // 短信登录 token
}

#[derive(Debug, Deserialize, Serialize)]
struct SMSLoginData {
    is_new: bool, // 是否为新用户
    status: i32,  // 0:成功
    url: String,  // 跳转url
}

/// 发送 Web SMS 登录验证码的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginSmsCodeParams {
    cid: u32,
    tel: String,
    source: String,
    token: String,
    challenge: String,
    validate: String,
    seccode: String,
}

impl LoginSmsCodeParams {
    /// 创建 SMS 验证码请求参数。
    pub fn new(
        cid: u32,
        tel: impl Into<String>,
        token: impl Into<String>,
        challenge: impl Into<String>,
        validate: impl Into<String>,
        seccode: impl Into<String>,
    ) -> BpiResult<Self> {
        let params = Self {
            cid,
            tel: tel.into(),
            source: "main_web".to_string(),
            token: token.into(),
            challenge: challenge.into(),
            validate: validate.into(),
            seccode: seccode.into(),
        };
        params.validate()?;
        Ok(params)
    }

    /// 设置 Bilibili 登录 source 标记。默认值为 `main_web`。
    pub fn source(mut self, source: impl Into<String>) -> BpiResult<Self> {
        self.source = source.into();
        self.validate()?;
        Ok(self)
    }

    fn validate(&self) -> BpiResult<()> {
        if self.cid == 0 {
            return Err(BpiError::invalid_parameter("cid", "cid must be non-zero"));
        }
        validate_required("tel", &self.tel)?;
        validate_required("source", &self.source)?;
        validate_required("token", &self.token)?;
        validate_required("challenge", &self.challenge)?;
        validate_required("validate", &self.validate)?;
        validate_required("seccode", &self.seccode)?;
        Ok(())
    }

    fn form_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("cid", self.cid.to_string()),
            ("tel", self.tel.clone()),
            ("source", self.source.clone()),
            ("token", self.token.clone()),
            ("challenge", self.challenge.clone()),
            ("validate", self.validate.clone()),
            ("seccode", self.seccode.clone()),
        ]
    }
}

fn validate_required(field: &'static str, value: &str) -> BpiResult<()> {
    if value.trim().is_empty() {
        return Err(BpiError::invalid_parameter(field, "value cannot be blank"));
    }

    Ok(())
}

impl<'a> LoginClient<'a> {
    /// 发送短信验证码（Web端）
    pub async fn send_sms_code(&self, params: LoginSmsCodeParams) -> BpiResult<SMSSendData> {
        self.client
            .post("https://passport.bilibili.com/x/passport-login/web/sms/send")
            .form(&params.form_pairs())
            .send_bpi_payload("login.sms.send")
            .await
    }

    /// 短信登录
    ///
    /// * `cid` - 国际冠字码
    /// * `tel` - 手机号码
    /// * `captcha_key` - 短信登录 token(基于send_sms_code)
    /// * `code` - 短信验证码 5min过期
    pub async fn login_with_sms(
        &self,
        cid: u32,
        tel: u32,
        captcha_key: &str,
        code: &str,
    ) -> Result<(), String> {
        let form = vec![
            ("cid", cid.to_string()),
            ("tel", tel.to_string()),
            ("code", code.to_string()),
            ("source", "main_web".to_string()),
            ("captcha_key", captcha_key.to_string()),
            ("go_url", "https://www.bilibili.com".to_string()),
            ("keep", true.to_string()),
        ];

        let response = self
            .client
            .post("https://passport.bilibili.com/x/passport-login/web/login/sms")
            .form(&form)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if let Some(cookies) = response
            .headers()
            .get_all(SET_COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap_or(""))
            .collect::<Vec<_>>()
            .join("; ")
            .into()
        {
            info!("登录返回的 Cookie: {}", cookies);
        }

        let resp = response
            .json::<ApiEnvelope<SMSLoginData>>()
            .await
            .map_err(|e| {
                error!("解析短信登录响应失败: {:?}", e);
                e.to_string()
            })?;

        if resp.code != 0 {
            error!("短信登录失败: code={}, message={}", resp.code, resp.message);
            return Err(resp.message);
        }

        match resp.code {
            0 => {
                info!("短信登录成功");
                Ok(())
            }
            code => {
                error!("验证码发送失败: code={}, message={}", code, resp.message);
                let msg = match code {
                    -400 => "请求错误".to_string(),
                    1006 => "请输入正确的短信验证码".to_string(),
                    1007 => "短信验证码已过期".to_string(),

                    _ => resp.message,
                };
                Err(msg)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_sms_code_params_rejects_blank_token() {
        let err = LoginSmsCodeParams::new(
            86,
            "13800138000",
            " ",
            "challenge",
            "validate",
            "validate|jordan",
        )
        .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "token", .. }
        ));
    }
}
