use crate::ids::{Aid, Bvid, Cid};
use crate::{BpiError, BpiResult};

/// 用 AV 数字 ID 或 BV 字符串 ID 标识一个 Bilibili 视频。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VideoId {
    /// AV 数字视频 ID。
    Aid(Aid),
    /// BV 字符串视频 ID。
    Bvid(Bvid),
}

/// `/x/web-interface/view` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoViewParams {
    id: VideoId,
}

impl VideoViewParams {
    /// 使用已验证的 AV ID 创建视频详情参数。
    pub fn from_aid(aid: Aid) -> Self {
        Self {
            id: VideoId::Aid(aid),
        }
    }

    /// 使用已验证的 BV ID 创建视频详情参数。
    pub fn from_bvid(bvid: Bvid) -> Self {
        Self {
            id: VideoId::Bvid(bvid),
        }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        video_id_query_pairs(&self.id)
    }
}

/// `/x/web-interface/view/detail` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoDetailParams {
    id: VideoId,
    need_elec: Option<u8>,
}

impl VideoDetailParams {
    /// 使用已验证的 AV ID 创建详情参数。
    pub fn from_aid(aid: Aid) -> Self {
        Self::new(VideoId::Aid(aid))
    }

    /// 使用已验证的 BV ID 创建详情参数。
    pub fn from_bvid(bvid: Bvid) -> Self {
        Self::new(VideoId::Bvid(bvid))
    }

    fn new(id: VideoId) -> Self {
        Self {
            id,
            need_elec: None,
        }
    }

    /// 控制详情接口是否包含充电数据。
    pub fn need_elec(mut self, need_elec: bool) -> Self {
        self.need_elec = Some(u8::from(need_elec));
        self
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = video_id_query_pairs(&self.id);

        if let Some(need_elec) = self.need_elec {
            params.push(("need_elec", need_elec.to_string()));
        }

        params
    }
}

/// `/x/player/pagelist` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoPageListParams {
    id: VideoId,
}

impl VideoPageListParams {
    /// 使用已验证的 AV ID 创建分 P 列表参数。
    pub fn from_aid(aid: Aid) -> Self {
        Self {
            id: VideoId::Aid(aid),
        }
    }

    /// 使用已验证的 BV ID 创建分 P 列表参数。
    pub fn from_bvid(bvid: Bvid) -> Self {
        Self {
            id: VideoId::Bvid(bvid),
        }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        video_id_query_pairs(&self.id)
    }
}

/// `/x/web-interface/archive/desc` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoDescParams {
    id: VideoId,
}

impl VideoDescParams {
    /// 使用已验证的 AV ID 创建简介参数。
    pub fn from_aid(aid: Aid) -> Self {
        Self {
            id: VideoId::Aid(aid),
        }
    }

    /// 使用已验证的 BV ID 创建简介参数。
    pub fn from_bvid(bvid: Bvid) -> Self {
        Self {
            id: VideoId::Bvid(bvid),
        }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        video_id_query_pairs(&self.id)
    }
}

/// `/x/player/wbi/playurl` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoPlayUrlParams {
    id: VideoId,
    cid: Cid,
    qn: Option<u64>,
    fnval: Option<u64>,
    fnver: Option<u64>,
    fourk: Option<u8>,
    platform: String,
    high_quality: Option<u8>,
    try_look: Option<u8>,
}

impl VideoPlayUrlParams {
    /// 使用已验证的 AV ID 和分 P/内容 ID 创建播放 URL 参数。
    pub fn from_aid(aid: Aid, cid: Cid) -> Self {
        Self::new(VideoId::Aid(aid), cid)
    }

    /// 使用已验证的 BV ID 和分 P/内容 ID 创建播放 URL 参数。
    pub fn from_bvid(bvid: Bvid, cid: Cid) -> Self {
        Self::new(VideoId::Bvid(bvid), cid)
    }

    fn new(id: VideoId, cid: Cid) -> Self {
        Self {
            id,
            cid,
            qn: None,
            fnval: None,
            fnver: None,
            fourk: None,
            platform: "pc".to_string(),
            high_quality: None,
            try_look: None,
        }
    }

    /// 设置请求的清晰度代码。
    pub fn quality(mut self, qn: u64) -> Self {
        self.qn = Some(qn);
        self
    }

    /// 设置 Bilibili 流格式位掩码。
    pub fn format_flags(mut self, fnval: u64) -> Self {
        self.fnval = Some(fnval);
        self
    }

    /// 设置流格式版本。
    pub fn format_version(mut self, fnver: u64) -> Self {
        self.fnver = Some(fnver);
        self
    }

