//! 番剧
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/bangumi)
//!
//! # 相关链接 (轻音少女)
//! * 剧集: [查看详情](https://www.bilibili.com/bangumi/play/ep1746689)
//! * 番剧: [查看详情](https://www.bilibili.com/bangumi/play/ss1172)
//! * 媒体: [查看详情](https://www.bilibili.com/bangumi/media/md28220978)
//! * 流: [查看详情](https://api.bilibili.com/pgc/player/web/playurl?qn=127&fnver=0&fnval=12240&fourk=&ep_id=65709)

pub mod client;
pub mod follow;
pub mod info;
pub mod params;
pub mod timeline;
pub mod videostream_url;

pub use client::BangumiClient;
pub use follow::BangumiFollowParams;
pub use params::{
    BangumiDetailId, BangumiDetailParams, BangumiInfoParams, BangumiSectionsParams,
    BangumiTimelineParams, BangumiVideoStreamId, BangumiVideoStreamParams,
};
