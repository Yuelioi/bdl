//! 文集基本信息
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/article/articles.md)

use crate::article::models::{ArticleAuthor, ArticleCategory, ArticleStats};
use serde::{Deserialize, Serialize};

/// 文集基本信息数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticlesData {
    /// 文集概览
    pub list: ArticleList,
    /// 文集内的文章列表
    pub articles: Vec<ArticleItem>,
    /// 文集作者信息
    pub author: ArticleAuthor,
    /// 作用尚不明确 结构与data.articles[]中相似
    pub last: ArticleItem,
    /// 是否关注文集作者 false：未关注 true：已关注 需要登录(Cookie) 未登录为false
    pub attention: bool,
}

/// 文集概览
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleList {
    /// 文集rlid
    pub id: i64,
    /// 文集作者mid
    pub mid: i64,
    /// 文集名称
    pub name: String,
    /// 文集封面图片url
    pub image_url: String,
    /// 文集更新时间 时间戳
    pub update_time: i64,
    /// 文集创建时间 时间戳
    pub ctime: i64,
    /// 文集发布时间 时间戳
    pub publish_time: i64,
    /// 文集简介
    pub summary: String,
    /// 文集字数
    pub words: i64,
    /// 文集阅读量
    pub read: i64,
    /// 文集内文章数量
    pub articles_count: i32,
    /// 1或3 作用尚不明确
    pub state: i32,
    /// 空 作用尚不明确
    pub reason: String,
    /// 空 作用尚不明确
    pub apply_time: String,
    /// 空 作用尚不明确
    pub check_time: String,
}

/// 文章项目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleItem {
    /// 专栏cvid
    pub id: i64,
    /// 文章标题
    pub title: String,
    /// 0 作用尚不明确
    pub state: i32,
    /// 发布时间 秒时间戳
    pub publish_time: i64,
    /// 文章字数
    pub words: i64,
    /// 文章封面
    pub image_urls: Vec<String>,
    /// 文章标签
    pub category: ArticleCategory,
    /// 文章标签列表
    pub categories: Vec<ArticleCategory>,
    /// 文章摘要
    pub summary: String,
    // 文章状态数信息
    pub stats: Option<ArticleStats>,
    /// 是否点赞 0：未点赞 1：已点赞 需要登录(Cookie) 未登录为0
    pub like_state: Option<i32>,
}

/// 作者大会员状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorVip {
    /// 大会员类型
    pub r#type: i32,
    /// 大会员状态
    pub status: i32,
    /// 到期时间
    pub due_date: i64,
    /// 支付类型
    pub vip_pay_type: i32,
    /// 主题类型
    pub theme_type: i32,
    /// 标签
    pub label: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::article::params::ArticleArticlesInfoParams;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    const TEST_LIST_ID: i64 = 207146;

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/article/articles/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_articles_info() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");

        let params = ArticleArticlesInfoParams::new(TEST_LIST_ID)?;

        let data = bpi.article().articles(params).await?;
        tracing::info!("{:#?}", data);

        assert!(!data.list.name.is_empty());
        assert!(!data.articles.is_empty());
        assert!(!data.author.name.is_empty());

        Ok(())
    }

    #[test]
    fn article_articles_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params = ArticleArticlesInfoParams::new(TEST_LIST_ID)?;

        assert_eq!(contract.name, "article.articles_info");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/article/list/web/articles"
        );
        assert_eq!(
            contract.request.query.get("id").map(String::as_str),
            Some("207146")
        );
        assert_eq!(params.query_pairs(), vec![("id", "207146".to_string())]);
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("ArticlesData")
        );
        Ok(())
    }

    #[test]
    fn article_articles_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/article/articles/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!("../../tests/contracts/article/articles/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/article/articles/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<ArticlesData>::from_slice(bytes)?.into_payload()?;

            assert_eq!(payload.list.id, TEST_LIST_ID);
            assert!(!payload.articles.is_empty());
        }
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/article/read/articles/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn article_articles_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<ArticlesData>>(body)?.into_payload()?;

            assert_eq!(payload.list.id, TEST_LIST_ID);
        }
        Ok(())
    }
}
