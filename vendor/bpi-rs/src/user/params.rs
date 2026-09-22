use crate::ids::Mid;
use crate::{BpiError, BpiResult};

/// 控制 `/x/web-interface/card` 是否包含用户空间头图。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserCardPhoto {
    /// Bilibili 返回时包含用户空间头图。
    Include,
    /// 不包含用户空间头图。
    Exclude,
}

/// `/x/web-interface/card` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserCardParams {
    mid: Mid,
    photo: Option<UserCardPhoto>,
}

impl UserCardParams {
    /// 为已验证的用户 ID 创建卡片参数。
    pub fn new(mid: Mid) -> Self {
        Self { mid, photo: None }
    }

    /// 设置响应是否包含用户空间头图。
    pub fn with_photo(mut self, photo: UserCardPhoto) -> Self {
        self.photo = Some(photo);
        self
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = vec![("mid", self.mid.to_string())];

        if let Some(photo) = self.photo {
            let value = match photo {
                UserCardPhoto::Include => "true",
                UserCardPhoto::Exclude => "false",
            };
            pairs.push(("photo", value.to_string()));
        }

        pairs
    }
}

/// `/account/v1/user/cards` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserCardsParams {
    mids: Vec<Mid>,
}

impl UserCardsParams {
    /// 为一个或多个已验证的用户 ID 创建批量卡片参数。
    pub fn new<I>(mids: I) -> BpiResult<Self>
    where
        I: IntoIterator<Item = Mid>,
    {
        let mids = mids.into_iter().collect::<Vec<_>>();

        if mids.is_empty() {
            return Err(BpiError::invalid_parameter(
                "uids",
                "at least one user id is required",
            ));
        }

        Ok(Self { mids })
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![(
            "uids",
            self.mids
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        )]
    }
}

/// `/x/im/user_infos` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserInfosParams {
    mids: Vec<Mid>,
}

impl UserInfosParams {
    /// 为一个或多个已验证的用户 ID 创建批量信息参数。
    pub fn new<I>(mids: I) -> BpiResult<Self>
    where
        I: IntoIterator<Item = Mid>,
    {
        let mids = mids.into_iter().collect::<Vec<_>>();

        if mids.is_empty() {
            return Err(BpiError::invalid_parameter(
                "uids",
                "at least one user id is required",
            ));
        }

        Ok(Self { mids })
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![(
            "uids",
            self.mids
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
        )]
    }
}

/// `/x/space/wbi/acc/info` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserSpaceParams {
    mid: Mid,
}

impl UserSpaceParams {
    /// 为已验证的用户 ID 创建空间信息参数。
    pub fn new(mid: Mid) -> Self {
        Self { mid }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("mid", self.mid.to_string())]
    }
}

/// `/x/space/notice` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserSpaceNoticeParams {
    mid: Mid,
}

impl UserSpaceNoticeParams {
    /// 为已验证的用户 ID 创建空间公告参数。
    pub fn new(mid: Mid) -> Self {
        Self { mid }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("mid", self.mid.to_string())]
    }
}

/// `/x/space/bangumi/follow/list` 的追番列表分类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserBangumiFollowKind {
    /// 已追番剧或动画 season。
    Bangumi,
    /// 已追影视或剧集 season。
    Cinema,
}

impl UserBangumiFollowKind {
    fn as_query_value(self) -> &'static str {
        match self {
            Self::Bangumi => "1",
            Self::Cinema => "2",
        }
    }
}

/// `/x/space/bangumi/follow/list` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserBangumiFollowListParams {
    mid: Mid,
    kind: UserBangumiFollowKind,
    page: u32,
    page_size: u32,
}

impl UserBangumiFollowListParams {
    /// 为已验证的用户 ID 创建番剧追番列表参数。
    pub fn new(mid: Mid) -> Self {
        Self {
            mid,
            kind: UserBangumiFollowKind::Bangumi,
            page: 1,
            page_size: 15,
        }
    }

