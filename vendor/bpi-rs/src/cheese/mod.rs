//! 课堂
//!
//! 示例 [https://www.bilibili.com/cheese/play/ss556](https://www.bilibili.com/cheese/play/ss556)
pub mod client;
pub mod info;
pub mod params;
pub mod videostream_url;

pub use client::CheeseClient;
pub use params::{CheeseEpListParams, CheeseInfoId, CheeseInfoParams, CheeseVideoStreamParams};
