//! 漫画
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/manga)

pub mod activity;
pub mod client;
pub mod clockin;
pub mod comic;
pub mod download;
pub mod point_shop;
pub mod season;
pub mod user;

pub use client::MangaClient;