    /// 设置获取已追番剧还是影视 season。
    pub fn with_kind(mut self, kind: UserBangumiFollowKind) -> Self {
        self.kind = kind;
        self
    }

    /// 设置从 1 开始的页码。
    pub fn with_page(mut self, page: u32) -> Self {
        self.page = page;
        self
    }

    /// 设置每页数量。Bilibili 接受 1 到 30。
    pub fn with_page_size(mut self, page_size: u32) -> BpiResult<Self> {
        if !(1..=30).contains(&page_size) {
            return Err(BpiError::invalid_parameter(
                "page_size",
                "page size must be between 1 and 30",
            ));
        }

        self.page_size = page_size;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("vmid", self.mid.to_string()),
            ("type", self.kind.as_query_value().to_string()),
            ("pn", self.page.to_string()),
            ("ps", self.page_size.to_string()),
        ]
    }
}

/// `/x/relation/stat` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserRelationStatParams {
    mid: Mid,
}

impl UserRelationStatParams {
    /// 为已验证的用户 ID 创建关系统计参数。
    pub fn new(mid: Mid) -> Self {
        Self { mid }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("vmid", self.mid.to_string())]
    }
}

/// `/x/relation/followings` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserFollowingsParams {
    mid: Mid,
    order_type: Option<String>,
    page_size: Option<u32>,
    page: Option<u32>,
}

impl UserFollowingsParams {
    /// 为已验证的用户 ID 创建关注列表参数。
    pub fn new(mid: Mid) -> Self {
        Self {
            mid,
            order_type: None,
            page_size: None,
            page: None,
        }
    }

    /// 设置 Bilibili 原始排序类型，例如 `attention`。
    pub fn with_order_type(mut self, order_type: impl Into<String>) -> Self {
        let order_type = order_type.into();
        if !order_type.trim().is_empty() {
            self.order_type = Some(order_type);
        }
        self
    }

    /// 设置每页数量。
    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    /// 设置从 1 开始的页码。
    pub fn with_page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = vec![("vmid", self.mid.to_string())];

        if let Some(order_type) = &self.order_type {
            pairs.push(("order_type", order_type.to_string()));
        }
        if let Some(page_size) = self.page_size {
            pairs.push(("ps", page_size.to_string()));
        }
        if let Some(page) = self.page {
            pairs.push(("pn", page.to_string()));
        }

        pairs
    }
}

/// `/x/relation/fans` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserFollowersParams {
    mid: Mid,
    page_size: Option<u32>,
    page: Option<u32>,
    offset: Option<String>,
    last_access_ts: Option<u64>,
    from: Option<String>,
}

impl UserFollowersParams {
    /// 为已验证的用户 ID 创建粉丝列表参数。
    pub fn new(mid: Mid) -> Self {
        Self {
            mid,
            page_size: None,
            page: None,
            offset: None,
            last_access_ts: None,
            from: None,
        }
    }

    /// 设置每页数量。
    pub fn with_page_size(mut self, page_size: u32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    /// 设置从 1 开始的页码。
    pub fn with_page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// 设置 Bilibili 返回的分页 offset。
    pub fn with_offset(mut self, offset: impl Into<String>) -> Self {
        let offset = offset.into();
        if !offset.trim().is_empty() {
            self.offset = Some(offset);
        }
        self
    }

    /// 设置最后访问时间戳，单位秒。
    pub fn with_last_access_ts(mut self, last_access_ts: u64) -> Self {
        self.last_access_ts = Some(last_access_ts);
        self
    }

    /// 设置 Bilibili 原始来源标记，例如 `main`。
    pub fn with_from(mut self, from: impl Into<String>) -> Self {
        let from = from.into();
        if !from.trim().is_empty() {
            self.from = Some(from);
        }
        self
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = vec![("vmid", self.mid.to_string())];

        if let Some(page_size) = self.page_size {
            pairs.push(("ps", page_size.to_string()));
        }
        if let Some(page) = self.page {
            pairs.push(("pn", page.to_string()));
        }
        if let Some(offset) = &self.offset {
            pairs.push(("offset", offset.to_string()));
        }
        if let Some(last_access_ts) = self.last_access_ts {
            pairs.push(("last_access_ts", last_access_ts.to_string()));
        }
        if let Some(from) = &self.from {
            pairs.push(("from", from.to_string()));
        }

        pairs
    }
}

/// `/xlive/web-ucenter/user/MedalWall` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserMedalWallParams {
    target_id: Mid,
}

