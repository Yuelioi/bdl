//! 查询视频分P列表 (avid/bvid 转 cid)
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/video)
//!
//! [文档](https://socialsisteryi.github.io/bilibili-API-collect/docs/video/video.html#查询视频分p列表)

use serde::{Deserialize, Serialize};

/// 分P分辨率信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageDimension {
    /// 宽度
    pub width: u32,
    /// 高度
    pub height: u32,
    /// 是否将宽高对换，0: 正常，1: 对换
    pub rotate: u8,
}

/// 分P信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageItem {
    /// 当前分P cid
    pub cid: u64,
    /// 当前分P页码
    pub page: u32,
    /// 视频来源：vupload/芒果TV/腾讯
    pub from: String,
    /// 当前分P标题
    pub part: String,
    /// 当前分P持续时间（秒）
    pub duration: u32,
    /// 站外视频 vid
    pub vid: String,
    /// 站外跳转 URL
    pub weblink: String,
    /// 分辨率信息，部分视频可能不存在
    pub dimension: Option<PageDimension>,
    /// 分P封面
    pub first_frame: Option<String>,

    pub ctime: u64,
}

#[cfg(test)]
mod tests {
    use crate::ids::Aid;
    use crate::video::params::VideoPageListParams;
    use crate::{BpiClient, BpiError};

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_pagelist() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");

        let data = bpi
            .video()
            .page_list(VideoPageListParams::from_aid(Aid::new(10001)?))
            .await?;

        for item in data {
            tracing::info!(
                "P{}: {}, cid={}, duration={}秒",
                item.page,
                item.part,
                item.cid,
                item.duration
            );
        }

        Ok(())
    }
}
