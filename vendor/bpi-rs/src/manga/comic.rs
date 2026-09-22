// 购买漫画章节
//
// [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/manga/Comic.md)

// ================= 数据结构 =================

use crate::BpiError;
use crate::manga::MangaClient;
use crate::request::send_bpi_envelope;
use crate::response::BpiResponse;
use serde::Serialize;

/// 购买漫画章节请求参数

#[derive(Debug, Clone, Serialize)]
pub struct BuyEpisodeRequest {
    /// 章节id
    #[serde(rename = "epId")]
    pub ep_id: i32,

    /// 购买方式
    /// 2：漫读券
    /// 4：新人等免
    /// 5：通用券
    #[serde(rename = "buyMethod")]
    pub buy_method: i32,

    /// 漫读券id
    #[serde(rename = "couponId")]
    pub coupon_id: i32,

    /// 漫画id，buyMethod=4时必填
    #[serde(rename = "comicId", skip_serializing_if = "Option::is_none")]
    pub comic_id: Option<i32>,

    /// 自动支付状态，buyMethod=2,5时必填
    #[serde(rename = "autoPayGoldStatus", skip_serializing_if = "Option::is_none")]
    pub auto_pay_gold_status: Option<i32>,

    /// 是否预售，buyMethod=2,5时必填
    #[serde(rename = "isPresale", skip_serializing_if = "Option::is_none")]
    pub is_presale: Option<i32>,

    /// 支付金额，buyMethod=5时必填
    #[serde(rename = "payAmount", skip_serializing_if = "Option::is_none")]
    pub pay_amount: Option<i32>,
}

// ================= 实现 =================

impl<'a> MangaClient<'a> {
    /// 购买漫画章节
    ///
    /// [网页入口](https://manga.bilibili.com/twirp/comic.v1.Comic/BuyEpisode)
    pub async fn manga_buy_episode(
        &self,
        request: BuyEpisodeRequest,
    ) -> Result<BpiResponse<serde_json::Value>, BpiError> {
        let request = self
            .client
            .post("https://manga.bilibili.com/twirp/comic.v1.Comic/BuyEpisode?platform=web")
            .json(&request);

        let result = send_bpi_envelope(request, "购买漫画章节").await?;

        Ok(result)
    }

    /// 使用漫读券购买漫画章节
    ///
    /// [网页入口](https://manga.bilibili.com/twirp/comic.v1.Comic/BuyEpisode)
    pub async fn manga_buy_episode_with_coupon(
        &self,
        ep_id: i32,
        coupon_id: i32,
    ) -> Result<BpiResponse<serde_json::Value>, BpiError> {
        let request = BuyEpisodeRequest {
            ep_id,
            buy_method: 2,
            coupon_id,
            comic_id: None,
            auto_pay_gold_status: Some(2),
            is_presale: Some(0),
            pay_amount: None,
        };

        self.manga_buy_episode(request).await
    }

    /// 使用新人等免购买漫画章节
    ///
    /// [网页入口](https://manga.bilibili.com/twirp/comic.v1.Comic/BuyEpisode)
    pub async fn manga_buy_episode_with_free(
        &self,
        comic_id: i32,
        ep_id: i32,
    ) -> Result<BpiResponse<serde_json::Value>, BpiError> {
        let request = BuyEpisodeRequest {
            ep_id,
            buy_method: 4,
            coupon_id: 0,
            comic_id: Some(comic_id),
            auto_pay_gold_status: None,
            is_presale: None,
            pay_amount: None,
        };

        self.manga_buy_episode(request).await
    }

    /// 使用通用券购买漫画章节
    ///
    /// [网页入口](https://manga.bilibili.com/twirp/comic.v1.Comic/BuyEpisode)
    pub async fn manga_buy_episode_with_general_coupon(
        &self,
        ep_id: i32,
        pay_amount: i32,
    ) -> Result<BpiResponse<serde_json::Value>, BpiError> {
        let request = BuyEpisodeRequest {
            ep_id,
            buy_method: 5,
            coupon_id: 0,
            comic_id: None,
            auto_pay_gold_status: Some(2),
            is_presale: Some(0),
            pay_amount: Some(pay_amount),
        };

        self.manga_buy_episode(request).await
    }
}

#[cfg(test)]
mod tests {}