impl UserMedalWallParams {
    /// 为已验证的用户 ID 创建勋章墙参数。
    pub fn new(target_id: Mid) -> Self {
        Self { target_id }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("target_id", self.target_id.to_string())]
    }
}

/// `/x/space/upstat` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserUpStatParams {
    mid: Mid,
}

impl UserUpStatParams {
    /// 为已验证的用户 ID 创建 UP 主统计参数。
    pub fn new(mid: Mid) -> Self {
        Self { mid }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("mid", self.mid.to_string())]
    }
}

/// `/x/space/navnum` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserNavStatParams {
    mid: Mid,
}

impl UserNavStatParams {
    /// 为已验证的用户 ID 创建导航统计参数。
    pub fn new(mid: Mid) -> Self {
        Self { mid }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("mid", self.mid.to_string())]
    }
}

/// `/link_draw/v1/doc/upload_count` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserAlbumCountParams {
    mid: Mid,
}

impl UserAlbumCountParams {
    /// 为已验证的用户 ID 创建相簿计数参数。
    pub fn new(mid: Mid) -> Self {
        Self { mid }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("uid", self.mid.to_string())]
    }
}

/// `/x/polymer/web-dynamic/v1/name-to-uid` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserNameToUidParams {
    names: Vec<String>,
}

impl UserNameToUidParams {
    /// 从一个或多个非空显示名称创建 name-to-UID 参数。
    pub fn new<I, S>(names: I) -> BpiResult<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let names = names
            .into_iter()
            .map(Into::into)
            .map(|name| name.trim().to_string())
            .collect::<Vec<_>>();

        if names.is_empty() {
            return Err(BpiError::invalid_parameter(
                "names",
                "at least one name is required",
            ));
        }

        if names.iter().any(|name| name.is_empty()) {
            return Err(BpiError::invalid_parameter(
                "names",
                "names cannot contain blank values",
            ));
        }

        Ok(Self { names })
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("names", self.names.join(","))]
    }
}

/// 用户空间投稿视频的排序方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserUploadedVideoOrder {
    /// 按发布时间排序。
    Pubdate,
    /// 按播放数排序。
    Click,
    /// 按收藏数排序。
    Stow,
}

impl UserUploadedVideoOrder {
    fn as_query_value(self) -> &'static str {
        match self {
            Self::Pubdate => "pubdate",
            Self::Click => "click",
            Self::Stow => "stow",
        }
    }
}

impl TryFrom<&str> for UserUploadedVideoOrder {
    type Error = BpiError;

    fn try_from(value: &str) -> BpiResult<Self> {
        match value.trim() {
            "pubdate" => Ok(Self::Pubdate),
            "click" => Ok(Self::Click),
            "stow" => Ok(Self::Stow),
            _ => Err(BpiError::invalid_parameter(
                "order",
                "uploaded video order must be pubdate, click, or stow",
            )),
        }
    }
}

/// `/x/space/wbi/arc/search` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserUploadedVideosParams {
    mid: Mid,
    order: UserUploadedVideoOrder,
    tid: u64,
    keyword: Option<String>,
    page: u32,
    page_size: u32,
}

impl UserUploadedVideosParams {
    /// 为已验证的用户 ID 创建投稿视频参数。
    pub fn new(mid: Mid) -> Self {
        Self {
            mid,
            order: UserUploadedVideoOrder::Pubdate,
            tid: 0,
            keyword: None,
            page: 1,
            page_size: 30,
        }
    }

