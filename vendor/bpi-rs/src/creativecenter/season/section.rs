//! 获取合集小节中的视频 API
//!
//! [参考文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/season.md)

use serde::{Deserialize, Serialize};

/// 小节中的视频信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonSectionEpisodesData {
    pub section: SeasonSectionInfo,
    pub episodes: Option<Vec<SeasonSectionEpisode>>,
}

/// 小节信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeasonSectionInfo {
    pub id: u64,
    #[serde(rename = "type")]
    pub section_type: u32,
    #[serde(rename = "seasonId")]
    pub season_id: i64,
    pub title: String,
    pub order: i64,
    pub state: i64,
    pub part_state: i64,
    pub reject_reason: String,
    pub ctime: i64,
    pub mtime: i64,
    pub ep_count: i64,
    pub cover: String,
    #[serde(rename = "has_charging_pay")]
    pub has_charging_pay: i64,
    #[serde(rename = "Episodes")]
    pub episodes: serde_json::Value,
    pub show: i64,
    #[serde(rename = "has_pugv_pay")]
    pub has_pugv_pay: i64,
}

/// 小节中的单个视频信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonSectionEpisode {
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
    pub state: u32,
    pub cover: String,
    #[serde(rename = "is_free")]
    pub is_free: u32,
    #[serde(rename = "aid_owner")]
    pub aid_owner: bool,
    #[serde(rename = "charging_pay")]
    pub charging_pay: u32,
}

#[cfg(test)]
mod tests {
    use crate::creativecenter::season::SeasonSectionEpisodesParams;
    use crate::ids::SeasonId;
    use crate::{BpiClient, BpiError};

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_season_section_episodes() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");

        let season_id = 176088;
        let params = SeasonSectionEpisodesParams::new(SeasonId::new(season_id)?);
        let data = bpi.creativecenter().season_section_episodes(params).await?;

        tracing::info!("小节: {} - {}", data.section.id, data.section.title);
        if let Some(episodes) = data.episodes
            && let Some(ep) = episodes.into_iter().next()
        {
            tracing::info!("视频: {} - {} (aid={})", ep.id, ep.title, ep.aid);
        }

        Ok(())
    }
}
