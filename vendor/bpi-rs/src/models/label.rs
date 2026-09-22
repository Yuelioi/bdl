use serde::{Deserialize, Serialize};

/// Label 跳转信息
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct LabelGoto {
    pub mobile: String,
    pub pc_web: String,
}
