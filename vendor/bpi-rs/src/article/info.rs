//! 专栏基本信息
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/article/info.md)

use crate::article::models::ArticleStats;
use serde::{Deserialize, Serialize};

/// 专栏基本信息数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleInfoData {
    /// 是否点赞 0：未点赞 1：已点赞 需要登录(Cookie) 未登录为0
    pub like: i32,
    /// 是否关注文章作者 false：未关注 true：已关注 需要登录(Cookie) 未登录为false
    pub attention: bool,
    /// 是否收藏 false：未收藏 true：已收藏 需要登录(Cookie) 未登录为false
    pub favorite: bool,
    /// 为文章投币数
    pub coin: i32,
    /// 状态数信息
    pub stats: ArticleStats,
    /// 文章标题
    pub title: String,
    /// 文章头图url
    pub banner_url: String,
    /// 文章作者mid
    pub mid: i64,
    /// 文章作者昵称
    pub author_name: String,
    /// true 作用尚不明确
    pub is_author: bool,
    /// 动态封面
    pub image_urls: Vec<String>,
    /// 封面图片
    pub origin_image_urls: Vec<String>,
    /// true 作用尚不明确
    pub shareable: bool,
    /// true 作用尚不明确
    pub show_later_watch: bool,
    /// true 作用尚不明确
    pub show_small_window: bool,
    /// 是否收于文集 false：否 true：是
    pub in_list: bool,
    /// 上一篇文章cvid 无为0
    pub pre: i64,
    /// 下一篇文章cvid 无为0
    pub next: i64,
    /// 分享方式列表
    pub share_channels: Vec<ShareChannel>,
    /// 文章类别 0：文章 2：笔记
    pub r#type: i32,
    /// 视频URL
    #[serde(default)]
    pub video_url: String,
    /// 位置信息
    #[serde(default)]
    pub location: String,
    /// 是否禁用分享
    #[serde(default)]
    pub disable_share: bool,
}

/// 分享方式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareChannel {
    /// 分享名称
    pub name: String,
    /// 分享图片url
    pub picture: String,
    /// 分享代号
    pub share_channel: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::article::params::ArticleInfoParams;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    const TEST_CVID: i64 = 2;

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/article/info/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_article_info() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");

        let params = ArticleInfoParams::new(TEST_CVID)?;

        let data = bpi.article().info(params).await?;
        assert!(!data.title.is_empty());
        assert!(!data.author_name.is_empty());
        assert!(data.mid > 0);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_article_stats() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");

        let params = ArticleInfoParams::new(TEST_CVID)?;
        let data = bpi.article().info(params).await?;
        let stats = &data.stats;
        assert!(stats.view >= 0);
        assert!(stats.favorite >= 0);
        assert!(stats.like >= 0);
        assert!(stats.reply >= 0);

        Ok(())
    }

    #[test]
    fn article_info_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params = ArticleInfoParams::new(TEST_CVID)?;

        assert_eq!(contract.name, "article.info");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/article/viewinfo"
        );
        assert_eq!(
            contract.request.query.get("id").map(String::as_str),
            Some("2")
        );
        assert_eq!(params.query_pairs(), vec![("id", "2".to_string())]);
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("ArticleInfoData")
        );
        Ok(())
    }

    #[test]
    fn article_info_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!("../../tests/contracts/article/info/responses/anonymous.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/article/info/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/article/info/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<ArticleInfoData>::from_slice(bytes)?.into_payload()?;

            assert!(!payload.title.is_empty());
            assert!(!payload.author_name.is_empty());
            assert!(payload.mid > 0);
        }
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/article/read/info/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn article_info_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<ArticleInfoData>>(body)?.into_payload()?;

            assert!(payload.mid > 0);
        }
        Ok(())
    }
}
