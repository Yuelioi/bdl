//! 视频主模块
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/video)
pub mod action;
pub mod appeal;
pub mod attribute_data;
pub mod client;
pub mod collection;
pub mod info;
pub mod interact_video;
pub mod model;
pub mod online;
pub mod params;
pub mod pbp;
pub mod player;
pub mod recommend;
pub mod report;
pub mod snapshot;
pub mod summary;
pub mod tags;
pub mod video_zone;
pub mod video_zone_v2;
pub mod videostream_url;

pub use action::{
    VideoCoinParams, VideoCoinStatusData, VideoCoinStatusParams, VideoFavoriteParams,
    VideoLikeParams,
};
pub use client::VideoClient;
pub use collection::{
    CollectionArchiveSort, CollectionArchivesMutationParams, CollectionCreateAndAddArchivesParams,
    CollectionDeleteSeriesParams, CollectionUpdateSeriesParams,
    VideoCollectionHomeSeasonsSeriesParams, VideoCollectionSeasonsArchivesParams,
    VideoCollectionSeasonsSeriesParams, VideoCollectionSeriesArchivesParams,
    VideoCollectionSeriesInfoParams,
};
pub use model::{VideoDetail, VideoOwner, VideoPage, VideoRelated, VideoStat, VideoTag, VideoView};
pub use params::{
    InteractiveVideoInfoParams, VideoAiSummaryParams, VideoDescParams, VideoDetailParams,
    VideoHomepageRecommendationsParams, VideoId, VideoOnlineTotalParams, VideoPageListParams,
    VideoPlayUrlParams, VideoPlayerInfoParams, VideoRelatedParams, VideoTagsParams,
    VideoViewParams,
};
pub use report::VideoWatchProgressParams;
