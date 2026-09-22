// 评论区相关操作 API
//
// [参考文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/comment/action)

use crate::BilibiliRequest;
use crate::BpiError;
use crate::comment::CommentClient;
use crate::response::BpiResult;
use serde::{Deserialize, Serialize};

const ADD_ENDPOINT: &str = "https://api.bilibili.com/x/v2/reply/add";
const LIKE_ENDPOINT: &str = "https://api.bilibili.com/x/v2/reply/action";
const DISLIKE_ENDPOINT: &str = "https://api.bilibili.com/x/v2/reply/hate";
const DELETE_ENDPOINT: &str = "https://api.bilibili.com/x/v2/reply/del";
const TOP_ENDPOINT: &str = "https://api.bilibili.com/x/v2/reply/top";
const REPORT_ENDPOINT: &str = "https://api.bilibili.com/x/v2/reply/report";

/// 评论区类型枚举（部分示例，需按需求扩展）

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CommentType {
    Video = 1,    // 视频
    Article = 12, // 专栏
    Dynamic = 17, // 动态
    Unknown = 0,
}

/// 举报原因枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ReportReason {
    Other = 0,
    Ad = 1,
    Porn = 2,
    Spam = 3,
    Flame = 4,
    Spoiler = 5,
    Politics = 6,
    Abuse = 7,
    Irrelevant = 8,
    Illegal = 9,
    Vulgar = 10,
    Phishing = 11,
    Scam = 12,
    Rumor = 13,
    Incitement = 14,
    Privacy = 15,
    FloorSnatching = 16,
    HarmfulToYouth = 17,
}

/// 评论成功返回数据
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CommentData {
    pub rpid: u64,
    pub rpid_str: String,
    pub root: u64,
    pub root_str: String,
    pub parent: u64,
    pub parent_str: String,
    pub dialog: u64,
    pub dialog_str: String,
    pub success_toast: Option<String>,
}

/// 发布评论的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentAddParams {
    r#type: CommentType,
    oid: u64,
    message: String,
    root: Option<u64>,
    parent: Option<u64>,
    plat: u8,
}

