use crate::{BpiError, BpiResult};

/// 搜索目标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchType {
    Video,
    MediaBangumi,
    MediaFt,
    Live,
    LiveRoom,
    LiveUser,
    Article,
    BiliUser,
}

impl SearchType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SearchType::Video => "video",
            SearchType::MediaBangumi => "media_bangumi",
            SearchType::MediaFt => "media_ft",
            SearchType::Live => "live",
            SearchType::LiveRoom => "live_room",
            SearchType::LiveUser => "live_user",
            SearchType::Article => "article",
            SearchType::BiliUser => "bili_user",
        }
    }
}

/// 搜索结果排序
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchOrder {
    // 视频、专栏、相簿
    TotalRank,
    Click,
    PubDate,
    Dm,
    Stow,
    Scores,
    Attention, // 专栏
    // 直播
    Online,
    LiveTime,
    // 用户
    Default,
    Fans,
    Level,
}

impl SearchOrder {
    pub fn as_str(&self) -> &'static str {
        match self {
            SearchOrder::TotalRank => "totalrank",
            SearchOrder::Click => "click",
            SearchOrder::PubDate => "pubdate",
            SearchOrder::Dm => "dm",
            SearchOrder::Stow => "stow",
            SearchOrder::Scores => "scores",
            SearchOrder::Attention => "attention",
            SearchOrder::Online => "online",
            SearchOrder::LiveTime => "live_time",
            SearchOrder::Default => "0",
            SearchOrder::Fans => "fans",
            SearchOrder::Level => "level",
        }
    }
}

/// 用户粉丝数及等级排序顺序
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderSort {
    Descending, // 由高到低
    Ascending,  // 由低到高
}

impl OrderSort {
    pub fn as_num(&self) -> u8 {
        match self {
            OrderSort::Descending => 0,
            OrderSort::Ascending => 1,
        }
    }
}

/// 用户分类筛选
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserType {
    All,
    Up,
    Normal,
    Verified,
}

impl UserType {
    pub fn as_num(&self) -> u8 {
        match self {
            UserType::All => 0,
            UserType::Up => 1,
            UserType::Normal => 2,
            UserType::Verified => 3,
        }
    }
}

/// 视频时长筛选
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Duration {
    All,
    Under10,
    From10To30,
    From30To60,
    Over60,
}

impl Duration {
    pub fn as_num(&self) -> u8 {
        match self {
            Duration::All => 0,
            Duration::Under10 => 1,
            Duration::From10To30 => 2,
            Duration::From30To60 => 3,
            Duration::Over60 => 4,
        }
    }
}

/// 专栏及相簿分区筛选
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CategoryId {
    All,
    Animation,
    Game,
    Movie,
    Life,
    Interest,
    LightNovel,
    Technology,
    Huayou,      // 相簿画友
    Photography, // 相簿摄影
}

impl CategoryId {
    pub fn as_num(&self) -> u8 {
        match self {
            CategoryId::All => 0,
            CategoryId::Animation => 2,
            CategoryId::Game => 1,
            CategoryId::Movie => 28,
            CategoryId::Life => 3,
            CategoryId::Interest => 29,
            CategoryId::LightNovel => 16,
            CategoryId::Technology => 17,
            CategoryId::Huayou => 1,
            CategoryId::Photography => 2,
        }
    }
}

/// 专栏搜索参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchArticleParams {
    keyword: String,
    order: SearchOrder,
    category_id: CategoryId,
    page: u32,
}

impl SearchArticleParams {
    pub fn new(keyword: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            keyword: normalize_search_keyword(keyword)?,
            order: SearchOrder::TotalRank,
            category_id: CategoryId::All,
            page: 1,
        })
    }

    pub fn with_order(mut self, order: SearchOrder) -> Self {
        self.order = order;
        self
    }

    pub fn with_category_id(mut self, category_id: CategoryId) -> Self {
        self.category_id = category_id;
        self
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.page = validate_search_page(page)?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("search_type", SearchType::Article.as_str().to_string()),
            ("keyword", self.keyword.clone()),
            ("order", self.order.as_str().to_string()),
            ("category_id", self.category_id.as_num().to_string()),
            ("page", self.page.to_string()),
        ]
    }
}

/// bangumi 搜索参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchBangumiParams {
    keyword: String,
    page: u32,
}

impl SearchBangumiParams {
    pub fn new(keyword: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            keyword: normalize_search_keyword(keyword)?,
            page: 1,
        })
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.page = validate_search_page(page)?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("search_type", SearchType::MediaBangumi.as_str().to_string()),
            ("keyword", self.keyword.clone()),
            ("page", self.page.to_string()),
        ]
    }
}

