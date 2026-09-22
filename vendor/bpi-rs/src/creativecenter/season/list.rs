//! 获取合集列表 API
//!
//! [参考文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/season/list.md)

use serde::{Deserialize, Serialize};

use super::models::{Season, Section};

/// 合集列表返回结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonListData {
    pub seasons: Vec<SeasonItem>,
    pub tip: serde_json::Value,
    pub total: u32,
    pub play_type: u32,
}

/// 单个合集条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonItem {
    pub season: Season,
    pub course: Option<serde_json::Value>,
    pub checkin: Option<CheckInInfo>,
    #[serde(rename = "seasonStat")]
    pub season_stat: Option<SeasonStat>,
    pub sections: Option<SectionsWrapper>,
    pub part_episodes: Vec<PartEpisode>,
}

/// 合集审核信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckInInfo {
    pub status: i32,
    pub status_reason: Option<String>,
    pub season_status: i32,
}

/// 合集统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonStat {
    pub view: u64,
    pub danmaku: u64,
    pub reply: u64,
    pub fav: u64,
    pub coin: u64,
    pub share: u64,
    #[serde(rename = "nowRank")]
    pub now_rank: u32,
    #[serde(rename = "hisRank")]
    pub his_rank: u32,
    pub like: u64,
    pub subscription: u64,
    pub vt: u32,
}

/// 小节包装器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionsWrapper {
    pub sections: Vec<Section>,
}

/// 合集内视频条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartEpisode {
    pub id: u64,
    pub title: String,
    pub aid: u64,
    pub bvid: String,
    pub cid: u64,
    #[serde(rename = "seasonId")]
    pub season_id: u64,
    #[serde(rename = "sectionId")]
    pub section_id: u64,
    pub order: u32,
    #[serde(rename = "videoTitle")]
    pub video_title: Option<String>,
    #[serde(rename = "archiveTitle")]
    pub archive_title: Option<String>,
    #[serde(rename = "archiveState")]
    pub archive_state: i32,
    #[serde(rename = "rejectReason")]
    pub reject_reason: Option<String>,
    pub state: i32,
    pub cover: String,
    pub is_free: i32,
    pub aid_owner: bool,
    #[serde(rename = "charging_pay")]
    pub charging_pay: u32,
}

#[cfg(test)]
mod tests {
    use crate::creativecenter::season::{SeasonListOrder, SeasonListParams, SeasonListSort};
    use crate::{BpiClient, BpiError};

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_season_list() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let params = SeasonListParams::new(1, 10)?
            .with_order(SeasonListOrder::CreatedAt)
            .with_sort(SeasonListSort::Desc);
        let data = bpi.creativecenter().season_list(params).await?;

        tracing::info!("共 {} 个合集", data.total);
        for s in data.seasons {
            tracing::info!("合集: {} - {}", s.season.id, s.season.title);
        }

        Ok(())
    }
}
