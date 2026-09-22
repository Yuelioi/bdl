use std::sync::{Arc, Mutex};
use std::time::Duration;

use reqwest::cookie::CookieStore;
use reqwest::header::{COOKIE, HeaderValue, ORIGIN, REFERER, USER_AGENT};
use reqwest::{Client, RequestBuilder, Url};

use crate::BpiError;
#[cfg(feature = "activity")]
use crate::activity::ActivityClient;
#[cfg(feature = "article")]
use crate::article::ArticleClient;
#[cfg(feature = "audio")]
use crate::audio::AudioClient;
#[cfg(feature = "bangumi")]
use crate::bangumi::BangumiClient;
#[cfg(feature = "cheese")]
use crate::cheese::CheeseClient;
#[cfg(feature = "clientinfo")]
use crate::clientinfo::ClientInfoClient;
#[cfg(feature = "comment")]
use crate::comment::CommentClient;
#[cfg(feature = "creativecenter")]
use crate::creativecenter::CreativeCenterClient;
#[cfg(feature = "danmaku")]
use crate::danmaku::DanmakuClient;
#[cfg(feature = "dynamic")]
use crate::dynamic::DynamicClient;
#[cfg(feature = "electric")]
use crate::electric::ElectricClient;
#[cfg(feature = "fav")]
use crate::fav::FavClient;
#[cfg(feature = "historytoview")]
use crate::historytoview::HistoryToViewClient;
#[cfg(feature = "live")]
use crate::live::LiveClient;
#[cfg(feature = "login")]
use crate::login::LoginClient;
#[cfg(feature = "manga")]
use crate::manga::MangaClient;
#[cfg(feature = "message")]
use crate::message::MessageClient;
#[cfg(feature = "misc")]
use crate::misc::MiscClient;
#[cfg(feature = "note")]
use crate::note::NoteClient;
#[cfg(feature = "opus")]
use crate::opus::OpusClient;
#[cfg(feature = "search")]
use crate::search::SearchClient;
use crate::session::Account;
use crate::session::cookie::{format_cookie_pairs, parse_cookie_header as parse_cookie_pairs};
use crate::sign::wbi::WbiKeyCache;
#[cfg(feature = "user")]
use crate::user::UserClient;
#[cfg(feature = "video")]
use crate::video::VideoClient;
#[cfg(feature = "video_ranking")]
use crate::video_ranking::VideoRankingClient;
#[cfg(feature = "vip")]
use crate::vip::VipClient;
#[cfg(feature = "wallet")]
use crate::wallet::WalletClient;
#[cfg(feature = "web_widget")]
use crate::web_widget::WebWidgetClient;

const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
    AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
const DEFAULT_REFERER: &str = "https://www.bilibili.com/";
const DEFAULT_ORIGIN: &str = "https://www.bilibili.com";
const BILIBILI_URL: &str = "https://www.bilibili.com";
const API_BILIBILI_URL: &str = "https://api.bilibili.com";
const AUTH_COOKIE_NAMES: &[&str] = &[
    "DedeUserID",
    "DedeUserID__ckMd5",
    "SESSDATA",
    "bili_jct",
    "buvid3",
];

/// 在构造前配置 [`BpiClient`]。
#[derive(Debug)]
pub struct BpiClientBuilder {
    timeout: Duration,
    connect_timeout: Duration,
    user_agent: String,
    referer: String,
    origin: String,
    no_proxy: bool,
    proxies: Vec<reqwest::Proxy>,
    cookie: Option<String>,
    account: Option<Account>,
    reqwest_client: Option<Client>,
}

impl Default for BpiClientBuilder {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(10),
            connect_timeout: Duration::from_secs(10),
            user_agent: DEFAULT_USER_AGENT.to_string(),
            referer: DEFAULT_REFERER.to_string(),
            origin: DEFAULT_ORIGIN.to_string(),
            no_proxy: true,
            proxies: Vec::new(),
            cookie: None,
            account: None,
            reqwest_client: None,
        }
    }
}

