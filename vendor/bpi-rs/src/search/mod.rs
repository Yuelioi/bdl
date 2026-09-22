//! 搜索

pub mod client;
pub mod hot;

mod result;
mod search_params;
pub mod suggest;

mod typed;

pub use client::SearchClient;
pub use result::{
    Article, Bangumi, BiliUser, LiveData, LiveRoom, LiveUser, Movie, SearchData, Video,
};

pub use search_params::{
    CategoryId, Duration, OrderSort, SearchArticleParams, SearchBangumiParams,
    SearchBiliUserParams, SearchLiveParams, SearchLiveRoomParams, SearchLiveUserParams,
    SearchMovieParams, SearchOrder, SearchVideoParams, UserType,
};
