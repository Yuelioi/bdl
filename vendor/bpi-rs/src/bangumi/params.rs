use crate::ids::{Cid, EpisodeId, MediaId, SeasonId};
use crate::models::{Fnval, VideoQuality};
use crate::{BpiError, BpiResult};

use super::timeline::BangumiTimelineType;

/// `/pgc/review/user` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BangumiInfoParams {
    media_id: MediaId,
}

impl BangumiInfoParams {
    pub fn new(media_id: MediaId) -> Self {
        Self { media_id }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("media_id", self.media_id.to_string())]
    }
}

/// 通过 season ID 或 episode ID 标识 bangumi 详情请求。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BangumiDetailId {
    Season(SeasonId),
    Episode(EpisodeId),
}

/// `/pgc/view/web/season` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BangumiDetailParams {
    id: BangumiDetailId,
}

impl BangumiDetailParams {
    pub fn from_season_id(season_id: SeasonId) -> Self {
        Self {
            id: BangumiDetailId::Season(season_id),
        }
    }

    pub fn from_episode_id(episode_id: EpisodeId) -> Self {
        Self {
            id: BangumiDetailId::Episode(episode_id),
        }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        match self.id {
            BangumiDetailId::Season(season_id) => {
                vec![("season_id", season_id.to_string())]
            }
            BangumiDetailId::Episode(episode_id) => {
                vec![("ep_id", episode_id.to_string())]
            }
        }
    }
}

/// `/pgc/web/season/section` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BangumiSectionsParams {
    season_id: SeasonId,
}

impl BangumiSectionsParams {
    pub fn new(season_id: SeasonId) -> Self {
        Self { season_id }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("season_id", self.season_id.to_string())]
    }
}

/// 通过 episode ID 或内容 ID 标识 bangumi 播放 URL 请求。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BangumiVideoStreamId {
    Episode(EpisodeId),
    Content(Cid),
}

/// `/pgc/player/web/playurl` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BangumiVideoStreamParams {
    id: BangumiVideoStreamId,
    quality: Option<VideoQuality>,
    fnval: Option<Fnval>,
}

impl BangumiVideoStreamParams {
    pub fn from_episode_id(episode_id: EpisodeId) -> Self {
        Self {
            id: BangumiVideoStreamId::Episode(episode_id),
            quality: None,
            fnval: None,
        }
    }

    pub fn from_cid(cid: Cid) -> Self {
        Self {
            id: BangumiVideoStreamId::Content(cid),
            quality: None,
            fnval: None,
        }
    }

    pub fn with_quality(mut self, quality: VideoQuality) -> Self {
        self.quality = Some(quality);
        self
    }

    pub fn with_fnval(mut self, fnval: Fnval) -> Self {
        self.fnval = Some(fnval);
        self
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![("fnver", "0".to_string())];

        if self.fnval.is_some_and(|fnval| fnval.is_fourk()) {
            params.push(("fourk", "1".to_string()));
        }

        match self.id {
            BangumiVideoStreamId::Episode(episode_id) => {
                params.push(("ep_id", episode_id.to_string()));
            }
            BangumiVideoStreamId::Content(cid) => {
                params.push(("cid", cid.to_string()));
            }
        }

        if let Some(quality) = self.quality {
            params.push(("qn", quality.as_u32().to_string()));
        }

        if let Some(fnval) = self.fnval {
            params.push(("fnval", fnval.bits().to_string()));
        }

        params
    }
}

/// `/pgc/web/timeline` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BangumiTimelineParams {
    timeline_type: BangumiTimelineType,
    before: i32,
    after: i32,
}

impl BangumiTimelineParams {
    pub fn new(timeline_type: BangumiTimelineType, before: i32, after: i32) -> BpiResult<Self> {
        Ok(Self {
            timeline_type,
            before: validate_timeline_day_offset("before", before)?,
            after: validate_timeline_day_offset("after", after)?,
        })
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("types", self.timeline_type.as_i32().to_string()),
            ("before", self.before.to_string()),
            ("after", self.after.to_string()),
        ]
    }
}

fn validate_timeline_day_offset(field: &'static str, value: i32) -> BpiResult<i32> {
    if !(0..=7).contains(&value) {
        return Err(BpiError::invalid_parameter(
            field,
            "value must be between 0 and 7",
        ));
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bangumi_info_params_serializes_media_id() -> BpiResult<()> {
        let params = BangumiInfoParams::new(MediaId::new(28_220_978)?);

        assert_eq!(
            params.query_pairs(),
            vec![("media_id", "28220978".to_string())]
        );
        Ok(())
    }

    #[test]
    fn bangumi_detail_params_serializes_season_id() -> BpiResult<()> {
        let params = BangumiDetailParams::from_season_id(SeasonId::new(1172)?);

        assert_eq!(
            params.query_pairs(),
            vec![("season_id", "1172".to_string())]
        );
        Ok(())
    }

    #[test]
    fn bangumi_detail_params_serializes_episode_id() -> BpiResult<()> {
        let params = BangumiDetailParams::from_episode_id(EpisodeId::new(21265)?);

        assert_eq!(params.query_pairs(), vec![("ep_id", "21265".to_string())]);
        Ok(())
    }

    #[test]
    fn bangumi_sections_params_serializes_season_id() -> BpiResult<()> {
        let params = BangumiSectionsParams::new(SeasonId::new(1172)?);

        assert_eq!(
            params.query_pairs(),
            vec![("season_id", "1172".to_string())]
        );
        Ok(())
    }

    #[test]
    fn bangumi_video_stream_params_serializes_episode_id() -> BpiResult<()> {
        let params = BangumiVideoStreamParams::from_episode_id(EpisodeId::new(21_265)?)
            .with_quality(VideoQuality::P480)
            .with_fnval(Fnval::DASH);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("fnver", "0".to_string()),
                ("ep_id", "21265".to_string()),
                ("qn", "32".to_string()),
                ("fnval", "16".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn bangumi_video_stream_params_serializes_cid_and_fourk_flag() -> BpiResult<()> {
        let params = BangumiVideoStreamParams::from_cid(Cid::new(91_549_662)?)
            .with_quality(VideoQuality::P4K)
            .with_fnval(Fnval::DASH | Fnval::FOURK);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("fnver", "0".to_string()),
                ("fourk", "1".to_string()),
                ("cid", "91549662".to_string()),
                ("qn", "120".to_string()),
                ("fnval", "144".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn bangumi_timeline_params_serializes_query() -> BpiResult<()> {
        let params = BangumiTimelineParams::new(BangumiTimelineType::Anime, 3, 7)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("types", "1".to_string()),
                ("before", "3".to_string()),
                ("after", "7".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn bangumi_timeline_params_rejects_large_before() {
        let err = BangumiTimelineParams::new(BangumiTimelineType::Anime, 8, 7).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "before",
                ..
            }
        ));
    }
}
