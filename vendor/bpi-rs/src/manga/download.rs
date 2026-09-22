//! 漫画图片下载模型
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/manga/Comic.md)
//!
//! 状态：未实现。
//!
//! 当前 Web 阅读器需要浏览器生成的 proof 字段（`GetImageIndex` 使用 `m2`，
//! `ImageToken` 使用 `m1`）。旧请求形态会返回 API `code = 99`，
//! 因此在 SDK 具备明确的 proof-provider 边界前，此模块有意只暴露数据模型。

use serde::{Deserialize, Serialize};

/// 漫画图片信息
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct MangaImage {
    /// 图片的路径，不包含host
    pub path: String,
    /// 图片宽度，单位：像素px
    pub x: i32,
    /// 图片高度，单位：像素px
    pub y: i32,
    /// 视频路径
    pub video_path: String,
    /// 视频大小
    pub video_size: String,
}

/// 漫画视频信息
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct MangaVideo {
    /// 视频ID
    pub svid: String,
    /// 文件名
    pub filename: String,
    /// 路由
    pub route: String,
    /// 资源
    pub resource: Vec<serde_json::Value>,
    /// 原始宽度
    pub raw_width: String,
    /// 原始高度
    pub raw_height: String,
    /// 原始旋转
    pub raw_rotate: String,
    /// 图片URL列表
    pub img_urls: Vec<String>,
    /// 二进制URL
    pub bin_url: String,
    /// 图片X长度
    pub img_x_len: i32,
    /// 图片X大小
    pub img_x_size: i32,
    /// 图片Y长度
    pub img_y_len: i32,
    /// 图片Y大小
    pub img_y_size: i32,
}

/// 漫画图片索引数据
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ImageIndexData {
    /// .index 文件路径
    pub path: String,
    /// 本话图片信息
    pub images: Vec<MangaImage>,
    /// 本话信息最后修改时间
    pub last_modified: String,
    /// 图片host，通常为 `https://manga.hdslb.com`
    pub host: String,
    /// 视频信息
    pub video: MangaVideo,
}

/// 图片token信息
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ImageToken {
    /// 图片下载的地址
    pub url: String,
    /// 图片下载需要的token
    pub token: String,
}
