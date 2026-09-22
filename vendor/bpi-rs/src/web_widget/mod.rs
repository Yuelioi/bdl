//! web 小组件模块
//!
//! [查看 API 文档](https://socialsisteryi.github.io/bilibili-API-collect/docs/web_widget/)
pub mod banner;
mod client;
pub mod header;
pub mod params;
pub mod zone_upload;

pub use banner::RegionBannerData;
pub use client::WebWidgetClient;
pub use header::HeaderData;
pub use params::{WebWidgetHeaderPageParams, WebWidgetRegionBannerParams};
pub use zone_upload::OnlineData;