    /// 控制是否允许 4K 流。
    pub fn fourk(mut self, enabled: bool) -> Self {
        self.fourk = Some(u8::from(enabled));
        self
    }

    /// 设置 API 平台标记。默认值为 `pc`。
    pub fn platform(mut self, platform: impl Into<String>) -> Self {
        self.platform = platform.into();
        self
    }

    /// 设置高画质播放标记。
    pub fn high_quality(mut self, enabled: bool) -> Self {
        self.high_quality = Some(u8::from(enabled));
        self
    }

    /// 控制是否请求试看。
    pub fn try_look(mut self, enabled: bool) -> Self {
        self.try_look = Some(u8::from(enabled));
        self
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![("cid", self.cid.to_string())];

        match &self.id {
            VideoId::Aid(aid) => params.push(("avid", aid.to_string())),
            VideoId::Bvid(bvid) => params.push(("bvid", bvid.to_string())),
        }
        if let Some(qn) = self.qn {
            params.push(("qn", qn.to_string()));
        }
        if let Some(fnval) = self.fnval {
            params.push(("fnval", fnval.to_string()));
        }
        if let Some(fnver) = self.fnver {
            params.push(("fnver", fnver.to_string()));
        }
        if let Some(fourk) = self.fourk {
            params.push(("fourk", fourk.to_string()));
        }
        params.push(("platform", self.platform.clone()));
        if let Some(high_quality) = self.high_quality {
            params.push(("high_quality", high_quality.to_string()));
        }
        if let Some(try_look) = self.try_look {
            params.push(("try_look", try_look.to_string()));
        }

        params
    }
}

/// `/x/player/online/total` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoOnlineTotalParams {
    id: VideoId,
    cid: Cid,
}

impl VideoOnlineTotalParams {
    pub fn from_aid(aid: Aid, cid: Cid) -> Self {
        Self {
            id: VideoId::Aid(aid),
            cid,
        }
    }

    pub fn from_bvid(bvid: Bvid, cid: Cid) -> Self {
        Self {
            id: VideoId::Bvid(bvid),
            cid,
        }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![("cid", self.cid.to_string())];
        params.extend(video_id_query_pairs(&self.id));
        params
    }
}

/// `/x/player/wbi/v2` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoPlayerInfoParams {
    id: VideoId,
    cid: Cid,
    season_id: Option<u64>,
    ep_id: Option<u64>,
}

impl VideoPlayerInfoParams {
    pub fn from_aid(aid: Aid, cid: Cid) -> Self {
        Self::new(VideoId::Aid(aid), cid)
    }

    pub fn from_bvid(bvid: Bvid, cid: Cid) -> Self {
        Self::new(VideoId::Bvid(bvid), cid)
    }

    fn new(id: VideoId, cid: Cid) -> Self {
        Self {
            id,
            cid,
            season_id: None,
            ep_id: None,
        }
    }

    pub fn season_id(mut self, season_id: u64) -> BpiResult<Self> {
        self.season_id = Some(validate_nonzero_u64("season_id", season_id)?);
        Ok(self)
    }

    pub fn ep_id(mut self, ep_id: u64) -> BpiResult<Self> {
        self.ep_id = Some(validate_nonzero_u64("ep_id", ep_id)?);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![("cid", self.cid.to_string())];
        params.extend(video_id_query_pairs(&self.id));
        if let Some(season_id) = self.season_id {
            params.push(("season_id", season_id.to_string()));
        }
        if let Some(ep_id) = self.ep_id {
            params.push(("ep_id", ep_id.to_string()));
        }
        params
    }
}

/// `/x/web-interface/archive/related` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoRelatedParams {
    id: VideoId,
}

impl VideoRelatedParams {
    pub fn from_aid(aid: Aid) -> Self {
        Self {
            id: VideoId::Aid(aid),
        }
    }

    pub fn from_bvid(bvid: Bvid) -> Self {
        Self {
            id: VideoId::Bvid(bvid),
        }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        video_id_query_pairs(&self.id)
    }
}

/// `/x/web-interface/wbi/index/top/feed/rcmd` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoHomepageRecommendationsParams {
    page_size: u8,
    fresh_idx: u32,
    fetch_row: u32,
}

impl VideoHomepageRecommendationsParams {
    pub fn new() -> Self {
        Self {
            page_size: 12,
            fresh_idx: 1,
            fetch_row: 1,
        }
    }

    pub fn page_size(mut self, page_size: u8) -> BpiResult<Self> {
        if page_size == 0 || page_size > 30 {
            return Err(BpiError::invalid_parameter(
                "ps",
                "value must be between 1 and 30",
            ));
        }

        self.page_size = page_size;
        Ok(self)
    }