/// Bilibili 用户搜索参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchBiliUserParams {
    keyword: String,
    order_sort: OrderSort,
    user_type: UserType,
    page: u32,
}

impl SearchBiliUserParams {
    pub fn new(keyword: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            keyword: normalize_search_keyword(keyword)?,
            order_sort: OrderSort::Ascending,
            user_type: UserType::All,
            page: 1,
        })
    }

    pub fn with_order_sort(mut self, order_sort: OrderSort) -> Self {
        self.order_sort = order_sort;
        self
    }

    pub fn with_user_type(mut self, user_type: UserType) -> Self {
        self.user_type = user_type;
        self
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.page = validate_search_page(page)?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("search_type", SearchType::BiliUser.as_str().to_string()),
            ("keyword", self.keyword.clone()),
            ("order_sort", self.order_sort.as_num().to_string()),
            ("user_type", self.user_type.as_num().to_string()),
            ("page", self.page.to_string()),
        ]
    }
}

/// 直播间/直播用户组合搜索参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchLiveParams {
    keyword: String,
    page: u32,
}

impl SearchLiveParams {
    pub fn new(keyword: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            keyword: normalize_search_keyword(keyword)?,
            page: 1,
        })
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.page = validate_search_page(page)?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("search_type", SearchType::Live.as_str().to_string()),
            ("keyword", self.keyword.clone()),
            ("page", self.page.to_string()),
        ]
    }
}

/// 直播间搜索参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchLiveRoomParams {
    keyword: String,
    order: SearchOrder,
    page: u32,
}

impl SearchLiveRoomParams {
    pub fn new(keyword: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            keyword: normalize_search_keyword(keyword)?,
            order: SearchOrder::Online,
            page: 1,
        })
    }

    pub fn with_order(mut self, order: SearchOrder) -> Self {
        self.order = order;
        self
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.page = validate_search_page(page)?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("search_type", SearchType::LiveRoom.as_str().to_string()),
            ("keyword", self.keyword.clone()),
            ("order", self.order.as_str().to_string()),
            ("page", self.page.to_string()),
        ]
    }
}

/// 直播用户搜索参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchLiveUserParams {
    keyword: String,
    order_sort: OrderSort,
    user_type: UserType,
    page: u32,
}

impl SearchLiveUserParams {
    pub fn new(keyword: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            keyword: normalize_search_keyword(keyword)?,
            order_sort: OrderSort::Ascending,
            user_type: UserType::All,
            page: 1,
        })
    }

    pub fn with_order_sort(mut self, order_sort: OrderSort) -> Self {
        self.order_sort = order_sort;
        self
    }

    pub fn with_user_type(mut self, user_type: UserType) -> Self {
        self.user_type = user_type;
        self
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.page = validate_search_page(page)?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("search_type", SearchType::LiveUser.as_str().to_string()),
            ("keyword", self.keyword.clone()),
            ("order_sort", self.order_sort.as_num().to_string()),
            ("user_type", self.user_type.as_num().to_string()),
            ("page", self.page.to_string()),
        ]
    }
}

/// 影视搜索参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchMovieParams {
    keyword: String,
    page: u32,
}

impl SearchMovieParams {
    pub fn new(keyword: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            keyword: normalize_search_keyword(keyword)?,
            page: 1,
        })
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.page = validate_search_page(page)?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("search_type", SearchType::MediaFt.as_str().to_string()),
            ("keyword", self.keyword.clone()),
            ("page", self.page.to_string()),
        ]
    }
}

/// 视频搜索参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchVideoParams {
    keyword: String,
    order: SearchOrder,
    duration: Duration,
    tids: u32,
    page: u32,
}

impl SearchVideoParams {
    pub fn new(keyword: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            keyword: normalize_search_keyword(keyword)?,
            order: SearchOrder::TotalRank,
            duration: Duration::All,
            tids: 0,
            page: 1,
        })
    }

    pub fn with_order(mut self, order: SearchOrder) -> Self {
        self.order = order;
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn with_tid(mut self, tid: u32) -> Self {
        self.tids = tid;
        self
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.page = validate_search_page(page)?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("search_type", SearchType::Video.as_str().to_string()),
            ("keyword", self.keyword.clone()),
            ("order", self.order.as_str().to_string()),
            ("duration", self.duration.as_num().to_string()),
            ("tids", self.tids.to_string()),
            ("page", self.page.to_string()),
        ]
    }
}

fn normalize_search_keyword(keyword: impl Into<String>) -> BpiResult<String> {
    let keyword = keyword.into().trim().to_string();
    if keyword.is_empty() {
        return Err(BpiError::invalid_parameter(
            "keyword",
            "search keyword cannot be blank",
        ));
    }

    Ok(keyword)
}

