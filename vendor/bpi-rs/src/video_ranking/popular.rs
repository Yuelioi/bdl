use serde::{Deserialize, Serialize};

#[cfg(test)]
use super::params::{PopularSeriesOneParams, VideoPopularListParams};

// --- 获取当前热门视频列表 ---

/// 热门视频列表的页面信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PopularListData {
    /// 视频列表
    pub list: Vec<serde_json::Value>, // 视频内容复杂，这里用Value代替
    /// 是否有更多数据
    pub no_more: bool,
}

// --- 每周必看全部列表 ---

/// 每周必看列表中的单个必看
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PopularSeriesItem {
    /// 期数
    pub number: u32,
    /// 主题
    pub subject: String,
    /// 状态，2: 已结束
    pub status: u8,
    /// 名称，如 "yyyy第n期 MM.dd - MM.dd"
    pub name: String,
}

/// 每周必看全部列表数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PopularSeriesListData {
    /// 全部信息列表
    pub list: Vec<PopularSeriesItem>,
}

// --- 每周必看选期详细信息 ---

/// 每周必看选期信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PopularSeriesConfig {
    /// 选期 ID
    pub id: u64,
    /// 选期类型
    #[serde(rename = "type")]
    pub type_name: String,
    /// 期数
    pub number: u32,
    /// 主题
    pub subject: String,
    /// 开始时间
    pub stime: u64,
    /// 结束时间
    pub etime: u64,
    /// 状态，2: 已结束
    pub status: u8,
    /// 名称
    pub name: String,
    /// 标题
    pub label: String,
    /// 提示
    pub hint: String,
    /// 颜色
    pub color: u32,
    /// 封面
    pub cover: String,
    /// 分享标题
    pub share_title: String,
    /// 分享副标题
    pub share_subtitle: String,
    /// 媒体 ID
    pub media_id: u64,
}

/// 每周必看选期详细信息数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PopularSeriesOneData {
    /// 选期信息
    pub config: PopularSeriesConfig,
    /// 提醒
    pub reminder: Option<String>,
    /// 选期视频列表
    pub list: Vec<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BpiClient;
    use tracing::info;

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_popular_list() {
        let bpi = BpiClient::new().expect("client should build");
        let params = VideoPopularListParams::new()
            .with_page(1)
            .expect("page is valid")
            .with_page_size(2)
            .expect("page size is valid");
        let resp = bpi.video_ranking().popular_list(params).await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("no_more: {}", data.no_more);
        info!("first item: {:?}", data.list.first());
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_popular_series_list() {
        let bpi = BpiClient::new().expect("client should build");
        let resp = bpi.video_ranking().popular_series_list().await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("first series: {:?}", data.list.first());
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_popular_series_one() {
        let bpi = BpiClient::new().expect("client should build");
        let params = PopularSeriesOneParams::new(1).expect("number is valid");
        let resp = bpi.video_ranking().popular_series_one(params).await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("config: {:?}", data.config);
        info!("first video: {:?}", data.list.first());
    }
}
