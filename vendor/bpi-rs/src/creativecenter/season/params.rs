use crate::ids::{Aid, SeasonId};
use crate::{BpiError, BpiResult};

/// `/x2/creative/web/seasons` 的排序字段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeasonListOrder {
    CreatedAt,
    UpdatedAt,
}

impl SeasonListOrder {
    const fn as_str(self) -> &'static str {
        match self {
            Self::CreatedAt => "ctime",
            Self::UpdatedAt => "mtime",
        }
    }
}

/// `/x2/creative/web/seasons` 的排序方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeasonListSort {
    Asc,
    Desc,
}

impl SeasonListSort {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

/// `/x2/creative/web/seasons` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeasonListParams {
    page: u32,
    page_size: u32,
    order: Option<SeasonListOrder>,
    sort: Option<SeasonListSort>,
}

impl SeasonListParams {
    pub fn new(page: u32, page_size: u32) -> BpiResult<Self> {
        Ok(Self {
            page: validate_non_zero_u32("pn", page)?,
            page_size: validate_non_zero_u32("ps", page_size)?,
            order: None,
            sort: None,
        })
    }

    pub fn with_order(mut self, order: SeasonListOrder) -> Self {
        self.order = Some(order);
        self
    }

    pub fn with_sort(mut self, sort: SeasonListSort) -> Self {
        self.sort = Some(sort);
        self
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut query = vec![
            ("pn", self.page.to_string()),
            ("ps", self.page_size.to_string()),
        ];

        if let Some(order) = self.order {
            query.push(("order", order.as_str().to_string()));
        }

        if let Some(sort) = self.sort {
            query.push(("sort", sort.as_str().to_string()));
        }

        query
    }
}

/// `/x2/creative/web/season/aid` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeasonByAidParams {
    aid: Aid,
}

impl SeasonByAidParams {
    pub fn new(aid: Aid) -> Self {
        Self { aid }
    }

    pub fn query_pairs(self) -> [(&'static str, String); 1] {
        [("id", self.aid.to_string())]
    }
}

/// `/x2/creative/web/season` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeasonInfoParams {
    season_id: SeasonId,
}

impl SeasonInfoParams {
    pub fn new(season_id: SeasonId) -> Self {
        Self { season_id }
    }

    pub fn query_pairs(self) -> [(&'static str, String); 1] {
        [("id", self.season_id.to_string())]
    }
}

/// `/x2/creative/web/season/section` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeasonSectionEpisodesParams {
    season_id: SeasonId,
}

impl SeasonSectionEpisodesParams {
    pub fn new(season_id: SeasonId) -> Self {
        Self { season_id }
    }

    pub fn query_pairs(self) -> [(&'static str, String); 1] {
        [("id", self.season_id.to_string())]
    }
}

fn validate_non_zero_u32(field: &'static str, value: u32) -> BpiResult<u32> {
    if value == 0 {
        return Err(BpiError::invalid_parameter(field, "value must be non-zero"));
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn season_list_params_serializes_required_query() -> Result<(), BpiError> {
        let params = SeasonListParams::new(1, 10)?;

        assert_eq!(
            params.query_pairs(),
            vec![("pn", "1".to_string()), ("ps", "10".to_string())]
        );
        Ok(())
    }

    #[test]
    fn season_list_params_serializes_sorting_query() -> Result<(), BpiError> {
        let params = SeasonListParams::new(1, 10)?
            .with_order(SeasonListOrder::CreatedAt)
            .with_sort(SeasonListSort::Desc);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("pn", "1".to_string()),
                ("ps", "10".to_string()),
                ("order", "ctime".to_string()),
                ("sort", "desc".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn season_list_params_rejects_zero_page() {
        let err = SeasonListParams::new(0, 10).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "pn", .. }
        ));
    }

    #[test]
    fn season_info_params_serializes_query() -> Result<(), BpiError> {
        let params = SeasonInfoParams::new(SeasonId::new(4294056)?);

        assert_eq!(params.query_pairs(), [("id", "4294056".to_string())]);
        Ok(())
    }

    #[test]
    fn season_section_episodes_params_serializes_query() -> Result<(), BpiError> {
        let params = SeasonSectionEpisodesParams::new(SeasonId::new(176088)?);

        assert_eq!(params.query_pairs(), [("id", "176088".to_string())]);
        Ok(())
    }
}