fn validate_search_page(page: u32) -> BpiResult<u32> {
    if page == 0 {
        return Err(BpiError::invalid_parameter(
            "page",
            "page number must be at least 1",
        ));
    }

    Ok(page)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_article_params_serializes_optional_filters() -> Result<(), BpiError> {
        let params = SearchArticleParams::new("  Rust  ")?
            .with_order(SearchOrder::PubDate)
            .with_category_id(CategoryId::Technology)
            .with_page(2)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("search_type", "article".to_string()),
                ("keyword", "Rust".to_string()),
                ("order", "pubdate".to_string()),
                ("category_id", "17".to_string()),
                ("page", "2".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn search_bangumi_params_serializes_default_query() -> Result<(), BpiError> {
        let params = SearchBangumiParams::new("  天气之子  ")?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("search_type", "media_bangumi".to_string()),
                ("keyword", "天气之子".to_string()),
                ("page", "1".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn search_bangumi_params_serializes_page() -> Result<(), BpiError> {
        let params = SearchBangumiParams::new("天气之子")?.with_page(2)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("search_type", "media_bangumi".to_string()),
                ("keyword", "天气之子".to_string()),
                ("page", "2".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn search_bangumi_params_rejects_blank_keyword() {
        let err = SearchBangumiParams::new("  ").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "keyword",
                ..
            }
        ));
    }

    #[test]
    fn search_bili_user_params_serializes_optional_filters() -> Result<(), BpiError> {
        let params = SearchBiliUserParams::new("  老番茄  ")?
            .with_order_sort(OrderSort::Descending)
            .with_user_type(UserType::Verified)
            .with_page(3)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("search_type", "bili_user".to_string()),
                ("keyword", "老番茄".to_string()),
                ("order_sort", "0".to_string()),
                ("user_type", "3".to_string()),
                ("page", "3".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn search_live_params_serializes_default_query() -> Result<(), BpiError> {
        let params = SearchLiveParams::new("  游戏  ")?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("search_type", "live".to_string()),
                ("keyword", "游戏".to_string()),
                ("page", "1".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn search_live_room_params_serializes_optional_filters() -> Result<(), BpiError> {
        let params = SearchLiveRoomParams::new("  游戏  ")?
            .with_order(SearchOrder::LiveTime)
            .with_page(2)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("search_type", "live_room".to_string()),
                ("keyword", "游戏".to_string()),
                ("order", "live_time".to_string()),
                ("page", "2".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn search_live_user_params_serializes_optional_filters() -> Result<(), BpiError> {
        let params = SearchLiveUserParams::new("  散人  ")?
            .with_order_sort(OrderSort::Descending)
            .with_user_type(UserType::Up)
            .with_page(2)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("search_type", "live_user".to_string()),
                ("keyword", "散人".to_string()),
                ("order_sort", "0".to_string()),
                ("user_type", "1".to_string()),
                ("page", "2".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn search_movie_params_serializes_default_query() -> Result<(), BpiError> {
        let params = SearchMovieParams::new("  哈利波特  ")?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("search_type", "media_ft".to_string()),
                ("keyword", "哈利波特".to_string()),
                ("page", "1".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn search_movie_params_rejects_zero_page() -> Result<(), BpiError> {
        let err = SearchMovieParams::new("哈利波特")?
            .with_page(0)
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "page", .. }
        ));
        Ok(())
    }

    #[test]
    fn search_video_params_serializes_default_query() -> Result<(), BpiError> {
        let params = SearchVideoParams::new("rust")?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("search_type", "video".to_string()),
                ("keyword", "rust".to_string()),
                ("order", "totalrank".to_string()),
                ("duration", "0".to_string()),
                ("tids", "0".to_string()),
                ("page", "1".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn search_video_params_serializes_optional_filters() -> Result<(), BpiError> {
        let params = SearchVideoParams::new("  rust 教程  ")?
            .with_order(SearchOrder::Online)
            .with_duration(Duration::From10To30)
            .with_tid(171)
            .with_page(2)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("search_type", "video".to_string()),
                ("keyword", "rust 教程".to_string()),
                ("order", "online".to_string()),
                ("duration", "2".to_string()),
                ("tids", "171".to_string()),
                ("page", "2".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn search_video_params_rejects_blank_keyword() {
        let err = SearchVideoParams::new(" \t ").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "keyword",
                ..
            }
        ));
    }

    #[test]
    fn search_video_params_rejects_zero_page() -> Result<(), BpiError> {
        let err = SearchVideoParams::new("rust")?.with_page(0).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "page", .. }
        ));
        Ok(())
    }
}