impl BpiClientBuilder {
    /// 设置请求总超时时间。
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 设置 TCP 连接超时时间。
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// 设置 [`BpiClient::get`] 和 [`BpiClient::post`] 使用的默认 user-agent。
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// 设置 [`BpiClient::get`] 和 [`BpiClient::post`] 使用的默认 referer 请求头。
    pub fn referer(mut self, referer: impl Into<String>) -> Self {
        self.referer = referer.into();
        self
    }

    /// 设置 [`BpiClient::get`] 和 [`BpiClient::post`] 使用的默认 origin 请求头。
    pub fn origin(mut self, origin: impl Into<String>) -> Self {
        self.origin = origin.into();
        self
    }

    /// 控制 reqwest 是否绕过系统代理。
    pub fn no_proxy(mut self, enabled: bool) -> Self {
        self.no_proxy = enabled;
        self
    }

    /// 向 reqwest client builder 添加显式代理。
    pub fn proxy(mut self, proxy: reqwest::Proxy) -> Self {
        self.proxies.push(proxy);
        self
    }

    /// 从原始 Cookie 请求头字符串初始化客户端会话。
    pub fn cookie(mut self, cookie: impl Into<String>) -> Self {
        self.cookie = Some(cookie.into());
        self
    }

    /// 从结构化账号值初始化客户端会话。
    pub fn account(mut self, account: Account) -> Self {
        self.account = Some(account);
        self
    }

    /// 使用外部配置的 reqwest client。
    pub fn reqwest_client(mut self, client: Client) -> Self {
        self.reqwest_client = Some(client);
        self
    }

    /// 构建客户端，不读取文件、不初始化全局日志，也不使用共享状态。
    pub fn build(self) -> Result<BpiClient, BpiError> {
        let jar = Arc::new(reqwest::cookie::Jar::default());
        let mut account = None;
        let mut cookie_header = None;

        if let Some(cookie) = self.cookie {
            let pairs = parse_cookie_pairs(&cookie)?;
            add_cookie_pairs(&jar, &pairs);
            cookie_header = Some(format_cookie_pairs(&pairs));

            let cookie_account = Account::from_cookie_pairs(&pairs);
            if cookie_account.is_complete() {
                account = Some(cookie_account);
            }
        }

        if let Some(configured_account) = self.account {
            configured_account.validate_complete()?;
            let pairs = configured_account.cookie_pairs();
            add_cookie_pairs(&jar, &pairs);
            cookie_header = Some(format_cookie_pairs(&pairs));
            account = Some(configured_account);
        }

        let client = match self.reqwest_client {
            Some(client) => client,
            None => {
                let mut builder = Client::builder()
                    .timeout(self.timeout)
                    .connect_timeout(self.connect_timeout)
                    .gzip(true)
                    .deflate(true)
                    .brotli(true)
                    .cookie_provider(jar.clone())
                    .pool_max_idle_per_host(0);

                if self.no_proxy {
                    builder = builder.no_proxy();
                }

                for proxy in self.proxies {
                    builder = builder.proxy(proxy);
                }

                builder.build()?
            }
        };

        Ok(BpiClient {
            client,
            jar,
            account: Mutex::new(account),
            user_agent: validate_header("user_agent", &self.user_agent)?,
            referer: validate_header("referer", &self.referer)?,
            origin: validate_header("origin", &self.origin)?,
            cookie_header: Mutex::new(cookie_header),
            wbi_key_cache: WbiKeyCache::default(),
        })
    }
}

/// Bilibili API 客户端。
pub struct BpiClient {
    client: Client,
    jar: Arc<reqwest::cookie::Jar>,
    account: Mutex<Option<Account>>,
    user_agent: HeaderValue,
    referer: HeaderValue,
    origin: HeaderValue,
    cookie_header: Mutex<Option<String>>,
    wbi_key_cache: WbiKeyCache,
}

impl BpiClient {
    /// 使用默认配置创建自有客户端。
    pub fn new() -> Result<Self, BpiError> {
        Self::builder().build()
    }

    /// 开始配置客户端。
    pub fn builder() -> BpiClientBuilder {
        BpiClientBuilder::default()
    }

    /// 设置账号信息并更新此客户端的 cookie 状态。
    pub fn set_account(&self, account: Account) -> Result<(), BpiError> {
        account.validate_complete()?;

        let pairs = account.cookie_pairs();
        add_cookie_pairs(&self.jar, &pairs);
        *self
            .cookie_header
            .lock()
            .expect("cookie header mutex poisoned") = Some(format_cookie_pairs(&pairs));
        *self.account.lock().expect("account mutex poisoned") = Some(account);
        tracing::info!("Bilibili account configured");
        Ok(())
    }

