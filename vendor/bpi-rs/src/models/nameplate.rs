use serde::{Deserialize, Serialize};

/// 勋章信息
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Nameplate {
    /// 勋章id
    pub nid: u64,
    /// 勋章名称
    pub name: String,
    /// 勋章图标
    pub image: String,
    /// 勋章图标（小）
    pub image_small: String,
    /// 勋章等级
    pub level: String,
    /// 获取条件
    pub condition: String,
}
