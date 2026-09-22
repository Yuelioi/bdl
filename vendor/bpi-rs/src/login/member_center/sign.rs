// 修改签名
//
// [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/login/member_center.md)

use crate::BilibiliRequest;
use crate::BpiError;
use crate::BpiResult;
use crate::login::LoginClient;

const UPDATE_USER_SIGN_ENDPOINT: &str = "https://api.bilibili.com/x/member/web/sign/update";

/// 更新会员中心用户签名的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoginUserSignParams {
    user_sign: String,
}

impl LoginUserSignParams {
    pub fn new(user_sign: impl Into<String>) -> BpiResult<Self> {
        let user_sign = user_sign.into();
        if user_sign.len() > 70 {
            return Err(BpiError::invalid_parameter(
                "user_sign",
                "length cannot exceed 70 bytes",
            ));
        }

        Ok(Self { user_sign })
    }

    fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("user_sign", self.user_sign.clone()),
            ("csrf", csrf.to_string()),
        ]
    }
}

impl<'a> LoginClient<'a> {
    /// 更新会员中心用户签名并返回标准 payload 结果。
    pub async fn update_user_sign(
        &self,
        params: LoginUserSignParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        self.client
            .post(UPDATE_USER_SIGN_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("login.member_center.user_sign.update")
            .await
    }
}
