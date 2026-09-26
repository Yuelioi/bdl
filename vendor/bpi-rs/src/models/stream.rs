use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use serde_with::DefaultOnError;
use serde_with::serde_as;

/// 视频清晰度标识
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoQuality {
    /// 240P 极速 (仅mp4方式支持)
    P240 = 6,
    /// 360P 流畅
    P360 = 16,
    /// 480P 清晰
    P480 = 32,
    /// 720P 高清 (web端默认值)
    /// B站前端需要登录才能选择，但是直接发送请求可以不登录就拿到720P的取流地址
    P720 = 64,
    /// 720P60 高帧率
    P720_60 = 74,
    /// 1080P 高清
    P1080 = 80,
    /// 智能修复
    /// 仅支持dash方式
    /// 需要fnval&12240=12240
    Smart = 100,
    /// 1080P+ 高码率
    P1080Plus = 112,
    /// 1080P60 高帧率
    P1080_60 = 116,
    /// 4K 超清
    P4K = 120,
    /// HDR 真彩色
    HDR = 125,
    /// 杜比视界
    DolbyVision = 126,
    /// 8K 超高清
    P8K = 127,
}

impl VideoQuality {
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Fnval: u32 {
        const FLV          = 0; // flv格式 仅H.264编码 0
        const MP4          = 1; // mp4格式 仅H.264编码 1
        const DASH         = 1 << 4;  // H.264编码或H.265编码 16

        const HDR          = 1 << 6;  // HDR 视频 必须为dash格式 需要qn=125 64
        const FOURK        = 1 << 7;  // 4K 分辨率 qn=120 128
        const DOLBY_AUDIO  = 1 << 8;  // 杜比音频 256
        const DOLBY_VISION = 1 << 9;  // 杜比视界 512
        const EIGHTK       = 1 << 10; //  8K 分辨率 qn=127 1024
        const AV1          = 1 << 11; // av1 编码  2048

        const AI_FIX       = 12240; // 智能修复 会顶掉其他值 只能单独使用
    }
}
impl Fnval {
    pub fn is_fourk(&self) -> bool {
        self.contains(Fnval::FOURK) || self.contains(Fnval::EIGHTK)
    }
}

/// 视频编码代码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoCodec {
    /// AVC编码
    Avc = 7,
    /// HEVC编码
    Hevc = 12,
    /// AV1编码
    Av1 = 13,
}

impl VideoCodec {
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// 视频伴音音质代码
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioQuality {
    /// 64K
    K64 = 30216,
    /// 132K
    K132 = 30232,
    /// 192K
    K192 = 30280,
}

impl AudioQuality {
    pub fn as_u32(self) -> u32 {
        self as u32
    }
}

/// 通用视频流响应数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoStreamData {
    /// 当前分辨率代码
    pub quality: u32,
    /// 分辨率代码列表；部分课程响应省略时为空
    #[serde(default)]
    pub accept_quality: Vec<u32>,
    /// 支持的格式；部分课程响应省略时为空
    #[serde(default)]
    pub accept_format: String,
    /// 分辨率描述；部分课程响应省略时为空
    #[serde(default)]
    pub accept_description: Vec<String>,
    /// 当前格式
    pub format: String,
    /// 视频编码ID
    pub video_codecid: u32,

    /// FLV/MP4 直链（可选）
    #[serde(default)]
    pub durl: Option<Vec<Durl>>,
    /// DASH 流（可选）
    #[serde(default)]
    pub dash: Option<DashStreams>,
    /// 是否已付费
    pub has_paid: bool,
    /// 支持的格式列表
    pub support_formats: Vec<SupportFormat>,
    /// 时长，毫秒
    #[serde(default)]
    pub timelength: Option<u64>,
    /// fnval参数
    #[serde(default)]
    pub fnval: Option<u32>,
    /// 是否为预览
    #[serde(default)]
    pub is_preview: Option<u32>,
}

impl VideoStreamData {
    /// 获取最佳格式
    pub fn best_format(&self) -> Option<&SupportFormat> {
        self.support_formats.iter().max_by_key(|f| f.quality)
    }

    /// 获取最佳视频流
    pub fn best_video(&self) -> Option<&DashTrack> {
        self.dash.as_ref().and_then(|dash| {
            dash.video.iter().max_by(|a, b| {
                // 分辨率 > 带宽 > 编码格式
                let res_a = a.width * a.height;
                let res_b = b.width * b.height;

                res_a
                    .cmp(&res_b)
                    .then_with(|| a.bandwidth.cmp(&b.bandwidth))
                    .then_with(|| {
                        let codec_priority = |c: &str| {
                            if c.starts_with("hev1") || c.starts_with("hvc1") {
                                1
                            } else {
                                0
                            }
                        };
                        codec_priority(&a.codecs).cmp(&codec_priority(&b.codecs))
                    })
            })
        })
    }

    /// 获取最佳音频流
    pub fn best_audio(&self) -> Option<&DashTrack> {
        self.dash.as_ref().and_then(|dash| {
            dash.audio.iter().max_by(|a, b| {
                a.bandwidth
                    .cmp(&b.bandwidth)
                    .then_with(|| a.size.cmp(&b.size))
            })
        })
    }

    /// 检查是否支持DASH格式
    pub fn supports_dash(&self) -> bool {
        self.dash.is_some()
    }

    /// 检查是否支持直链格式
    pub fn supports_direct_url(&self) -> bool {
        self.durl.is_some() && !self.durl.as_ref().unwrap().is_empty()
    }

    /// 获取视频时长（秒）
    pub fn duration_seconds(&self) -> Option<u64> {
        self.timelength.map(|ms| ms / 1000)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Durl {
    /// 单位 Byte
    pub size: u64,
    pub ahead: String,
    /// 毫秒
    pub length: u64,
    pub vhead: String,
    /// 备用 URL
    pub backup_url: Vec<String>,
    /// 视频流 URL（120 分钟有效）
    pub url: String,
    /// 分段序号
    pub order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportFormat {
    pub display_desc: String,
    pub format: String,
    pub description: String,
    pub quality: u32,
    pub new_description: String,

    pub superscript: String,

    #[serde(default)]
    // 课程分段不存在
    pub codecs: Vec<String>,

    pub attribute: Option<u32>,
    pub has_preview: Option<bool>,
    pub sub_description: Option<String>,
    pub need_login: Option<bool>,
    pub need_vip: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashStreams {
    pub duration: u64,
    pub min_buffer_time: f64,
    pub video: Vec<DashTrack>,
    pub audio: Vec<DashTrack>,
    pub dolby: Option<DashDolby>,
    pub flac: Option<DashFlac>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashFlac {
    pub display_sample_rate: String,
    pub audio: DashTrack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SegmentBase {
    pub initialization: String,
    pub index_range: String,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashDolby {
    #[serde_as(as = "DefaultOnError")]
    pub r#type: u32,
    pub audio: Vec<DashTrack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashTrack {
    pub id: u32,
    pub base_url: String,
    pub backup_url: Vec<String>,
    pub bandwidth: u32,
    pub mime_type: String,
    pub codecs: String,
    pub width: u32,
    pub height: u32,
    pub frame_rate: String,
    pub sar: String,
    pub start_with_sap: u32,
    pub segment_base: SegmentBase,
    pub codecid: u32,
    pub size: u64,
    pub md5: Option<String>,
}