    /// 清除此客户端的账号信息。
    pub fn clear_account(&self) {
        *self.account.lock().expect("account mutex poisoned") = None;
        *self
            .cookie_header
            .lock()
            .expect("cookie header mutex poisoned") = None;
        expire_auth_cookies(&self.jar);
        tracing::info!("Bilibili account cleared");
    }

    /// 从原始 Cookie 请求头字符串设置账号信息。
    pub fn set_account_from_cookie_str(&self, cookie_str: &str) -> Result<(), BpiError> {
        let pairs = parse_cookie_pairs(cookie_str)?;
        let account = Account::from_cookie_pairs(&pairs);
        account.validate_complete()?;

        add_cookie_pairs(&self.jar, &pairs);
        *self
            .cookie_header
            .lock()
            .expect("cookie header mutex poisoned") = Some(format_cookie_pairs(&pairs));
        *self.account.lock().expect("account mutex poisoned") = Some(account);
        Ok(())
    }

    /// 检查此客户端是否有登录 cookie。
    pub fn has_login_cookies(&self) -> bool {
        if self
            .cookie_header
            .lock()
            .expect("cookie header mutex poisoned")
            .as_deref()
            .is_some_and(contains_login_cookie)
        {
            return true;
        }

        let url = Url::parse(API_BILIBILI_URL).expect("static Bilibili API URL is valid");
        self.jar
            .cookies(&url)
            .and_then(|cookies| cookies.to_str().ok().map(contains_login_cookie))
            .unwrap_or(false)
    }

    /// 返回当前账号信息。
    pub fn get_account(&self) -> Option<Account> {
        self.account.lock().expect("account mutex poisoned").clone()
    }

    /// 从账号信息获取当前 CSRF token。
    pub fn csrf(&self) -> Result<String, BpiError> {
        let account = self.account.lock().expect("account mutex poisoned");
        let account = account.as_ref().ok_or_else(BpiError::auth_required)?;

        account.csrf().map(str::to_owned)
    }

    /// 使用此客户端默认的 Bilibili 请求头创建 GET 请求。
    pub fn get(&self, url: &str) -> RequestBuilder {
        self.apply_default_headers(url, self.client.get(url))
    }

    /// 创建保留原始压缩响应字节的 GET 请求。
    #[cfg(feature = "danmaku")]
    pub(crate) fn get_without_response_decoding(
        &self,
        url: &str,
    ) -> Result<RequestBuilder, BpiError> {
        let client = Client::builder()
            .no_gzip()
            .no_brotli()
            .no_deflate()
            .no_proxy()
            .cookie_provider(self.jar.clone())
            .pool_max_idle_per_host(0)
            .build()?;

        Ok(self.apply_default_headers(url, client.get(url)))
    }

    /// 使用此客户端默认的 Bilibili 请求头创建 POST 请求。
    pub fn post(&self, url: &str) -> RequestBuilder {
        self.apply_default_headers(url, self.client.post(url))
    }

    fn apply_default_headers(&self, url: &str, builder: RequestBuilder) -> RequestBuilder {
        let builder = builder
            .header(USER_AGENT, self.user_agent.clone())
            .header(REFERER, self.referer.clone())
            .header(ORIGIN, self.origin.clone());

        if !is_bilibili_url(url) {
            return builder;
        }

        match self
            .cookie_header
            .lock()
            .expect("cookie header mutex poisoned")
            .as_ref()
        {
            Some(cookie_header) => builder.header(COOKIE, cookie_header),
            None => builder,
        }
    }

    pub(crate) fn wbi_key_cache(&self) -> &WbiKeyCache {
        &self.wbi_key_cache
    }

    #[cfg(test)]
    fn cookie_header_for_test(&self) -> Option<String> {
        self.cookie_header
            .lock()
            .expect("cookie header mutex poisoned")
            .clone()
    }