impl CommentAddParams {
    pub fn new(r#type: CommentType, oid: u64, message: impl Into<String>) -> BpiResult<Self> {
        validate_comment_type(r#type)?;

        Ok(Self {
            r#type,
            oid: validate_nonzero_u64("oid", oid)?,
            message: normalize_non_blank("message", message.into())?,
            root: None,
            parent: None,
            plat: 1,
        })
    }

    pub fn root(mut self, root: u64) -> BpiResult<Self> {
        self.root = Some(validate_nonzero_u64("root", root)?);
        Ok(self)
    }

    pub fn parent(mut self, parent: u64) -> BpiResult<Self> {
        self.parent = Some(validate_nonzero_u64("parent", parent)?);
        Ok(self)
    }

    pub(crate) fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        let mut pairs = vec![
            ("type", comment_type_value(self.r#type).to_string()),
            ("oid", self.oid.to_string()),
            ("message", self.message.clone()),
            ("plat", self.plat.to_string()),
            ("csrf", csrf.to_string()),
        ];

        if let Some(root) = self.root {
            pairs.push(("root", root.to_string()));
        }
        if let Some(parent) = self.parent {
            pairs.push(("parent", parent.to_string()));
        }

        pairs
    }
}

/// 点赞、点踩、置顶等二元评论操作的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentActionParams {
    r#type: CommentType,
    oid: u64,
    rpid: u64,
    action: u8,
}

impl CommentActionParams {
    pub fn new(r#type: CommentType, oid: u64, rpid: u64, action: u8) -> BpiResult<Self> {
        validate_comment_type(r#type)?;

        Ok(Self {
            r#type,
            oid: validate_nonzero_u64("oid", oid)?,
            rpid: validate_nonzero_u64("rpid", rpid)?,
            action: validate_binary_action(action)?,
        })
    }

    pub(crate) fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("type", comment_type_value(self.r#type).to_string()),
            ("oid", self.oid.to_string()),
            ("rpid", self.rpid.to_string()),
            ("action", self.action.to_string()),
            ("csrf", csrf.to_string()),
        ]
    }
}

/// 删除评论的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CommentDeleteParams {
    r#type: CommentType,
    oid: u64,
    rpid: u64,
}

impl CommentDeleteParams {
    pub fn new(r#type: CommentType, oid: u64, rpid: u64) -> BpiResult<Self> {
        validate_comment_type(r#type)?;

        Ok(Self {
            r#type,
            oid: validate_nonzero_u64("oid", oid)?,
            rpid: validate_nonzero_u64("rpid", rpid)?,
        })
    }

    pub(crate) fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("type", comment_type_value(self.r#type).to_string()),
            ("oid", self.oid.to_string()),
            ("rpid", self.rpid.to_string()),
            ("csrf", csrf.to_string()),
        ]
    }
}

/// 举报评论的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentReportParams {
    r#type: CommentType,
    oid: u64,
    rpid: u64,
    reason: ReportReason,
    content: Option<String>,
}

impl CommentReportParams {
    pub fn new(r#type: CommentType, oid: u64, rpid: u64, reason: ReportReason) -> BpiResult<Self> {
        validate_comment_type(r#type)?;

        Ok(Self {
            r#type,
            oid: validate_nonzero_u64("oid", oid)?,
            rpid: validate_nonzero_u64("rpid", rpid)?,
            reason,
            content: None,
        })
    }

    pub fn content(mut self, content: impl Into<String>) -> BpiResult<Self> {
        self.content = Some(normalize_non_blank("content", content.into())?);
        Ok(self)
    }

    pub(crate) fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        let mut pairs = vec![
            ("type", comment_type_value(self.r#type).to_string()),
            ("oid", self.oid.to_string()),
            ("rpid", self.rpid.to_string()),
            ("reason", report_reason_value(self.reason).to_string()),
            ("csrf", csrf.to_string()),
        ];

        if let Some(content) = &self.content {
            pairs.push(("content", content.clone()));
        }

        pairs
    }
}

impl<'a> CommentClient<'a> {
    /// 发布评论并返回标准 payload 结果。
    pub async fn add(&self, params: CommentAddParams) -> BpiResult<CommentData> {
        let csrf = self.client.csrf()?;
        self.client
            .post(ADD_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_payload("comment.action.add")
            .await
    }

    /// 点赞或取消点赞评论，并返回标准 payload 结果。
    pub async fn like(&self, params: CommentActionParams) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;
        self.client
            .post(LIKE_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("comment.action.like")
            .await
    }

    /// 点踩或取消点踩评论，并返回标准 payload 结果。
    pub async fn dislike(
        &self,
        params: CommentActionParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;
        self.client
            .post(DISLIKE_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("comment.action.dislike")
            .await
    }

    /// 删除评论并返回标准 payload 结果。
    pub async fn delete(
        &self,
        params: CommentDeleteParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;
        self.client
            .post(DELETE_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("comment.action.delete")
            .await
    }

    /// 置顶或取消置顶评论，并返回标准 payload 结果。
    pub async fn top(&self, params: CommentActionParams) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;
        self.client
            .post(TOP_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("comment.action.top")
            .await
    }

    /// 举报评论并返回标准 payload 结果。
    pub async fn report(
        &self,
        params: CommentReportParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;
        self.client
            .post(REPORT_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("comment.action.report")
            .await
    }
}

fn validate_comment_type(value: CommentType) -> BpiResult<()> {
    if value == CommentType::Unknown {
        return Err(BpiError::invalid_parameter(
            "type",
            "comment type must be known",
        ));
    }

    Ok(())
}

fn comment_type_value(value: CommentType) -> u32 {
    value as u32
}

fn report_reason_value(value: ReportReason) -> u32 {
    value as u32
}

fn validate_nonzero_u64(field: &'static str, value: u64) -> BpiResult<u64> {
    if value == 0 {
        return Err(BpiError::invalid_parameter(field, "value must be non-zero"));
    }

    Ok(value)
}

fn validate_binary_action(value: u8) -> BpiResult<u8> {
    if matches!(value, 0 | 1) {
        return Ok(value);
    }

    Err(BpiError::invalid_parameter(
        "action",
        "value must be 0 or 1",
    ))
}

fn normalize_non_blank(field: &'static str, value: String) -> BpiResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BpiError::invalid_parameter(field, "value cannot be blank"));
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use crate::BpiError;

    use super::{CommentActionParams, CommentAddParams, CommentReportParams, CommentType};

    #[test]
    fn comment_add_params_rejects_blank_message() {
        let err = CommentAddParams::new(CommentType::Video, 23199, "  ").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "message",
                ..
            }
        ));
    }

    #[test]
    fn comment_add_params_serializes_reply_fields() -> Result<(), BpiError> {
        let params = CommentAddParams::new(CommentType::Video, 23199, "hello")?
            .root(2554491176)?
            .parent(2554491177)?;

        assert_eq!(
            params.form_pairs("csrf-token"),
            vec![
                ("type", "1".to_string()),
                ("oid", "23199".to_string()),
                ("message", "hello".to_string()),
                ("plat", "1".to_string()),
                ("csrf", "csrf-token".to_string()),
                ("root", "2554491176".to_string()),
                ("parent", "2554491177".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn comment_action_params_rejects_invalid_action() {
        let err = CommentActionParams::new(CommentType::Video, 23199, 2554491176, 2).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "action",
                ..
            }
        ));
    }

    #[test]
    fn comment_report_params_rejects_blank_content() -> Result<(), BpiError> {
        let err = CommentReportParams::new(
            CommentType::Video,
            23199,
            2554491176,
            super::ReportReason::Other,
        )?
        .content(" ")
        .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "content",
                ..
            }
        ));
        Ok(())
    }
}
