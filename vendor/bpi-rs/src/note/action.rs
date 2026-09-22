// --- 保存视频笔记 ---

use crate::BilibiliRequest;
use crate::BpiError;
use crate::BpiResult;
use crate::ids::Aid;
use crate::note::NoteClient;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// 保存视频笔记的响应数据

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NoteAddResponseData {
    /// 笔记ID
    pub note_id: String,
}

/// 保存视频笔记的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteAddParams {
    oid: Aid,
    title: String,
    summary: String,
    content: String,
    note_id: Option<String>,
    tags: Option<String>,
    publish: Option<bool>,
    auto_comment: Option<bool>,
}

/// 删除视频笔记的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteDeleteParams {
    oid: Aid,
    note_id: Option<String>,
}

impl NoteDeleteParams {
    pub fn new(oid: Aid) -> Self {
        Self { oid, note_id: None }
    }

    pub fn note_id(mut self, note_id: impl Into<String>) -> BpiResult<Self> {
        self.note_id = Some(normalize_non_blank("note_id", note_id.into())?);
        Ok(self)
    }

    fn form_pairs(&self, csrf: impl Into<String>) -> Vec<(&'static str, String)> {
        let mut form = vec![("oid", self.oid.to_string()), ("csrf", csrf.into())];

        if let Some(note_id) = &self.note_id {
            form.push(("note_id", note_id.clone()));
        }

        form
    }
}

impl NoteAddParams {
    /// 为视频 AV ID 创建笔记保存参数。
    pub fn new(
        oid: Aid,
        title: impl Into<String>,
        summary: impl Into<String>,
        content: impl Into<String>,
    ) -> BpiResult<Self> {
        let params = Self {
            oid,
            title: title.into(),
            summary: summary.into(),
            content: content.into(),
            note_id: None,
            tags: None,
            publish: None,
            auto_comment: None,
        };
        params.validate()?;
        Ok(params)
    }

    /// 更新现有笔记时设置笔记 ID。
    pub fn note_id(mut self, note_id: impl Into<String>) -> BpiResult<Self> {
        self.note_id = Some(note_id.into());
        self.validate()?;
        Ok(self)
    }

    /// 设置笔记跳转 tag。
    pub fn tags(mut self, tags: impl Into<String>) -> BpiResult<Self> {
        self.tags = Some(tags.into());
        self.validate()?;
        Ok(self)
    }

    /// 控制笔记是否公开。
    pub fn publish(mut self, publish: bool) -> Self {
        self.publish = Some(publish);
        self
    }

    /// 控制是否将笔记添加到评论。
    pub fn auto_comment(mut self, auto_comment: bool) -> Self {
        self.auto_comment = Some(auto_comment);
        self
    }

    fn validate(&self) -> BpiResult<()> {
        normalize_non_blank("title", self.title.clone())?;
        normalize_non_blank("summary", self.summary.clone())?;
        normalize_non_blank("content", self.content.clone())?;
        if let Some(note_id) = self.note_id.as_deref() {
            normalize_non_blank("note_id", note_id.to_string())?;
        }
        if let Some(tags) = self.tags.as_deref() {
            normalize_non_blank("tags", tags.to_string())?;
        }
        Ok(())
    }

    fn form_pairs(&self, csrf: impl Into<String>) -> Vec<(&'static str, String)> {
        let content = json!([{"insert": self.content}]);
        let mut form = vec![
            ("oid", self.oid.to_string()),
            ("oid_type", "0".to_string()),
            ("title", self.title.clone()),
            ("summary", self.summary.clone()),
            ("content", content.to_string()),
            ("cls", "1".to_string()),
            ("from", "save".to_string()),
            ("platform", "web".to_string()),
            ("csrf", csrf.into()),
        ];

        if let Some(tags) = self.tags.as_ref() {
            form.push(("tags", tags.clone()));
        }
        if let Some(note_id) = self.note_id.as_ref() {
            form.push(("note_id", note_id.clone()));
        }
        if let Some(publish) = self.publish {
            form.push(("publish", bool_flag(publish)));
        }
        if let Some(auto_comment) = self.auto_comment {
            form.push(("auto_comment", bool_flag(auto_comment)));
        }

        form
    }
}

fn normalize_non_blank(field: &'static str, value: String) -> BpiResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BpiError::invalid_parameter(field, "value cannot be blank"));
    }

    Ok(value)
}

fn bool_flag(value: bool) -> String {
    if value {
        "1".to_string()
    } else {
        "0".to_string()
    }
}

// --- 删除视频笔记 ---

impl<'a> NoteClient<'a> {
    /// 保存视频笔记并返回标准 payload 结果。
    pub async fn add(&self, params: NoteAddParams) -> BpiResult<NoteAddResponseData> {
        let csrf = self.client.csrf()?;
        let form = params.form_pairs(csrf);

        self.client
            .post("https://api.bilibili.com/x/note/add")
            .form(&form)
            .send_bpi_payload("note.add")
            .await
    }

    /// 删除视频笔记并返回标准 payload 结果。
    pub async fn delete(&self, params: NoteDeleteParams) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;
        let form = params.form_pairs(csrf);

        self.client
            .post("https://api.bilibili.com/x/note/del")
            .form(&form)
            .send_bpi_optional_payload("note.delete")
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_add_params_rejects_blank_content() {
        let err = NoteAddParams::new(
            Aid::new(170001).expect("test aid should be valid"),
            "title",
            "summary",
            "  ",
        )
        .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "content",
                ..
            }
        ));
    }
}
