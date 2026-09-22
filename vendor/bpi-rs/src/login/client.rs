use crate::{ApiEnvelope, BilibiliRequest, BpiClient, BpiError, BpiResult};

use super::login_action::captcha::{GeetestData, GenerateCaptcha};
use super::login_action::qr::{CheckQrCodeStatusData, GenerateQrCodeData};
use super::login_notice::{LoginLogData, LoginNoticeData};
use super::model::{
    LoginAccountInfo, LoginCoinBalance, LoginDailyReward, LoginNav, LoginStats, LoginTodayCoinExp,
    LoginVipInfo,
};
use super::params::{LoginLogParams, LoginNoticeParams, LoginQrPollParams};

const NAV_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/nav";
const STAT_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/nav/stat";
const COIN_ENDPOINT: &str = "https://account.bilibili.com/site/getCoin";
const TODAY_COIN_EXP_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/coin/today/exp";
const DAILY_REWARD_ENDPOINT: &str = "https://api.bilibili.com/x/member/web/exp/reward";
const ACCOUNT_INFO_ENDPOINT: &str = "https://api.bilibili.com/x/member/web/account";
const VIP_INFO_ENDPOINT: &str = "https://api.bilibili.com/x/vip/web/user/info";
const NOTICE_ENDPOINT: &str = "https://api.bilibili.com/x/safecenter/login_notice";
const LOG_ENDPOINT: &str = "https://api.bilibili.com/x/member/web/login/log";
const CAPTCHA_GENERATE_ENDPOINT: &str = "https://passport.bilibili.com/x/passport-login/captcha";
const CAPTCHA_SOURCE: &str = "main_web";
const QR_GENERATE_ENDPOINT: &str =
    "https://passport.bilibili.com/x/passport-login/web/qrcode/generate";
const QR_POLL_ENDPOINT: &str = "https://passport.bilibili.com/x/passport-login/web/qrcode/poll";

