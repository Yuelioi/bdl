use crate::ids::{Aid, Cid, EpisodeId, SeasonId};
use crate::models::{Fnval, VideoQuality};
use crate::{BpiError, BpiResult};

/// 通过 season ID 或 episode ID 标识课堂课程请求。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheeseInfoId {
    Season(SeasonId),
    Episode(EpisodeId),
}

/// `/pugv/view/web/season` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheeseInfoParams {
    id: CheeseInfoId,
}

impl CheeseInfoParams {
    pub fn from_season_id(season_id: SeasonId) -> Self {
        Self {
            id: CheeseInfoId::Season(season_id),
        }
    }

    pub fn from_episode_id(episode_id: EpisodeId) -> Self {
        Self {
            id: CheeseInfoId::Episode(episode_id),
        }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        match self.id {
            CheeseInfoId::Season(season_id) => vec![("season_id", season_id.to_string())],
            CheeseInfoId::Episode(episode_id) => vec![("ep_id", episode_id.to_string())],
        }
    }
}

/// `/pugv/view/web/ep/list` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheeseEpListParams {
    season_id: SeasonId,
    page_size: Option<u32>,
    page: Option<u32>,
}

impl CheeseEpListParams {
    pub fn new(season_id: SeasonId) -> Self {
        Self {
            season_id,
            page_size: None,
            page: None,
        }
    }

    pub fn with_page_size(mut self, page_size: u32) -> BpiResult<Self> {
        self.page_size = Some(validate_positive("ps", page_size)?);
        Ok(self)
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.page = Some(validate_positive("pn", page)?);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![("season_id", self.season_id.to_string())];

        if let Some(page_size) = self.page_size {
            params.push(("ps", page_size.to_string()));
        }

        if let Some(page) = self.page {
            params.push(("pn", page.to_string()));
        }

        params
    }
}

/// `/pugv/player/web/playurl` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheeseVideoStreamParams {
    aid: Aid,
    episode_id: EpisodeId,
    cid: Cid,
    quality: Option<VideoQuality>,
    fnval: Option<Fnval>,
}

impl CheeseVideoStreamParams {
    pub fn new(aid: Aid, episode_id: EpisodeId, cid: Cid) -> Self {
        Self {
            aid,
            episode_id,
            cid,
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
        let mut params = vec![
            ("avid", self.aid.to_string()),
            ("ep_id", self.episode_id.to_string()),
            ("cid", self.cid.to_string()),
            ("fnver", "0".to_string()),
        ];

        if self.fnval.is_some_and(|fnval| fnval.is_fourk()) {
            params.push(("fourk", "1".to_string()));
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

fn validate_positive(field: &'static str, value: u32) -> BpiResult<u32> {
    if value == 0 {
        return Err(BpiError::invalid_parameter(
            field,
            "value must be greater than zero",
        ));
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cheese_info_params_serializes_episode_id() -> BpiResult<()> {
        let params = CheeseInfoParams::from_episode_id(EpisodeId::new(20_767)?);

        assert_eq!(params.query_pairs(), vec![("ep_id", "20767".to_string())]);
        Ok(())
    }

    #[test]
    fn cheese_ep_list_params_serializes_optional_pagination() -> BpiResult<()> {
        let params = CheeseEpListParams::new(SeasonId::new(556)?)
            .with_page_size(50)?
            .with_page(1)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("season_id", "556".to_string()),
                ("ps", "50".to_string()),
                ("pn", "1".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn cheese_video_stream_params_serializes_required_query() -> BpiResult<()> {
        let params = CheeseVideoStreamParams::new(
            Aid::new(997_984_154)?,
            EpisodeId::new(163_956)?,
            Cid::new(1_183_682_680)?,
        );

        assert_eq!(
            params.query_pairs(),
            vec![
                ("avid", "997984154".to_string()),
                ("ep_id", "163956".to_string()),
                ("cid", "1183682680".to_string()),
                ("fnver", "0".to_string()),
            ]
        );
        Ok(())
    }
}
