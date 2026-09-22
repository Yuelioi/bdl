use serde::{Deserialize, Serialize};

/// 合集 Season 信息
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Season {
    pub id: u64,
    pub title: String,
    pub desc: String,
    pub cover: String,

    #[serde(rename = "isEnd")]
    pub is_end: u32,

    pub mid: u64,

    #[serde(rename = "isAct")]
    pub is_act: u32,

    pub is_pay: u32,
    pub state: u32,

    #[serde(rename = "partState")]
    pub part_state: u32,

    #[serde(rename = "signState")]
    pub sign_state: u32,

    #[serde(rename = "rejectReason")]
    pub reject_reason: Option<String>,

    pub ctime: u64,
    pub mtime: u64,

    #[serde(rename = "no_section")]
    pub no_section: u32,

    pub forbid: u32,

    pub protocol_id: Option<String>,
    pub ep_num: u32,

    #[serde(rename = "season_price")]
    pub season_price: u32,

    #[serde(rename = "is_opened")]
    pub is_opened: u32,

    #[serde(rename = "has_charging_pay")]
    pub has_charging_pay: u32,

    /// 只有 Season 有
    pub has_pugv_pay: Option<u32>,

    /// 只有 Season 有，注意 serde 名字
    #[serde(rename = "SeasonUpfrom")]
    pub season_upfrom: Option<u32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Section {
    pub id: u64,

    #[serde(rename = "type")]
    pub section_type: u32,

    #[serde(rename = "seasonId")]
    pub season_id: u64,

    pub title: String,
    pub order: u32,
    pub state: u32,

    #[serde(rename = "partState")]
    pub part_state: u32,

    pub ctime: u64,
    pub mtime: u64,

    #[serde(rename = "epCount")]
    pub ep_count: u32,

    pub cover: String,

    #[serde(rename = "has_charging_pay")]
    pub has_charging_pay: u32,

    pub show: Option<u32>,

    #[serde(rename = "has_pugv_pay")]
    pub has_pugv_pay: Option<u32>,

    #[serde(rename = "rejectReason")]
    pub reject_reason: Option<String>,

    #[serde(rename = "Episodes")]
    pub episodes: Option<serde_json::Value>,
}
