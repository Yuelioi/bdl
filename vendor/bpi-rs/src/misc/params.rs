use crate::ids::Aid;
use crate::{BpiError, BpiResult};

const DEFAULT_PLATFORM: &str = "unix";
const DEFAULT_SHARE_CHANNEL: &str = "COPY";
const DEFAULT_SHARE_ID: &str = "main.ugc-video-detail.0.0.pv";
const DEFAULT_SHARE_MODE: u32 = 4;
const DEFAULT_BUVID: &str = "qwq";
const DEFAULT_BUILD: u64 = 6_114_514;

/// `/x/share/click` b23.tv 短链接生成参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MiscB23ShortLinkParams {
    aid: Aid,
    platform: String,
    share_channel: String,
    share_id: String,
    share_mode: u32,
    buvid: String,
    build: u64,
}

impl MiscB23ShortLinkParams {
    pub fn new(aid: Aid) -> Self {
        Self {
            aid,
            platform: DEFAULT_PLATFORM.to_string(),
            share_channel: DEFAULT_SHARE_CHANNEL.to_string(),
            share_id: DEFAULT_SHARE_ID.to_string(),
            share_mode: DEFAULT_SHARE_MODE,
            buvid: DEFAULT_BUVID.to_string(),
            build: DEFAULT_BUILD,
        }
    }

    pub fn with_platform(mut self, platform: impl Into<String>) -> BpiResult<Self> {
        self.platform = normalize_non_blank("platform", platform.into())?;
        Ok(self)
    }

    pub fn with_share_channel(mut self, share_channel: impl Into<String>) -> BpiResult<Self> {
        self.share_channel = normalize_non_blank("share_channel", share_channel.into())?;
        Ok(self)
    }

    pub fn with_share_id(mut self, share_id: impl Into<String>) -> BpiResult<Self> {
        self.share_id = normalize_non_blank("share_id", share_id.into())?;
        Ok(self)
    }

    pub fn with_share_mode(mut self, share_mode: u32) -> Self {
        self.share_mode = share_mode;
        self
    }

    pub fn with_buvid(mut self, buvid: impl Into<String>) -> BpiResult<Self> {
        self.buvid = normalize_non_blank("buvid", buvid.into())?;
        Ok(self)
    }

    pub fn with_build(mut self, build: u64) -> Self {
        self.build = build;
        self
    }

    pub fn form_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("platform", self.platform.clone()),
            ("share_channel", self.share_channel.clone()),
            ("share_id", self.share_id.clone()),
            ("share_mode", self.share_mode.to_string()),
            ("oid", self.aid.to_string()),
            ("buvid", self.buvid.clone()),
            ("build", self.build.to_string()),
        ]
    }
}

fn normalize_non_blank(field: &'static str, value: String) -> BpiResult<String> {
    let value = value.trim();
    if value.is_empty() {
        return Err(BpiError::invalid_parameter(field, "value cannot be blank"));
    }

    Ok(value.to_string())
}
