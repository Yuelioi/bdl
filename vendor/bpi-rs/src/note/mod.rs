//! 视频笔记

pub mod action;
mod client;
pub mod info;
pub mod list;
pub mod params;

pub use action::{NoteAddParams, NoteDeleteParams};
pub use client::NoteClient;
pub use params::{
    NoteArchiveListParams, NoteIsForbidParams, NotePrivateInfoParams, NotePublicArchiveListParams,
    NotePublicInfoParams, NoteUserPrivateListParams, NoteUserPublicListParams,
};
