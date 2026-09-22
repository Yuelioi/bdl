//! 根据 aid 反查合集信息 API
//!
//! [参考文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/season.md)

use serde::{Deserialize, Serialize};

/// 合集信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonByAidData {
    /// 合集 ID
    pub id: u64,
    /// 合集标题
    pub title: String,
    /// 合集描述
    pub desc: Option<String>,
    /// 合集封面 URL
    pub cover: String,

    /// 是否已完结
    /// 0: 未完结 | 1: 已完结
    #[serde(rename = "isEnd")]
    pub is_end: u32,

    /// 合集作者 ID
    pub mid: u64,

    /// 是否为活动合集
    /// 0: 否 | 1: 是
    #[serde(rename = "isAct")]
    pub is_act: u32,

    /// 是否付费
    /// 0: 否 | 1: 是
    pub is_pay: u32,

    /// 合集状态
    /// 0: 正常显示 | -6: 正在审核
    pub state: i32,

    /// 合集分段状态
    /// 0: 正常
    #[serde(rename = "partState")]
    pub part_state: u32,

    /// 合集签名状态
    /// 0: 正常
    #[serde(rename = "signState")]
    pub sign_state: u32,

    /// 合集拒绝原因
    #[serde(rename = "rejectReason")]
    pub reject_reason: Option<String>,

    /// 创建时间 (UNIX 时间戳)
    pub ctime: u64,
    /// 修改时间 (UNIX 时间戳)
    pub mtime: u64,

    /// 是否设小节
    /// 1: 不设小节
    pub no_section: u32,

    /// 合集是否禁止
    /// 0: 否 | 1: 是
    pub forbid: u32,

    /// 协议 ID
    pub protocol_id: Option<String>,

    /// 视频数量
    pub ep_num: u32,

    /// 合集价格
    /// 0: 免费
    pub season_price: u32,

    /// 是否公开
    /// 1: 公开 | 0: 不公开
    pub is_opened: u32,

    /// 是否充电付费
    /// 0: 否 | 1: 是
    pub has_charging_pay: u32,

    /// 是否 PUGV 付费
    /// 0: 否 | 1: 是
    pub has_pugv_pay: u32,
}

#[cfg(test)]
mod tests {
    use crate::creativecenter::season::SeasonByAidParams;
    use crate::ids::Aid;
    use crate::{BpiClient, BpiError};

    const TEST_AID: u64 = 113602455409683;

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_season_by_aid() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let params = SeasonByAidParams::new(Aid::new(TEST_AID)?);

        let data = bpi.creativecenter().season_by_aid(params).await?;
        tracing::info!("视频 {} 所属合集 {} - {}", TEST_AID, data.id, data.title);

        Ok(())
    }

    #[test]
    fn season_by_aid_params_serializes_query() -> Result<(), BpiError> {
        let params = SeasonByAidParams::new(Aid::new(TEST_AID)?);

        assert_eq!(params.query_pairs(), [("id", TEST_AID.to_string())]);
        Ok(())
    }
}
