//! 用户
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/user)

pub mod batch;
pub mod client;
pub mod contract;
pub mod info;
pub mod medals;
pub mod model;
pub mod official_role;
pub mod params;
pub mod register;
pub mod relation;
pub mod search;
pub mod space;
pub mod status_number;

pub use client::UserClient;
pub use model::{
    UserAlbumCount, UserBangumiFollow, UserBangumiFollowList, UserBangumiLatestEpisode,
    UserBangumiRating, UserBatchCard, UserBatchInfo, UserBatchVip, UserCardProfile,
    UserCardSummary, UserFollowTag, UserFollower, UserFollowers, UserFollowing, UserFollowings,
    UserMedalInfo, UserMedalOwnerInfo, UserMedalWall, UserMedalWallItem, UserNameToUid,
    UserNameToUidItem, UserNavStat, UserNavStatPair, UserOfficialSummary,
    UserRelationOfficialVerify, UserRelationStat, UserSpaceLiveRoom, UserSpaceNotice,
    UserSpaceProfile, UserUpStat, UserUpStatArchive, UserUpStatArticle, UserUploadedVideo,
    UserUploadedVideoList, UserUploadedVideos, UserUploadedVideosButton, UserUploadedVideosPage,
    UserVipSummary,
};
pub use params::{
    UserAlbumCountParams, UserBangumiFollowKind, UserBangumiFollowListParams, UserCardParams,
    UserCardPhoto, UserCardsParams, UserFollowersParams, UserFollowingsParams, UserInfosParams,
    UserMedalWallParams, UserNameToUidParams, UserNavStatParams, UserRelationStatParams,
    UserSpaceNoticeParams, UserSpaceParams, UserUpStatParams, UserUploadedVideoOrder,
    UserUploadedVideosParams,
};
pub use relation::{
    ModifyRelationResponseData, RelationAction, RelationSource, UserGroupCreateParams,
    UserGroupDeleteParams, UserGroupMoveUsersParams, UserGroupUpdateParams, UserGroupUsersParams,
    UserModifyRelationParams,
};
pub use space::UserSpaceNoticeSetParams;
