use crate::article::articles::ArticlesData;
use crate::article::card::CardData;
use crate::article::info::ArticleInfoData;
use crate::article::params::{
    ArticleArticlesInfoParams, ArticleCardsParams, ArticleInfoParams, ArticleViewParams,
};
use crate::article::view::ArticleViewData;
use crate::{BilibiliRequest, BpiClient, BpiResult};

const INFO_ENDPOINT: &str = "https://api.bilibili.com/x/article/viewinfo";
const VIEW_ENDPOINT: &str = "https://api.bilibili.com/x/article/view";
const CARDS_ENDPOINT: &str = "https://api.bilibili.com/x/article/cards";
const ARTICLES_ENDPOINT: &str = "https://api.bilibili.com/x/article/list/web/articles";

/// 专栏 API 客户端。
#[derive(Clone, Copy)]
pub struct ArticleClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> ArticleClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn info_endpoint(&self) -> &'static str {
        INFO_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn view_endpoint(&self) -> &'static str {
        VIEW_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn cards_endpoint(&self) -> &'static str {
        CARDS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn articles_endpoint(&self) -> &'static str {
        ARTICLES_ENDPOINT
    }

    /// 获取专栏概要信息。
    pub async fn info(&self, params: ArticleInfoParams) -> BpiResult<ArticleInfoData> {
        self.client
            .get(INFO_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("article.info")
            .await
    }

    /// 获取专栏内容。
    pub async fn view(&self, params: ArticleViewParams) -> BpiResult<ArticleViewData> {
        let signed_params = self.client.get_wbi_sign2(params.query_pairs()).await?;

        self.client
            .get(VIEW_ENDPOINT)
            .query(&signed_params)
            .send_bpi_payload("article.view")
            .await
    }

    /// 获取专栏内容引用的专栏、视频或直播卡片。
    pub async fn cards(&self, params: ArticleCardsParams) -> BpiResult<CardData> {
        let signed_params = self.client.get_wbi_sign2(params.query_pairs()).await?;

        self.client
            .get(CARDS_ENDPOINT)
            .query(&signed_params)
            .send_bpi_payload("article.cards")
            .await
    }

    /// 获取专栏列表信息。
    pub async fn articles(&self, params: ArticleArticlesInfoParams) -> BpiResult<ArticlesData> {
        self.client
            .get(ARTICLES_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("article.articles_info")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::article::articles::ArticlesData;
    use crate::article::card::CardData;
    use crate::article::info::ArticleInfoData;
    use crate::article::params::{
        ArticleArticlesInfoParams, ArticleCardsParams, ArticleInfoParams, ArticleViewParams,
    };
    use crate::article::view::ArticleViewData;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiResult};

    const TEST_CVID: i64 = 2;
    const TEST_LIST_ID: i64 = 207146;

    fn assert_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ArticleInfoData>>,
    {
    }

    fn assert_view_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ArticleViewData>>,
    {
    }

    fn assert_cards_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<CardData>>,
    {
    }

    fn assert_articles_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ArticlesData>>,
    {
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "info" => include_bytes!("../../tests/contracts/article/info/contract.json").as_slice(),
            "view" => include_bytes!("../../tests/contracts/article/view/contract.json").as_slice(),
            "cards" => {
                include_bytes!("../../tests/contracts/article/cards/contract.json").as_slice()
            }
            "articles" => {
                include_bytes!("../../tests/contracts/article/articles/contract.json").as_slice()
            }
            _ => unreachable!("unknown article contract"),
        };
        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn article_client_exposes_promoted_endpoint_urls() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let article = client.article();

        assert_eq!(
            article.info_endpoint(),
            "https://api.bilibili.com/x/article/viewinfo"
        );
        assert_eq!(
            article.view_endpoint(),
            "https://api.bilibili.com/x/article/view"
        );
        assert_eq!(
            article.cards_endpoint(),
            "https://api.bilibili.com/x/article/cards"
        );
        assert_eq!(
            article.articles_endpoint(),
            "https://api.bilibili.com/x/article/list/web/articles"
        );
        Ok(())
    }

    #[test]
    fn article_methods_return_payload_futures() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let article = client.article();

        assert_info_future(article.info(ArticleInfoParams::new(TEST_CVID)?));
        assert_view_future(article.view(ArticleViewParams::new(TEST_CVID)?));
        assert_cards_future(article.cards(ArticleCardsParams::new("av2,cv1,cv2")?));
        assert_articles_future(article.articles(ArticleArticlesInfoParams::new(TEST_LIST_ID)?));
        Ok(())
    }

    #[test]
    fn article_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let article = client.article();
        let info = contract("info")?;
        let view = contract("view")?;
        let cards = contract("cards")?;
        let articles = contract("articles")?;

        assert_eq!(info.name, "article.info");
        assert_eq!(info.request.method, HttpMethod::Get);
        assert_eq!(info.request.url.as_str(), article.info_endpoint());
        assert_eq!(info.request.query.get("id").map(String::as_str), Some("2"));

        assert_eq!(view.name, "article.view");
        assert_eq!(view.request.method, HttpMethod::Get);
        assert_eq!(view.request.url.as_str(), article.view_endpoint());
        assert!(view.request.auth.requires_wbi());

        assert_eq!(cards.name, "article.cards");
        assert_eq!(cards.request.method, HttpMethod::Get);
        assert_eq!(cards.request.url.as_str(), article.cards_endpoint());
        assert!(cards.request.auth.requires_wbi());

        assert_eq!(articles.name, "article.articles_info");
        assert_eq!(articles.request.method, HttpMethod::Get);
        assert_eq!(articles.request.url.as_str(), article.articles_endpoint());
        assert_eq!(
            articles.request.query.get("id").map(String::as_str),
            Some("207146")
        );
        Ok(())
    }
}
