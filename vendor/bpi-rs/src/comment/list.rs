//! 评论查询 API
//!
//! [参考文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/comment/list.md)

use crate::{BpiError, BpiResult};
use serde::{Deserialize, Serialize};

use super::types::{
    Comment, // 评论条目对象，包含评论内容、发送者信息、回复等
    Config,
    Control,
    Cursor,
    PageInfo,
    Top,
    Upper,
};

/// 目标评论区。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentTarget {
    r#type: i32,
    oid: i64,
}

impl CommentTarget {
    pub fn new(r#type: i32, oid: i64) -> BpiResult<Self> {
        if r#type <= 0 {
            return Err(BpiError::invalid_parameter(
                "type",
                "value must be greater than zero",
            ));
        }
        if oid <= 0 {
            return Err(BpiError::invalid_parameter(
                "oid",
                "value must be greater than zero",
            ));
        }
        Ok(Self { r#type, oid })
    }

    fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("type", self.r#type.to_string()),
            ("oid", self.oid.to_string()),
        ]
    }
}

/// `/x/v2/reply` 的排序方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommentSort {
    Time,
    Like,
    Replies,
}

impl CommentSort {
    fn as_i32(self) -> i32 {
        match self {
            Self::Time => 0,
            Self::Like => 1,
            Self::Replies => 2,
        }
    }
}

/// `/x/v2/reply` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentListParams {
    target: CommentTarget,
    page: Option<u32>,
    page_size: Option<u32>,
    sort: Option<CommentSort>,
    nohot: Option<bool>,
}

impl CommentListParams {
    pub fn new(target: CommentTarget) -> Self {
        Self {
            target,
            page: None,
            page_size: None,
            sort: None,
            nohot: None,
        }
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.page = Some(validate_positive("pn", page)?);
        Ok(self)
    }

    pub fn with_page_size(mut self, page_size: u32) -> BpiResult<Self> {
        let page_size = validate_positive("ps", page_size)?;
        if page_size > 20 {
            return Err(BpiError::invalid_parameter(
                "ps",
                "value must be less than or equal to 20",
            ));
        }
        self.page_size = Some(page_size);
        Ok(self)
    }

    pub fn with_sort(mut self, sort: CommentSort) -> Self {
        self.sort = Some(sort);
        self
    }

    pub fn without_hot(mut self, nohot: bool) -> Self {
        self.nohot = Some(nohot);
        self
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = self.target.query_pairs();
        if let Some(page) = self.page {
            params.push(("pn", page.to_string()));
        }
        if let Some(page_size) = self.page_size {
            params.push(("ps", page_size.to_string()));
        }
        if let Some(sort) = self.sort {
            params.push(("sort", sort.as_i32().to_string()));
        }
        if let Some(nohot) = self.nohot {
            params.push(("nohot", i32::from(nohot).to_string()));
        }
        params
    }
}

/// `/x/v2/reply/reply` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentRepliesParams {
    target: CommentTarget,
    root: i64,
    page: Option<u32>,
    page_size: Option<u32>,
}

impl CommentRepliesParams {
    pub fn new(target: CommentTarget, root: i64) -> BpiResult<Self> {
        if root <= 0 {
            return Err(BpiError::invalid_parameter(
                "root",
                "value must be greater than zero",
            ));
        }
        Ok(Self {
            target,
            root,
            page: None,
            page_size: None,
        })
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.page = Some(validate_positive("pn", page)?);
        Ok(self)
    }

    pub fn with_page_size(mut self, page_size: u32) -> BpiResult<Self> {
        self.page_size = Some(validate_positive("ps", page_size)?);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = self.target.query_pairs();
        params.push(("root", self.root.to_string()));
        if let Some(page) = self.page {
            params.push(("pn", page.to_string()));
        }
        if let Some(page_size) = self.page_size {
            params.push(("ps", page_size.to_string()));
        }
        params
    }
}

/// `/x/v2/reply/hot` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentHotParams {
    target: CommentTarget,
    root: i64,
    page: Option<u32>,
    page_size: Option<u32>,
}

