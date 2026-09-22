//! 查询稿件简介相关接口
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/video)

use serde::{Deserialize, Serialize};

/// 稿件简介响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoDescResponse {
    /// 返回码
    /// - 0：成功
    /// - -400：请求错误
    /// - 62002：稿件不可见
    pub code: i32,
    /// 错误信息，默认为 "0"
    pub message: String,
    /// ttl，一般为1
    pub ttl: i32,
    /// 简介内容
    pub data: String,
}

#[cfg(test)]
mod tests {
    use crate::ids::Aid;
    use crate::video::params::VideoDescParams;
    use crate::{BpiClient, BpiError};

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_desc() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");

        let data = bpi
            .video()
            .desc(VideoDescParams::from_aid(Aid::new(10001)?))
            .await?;

        tracing::info!("稿件简介: {}", data);

        Ok(())
    }
}
