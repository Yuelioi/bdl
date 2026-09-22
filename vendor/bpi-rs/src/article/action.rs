// 专栏点赞&投币&收藏
//
// [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/article/action.md)

use crate::BilibiliRequest;
use crate::article::ArticleClient;
use crate::article::params::{ArticleCoinParams, ArticleFavoriteParams, ArticleLikeParams};
use crate::response::BpiResult;

const LIKE_ENDPOINT: &str = "https://api.bilibili.com/x/article/like";
const COIN_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/coin/add";
const FAVORITE_ADD_ENDPOINT: &str = "https://api.bilibili.com/x/article/favorites/add";
const FAVORITE_DEL_ENDPOINT: &str = "https://api.bilibili.com/x/article/favorites/del";

/// 投币响应数据

#[derive(Debug, Clone, serde::Deserialize)]
pub struct CoinResponseData {
    /// 是否点赞成功 true：成功 false：失败 已赞过则附加点赞失败
    pub like: bool,
}

impl<'a> ArticleClient<'a> {
    /// 点赞或取消点赞专栏，并返回标准 payload 结果。
    pub async fn like(&self, params: ArticleLikeParams) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        self.client
            .post(LIKE_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("article.like")
            .await
    }

    /// 给专栏投币并返回标准 payload 结果。
    pub async fn coin(&self, params: ArticleCoinParams) -> BpiResult<CoinResponseData> {
        let csrf = self.client.csrf()?;

        self.client
            .post(COIN_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_payload("article.coin")
            .await
    }

    /// 收藏专栏并返回标准 payload 结果。
    pub async fn favorite(
        &self,
        params: ArticleFavoriteParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        self.client
            .post(FAVORITE_ADD_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("article.favorite")
            .await
    }

    /// 从收藏中移除专栏，并返回标准 payload 结果。
    pub async fn unfavorite(
        &self,
        params: ArticleFavoriteParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        self.client
            .post(FAVORITE_DEL_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("article.unfavorite")
            .await
    }
}

#[cfg(test)]
mod tests {}