    pub fn fresh_idx(mut self, fresh_idx: u32) -> BpiResult<Self> {
        self.fresh_idx = validate_nonzero_u32("fresh_idx", fresh_idx)?;
        Ok(self)
    }

    pub fn fetch_row(mut self, fetch_row: u32) -> BpiResult<Self> {
        self.fetch_row = validate_nonzero_u32("fetch_row", fetch_row)?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("fresh_type", "4".to_string()),
            ("ps", self.page_size.to_string()),
            ("fresh_idx", self.fresh_idx.to_string()),
            ("fresh_idx_1h", self.fresh_idx.to_string()),
            ("brush", self.fresh_idx.to_string()),
            ("fetch_row", self.fetch_row.to_string()),
        ]
    }
}

impl Default for VideoHomepageRecommendationsParams {
    fn default() -> Self {
        Self::new()
    }
}

/// `/x/web-interface/view/conclusion/get` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoAiSummaryParams {
    id: VideoId,
    cid: Cid,
    up_mid: u64,
}

impl VideoAiSummaryParams {
    pub fn from_aid(aid: Aid, cid: Cid, up_mid: u64) -> BpiResult<Self> {
        Self::new(VideoId::Aid(aid), cid, up_mid)
    }

    pub fn from_bvid(bvid: Bvid, cid: Cid, up_mid: u64) -> BpiResult<Self> {
        Self::new(VideoId::Bvid(bvid), cid, up_mid)
    }

    fn new(id: VideoId, cid: Cid, up_mid: u64) -> BpiResult<Self> {
        Ok(Self {
            id,
            cid,
            up_mid: validate_nonzero_u64("up_mid", up_mid)?,
        })
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![
            ("cid", self.cid.to_string()),
            ("up_mid", self.up_mid.to_string()),
        ];
        params.extend(video_id_query_pairs(&self.id));
        params
    }
}

/// `/x/web-interface/view/detail/tag` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoTagsParams {
    id: VideoId,
    cid: Option<Cid>,
}

impl VideoTagsParams {
    pub fn from_aid(aid: Aid) -> Self {
        Self {
            id: VideoId::Aid(aid),
            cid: None,
        }
    }

    pub fn from_bvid(bvid: Bvid) -> Self {
        Self {
            id: VideoId::Bvid(bvid),
            cid: None,
        }
    }

    pub fn cid(mut self, cid: Cid) -> Self {
        self.cid = Some(cid);
        self
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = video_id_query_pairs(&self.id);
        if let Some(cid) = self.cid {
            params.push(("cid", cid.to_string()));
        }
        params
    }
}

/// `/x/stein/edgeinfo_v2` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InteractiveVideoInfoParams {
    id: VideoId,
    graph_version: u64,
    edge_id: Option<u64>,
}

impl InteractiveVideoInfoParams {
    pub fn from_aid(aid: Aid, graph_version: u64) -> BpiResult<Self> {
        Self::new(VideoId::Aid(aid), graph_version)
    }

    pub fn from_bvid(bvid: Bvid, graph_version: u64) -> BpiResult<Self> {
        Self::new(VideoId::Bvid(bvid), graph_version)
    }

    fn new(id: VideoId, graph_version: u64) -> BpiResult<Self> {
        Ok(Self {
            id,
            graph_version: validate_nonzero_u64("graph_version", graph_version)?,
            edge_id: None,
        })
    }

    pub fn edge_id(mut self, edge_id: u64) -> BpiResult<Self> {
        self.edge_id = Some(validate_nonzero_u64("edge_id", edge_id)?);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![("graph_version", self.graph_version.to_string())];
        params.extend(video_id_query_pairs(&self.id));
        if let Some(edge_id) = self.edge_id {
            params.push(("edge_id", edge_id.to_string()));
        }
        params
    }
}

fn video_id_query_pairs(id: &VideoId) -> Vec<(&'static str, String)> {
    match id {
        VideoId::Aid(aid) => vec![("aid", aid.to_string())],
        VideoId::Bvid(bvid) => vec![("bvid", bvid.to_string())],
    }
}

fn validate_nonzero_u32(field: &'static str, value: u32) -> BpiResult<u32> {
    if value == 0 {
        return Err(BpiError::invalid_parameter(field, "value must be non-zero"));
    }

    Ok(value)
}

