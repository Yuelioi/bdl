//! B站用户关系子模块
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/user)
pub mod followers;
pub mod following;
pub mod following_group;

mod action;
mod group;

pub use action::{
    ModifyRelationResponseData, RelationAction, RelationSource, UserModifyRelationParams,
};
pub use group::{
    UserGroupCreateParams, UserGroupDeleteParams, UserGroupMoveUsersParams, UserGroupUpdateParams,
    UserGroupUsersParams,
};