    /// 设置视频排序方式。
    pub fn with_order(mut self, order: UserUploadedVideoOrder) -> Self {
        self.order = order;
        self
    }

    /// 设置分区过滤条件。`0` 表示全部分区。
    pub fn with_tid(mut self, tid: u64) -> Self {
        self.tid = tid;
        self
    }

    /// 设置关键词过滤条件。
    pub fn with_keyword(mut self, keyword: impl Into<String>) -> Self {
        let keyword = keyword.into().trim().to_string();
        if !keyword.is_empty() {
            self.keyword = Some(keyword);
        }
        self
    }

    /// 设置从 1 开始的页码。
    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        if page == 0 {
            return Err(BpiError::invalid_parameter(
                "page",
                "page number must be at least 1",
            ));
        }

        self.page = page;
        Ok(self)
    }

    /// 设置每页数量。
    pub fn with_page_size(mut self, page_size: u32) -> BpiResult<Self> {
        if page_size == 0 {
            return Err(BpiError::invalid_parameter(
                "page_size",
                "page size must be at least 1",
            ));
        }

        self.page_size = page_size;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = vec![
            ("mid", self.mid.to_string()),
            ("order", self.order.as_query_value().to_string()),
            ("tid", self.tid.to_string()),
            ("pn", self.page.to_string()),
            ("ps", self.page_size.to_string()),
        ];

        if let Some(keyword) = &self.keyword {
            pairs.push(("keyword", keyword.to_string()));
        }

        pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_card_params_serializes_mid_query() -> Result<(), BpiError> {
        let params = UserCardParams::new(Mid::new(1001)?);

        assert_eq!(params.query_pairs(), vec![("mid", "1001".to_string())]);
        Ok(())
    }

    #[test]
    fn user_card_params_serializes_include_photo_query() -> Result<(), BpiError> {
        let params = UserCardParams::new(Mid::new(1001)?).with_photo(UserCardPhoto::Include);

        assert_eq!(
            params.query_pairs(),
            vec![("mid", "1001".to_string()), ("photo", "true".to_string())]
        );
        Ok(())
    }

    #[test]
    fn user_card_params_serializes_exclude_photo_query() -> Result<(), BpiError> {
        let params = UserCardParams::new(Mid::new(1001)?).with_photo(UserCardPhoto::Exclude);

        assert_eq!(
            params.query_pairs(),
            vec![("mid", "1001".to_string()), ("photo", "false".to_string())]
        );
        Ok(())
    }

    #[test]
    fn user_cards_params_serializes_uids_query() -> Result<(), BpiError> {
        let params = UserCardsParams::new([Mid::new(1001)?, Mid::new(1002)?])?;

        assert_eq!(
            params.query_pairs(),
            vec![("uids", "1001,1002".to_string())]
        );
        Ok(())
    }

    #[test]
    fn user_cards_params_rejects_empty_uid_list() {
        let err = UserCardsParams::new([]).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "uids", .. }
        ));
    }

    #[test]
    fn user_infos_params_serializes_uids_query() -> Result<(), BpiError> {
        let params = UserInfosParams::new([Mid::new(1001)?, Mid::new(1002)?])?;

        assert_eq!(
            params.query_pairs(),
            vec![("uids", "1001,1002".to_string())]
        );
        Ok(())
    }

    #[test]
    fn user_infos_params_rejects_empty_uid_list() {
        let err = UserInfosParams::new([]).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "uids", .. }
        ));
    }

    #[test]
    fn user_space_params_serializes_mid_query() -> Result<(), BpiError> {
        let params = UserSpaceParams::new(Mid::new(1001)?);

        assert_eq!(params.query_pairs(), vec![("mid", "1001".to_string())]);
        Ok(())
    }

    #[test]
    fn user_space_notice_params_serializes_mid_query() -> Result<(), BpiError> {
        let params = UserSpaceNoticeParams::new(Mid::new(1001)?);

        assert_eq!(params.query_pairs(), vec![("mid", "1001".to_string())]);
        Ok(())
    }

    #[test]
    fn user_bangumi_follow_list_params_serializes_default_query() -> Result<(), BpiError> {
        let params = UserBangumiFollowListParams::new(Mid::new(1001)?);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("vmid", "1001".to_string()),
                ("type", "1".to_string()),
                ("pn", "1".to_string()),
                ("ps", "15".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn user_bangumi_follow_list_params_serializes_optional_filters() -> Result<(), BpiError> {
        let params = UserBangumiFollowListParams::new(Mid::new(1001)?)
            .with_kind(UserBangumiFollowKind::Cinema)
            .with_page(2)
            .with_page_size(30)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("vmid", "1001".to_string()),
                ("type", "2".to_string()),
                ("pn", "2".to_string()),
                ("ps", "30".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn user_bangumi_follow_list_params_rejects_zero_page_size() -> Result<(), BpiError> {
        let err = UserBangumiFollowListParams::new(Mid::new(1001)?)
            .with_page_size(0)
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "page_size",
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn user_bangumi_follow_list_params_rejects_large_page_size() -> Result<(), BpiError> {
        let err = UserBangumiFollowListParams::new(Mid::new(1001)?)
            .with_page_size(31)
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "page_size",
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn user_uploaded_videos_params_serializes_default_query() -> Result<(), BpiError> {
        let params = UserUploadedVideosParams::new(Mid::new(1001)?);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("mid", "1001".to_string()),
                ("order", "pubdate".to_string()),
                ("tid", "0".to_string()),
                ("pn", "1".to_string()),
                ("ps", "30".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn user_uploaded_video_order_parses_supported_values() -> Result<(), BpiError> {
        assert_eq!(
            UserUploadedVideoOrder::try_from("pubdate")?,
            UserUploadedVideoOrder::Pubdate
        );
        assert_eq!(
            UserUploadedVideoOrder::try_from("click")?,
            UserUploadedVideoOrder::Click
        );
        assert_eq!(
            UserUploadedVideoOrder::try_from("stow")?,
            UserUploadedVideoOrder::Stow
        );
        Ok(())
    }

    #[test]
    fn user_uploaded_video_order_rejects_unknown_value() {
        let err = UserUploadedVideoOrder::try_from("invalid").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "order", .. }
        ));
    }

    #[test]
    fn user_uploaded_videos_params_serializes_optional_filters() -> Result<(), BpiError> {
        let params = UserUploadedVideosParams::new(Mid::new(1001)?)
            .with_order(UserUploadedVideoOrder::Click)
            .with_tid(33)
            .with_keyword("rust")
            .with_page(2)?
            .with_page_size(20)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("mid", "1001".to_string()),
                ("order", "click".to_string()),
                ("tid", "33".to_string()),
                ("pn", "2".to_string()),
                ("ps", "20".to_string()),
                ("keyword", "rust".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn user_uploaded_videos_params_trims_keyword() -> Result<(), BpiError> {
        let params = UserUploadedVideosParams::new(Mid::new(1001)?).with_keyword("  rust  ");

        assert_eq!(
            params.query_pairs(),
            vec![
                ("mid", "1001".to_string()),
                ("order", "pubdate".to_string()),
                ("tid", "0".to_string()),
                ("pn", "1".to_string()),
                ("ps", "30".to_string()),
                ("keyword", "rust".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn user_uploaded_videos_params_ignores_blank_keyword() -> Result<(), BpiError> {
        let params = UserUploadedVideosParams::new(Mid::new(1001)?).with_keyword("   ");

        assert_eq!(
            params.query_pairs(),
            vec![
                ("mid", "1001".to_string()),
                ("order", "pubdate".to_string()),
                ("tid", "0".to_string()),
                ("pn", "1".to_string()),
                ("ps", "30".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn user_uploaded_videos_params_rejects_zero_page() -> Result<(), BpiError> {
        let err = UserUploadedVideosParams::new(Mid::new(1001)?)
            .with_page(0)
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "page", .. }
        ));
        Ok(())
    }

    #[test]
    fn user_uploaded_videos_params_rejects_zero_page_size() -> Result<(), BpiError> {
        let err = UserUploadedVideosParams::new(Mid::new(1001)?)
            .with_page_size(0)
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "page_size",
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn user_relation_stat_params_serializes_mid_query() -> Result<(), BpiError> {
        let params = UserRelationStatParams::new(Mid::new(1001)?);

        assert_eq!(params.query_pairs(), vec![("vmid", "1001".to_string())]);
        Ok(())
    }

    #[test]
    fn user_followings_params_serializes_default_query() -> Result<(), BpiError> {
        let params = UserFollowingsParams::new(Mid::new(1001)?);

        assert_eq!(params.query_pairs(), vec![("vmid", "1001".to_string())]);
        Ok(())
    }

    #[test]
    fn user_followings_params_serializes_optional_filters() -> Result<(), BpiError> {
        let params = UserFollowingsParams::new(Mid::new(1001)?)
            .with_order_type("attention")
            .with_page_size(20)
            .with_page(2);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("vmid", "1001".to_string()),
                ("order_type", "attention".to_string()),
                ("ps", "20".to_string()),
                ("pn", "2".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn user_followers_params_serializes_default_query() -> Result<(), BpiError> {
        let params = UserFollowersParams::new(Mid::new(1001)?);

        assert_eq!(params.query_pairs(), vec![("vmid", "1001".to_string())]);
        Ok(())
    }

    #[test]
    fn user_followers_params_serializes_optional_filters() -> Result<(), BpiError> {
        let params = UserFollowersParams::new(Mid::new(1001)?)
            .with_page_size(20)
            .with_page(2)
            .with_offset("next-offset")
            .with_last_access_ts(1_700_000_000)
            .with_from("main");

        assert_eq!(
            params.query_pairs(),
            vec![
                ("vmid", "1001".to_string()),
                ("ps", "20".to_string()),
                ("pn", "2".to_string()),
                ("offset", "next-offset".to_string()),
                ("last_access_ts", "1700000000".to_string()),
                ("from", "main".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn user_medal_wall_params_serializes_target_id_query() -> Result<(), BpiError> {
        let params = UserMedalWallParams::new(Mid::new(1001)?);

        assert_eq!(
            params.query_pairs(),
            vec![("target_id", "1001".to_string())]
        );
        Ok(())
    }

    #[test]
    fn user_up_stat_params_serializes_mid_query() -> Result<(), BpiError> {
        let params = UserUpStatParams::new(Mid::new(1001)?);

        assert_eq!(params.query_pairs(), vec![("mid", "1001".to_string())]);
        Ok(())
    }

    #[test]
    fn user_nav_stat_params_serializes_mid_query() -> Result<(), BpiError> {
        let params = UserNavStatParams::new(Mid::new(1001)?);

        assert_eq!(params.query_pairs(), vec![("mid", "1001".to_string())]);
        Ok(())
    }

    #[test]
    fn user_album_count_params_serializes_uid_query() -> Result<(), BpiError> {
        let params = UserAlbumCountParams::new(Mid::new(1001)?);

        assert_eq!(params.query_pairs(), vec![("uid", "1001".to_string())]);
        Ok(())
    }

    #[test]
    fn user_name_to_uid_params_serializes_joined_names_query() -> Result<(), BpiError> {
        let params = UserNameToUidParams::new(["fixture_user", "another_user"])?;

        assert_eq!(
            params.query_pairs(),
            vec![("names", "fixture_user,another_user".to_string())]
        );
        Ok(())
    }

    #[test]
    fn user_name_to_uid_params_rejects_empty_names() {
        let err = UserNameToUidParams::new(Vec::<&str>::new()).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "names", .. }
        ));
    }

    #[test]
    fn user_name_to_uid_params_rejects_blank_name() {
        let err = UserNameToUidParams::new(["fixture_user", "  "]).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "names", .. }
        ));
    }
}
