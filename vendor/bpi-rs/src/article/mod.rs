//! 专栏
//!
//! 示例: [https://www.bilibili.com/read/cv1/?jump_opus=1](https://www.bilibili.com/read/cv1/?jump_opus=1)
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/article)

pub mod action;
pub mod articles;
pub mod card;
pub mod category;
mod client;
pub mod info;
mod models;
pub mod params;
pub mod view;

pub use client::ArticleClient;
pub use params::{
    ArticleArticlesInfoParams, ArticleCardsParams, ArticleCoinParams, ArticleFavoriteParams,
    ArticleInfoParams, ArticleLikeParams, ArticleViewParams,
};
