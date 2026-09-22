use serde::{Deserialize, Serialize};

/// 头像框信息
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Pendant {
    /// 头像框id
    pub pid: i64,
    /// 头像框名称
    pub name: String,
    /// 头像框图片url
    pub image: String,
    /// 过期时间 此接口返回恒为0
    pub expire: u64,
    /// 头像框图片url
    pub image_enhance: Option<String>,
    /// 头像框图片逐帧序列url
    pub image_enhance_frame: Option<String>,
    /// 新版头像框id
    pub n_pid: Option<u64>,
}
