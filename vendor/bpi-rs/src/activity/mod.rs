//! 活动
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/activity)

mod client;
pub mod info;
pub mod list;

pub use client::ActivityClient;
pub use info::{ActivityInfoData, ActivityInfoParams};
pub use list::{ActivityItem, ActivityListData, ActivityListParams};
