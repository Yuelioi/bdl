//! 观看历史与稍后再看

mod client;
pub mod history;
pub mod params;
pub mod toview;

pub use client::HistoryToViewClient;
pub use params::{
    HistoryBusiness, HistoryDeleteParams, HistoryListParams, HistoryListType,
    HistoryShadowSetParams, ToViewAddParams, ToViewDeleteParams,
};
