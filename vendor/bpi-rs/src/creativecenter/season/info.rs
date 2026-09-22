//! 获取合集列表 API
//!
//! [参考文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/season.md)

use serde::{Deserialize, Serialize};

use super::models::{Season, Section};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SeasonInfoData {
    pub season: Season,
    pub course: serde_json::Value,
    pub checkin: serde_json::Value,
    #[serde(rename = "seasonStat")]
    pub season_stat: serde_json::Value,
    pub sections: Sections,
    pub part_episodes: serde_json::Value,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Sections {
    pub sections: Vec<Section>,
    pub total: i64,
}

#[cfg(test)]
mod tests {
    use crate::creativecenter::season::SeasonInfoParams;
    use crate::ids::SeasonId;
    use crate::{BpiClient, BpiError};

    const TEST_SSID: u64 = 4294056;

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_season_list() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");
        let params = SeasonInfoParams::new(SeasonId::new(TEST_SSID)?);
        let data = bpi.creativecenter().season_info(params).await?;

        tracing::info!("共 {:?} 个合集", data);

        Ok(())
    }
}