    #[cfg(test)]
    fn insert_wbi_keys_for_test(
        &self,
        bucket: impl Into<String>,
        keys: crate::sign::wbi::WbiKeys,
    ) -> Result<(), BpiError> {
        self.wbi_key_cache.insert(bucket, keys)
    }

    #[cfg(test)]
    fn wbi_keys_for_test(
        &self,
        bucket: &str,
    ) -> Result<Option<crate::sign::wbi::WbiKeys>, BpiError> {
        self.wbi_key_cache.get(bucket)
    }
}

impl BpiClient {
    /// 创建 activity 领域客户端。
    #[cfg(feature = "activity")]
    pub fn activity(&self) -> ActivityClient<'_> {
        ActivityClient::new(self)
    }

    /// 创建 article 领域客户端。
    #[cfg(feature = "article")]
    pub fn article(&self) -> ArticleClient<'_> {
        ArticleClient::new(self)
    }

    /// 创建 audio 领域客户端。
    #[cfg(feature = "audio")]
    pub fn audio(&self) -> AudioClient<'_> {
        AudioClient::new(self)
    }

    /// 创建 bangumi 领域客户端。
    #[cfg(feature = "bangumi")]
    pub fn bangumi(&self) -> BangumiClient<'_> {
        BangumiClient::new(self)
    }

    /// 创建 cheese 课程领域客户端。
    #[cfg(feature = "cheese")]
    pub fn cheese(&self) -> CheeseClient<'_> {
        CheeseClient::new(self)
    }

    /// 创建 client info 领域客户端。
    #[cfg(feature = "clientinfo")]
    pub fn clientinfo(&self) -> ClientInfoClient<'_> {
        ClientInfoClient::new(self)
    }

    /// 创建 comment 领域客户端。
    #[cfg(feature = "comment")]
    pub fn comment(&self) -> CommentClient<'_> {
        CommentClient::new(self)
    }

    /// 创建 creative center 领域客户端。
    #[cfg(feature = "creativecenter")]
    pub fn creativecenter(&self) -> CreativeCenterClient<'_> {
        CreativeCenterClient::new(self)
    }

    /// 创建 danmaku 领域客户端。
    #[cfg(feature = "danmaku")]
    pub fn danmaku(&self) -> DanmakuClient<'_> {
        DanmakuClient::new(self)
    }

    /// 创建 dynamic 领域客户端。
    #[cfg(feature = "dynamic")]
    pub fn dynamic(&self) -> DynamicClient<'_> {
        DynamicClient::new(self)
    }

    /// 创建 electric charging 领域客户端。
    #[cfg(feature = "electric")]
    pub fn electric(&self) -> ElectricClient<'_> {
        ElectricClient::new(self)
    }

    /// 创建 favorite 领域客户端。
    #[cfg(feature = "fav")]
    pub fn fav(&self) -> FavClient<'_> {
        FavClient::new(self)
    }

    /// 创建 history and to-view 领域客户端。
    #[cfg(feature = "historytoview")]
    pub fn historytoview(&self) -> HistoryToViewClient<'_> {
        HistoryToViewClient::new(self)
    }

    /// 创建 login 领域客户端。
    #[cfg(feature = "login")]
    pub fn login(&self) -> LoginClient<'_> {
        LoginClient::new(self)
    }

    /// 创建 live 领域客户端。
    #[cfg(feature = "live")]
    pub fn live(&self) -> LiveClient<'_> {
        LiveClient::new(self)
    }

    /// 创建 manga 领域客户端。
    #[cfg(feature = "manga")]
    pub fn manga(&self) -> MangaClient<'_> {
        MangaClient::new(self)
    }

    /// 创建 misc 领域客户端。
    #[cfg(feature = "misc")]
    pub fn misc(&self) -> MiscClient<'_> {
        MiscClient::new(self)
    }

    /// 创建 message 领域客户端。
    #[cfg(feature = "message")]
    pub fn message(&self) -> MessageClient<'_> {
        MessageClient::new(self)
    }

    /// 创建 note 领域客户端。
    #[cfg(feature = "note")]
    pub fn note(&self) -> NoteClient<'_> {
        NoteClient::new(self)
    }

    /// 创建 opus 领域客户端。
    #[cfg(feature = "opus")]
    pub fn opus(&self) -> OpusClient<'_> {
        OpusClient::new(self)
    }

    /// 创建 search 领域客户端。
    #[cfg(feature = "search")]
    pub fn search(&self) -> SearchClient<'_> {
        SearchClient::new(self)
    }

    /// 创建 video 领域客户端。
    #[cfg(feature = "video")]
    pub fn video(&self) -> VideoClient<'_> {
        VideoClient::new(self)
    }

    /// 创建 video ranking 领域客户端。
    #[cfg(feature = "video_ranking")]
    pub fn video_ranking(&self) -> VideoRankingClient<'_> {
        VideoRankingClient::new(self)
    }

    /// 创建 VIP 领域客户端。
    #[cfg(feature = "vip")]
    pub fn vip(&self) -> VipClient<'_> {
        VipClient::new(self)
    }

    /// 创建 wallet 领域客户端。
    #[cfg(feature = "wallet")]
    pub fn wallet(&self) -> WalletClient<'_> {
        WalletClient::new(self)
    }

    /// 创建 user 领域客户端。
    #[cfg(feature = "user")]
    pub fn user(&self) -> UserClient<'_> {
        UserClient::new(self)
    }

    /// 创建 web widget 领域客户端。
    #[cfg(feature = "web_widget")]
    pub fn web_widget(&self) -> WebWidgetClient<'_> {
        WebWidgetClient::new(self)
    }

    /// 从结构化账号配置创建客户端。
    pub fn from_config(config: &Account) -> Result<Self, BpiError> {
        Self::builder().account(config.clone()).build()
    }
}