/// 登录领域 API 客户端。
#[derive(Clone, Copy)]
pub struct LoginClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> LoginClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn nav_endpoint(&self) -> &'static str {
        NAV_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn stat_endpoint(&self) -> &'static str {
        STAT_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn coin_endpoint(&self) -> &'static str {
        COIN_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn today_coin_exp_endpoint(&self) -> &'static str {
        TODAY_COIN_EXP_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn daily_reward_endpoint(&self) -> &'static str {
        DAILY_REWARD_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn account_info_endpoint(&self) -> &'static str {
        ACCOUNT_INFO_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn vip_info_endpoint(&self) -> &'static str {
        VIP_INFO_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn notice_endpoint(&self) -> &'static str {
        NOTICE_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn log_endpoint(&self) -> &'static str {
        LOG_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn captcha_generate_endpoint(&self) -> &'static str {
        CAPTCHA_GENERATE_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn qr_generate_endpoint(&self) -> &'static str {
        QR_GENERATE_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn qr_poll_endpoint(&self) -> &'static str {
        QR_POLL_ENDPOINT
    }

    /// 获取当前会话的导航/登录状态。
    pub async fn nav(&self) -> BpiResult<LoginNav> {
        self.client
            .get(NAV_ENDPOINT)
            .send_bpi_payload("login.nav")
            .await
    }

    /// 获取当前已认证用户的关注数、粉丝数和动态数。
    pub async fn stat(&self) -> BpiResult<LoginStats> {
        self.client
            .get(STAT_ENDPOINT)
            .send_bpi_payload("login.stat")
            .await
    }

    /// 获取当前已认证账号的硬币余额。
    pub async fn coin(&self) -> BpiResult<LoginCoinBalance> {
        self.client
            .get(COIN_ENDPOINT)
            .send_bpi_payload("login.coin")
            .await
    }

    /// 获取今天通过投币操作获得的经验值。
    pub async fn today_coin_exp(&self) -> BpiResult<LoginTodayCoinExp> {
        self.client
            .get(TODAY_COIN_EXP_ENDPOINT)
            .send_bpi_payload("login.today_coin_exp")
            .await
    }

    /// 获取当前已认证账号的每日奖励完成状态。
    pub async fn daily_reward(&self) -> BpiResult<LoginDailyReward> {
        self.client
            .get(DAILY_REWARD_ENDPOINT)
            .send_bpi_payload("login.daily_reward")
            .await
    }

    /// 获取当前已认证账号的个人资料。
    pub async fn account_info(&self) -> BpiResult<LoginAccountInfo> {
        self.client
            .get(ACCOUNT_INFO_ENDPOINT)
            .send_bpi_payload("login.account_info")
            .await
    }

    /// 获取当前已认证账号的 VIP 状态。
    pub async fn vip_info(&self) -> BpiResult<LoginVipInfo> {
        self.client
            .get(VIP_INFO_ENDPOINT)
            .send_bpi_payload("login.vip_info")
            .await
    }

    /// 获取已认证账号的指定登录通知。
    pub async fn notice(&self, params: LoginNoticeParams) -> BpiResult<LoginNoticeData> {
        self.client
            .get(NOTICE_ENDPOINT)
            .with_bilibili_headers()
            .query(&params.query_pairs())
            .send_bpi_payload("login.notice")
            .await
    }

    /// 获取已认证账号的近期登录日志条目。
    pub async fn log(&self, params: LoginLogParams) -> BpiResult<LoginLogData> {
        self.client
            .get(LOG_ENDPOINT)
            .with_bilibili_headers()
            .query(&params.query_pairs())
            .send_bpi_payload("login.log")
            .await
    }

    /// 为登录流程生成 Geetest 验证码挑战。
    pub async fn generate_captcha(&self) -> BpiResult<GenerateCaptcha> {
        let data: GeetestData = self
            .client
            .get(CAPTCHA_GENERATE_ENDPOINT)
            .with_bilibili_headers()
            .query(&[("source", CAPTCHA_SOURCE)])
            .send_bpi_payload("login.captcha_generate")
            .await?;
        let geetest = data.geetest;

        Ok(GenerateCaptcha {
            token: data.token,
            gt: geetest.gt,
            challenge: geetest.challenge,
        })
    }

    /// 生成 QR 登录 URL 和临时轮询 key。
    pub async fn qr_generate(&self) -> BpiResult<GenerateQrCodeData> {
        self.client
            .get(QR_GENERATE_ENDPOINT)
            .with_bilibili_headers()
            .send_bpi_payload("login.qr_generate")
            .await
    }

    /// 轮询 QR 登录状态。
    pub async fn qr_poll(&self, params: LoginQrPollParams) -> BpiResult<CheckQrCodeStatusData> {
        let response = self
            .client
            .get(QR_POLL_ENDPOINT)
            .with_bilibili_headers()
            .query(&params.query_pairs())
            .send()
            .await?;

        let cookies: Vec<(String, String)> = response
            .cookies()
            .map(|cookie| (cookie.name().to_string(), cookie.value().to_string()))
            .collect();
        let envelope: ApiEnvelope<CheckQrCodeStatusData> = response
            .json()
            .await
            .map_err(|err| BpiError::parse(err.to_string()))?;
        let mut data = envelope.into_data()?;

        if data.code == 0 {
            data.cookies = cookies;
        }

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use super::DAILY_REWARD_ENDPOINT;

    use crate::BpiClient;
    use crate::ids::Mid;
    use crate::login::login_action::captcha::GenerateCaptcha;
    use crate::login::login_action::qr::{CheckQrCodeStatusData, GenerateQrCodeData};
    use crate::login::login_notice::{LoginLogData, LoginNoticeData};
    use crate::login::{LoginLogParams, LoginNoticeParams, LoginQrPollParams};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiError, BpiResult};

    const READ_INFO_CONTRACTS: &[(&str, &str, &str)] = &[
        (
            "account-info",
            "login.account_info",
            "https://api.bilibili.com/x/member/web/account",
        ),
        (
            "coin",
            "login.coin",
            "https://account.bilibili.com/site/getCoin",
        ),
        (
            "nav",
            "login.nav",
            "https://api.bilibili.com/x/web-interface/nav",
        ),
        (
            "stat",
            "login.stat",
            "https://api.bilibili.com/x/web-interface/nav/stat",
        ),
        (
            "today-coin-exp",
            "login.today_coin_exp",
            "https://api.bilibili.com/x/web-interface/coin/today/exp",
        ),
    ];

    fn endpoint_contract(endpoint: &str) -> Result<EndpointContract, Box<dyn std::error::Error>> {
        let path = format!("tests/contracts/login/{endpoint}/contract.json");
        let bytes = std::fs::read(path)?;
        Ok(EndpointContract::from_slice(&bytes)?)
    }

    fn read_info_contract(endpoint: &str) -> Result<EndpointContract, Box<dyn std::error::Error>> {
        endpoint_contract(endpoint)
    }

    fn nested_contract(path: &str) -> Result<EndpointContract, BpiError> {
        let bytes = match path {
            "notice/login-notice" => {
                include_bytes!("../../tests/contracts/login/notice/login-notice/contract.json")
                    .as_slice()
            }
            "notice/login-log" => {
                include_bytes!("../../tests/contracts/login/notice/login-log/contract.json")
                    .as_slice()
            }
            "captcha/generate" => {
                include_bytes!("../../tests/contracts/login/captcha/generate/contract.json")
                    .as_slice()
            }
            "qr/generate" => {
                include_bytes!("../../tests/contracts/login/qr/generate/contract.json").as_slice()
            }
            "qr/poll" => {
                include_bytes!("../../tests/contracts/login/qr/poll/contract.json").as_slice()
            }
            _ => unreachable!("unknown login nested contract"),
        };

        EndpointContract::from_slice(bytes)
    }

    fn assert_notice_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<LoginNoticeData>>,
    {
    }

    fn assert_log_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<LoginLogData>>,
    {
    }

    fn assert_captcha_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<GenerateCaptcha>>,
    {
    }

    fn assert_qr_generate_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<GenerateQrCodeData>>,
    {
    }

    fn assert_qr_poll_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<CheckQrCodeStatusData>>,
    {
    }

    #[test]
    fn login_client_borrows_root_client() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let login = client.login();

        assert_eq!(
            login.nav_endpoint(),
            "https://api.bilibili.com/x/web-interface/nav"
        );
        Ok(())
    }

    #[test]
    fn login_client_exposes_stat_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let login = client.login();

        assert_eq!(
            login.stat_endpoint(),
            "https://api.bilibili.com/x/web-interface/nav/stat"
        );
        Ok(())
    }

    #[test]
    fn login_client_exposes_coin_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let login = client.login();

        assert_eq!(
            login.coin_endpoint(),
            "https://account.bilibili.com/site/getCoin"
        );
        Ok(())
    }

    #[test]
    fn login_client_exposes_today_coin_exp_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let login = client.login();

        assert_eq!(
            login.today_coin_exp_endpoint(),
            "https://api.bilibili.com/x/web-interface/coin/today/exp"
        );
        Ok(())
    }

    #[test]
    fn login_client_exposes_daily_reward_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let login = client.login();

        assert_eq!(
            login.daily_reward_endpoint(),
            "https://api.bilibili.com/x/member/web/exp/reward"
        );
        Ok(())
    }

    #[test]
    fn login_client_exposes_account_info_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let login = client.login();

        assert_eq!(
            login.account_info_endpoint(),
            "https://api.bilibili.com/x/member/web/account"
        );
        Ok(())
    }

    #[test]
    fn login_client_exposes_vip_info_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let login = client.login();

        assert_eq!(
            login.vip_info_endpoint(),
            "https://api.bilibili.com/x/vip/web/user/info"
        );
        Ok(())
    }

    #[test]
    fn login_safe_flow_client_methods_return_payload_futures() -> Result<(), BpiError> {
        let client = BpiClient::new()?;
        let login = client.login();

        assert_notice_future(login.notice(LoginNoticeParams::new(Mid::new(1_000_001)?)));
        assert_log_future(login.log(LoginLogParams::new()));
        assert_captcha_future(login.generate_captcha());
        assert_qr_generate_future(login.qr_generate());
        assert_qr_poll_future(login.qr_poll(LoginQrPollParams::new("sanitized-qrcode-key")?));
        Ok(())
    }

    #[test]
    fn login_read_info_contracts_match_endpoint_requests() -> Result<(), Box<dyn std::error::Error>>
    {
        for (endpoint, name, url) in READ_INFO_CONTRACTS {
            let contract = read_info_contract(endpoint)?;

            assert_eq!(contract.name, *name);
            assert_eq!(contract.request.method, HttpMethod::Get);
            assert_eq!(contract.request.url.as_str(), *url);
            assert!(contract.request.query.is_empty());
            assert_eq!(contract.cases.len(), 3);
            assert_eq!(contract.cases[0].response.api_code, Some(-101));
            assert_eq!(contract.cases[1].response.api_code, Some(0));
            assert_eq!(contract.cases[2].response.api_code, Some(0));
        }
        Ok(())
    }

    #[test]
    fn login_read_info_contracts_cover_vip_profile() -> Result<(), Box<dyn std::error::Error>> {
        for (endpoint, _, _) in READ_INFO_CONTRACTS {
            let contract = read_info_contract(endpoint)?;
            let vip = contract
                .cases
                .iter()
                .find(|case| case.name == "vip")
                .ok_or_else(|| {
                    crate::BpiError::unsupported_response("missing vip contract case")
                })?;

            assert_eq!(vip.profile.as_deref(), Some("vip"));
            assert!(vip.auth.requires_cookie());
            assert_eq!(vip.response.api_code, Some(0));
        }
        Ok(())
    }

    #[test]
    fn login_vip_info_contract_matches_endpoint_request() -> Result<(), Box<dyn std::error::Error>>
    {
        let contract = EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/login/vip-info/contract.json"
        ))?;

        assert_eq!(contract.name, "login.vip_info");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/vip/web/user/info"
        );
        assert_eq!(contract.cases.len(), 3);
        Ok(())
    }

    #[test]
    fn login_daily_reward_contract_matches_endpoint_request()
    -> Result<(), Box<dyn std::error::Error>> {
        let contract = endpoint_contract("daily-reward")?;

        assert_eq!(contract.name, "login.daily_reward");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), DAILY_REWARD_ENDPOINT);
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(contract.cases[0].response.http_status, Some(412));
        assert_eq!(contract.cases[1].response.api_code, Some(0));
        assert_eq!(contract.cases[2].response.http_status, Some(412));
        Ok(())
    }

    #[test]
    fn login_safe_flow_contracts_match_module_client_endpoints() -> Result<(), BpiError> {
        let client = BpiClient::new()?;
        let login = client.login();
        let cases = [
            (
                "notice/login-notice",
                "login.notice",
                login.notice_endpoint(),
            ),
            ("notice/login-log", "login.log", login.log_endpoint()),
            (
                "captcha/generate",
                "login.captcha_generate",
                login.captcha_generate_endpoint(),
            ),
            (
                "qr/generate",
                "login.qr_generate",
                login.qr_generate_endpoint(),
            ),
            ("qr/poll", "login.qr_poll", login.qr_poll_endpoint()),
        ];

        for (path, name, url) in cases {
            let contract = nested_contract(path)?;

            assert_eq!(contract.name, name);
            assert_eq!(contract.request.method, HttpMethod::Get);
            assert_eq!(contract.request.url.as_str(), url);
        }
        Ok(())
    }
}
