// --- 图片上传 API 结构体 ---

use crate::BilibiliRequest;
use crate::BpiError;
use crate::BpiResult;
use crate::dynamic::DynamicClient;
use reqwest::Body;
use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

const UPLOAD_PIC_ENDPOINT: &str = "https://api.bilibili.com/x/dynamic/feed/draw/upload_bfs";
const CREATE_TEXT_ENDPOINT: &str = "https://api.vc.bilibili.com/dynamic_svr/v1/dynamic_svr/create";
const CREATE_COMPLEX_ENDPOINT: &str = "https://api.bilibili.com/x/dynamic/feed/create/dyn";

/// 图片上传响应数据

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UploadPicData {
    /// 已上传图片 URL
    pub image_url: String,
    /// 已上传图片宽度
    pub image_width: u64,
    /// 已上传图片高度
    pub image_height: u64,
    /// 已上传图片大小k
    pub img_size: f64,
}

// --- 创建投票 API 结构体 ---

/// 创建投票响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateVoteData {
    /// 投票 ID
    pub vote_id: u64,
}

// --- 发布纯文本动态 API 结构体 ---

/// 纯文本动态响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateDynamicData {
    /// 动态 ID
    pub dynamic_id: u64,
    /// 动态 ID 字符串格式
    pub dynamic_id_str: String,
    // ... 其他字段
}

// 复杂动态

/// 动态内容组件
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicContentItem {
    /// 组件类型 ID，例如 @用户
    #[serde(rename = "type")]
    pub type_num: u8,
    /// 组件的内容 ID，例如用户的 mid
    pub biz_id: Option<String>,
    /// 文本内容
    pub raw_text: String,
}

/// 动态图片
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicPic {
    /// 图片 URL
    pub img_src: String,
    /// 图片高度
    pub img_height: u64,
    /// 图片宽度
    pub img_width: u64,
    /// 图片大小 (KB)
    pub img_size: f64,
}

/// 动态话题
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicTopic {
    /// 话题 ID
    pub id: u64,
    /// 话题名
    pub name: String,
    /// 来源，非必要
    pub from_source: Option<String>,
    /// 来源话题 ID，非必要
    pub from_topic_id: Option<u64>,
}

/// 动态互动设置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicOption {
    /// 开启精选评论
    pub up_choose_comment: Option<u8>,
    /// 关闭评论
    pub close_comment: Option<u8>,
}

/// 复杂动态请求体
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicRequest {
    /// 特殊卡片，非必要
    pub attach_card: Option<serde_json::Value>,
    /// 动态内容
    pub content: DynamicContent,
    /// 元信息，非必要
    pub meta: Option<serde_json::Value>,
    /// 动态类型
    pub scene: u8,
    /// 携带的图片
    pub pics: Option<Vec<DynamicPic>>,
    /// 话题
    pub topic: Option<DynamicTopic>,
    /// 互动设置
    pub option: Option<DynamicOption>,
}

/// 动态内容
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicContent {
    pub contents: Vec<DynamicContentItem>,
}

/// 复杂动态响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateComplexDynamicData {
    pub dyn_id: u64,
    pub dyn_id_str: String,
    pub dyn_type: u8,
}

/// 上传动态图片的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicUploadPicParams {
    file_path: PathBuf,
    category: String,
}

impl DynamicUploadPicParams {
    pub fn new(file_path: impl Into<PathBuf>) -> Self {
        Self {
            file_path: file_path.into(),
            category: "daily".to_string(),
        }
    }

    pub fn category(mut self, category: impl Into<String>) -> BpiResult<Self> {
        self.category = normalize_non_blank("category", category.into())?;
        Ok(self)
    }
}

/// 发布纯文本动态的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DynamicTextCreateParams {
    content: String,
}

impl DynamicTextCreateParams {
    pub fn new(content: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            content: normalize_non_blank("content", content.into())?,
        })
    }
}

/// 发布复杂动态的参数。
#[derive(Debug, Clone)]
pub struct DynamicComplexCreateParams {
    scene: u8,
    contents: Vec<DynamicContentItem>,
    pics: Option<Vec<DynamicPic>>,
    topic: Option<DynamicTopic>,
}