fn validate_nonzero_u64(field: &'static str, value: u64) -> BpiResult<u64> {
    if value == 0 {
        return Err(BpiError::invalid_parameter(field, "value must be non-zero"));
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BpiError;
    use crate::ids::{Aid, Cid};

    #[test]
    fn video_view_params_serializes_bvid_query() -> Result<(), BpiError> {
        let params = VideoViewParams::from_bvid("BV1xx411c7mD".parse()?);

        assert_eq!(
            params.query_pairs(),
            vec![("bvid", "BV1xx411c7mD".to_string())]
        );
        Ok(())
    }

    #[test]
    fn video_view_params_serializes_aid_query() -> Result<(), BpiError> {
        let params = VideoViewParams::from_aid(Aid::new(170001)?);

        assert_eq!(params.query_pairs(), vec![("aid", "170001".to_string())]);
        Ok(())
    }

    #[test]
    fn video_detail_params_serializes_bvid_query_with_electric_flag() -> Result<(), BpiError> {
        let params = VideoDetailParams::from_bvid("BV1xx411c7mD".parse()?).need_elec(false);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("bvid", "BV1xx411c7mD".to_string()),
                ("need_elec", "0".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn video_page_list_params_serializes_aid_query() -> Result<(), BpiError> {
        let params = VideoPageListParams::from_aid(Aid::new(170001)?);

        assert_eq!(params.query_pairs(), vec![("aid", "170001".to_string())]);
        Ok(())
    }

    #[test]
    fn video_desc_params_serializes_bvid_query() -> Result<(), BpiError> {
        let params = VideoDescParams::from_bvid("BV1xx411c7mD".parse()?);

        assert_eq!(
            params.query_pairs(),
            vec![("bvid", "BV1xx411c7mD".to_string())]
        );
        Ok(())
    }

    #[test]
    fn video_play_url_params_serializes_aid_query_with_default_platform() -> Result<(), BpiError> {
        let params = VideoPlayUrlParams::from_aid(Aid::new(170001)?, Cid::new(180001)?);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("cid", "180001".to_string()),
                ("avid", "170001".to_string()),
                ("platform", "pc".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn video_play_url_params_serializes_optional_playback_flags() -> Result<(), BpiError> {
        let params = VideoPlayUrlParams::from_bvid("BV1xx411c7mD".parse()?, Cid::new(180001)?)
            .quality(120)
            .format_flags(16 | 128)
            .format_version(0)
            .fourk(true)
            .high_quality(true)
            .try_look(false);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("cid", "180001".to_string()),
                ("bvid", "BV1xx411c7mD".to_string()),
                ("qn", "120".to_string()),
                ("fnval", "144".to_string()),
                ("fnver", "0".to_string()),
                ("fourk", "1".to_string()),
                ("platform", "pc".to_string()),
                ("high_quality", "1".to_string()),
                ("try_look", "0".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn video_online_total_params_serializes_bvid_and_cid_query() -> Result<(), BpiError> {
        let params = VideoOnlineTotalParams::from_bvid("BV1xx411c7mD".parse()?, Cid::new(62131)?);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("cid", "62131".to_string()),
                ("bvid", "BV1xx411c7mD".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn video_player_info_params_serializes_optional_context() -> Result<(), BpiError> {
        let params = VideoPlayerInfoParams::from_aid(Aid::new(170001)?, Cid::new(180001)?)
            .season_id(42)?
            .ep_id(43)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("cid", "180001".to_string()),
                ("aid", "170001".to_string()),
                ("season_id", "42".to_string()),
                ("ep_id", "43".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn video_homepage_recommendations_params_serializes_defaults() {
        let params = VideoHomepageRecommendationsParams::new();

        assert_eq!(
            params.query_pairs(),
            vec![
                ("fresh_type", "4".to_string()),
                ("ps", "12".to_string()),
                ("fresh_idx", "1".to_string()),
                ("fresh_idx_1h", "1".to_string()),
                ("brush", "1".to_string()),
                ("fetch_row", "1".to_string()),
            ]
        );
    }

    #[test]
    fn video_homepage_recommendations_params_rejects_oversized_page() {
        let err = VideoHomepageRecommendationsParams::new()
            .page_size(31)
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "ps", .. }
        ));
    }

    #[test]
    fn video_ai_summary_params_rejects_zero_up_mid() -> Result<(), BpiError> {
        let err = VideoAiSummaryParams::from_bvid("BV1xx411c7mD".parse()?, Cid::new(62131)?, 0)
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "up_mid",
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn video_tags_params_serializes_optional_cid() -> Result<(), BpiError> {
        let params = VideoTagsParams::from_bvid("BV1xx411c7mD".parse()?).cid(Cid::new(62131)?);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("bvid", "BV1xx411c7mD".to_string()),
                ("cid", "62131".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn interactive_video_info_params_serializes_start_node() -> Result<(), BpiError> {
        let params = InteractiveVideoInfoParams::from_aid(Aid::new(114347430905959)?, 1273647)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("graph_version", "1273647".to_string()),
                ("aid", "114347430905959".to_string())
            ]
        );
        Ok(())
    }
}
