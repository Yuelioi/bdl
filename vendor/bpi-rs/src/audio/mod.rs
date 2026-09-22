//! 音频投币
//!
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/audio/action.md)
//! # 相关链接
//! 入口: [查看](https://www.bilibili.com/audio/home)
//! 默认歌单: [查看](https://www.bilibili.com/audio/mycollection/19774063)
//! 其他歌单: [查看](https://www.bilibili.com/audio/am125312)
//! 歌曲: [查看](https://www.bilibili.com/audio/au305581)

pub mod action;
pub mod client;
pub mod info;
pub mod music_list;
pub mod musicstream_url;
pub mod params;
pub mod rank;
pub mod status_number;

pub use client::AudioClient;
pub use params::{
    AudioCoinParams, AudioCollectionInfoParams, AudioCollectionToFavParams,
    AudioCollectionToParams, AudioPageParams, AudioRankListParams, AudioRankListType,
    AudioRankPeriodParams, AudioSongParams, AudioStreamUrlParams, AudioStreamUrlWebParams,
};
pub use rank::AudioRankSubscribeParams;
