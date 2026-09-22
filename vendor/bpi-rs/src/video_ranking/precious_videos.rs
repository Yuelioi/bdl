use serde::{Deserialize, Serialize};

// --- 获取入站必刷视频 ---

/// 入站必刷视频列表中的单个视频
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PreciousVideoData {
    /// 标题
    pub title: String,
    /// media_id
    pub media_id: u64,
    /// 解释（概括）
    pub explain: String,
    /// 视频列表
    pub list: Vec<serde_json::Value>, // 视频内容复杂，这里用Value代替
}

#[cfg(test)]
mod tests {
    use crate::BpiClient;
    use tracing::info;

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_popular_precious() {
        let bpi = BpiClient::new().expect("client should build");
        let resp = bpi.video_ranking().popular_precious().await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("total lists: {}", data.list.len());
        info!("first list item: {:?}", data.list.first());
    }
}
