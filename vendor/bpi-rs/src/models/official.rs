use serde::{Deserialize, Serialize};

/// 官方认证信息 2
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct OfficialVerify {
    #[serde(rename = "type", default)]
    pub r#type: i32,
    pub desc: String,
}

/// 认证信息
#[derive(Default, Debug, Serialize, Clone, Deserialize)]
pub struct Official {
    /// 认证类型
    pub role: i32,
    /// 认证信息
    pub title: String,
    /// 认证备注
    pub desc: String,
    /// 是否认证 -1:无 0:认证
    #[serde(rename = "type")]
    pub r#type: i32,
}