impl CommentHotParams {
    pub fn new(target: CommentTarget, root: i64) -> BpiResult<Self> {
        if root <= 0 {
            return Err(BpiError::invalid_parameter(
                "root",
                "value must be greater than zero",
            ));
        }
        Ok(Self {
            target,
            root,
            page: None,
            page_size: None,
        })
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.page = Some(validate_positive("pn", page)?);
        Ok(self)
    }

    pub fn with_page_size(mut self, page_size: u32) -> BpiResult<Self> {
        self.page_size = Some(validate_positive("ps", page_size)?);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = self.target.query_pairs();
        params.push(("root", self.root.to_string()));
        if let Some(page) = self.page {
            params.push(("pn", page.to_string()));
        }
        if let Some(page_size) = self.page_size {
            params.push(("ps", page_size.to_string()));
        }
        params
    }
}

/// `/x/v2/reply/count` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentCountParams {
    target: CommentTarget,
}

impl CommentCountParams {
    pub fn new(target: CommentTarget) -> Self {
        Self { target }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        self.target.query_pairs()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentListData {
    pub page: Option<PageInfo>,
    pub cursor: Option<Cursor>,        // 评论列表游标
    pub replies: Option<Vec<Comment>>, // 评论列表，禁用时为 null
    pub top: Option<Top>,              // 评论列表顶部信息
    pub top_replies: Option<Vec<Comment>>,
    pub effects: Option<serde_json::Value>,
    pub assist: Option<u64>,    // 待确认
    pub blacklist: Option<u64>, // 待确认
    pub vote: Option<u64>,      // 投票评论？
    pub config: Option<Config>, // 评论区显示控制
    pub upper: Option<Upper>,   // 置顶评论

    pub control: Option<Control>, // 评论区输入属性
    pub note: Option<u32>,
    pub cm_info: Option<serde_json::Value>, // 评论区相关信息

                                            // pub page: Option<PageInfo>, // 页信息
                                            // pub hots: Option<Vec<Comment>>, // 热评列表，禁用时为 null
                                            // pub notice: Option<Notice>, // 评论区公告信息，无效时为 null
                                            // pub mode: Option<u64>, // 评论区类型 id
                                            // pub support_mode: Option<Vec<u64>>, // 评论区支持的类型 id
                                            // pub folder: Option<Folder>, // 折叠相关信息
                                            // pub lottery_card: Option<()>, // 待确认
                                            // pub show_bvid: Option<bool>, // 是否显示 bvid
}

/// 公告信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notice {
    pub content: Option<String>,
    pub id: Option<u64>,
    pub link: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HotCommentData {
    pub page: HotCommentPage,
    pub replies: Vec<Comment>, // 热评列表
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HotCommentPage {
    pub acount: i64, // 总评论数
    pub count: i64,  // 热评数
    pub num: i32,    // 当前页码
    pub size: i32,   // 每页项数
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CountData {
    pub count: u64,
}

fn validate_positive(field: &'static str, value: u32) -> BpiResult<u32> {
    if value == 0 {
        return Err(BpiError::invalid_parameter(
            field,
            "value must be greater than zero",
        ));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ApiEnvelope;
    use crate::BpiClient;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use std::collections::BTreeMap;
    use tracing::info;

    const TEST_TYPE: i32 = 1;
    const TEST_OID: i64 = 23199;
    const TEST_ROOT_RPID: i64 = 2554491176;

    fn target() -> BpiResult<CommentTarget> {
        CommentTarget::new(TEST_TYPE, TEST_OID)
    }

    fn contract(name: &str) -> BpiResult<EndpointContract> {
        let bytes = match name {
            "list" => {
                include_bytes!("../../tests/contracts/comment/read/list/contract.json").as_slice()
            }
            "replies" => include_bytes!("../../tests/contracts/comment/read/replies/contract.json")
                .as_slice(),
            "hot" => {
                include_bytes!("../../tests/contracts/comment/read/hot/contract.json").as_slice()
            }
            "count" => {
                include_bytes!("../../tests/contracts/comment/read/count/contract.json").as_slice()
            }
            _ => unreachable!("unknown comment read contract"),
        };
        EndpointContract::from_slice(bytes)
    }

    fn query_map(params: Vec<(&'static str, String)>) -> BTreeMap<String, String> {
        params
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect()
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_comment_list() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");

        let result = bpi
            .comment()
            .list(
                CommentListParams::new(CommentTarget::new(TEST_TYPE, TEST_OID)?)
                    .with_page(1)?
                    .with_page_size(5)?
                    .with_sort(CommentSort::Time)
                    .without_hot(false),
            )
            .await?;
        let data = result;
        info!("总评论数: {}", data.replies.unwrap().len());

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_comment_replies() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");

        let result = bpi
            .comment()
            .replies(
                CommentRepliesParams::new(
                    CommentTarget::new(TEST_TYPE, TEST_OID)?,
                    TEST_ROOT_RPID,
                )?
                .with_page(1)?
                .with_page_size(5)?,
            )
            .await?;
        let data = result;
        info!("总评论数: {}", data.replies.unwrap().len());

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_comment_hot() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let root_rpid = 654321;

        let result = bpi
            .comment()
            .hot(
                CommentHotParams::new(CommentTarget::new(TEST_TYPE, TEST_OID)?, root_rpid)?
                    .with_page(1)?
                    .with_page_size(5)?,
            )
            .await?;
        let data = result.ok_or_else(|| BpiError::unsupported_response("missing hot comments"))?;

        info!("热评数量: {}", data.replies.len());
        for comment in data.replies.iter() {
            info!("热评内容: {}", comment.content.message);
        }

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_comment_count() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");

        let result = bpi
            .comment()
            .count(CommentCountParams::new(CommentTarget::new(
                TEST_TYPE, TEST_OID,
            )?))
            .await?;

        let data = result;
        info!("评论总数: {}", data.count);

        Ok(())
    }

    #[test]
    fn comment_target_rejects_invalid_identifiers() {
        let type_err = CommentTarget::new(0, TEST_OID).unwrap_err();
        assert!(matches!(
            type_err,
            BpiError::InvalidParameter { field: "type", .. }
        ));

        let oid_err = CommentTarget::new(TEST_TYPE, 0).unwrap_err();
        assert!(matches!(
            oid_err,
            BpiError::InvalidParameter { field: "oid", .. }
        ));
    }

    #[test]
    fn comment_list_params_serializes_query() -> BpiResult<()> {
        let params = CommentListParams::new(target()?)
            .with_page(1)?
            .with_page_size(5)?
            .with_sort(CommentSort::Time)
            .without_hot(false);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("type", "1".to_string()),
                ("oid", "23199".to_string()),
                ("pn", "1".to_string()),
                ("ps", "5".to_string()),
                ("sort", "0".to_string()),
                ("nohot", "0".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn comment_list_params_rejects_large_page_size() -> BpiResult<()> {
        let err = CommentListParams::new(target()?)
            .with_page_size(21)
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "ps", .. }
        ));
        Ok(())
    }

    #[test]
    fn comment_replies_params_serializes_query() -> BpiResult<()> {
        let params = CommentRepliesParams::new(target()?, TEST_ROOT_RPID)?
            .with_page(1)?
            .with_page_size(5)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("type", "1".to_string()),
                ("oid", "23199".to_string()),
                ("root", "2554491176".to_string()),
                ("pn", "1".to_string()),
                ("ps", "5".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn comment_hot_params_serializes_query() -> BpiResult<()> {
        let params = CommentHotParams::new(target()?, TEST_ROOT_RPID)?
            .with_page(1)?
            .with_page_size(5)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("type", "1".to_string()),
                ("oid", "23199".to_string()),
                ("root", "2554491176".to_string()),
                ("pn", "1".to_string()),
                ("ps", "5".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn comment_count_params_serializes_query() -> BpiResult<()> {
        let params = CommentCountParams::new(target()?);

        assert_eq!(
            params.query_pairs(),
            vec![("type", "1".to_string()), ("oid", "23199".to_string())]
        );
        Ok(())
    }

    #[test]
    fn comment_read_contracts_match_endpoint_requests() -> BpiResult<()> {
        let list = contract("list")?;
        let list_params = CommentListParams::new(target()?)
            .with_page(1)?
            .with_page_size(5)?
            .with_sort(CommentSort::Time)
            .without_hot(false);
        assert_eq!(list.name, "comment.read.list");
        assert_eq!(list.request.method, HttpMethod::Get);
        assert_eq!(
            list.request.url.as_str(),
            "https://api.bilibili.com/x/v2/reply"
        );
        assert_eq!(query_map(list_params.query_pairs()), list.request.query);

        let replies = contract("replies")?;
        let replies_params = CommentRepliesParams::new(target()?, TEST_ROOT_RPID)?
            .with_page(1)?
            .with_page_size(5)?;
        assert_eq!(replies.name, "comment.read.replies");
        assert_eq!(
            replies.request.url.as_str(),
            "https://api.bilibili.com/x/v2/reply/reply"
        );
        assert_eq!(
            query_map(replies_params.query_pairs()),
            replies.request.query
        );

        let hot = contract("hot")?;
        let hot_params = CommentHotParams::new(target()?, TEST_ROOT_RPID)?
            .with_page(1)?
            .with_page_size(5)?;
        assert_eq!(hot.name, "comment.read.hot");
        assert_eq!(
            hot.request.url.as_str(),
            "https://api.bilibili.com/x/v2/reply/hot"
        );
        assert_eq!(query_map(hot_params.query_pairs()), hot.request.query);

        let count = contract("count")?;
        let count_params = CommentCountParams::new(target()?);
        assert_eq!(count.name, "comment.read.count");
        assert_eq!(
            count.request.url.as_str(),
            "https://api.bilibili.com/x/v2/reply/count"
        );
        assert_eq!(query_map(count_params.query_pairs()), count.request.query);
        Ok(())
    }

    #[test]
    fn comment_read_response_fixtures_parse_declared_models() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/comment/read/list/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!("../../tests/contracts/comment/read/list/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/comment/read/list/responses/vip.success.json")
                .as_slice(),
            include_bytes!(
                "../../tests/contracts/comment/read/replies/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/comment/read/replies/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!("../../tests/contracts/comment/read/replies/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<CommentListData>::from_slice(bytes)?.into_payload()?;
            assert!(payload.page.is_some());
        }

        for bytes in [
            include_bytes!(
                "../../tests/contracts/comment/read/count/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/comment/read/count/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!("../../tests/contracts/comment/read/count/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<CountData>::from_slice(bytes)?.into_payload()?;
            assert_eq!(payload.count, 10);
        }

        for bytes in [
            include_bytes!(
                "../../tests/contracts/comment/read/hot/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!("../../tests/contracts/comment/read/hot/responses/normal.success.json")
                .as_slice(),
            include_bytes!("../../tests/contracts/comment/read/hot/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload =
                ApiEnvelope::<HotCommentData>::from_slice(bytes)?.into_optional_payload()?;
            assert!(payload.is_none());
        }
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/comment/read/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn comment_read_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            for endpoint in ["list", "replies"] {
                let Some(body) = local_probe_body(endpoint, profile) else {
                    continue;
                };
                let payload =
                    serde_json::from_value::<ApiEnvelope<CommentListData>>(body)?.into_payload()?;
                assert!(payload.page.is_some());
            }

            let Some(count_body) = local_probe_body("count", profile) else {
                continue;
            };
            let count =
                serde_json::from_value::<ApiEnvelope<CountData>>(count_body)?.into_payload()?;
            assert_eq!(count.count, 10);

            let Some(hot_body) = local_probe_body("hot", profile) else {
                continue;
            };
            let hot = serde_json::from_value::<ApiEnvelope<HotCommentData>>(hot_body)?
                .into_optional_payload()?;
            assert!(hot.is_none());
        }
        Ok(())
    }
}
