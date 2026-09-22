//! B站视频合集模块
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/video)
pub mod action;
mod info;
mod params;

pub use action::{
    CollectionArchivesMutationParams, CollectionCreateAndAddArchivesParams,
    CollectionDeleteSeriesParams, CollectionUpdateSeriesParams,
};
pub use info::{
    Archive, ArchiveStat, GetSeasonsArchivesData, GetSeasonsSeriesData, GetSeriesArchivesData,
    GetSeriesData, ItemsList, PageInfo, SeasonsArchivesMeta, SeasonsItem, SeasonsMeta, SeriesItem,
    SeriesMeta,
};
pub(crate) use info::{
    HOME_SEASONS_SERIES_ENDPOINT, SEASONS_ARCHIVES_LIST_ENDPOINT, SEASONS_SERIES_LIST_ENDPOINT,
    SERIES_ARCHIVES_ENDPOINT, SERIES_INFO_ENDPOINT,
};
pub use params::{
    CollectionArchiveSort, VideoCollectionHomeSeasonsSeriesParams,
    VideoCollectionSeasonsArchivesParams, VideoCollectionSeasonsSeriesParams,
    VideoCollectionSeriesArchivesParams, VideoCollectionSeriesInfoParams,
};