fn add_cookie_pairs(jar: &reqwest::cookie::Jar, pairs: &[(String, String)]) {
    let url = Url::parse(BILIBILI_URL).expect("static Bilibili URL is valid");
    for (key, value) in pairs {
        let cookie = format!("{key}={value}; Domain=.bilibili.com; Path=/");
        jar.add_cookie_str(&cookie, &url);
    }
}

fn expire_auth_cookies(jar: &reqwest::cookie::Jar) {
    let url = Url::parse(BILIBILI_URL).expect("static Bilibili URL is valid");
    for key in AUTH_COOKIE_NAMES {
        let cookie = format!(
            "{key}=; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT; Domain=.bilibili.com; Path=/"
        );
        jar.add_cookie_str(&cookie, &url);
    }
}

fn contains_login_cookie(cookie_header: &str) -> bool {
    parse_cookie_pairs(cookie_header)
        .map(|pairs| {
            pairs
                .iter()
                .any(|(key, value)| key.eq_ignore_ascii_case("SESSDATA") && !value.is_empty())
        })
        .unwrap_or(false)
}

fn validate_header(field: &'static str, value: &str) -> Result<HeaderValue, BpiError> {
    HeaderValue::from_str(value)
        .map_err(|_| BpiError::invalid_parameter(field, "invalid header value"))
}

