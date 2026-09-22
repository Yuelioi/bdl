// 退出登录 (Web端)
//
// [文档](https://socialsisteryi.github.io/bilibili-API-collect/docs/login/exit.html#退出登录-web端)

use crate::BilibiliRequest;
use crate::BpiResult;
use crate::login::LoginClient;
use serde::{Deserialize, Serialize};

const LOGOUT_WEB_ENDPOINT: &str = "https://passport.bilibili.com/login/exit/v2";

/// 退出登录成功后的数据体

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutData {
    /// 重定向 URL
    #[serde(rename = "redirectUrl")]
    pub redirect_url: String,
}

/// 退出登录响应结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogoutResponse {
    /// 返回码
    /// - 0：成功
    /// - 2202：csrf 请求非法
    pub code: i32,
    /// 返回状态，成功为 true
    pub status: bool,
    /// 时间戳
    pub ts: u64,
    /// 错误信息（成功时不存在）
    pub message: Option<String>,
    /// 有效时的 data
    pub data: Option<LogoutData>,
}

/// Web 登出的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogoutWebParams {
    gourl: String,
}

impl Default for LogoutWebParams {
    fn default() -> Self {
        Self {
            gourl: "javascript:history.go(-1)".to_string(),
        }
    }
}

impl LogoutWebParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn gourl(mut self, gourl: impl Into<String>) -> Self {
        self.gourl = gourl.into();
        self
    }

    fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("biliCSRF", csrf.to_string()),
            ("gourl", self.gourl.clone()),
        ]
    }
}

impl<'a> LoginClient<'a> {
    /// 登出当前 Web 会话并返回标准 payload 结果。
    pub async fn logout(&self, params: LogoutWebParams) -> BpiResult<LogoutData> {
        let csrf = self.client.csrf()?;
        let form = params.form_pairs(&csrf);

        self.client
            .post(LOGOUT_WEB_ENDPOINT)
            .form(&form)
            .send_bpi_payload("login.logout")
            .await
    }
}

#[cfg(test)]
mod tests {}