impl DynamicComplexCreateParams {
    pub fn new(scene: u8, contents: Vec<DynamicContentItem>) -> BpiResult<Self> {
        if !matches!(scene, 1 | 2 | 4) {
            return Err(BpiError::invalid_parameter(
                "scene",
                "value must be 1, 2, or 4",
            ));
        }
        if contents.is_empty() {
            return Err(BpiError::invalid_parameter(
                "contents",
                "at least one content item is required",
            ));
        }

        Ok(Self {
            scene,
            contents,
            pics: None,
            topic: None,
        })
    }

    pub fn pics(mut self, pics: Vec<DynamicPic>) -> BpiResult<Self> {
        if pics.is_empty() {
            return Err(BpiError::invalid_parameter(
                "pics",
                "at least one picture is required",
            ));
        }
        self.pics = Some(pics);
        Ok(self)
    }

    pub fn topic(mut self, topic: DynamicTopic) -> Self {
        self.topic = Some(topic);
        self
    }

    fn request_body(self) -> serde_json::Value {
        let dyn_req = DynamicRequest {
            attach_card: None,
            content: DynamicContent {
                contents: self.contents,
            },
            meta: Some(json!({
                "app_meta": {
                    "from": "create.dynamic.web",
                    "mobi_app": "web"
                }
            })),
            scene: self.scene,
            pics: self.pics,
            topic: self.topic,
            option: None,
        };

        json!({ "dyn_req": dyn_req })
    }
}

impl<'a> DynamicClient<'a> {
    /// 上传动态图片并返回标准 payload 结果。
    pub async fn upload_pic(&self, params: DynamicUploadPicParams) -> BpiResult<UploadPicData> {
        let csrf = self.client.csrf()?;

        let file = File::open(&params.file_path)
            .await
            .map_err(|_| BpiError::parse("打开文件失败"))?;
        let stream = FramedRead::new(file, BytesCodec::new());
        let body = Body::wrap_stream(stream);

        let file_name = params.file_path.file_name().ok_or_else(|| {
            BpiError::parse("Invalid file path, cannot get file name".to_string())
        })?;

        let file_part = Part::stream(body)
            .file_name(file_name.to_string_lossy().into_owned())
            .mime_str("image/jpeg")?;

        let form = Form::new()
            .part("file_up", file_part)
            .text("csrf", csrf.clone())
            .text("category", params.category)
            .text("biz", "new_dyn".to_string());

        self.client
            .post(UPLOAD_PIC_ENDPOINT)
            .multipart(form)
            .send_bpi_payload("dynamic.pic.upload")
            .await
    }

    /// 发布纯文本动态并返回标准 payload 结果。
    pub async fn create_text(
        &self,
        params: DynamicTextCreateParams,
    ) -> BpiResult<CreateDynamicData> {
        let csrf = self.client.csrf()?;
        let form = Form::new()
            .text("dynamic_id", "0")
            .text("type", "4")
            .text("rid", "0")
            .text("content", params.content)
            .text("csrf", csrf.clone())
            .text("csrf_token", csrf);

        self.client
            .post(CREATE_TEXT_ENDPOINT)
            .multipart(form)
            .send_bpi_payload("dynamic.text.create")
            .await
    }

    /// 发布复杂动态并返回标准 payload 结果。
    pub async fn create_complex(
        &self,
        params: DynamicComplexCreateParams,
    ) -> BpiResult<CreateComplexDynamicData> {
        let csrf = self.client.csrf()?;
        let request_body = params.request_body();

        self.client
            .post(CREATE_COMPLEX_ENDPOINT)
            .header("Content-Type", "application/json")
            .query(&[("csrf", csrf)])
            .body(request_body.to_string())
            .send_bpi_payload("dynamic.complex.create")
            .await
    }
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
    use super::*;

    #[test]
    fn dynamic_text_create_params_rejects_blank_content() {
        let err = DynamicTextCreateParams::new("  ").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "content",
                ..
            }
        ));
    }
}
