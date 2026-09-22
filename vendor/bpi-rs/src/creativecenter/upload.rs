// 创作中心上传 API
//
// [参考文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/upload.md)

use crate::BilibiliRequest;
use crate::BpiError;
use crate::BpiResult;
use crate::creativecenter::CreativeCenterClient;
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

/// 上传封面返回结果

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadCoverData {
    pub url: String,
}

impl<'a> CreativeCenterClient<'a> {
    /// 上传视频封面
    ///
    /// 上传视频封面图片，支持多种输入格式。
    ///
    /// # 参数
    /// | 名称 | 类型 | 说明 |
    /// | ---- | ---- | ---- |
    /// | `mime_type` | &str | 图片 MIME 类型，如 image/jpeg |
    /// | `cover` | `AsRef<str>` | 封面数据，可以是：纯 base64、完整 data URI、文件路径 |
    ///
    /// # 注意
    /// 文件不得超过 20M
    ///
    /// # 文档
    /// [上传视频封面](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/upload.md#上传视频封面)
    pub async fn upload_cover(
        &self,
        mime_type: &str,
        cover: impl AsRef<str>,
    ) -> BpiResult<UploadCoverData> {
        let csrf = self.client.csrf()?;
        let cover_str = cover.as_ref();

        let final_cover = if cover_str.starts_with("data:") {
            cover_str.to_string()
        } else if cover_str
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
        {
            format!("data:{};base64,{}", mime_type, cover_str)
        } else {
            let file_bytes = fs::read(cover_str).map_err(|e| BpiError::Parse {
                message: format!("读取文件失败: {}", e),
            })?;
            let file_base64 = general_purpose::STANDARD.encode(file_bytes);
            format!("data:{};base64,{}", mime_type, file_base64)
        };

        let mut form = HashMap::new();
        form.insert("csrf", csrf);
        form.insert("cover", final_cover);

        self.client
            .post("https://member.bilibili.com/x/vu/web/cover/up")
            .form(&form)
            .send_bpi_payload("creativecenter.cover.upload")
            .await
    }
}