fn is_bilibili_url(url: &str) -> bool {
    Url::parse(url)
        .ok()
        .and_then(|url| url.host_str().map(|host| host.ends_with("bilibili.com")))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_creates_owned_client_without_account_side_effects() -> Result<(), BpiError> {
        let client = BpiClient::builder().build()?;

        assert!(client.get_account().is_none());
        Ok(())
    }

    #[test]
    fn builder_keeps_cookie_state_isolated_between_clients() -> Result<(), BpiError> {
        let first = BpiClient::builder().cookie("SESSDATA=first").build()?;
        let second = BpiClient::builder().cookie("SESSDATA=second").build()?;

        assert_ne!(
            first.cookie_header_for_test(),
            second.cookie_header_for_test()
        );
        Ok(())
    }

    #[test]
    fn clear_account_removes_login_cookie_state() -> Result<(), BpiError> {
        let client = BpiClient::builder()
            .cookie("DedeUserID=42; SESSDATA=session; bili_jct=csrf; buvid3=buvid")
            .build()?;
        assert!(client.has_login_cookies());

        client.clear_account();

        assert!(!client.has_login_cookies());
        assert!(client.cookie_header_for_test().is_none());
        assert!(client.get_account().is_none());
        Ok(())
    }

    #[test]
    fn has_login_cookies_requires_sessdata_cookie() -> Result<(), BpiError> {
        let client = BpiClient::builder().cookie("buvid3=buvid").build()?;

        assert!(!client.has_login_cookies());
        Ok(())
    }

    #[test]
    fn cookie_string_preserves_dede_user_id_ckmd5_cookie() -> Result<(), BpiError> {
        let client = BpiClient::builder()
            .cookie(
                "DedeUserID=42; DedeUserID__ckMd5=ck; SESSDATA=session; bili_jct=csrf; buvid3=buvid",
            )
            .build()?;

        let cookie_header = client
            .cookie_header_for_test()
            .ok_or_else(|| BpiError::unsupported_response("missing cookie header"))?;

        assert!(cookie_header.contains("DedeUserID__ckMd5=ck"));
        Ok(())
    }

    #[test]
    fn builder_rejects_cookie_strings_without_pairs() {
        let result = BpiClient::builder().cookie("not-a-cookie").build();

        assert!(matches!(
            result,
            Err(BpiError::InvalidParameter {
                field: "cookie",
                ..
            })
        ));
    }

    #[test]
    fn builder_rejects_incomplete_structured_account() {
        let result = BpiClient::builder().account(Account::default()).build();

        assert!(matches!(
            result,
            Err(BpiError::InvalidParameter {
                field: "account",
                ..
            })
        ));
    }

    #[test]
    fn set_account_rejects_incomplete_account_without_replacing_existing_state()
    -> Result<(), BpiError> {
        let client = BpiClient::builder()
            .cookie("DedeUserID=42; SESSDATA=session; bili_jct=csrf; buvid3=buvid")
            .build()?;

        let err = client.set_account(Account::default()).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "account",
                ..
            }
        ));
        assert_eq!(client.csrf()?, "csrf");
        assert!(client.has_login_cookies());
        Ok(())
    }

    #[test]
    fn set_account_from_cookie_str_rejects_incomplete_login_cookie_without_replacing_existing_state()
    -> Result<(), BpiError> {
        let client = BpiClient::builder()
            .cookie("DedeUserID=42; SESSDATA=session; bili_jct=csrf; buvid3=buvid")
            .build()?;

        let err = client
            .set_account_from_cookie_str("buvid3=guest-buvid")
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "account",
                ..
            }
        ));
        assert_eq!(client.csrf()?, "csrf");
        assert!(client.has_login_cookies());
        Ok(())
    }

    #[test]
    fn builder_applies_default_headers_to_requests() -> Result<(), BpiError> {
        let client = BpiClient::builder()
            .user_agent("test-agent")
            .referer("https://example.com/referer")
            .origin("https://example.com")
            .build()?;

        let request = client.get("https://api.bilibili.com/x/test").build()?;

        assert_eq!(request.headers()[USER_AGENT], "test-agent");
        assert_eq!(request.headers()[REFERER], "https://example.com/referer");
        assert_eq!(request.headers()[ORIGIN], "https://example.com");
        Ok(())
    }

    #[cfg(feature = "danmaku")]
    #[test]
    fn raw_response_request_keeps_default_headers_and_cookie() -> Result<(), BpiError> {
        let client = BpiClient::builder()
            .user_agent("test-agent")
            .cookie("DedeUserID=42; SESSDATA=session; bili_jct=csrf; buvid3=buvid")
            .build()?;

        let request = client
            .get_without_response_decoding("https://api.bilibili.com/x/v2/dm/history")?
            .build()?;

        assert_eq!(request.headers()[USER_AGENT], "test-agent");
        assert!(
            request
                .headers()
                .get(COOKIE)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.contains("SESSDATA=session"))
        );
        Ok(())
    }

    #[test]
    fn builder_accepts_explicit_proxy_configuration() -> Result<(), BpiError> {
        let proxy = reqwest::Proxy::http("http://127.0.0.1:8080")?;

        let client = BpiClient::builder().no_proxy(false).proxy(proxy).build()?;

        assert!(client.get_account().is_none());
        Ok(())
    }

    #[test]
    fn builder_keeps_wbi_key_cache_isolated_between_clients() -> Result<(), BpiError> {
        let first = BpiClient::new()?;
        let second = BpiClient::new()?;

        first.insert_wbi_keys_for_test(
            "2026-07-02T10",
            crate::sign::wbi::WbiKeys::new("abcdefghijklmnopqrstuvwxyz123456", "sub-key-a")?,
        )?;
        second.insert_wbi_keys_for_test(
            "2026-07-02T10",
            crate::sign::wbi::WbiKeys::new("ABCDEFGHIJKLMNOPQRSTUVWXYZ654321", "sub-key-b")?,
        )?;

        assert_ne!(
            first.wbi_keys_for_test("2026-07-02T10")?,
            second.wbi_keys_for_test("2026-07-02T10")?
        );
        Ok(())
    }

    #[cfg(feature = "web_widget")]
    #[test]
    fn web_widget_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _web_widget = client.web_widget();

        Ok(())
    }

    #[cfg(feature = "activity")]
    #[test]
    fn activity_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _activity = client.activity();

        Ok(())
    }

    #[cfg(feature = "audio")]
    #[test]
    fn audio_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _audio = client.audio();

        Ok(())
    }

    #[cfg(feature = "article")]
    #[test]
    fn article_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _article = client.article();

        Ok(())
    }

    #[cfg(feature = "bangumi")]
    #[test]
    fn bangumi_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _bangumi = client.bangumi();

        Ok(())
    }

    #[cfg(feature = "cheese")]
    #[test]
    fn cheese_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _cheese = client.cheese();

        Ok(())
    }

    #[cfg(feature = "wallet")]
    #[test]
    fn wallet_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _wallet = client.wallet();

        Ok(())
    }

    #[cfg(feature = "opus")]
    #[test]
    fn opus_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _opus = client.opus();

        Ok(())
    }

    #[cfg(feature = "misc")]
    #[test]
    fn misc_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _misc = client.misc();

        Ok(())
    }

    #[cfg(feature = "message")]
    #[test]
    fn message_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _message = client.message();

        Ok(())
    }

    #[cfg(feature = "vip")]
    #[test]
    fn vip_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _vip = client.vip();

        Ok(())
    }

    #[cfg(feature = "comment")]
    #[test]
    fn comment_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _comment = client.comment();

        Ok(())
    }

    #[cfg(feature = "creativecenter")]
    #[test]
    fn creativecenter_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _creativecenter = client.creativecenter();

        Ok(())
    }

    #[cfg(feature = "electric")]
    #[test]
    fn electric_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _electric = client.electric();

        Ok(())
    }

    #[cfg(feature = "fav")]
    #[test]
    fn fav_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _fav = client.fav();

        Ok(())
    }

    #[cfg(feature = "historytoview")]
    #[test]
    fn historytoview_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _historytoview = client.historytoview();

        Ok(())
    }

    #[cfg(feature = "manga")]
    #[test]
    fn manga_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _manga = client.manga();

        Ok(())
    }

    #[cfg(feature = "note")]
    #[test]
    fn note_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _note = client.note();

        Ok(())
    }

    #[cfg(feature = "dynamic")]
    #[test]
    fn dynamic_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _dynamic = client.dynamic();

        Ok(())
    }

    #[cfg(feature = "danmaku")]
    #[test]
    fn danmaku_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _danmaku = client.danmaku();

        Ok(())
    }

    #[cfg(feature = "search")]
    #[test]
    fn search_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _search = client.search();

        Ok(())
    }

    #[cfg(feature = "user")]
    #[test]
    fn user_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _user = client.user();

        Ok(())
    }

    #[cfg(feature = "video_ranking")]
    #[test]
    fn video_ranking_domain_client_can_be_created() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        let _video_ranking = client.video_ranking();

        Ok(())
    }

    #[cfg(all(feature = "article", feature = "video", feature = "fav"))]
    #[test]
    fn module_clients_expose_write_capability_futures() -> Result<(), BpiError> {
        let client = BpiClient::new()?;

        std::mem::drop(
            client
                .article()
                .like(crate::article::ArticleLikeParams::new(1, true)?),
        );
        std::mem::drop(
            client
                .video()
                .like(crate::video::VideoLikeParams::from_aid(1, 1)?),
        );
        std::mem::drop(
            client
                .fav()
                .add_folder(crate::fav::FavFolderAddParams::new("title")?.privacy(1)?),
        );

        Ok(())
    }
}
