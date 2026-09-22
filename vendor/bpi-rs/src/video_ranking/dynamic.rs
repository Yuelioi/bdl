use serde::{Deserialize, Serialize};

#[cfg(test)]
use super::params::{
    VideoRegionDynamicParams, VideoRegionNewListRankParams, VideoRegionTagDynamicParams,
};

// --- 获取分区最新视频列表 ---

/// 分区最新视频的页面信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegionPage {
    /// 总计视频数
    pub count: u32,
    /// 当前页码
    pub num: u32,
    /// 每页项数
    pub size: u32,
}

/// 分区最新视频列表的数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegionArchivesData {
    /// 视频列表
    pub archives: Vec<serde_json::Value>, // archives内容复杂，这里用Value代替
    /// 页面信息
    pub page: RegionPage,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewListRankResult {
    /// 发布时间
    #[serde(rename = "pubdate")]
    pub pub_date: String,
    /// 封面图
    pub pic: String,
    /// 标签
    pub tag: String,
    /// 时长 (秒)
    pub duration: u32,
    /// avid
    pub id: u64,
    /// 排序分数
    pub rank_score: Option<u64>,
    /// 是否有角标
    pub badgepay: bool,
    /// 发送时间 (UNIX 秒级时间戳)
    pub senddate: Option<u64>,
    /// UP 主名
    pub author: String,
    /// 评论数
    pub review: u64,
    /// UP 主 mid
    pub mid: u64,
    /// 是否为联合投稿
    pub is_union_video: u8,
    /// 排序索引号
    pub rank_index: Option<u64>,
    /// 类型
    #[serde(rename = "type")]
    pub type_name: String,
    /// 播放数
    pub play: String,
    /// 弹幕数
    #[serde(rename = "video_review")]
    pub video_review: u64,
    /// 是否付费
    pub is_pay: u8,
    /// 收藏数
    pub favorites: u64,
    /// 视频播放页 URL
    pub arcurl: String,
    /// bvid
    pub bvid: String,
    /// 标题
    pub title: String,
    /// 简介
    pub description: String,
    // 忽略其他作用不明确的字段
}

/// 带排序的分区投稿列表数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NewListRankData {
    /// 结果本体
    pub result: Option<Vec<NewListRankResult>>,
    /// 总计视频数
    #[serde(rename = "numResults")]
    pub num_results: u32,
    /// 页码
    pub page: u32,
    /// 视频数
    pub pagesize: u32,
    /// 结果信息
    pub msg: String,
}

#[cfg(test)]
mod tests {
    use super::super::params::VideoNewListRankOrder;
    use super::*;
    use crate::BpiClient;
    use chrono::{Duration, Local};
    use tracing::info;

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_region_dynamic() {
        let bpi = BpiClient::new().expect("client should build");
        let rid = 21; // 日常分区
        let params = VideoRegionDynamicParams::new(rid)
            .expect("rid is valid")
            .with_page(1)
            .expect("page is valid")
            .with_page_size(2)
            .expect("page size is valid");
        let resp = bpi.video_ranking().region_dynamic(params).await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("total videos: {}", data.page.count);
        info!("first item: {:?}", data.archives.first());
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_region_tag_dynamic() {
        let bpi = BpiClient::new().expect("client should build");
        let rid = 136; // 音游分区
        let tag_id = 10026108; // Phigros
        let params = VideoRegionTagDynamicParams::new(rid, tag_id)
            .expect("required ids are valid")
            .with_page(1)
            .expect("page is valid")
            .with_page_size(2)
            .expect("page size is valid");
        let resp = bpi.video_ranking().region_tag_dynamic(params).await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("total videos: {}", data.page.count);
        info!("first item: {:?}", data.archives.first());
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_region_newlist_rank() {
        let bpi = BpiClient::new().expect("client should build");
        let cate_id = 231; // 计算机技术
        let pagesize = 2;
        let today = Local::now().date_naive();
        let seven_days_ago = today - Duration::days(7);

        let time_from = seven_days_ago.format("%Y%m%d").to_string();
        let time_to = today.format("%Y%m%d").to_string();

        let params = VideoRegionNewListRankParams::new(cate_id, pagesize, time_from, time_to)
            .expect("rank params are valid")
            .with_order(VideoNewListRankOrder::Click)
            .with_page(1)
            .expect("page is valid");

        let resp = bpi.video_ranking().region_newlist_rank(params).await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("total results: {}", data.num_results);
        info!("first result: {:?}", data.result.unwrap().first());
    }
}
