#[cfg(feature = "activity")]
pub mod activity;

#[cfg(feature = "article")]
pub mod article;

#[cfg(feature = "audio")]
pub mod audio;

#[cfg(feature = "bangumi")]
pub mod bangumi;

#[cfg(feature = "cheese")]
pub mod cheese;

#[cfg(feature = "clientinfo")]
pub mod clientinfo;

#[cfg(feature = "comment")]
pub mod comment;

#[cfg(feature = "creativecenter")]
pub mod creativecenter;

#[cfg(feature = "dynamic")]
pub mod dynamic;

#[cfg(feature = "danmaku")]
pub mod danmaku;

#[cfg(feature = "electric")]
pub mod electric;

#[cfg(feature = "fav")]
pub mod fav;

#[cfg(feature = "historytoview")]
pub mod historytoview;

#[cfg(feature = "live")]
pub mod live;

#[cfg(feature = "login")]
pub mod login;

#[cfg(feature = "manga")]
pub mod manga;

#[cfg(feature = "message")]
pub mod message;

#[cfg(feature = "misc")]
pub mod misc;

#[cfg(feature = "note")]
pub mod note;

#[cfg(feature = "opus")]
pub mod opus;

#[cfg(feature = "search")]
pub mod search;

#[cfg(feature = "user")]
pub mod user;

#[cfg(feature = "video")]
pub mod video;

#[cfg(feature = "video_ranking")]
pub mod video_ranking;

#[cfg(feature = "vip")]
pub mod vip;

#[cfg(feature = "wallet")]
pub mod wallet;

#[cfg(feature = "web_widget")]
pub mod web_widget;

pub mod models;

pub mod client;
pub mod err;
pub mod ids;
pub mod log;
#[doc(hidden)]
pub mod probe;
pub mod request;
pub mod response;
pub mod session;
pub mod sign;
pub mod transport;

#[cfg(test)]
mod testing;

// bv aid互转, 以及生成wbi
pub mod utils;

pub use client::BpiClient;
pub use err::error::BpiError;
pub use request::BilibiliRequest;
pub use response::{ApiEnvelope, BpiResult};
pub use session::{Account, AccountProfile};
