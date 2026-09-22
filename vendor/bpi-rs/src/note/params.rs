use crate::ids::{Aid, Cvid, NoteId};
use crate::{BpiError, BpiResult};

/// `/x/note/is_forbid` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoteIsForbidParams {
    aid: Aid,
}

impl NoteIsForbidParams {
    pub fn new(aid: Aid) -> Self {
        Self { aid }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("aid", self.aid.to_string())]
    }
}

/// `/x/note/info` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotePrivateInfoParams {
    aid: Aid,
    note_id: NoteId,
}

impl NotePrivateInfoParams {
    pub fn new(aid: Aid, note_id: NoteId) -> Self {
        Self { aid, note_id }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("oid", self.aid.to_string()),
            ("oid_type", "0".to_string()),
            ("note_id", self.note_id.to_string()),
        ]
    }
}

/// `/x/note/publish/info` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotePublicInfoParams {
    cvid: Cvid,
}

impl NotePublicInfoParams {
    pub fn new(cvid: Cvid) -> Self {
        Self { cvid }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("cvid", self.cvid.to_string())]
    }
}

/// `/x/note/list/archive` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoteArchiveListParams {
    aid: Aid,
}

impl NoteArchiveListParams {
    pub fn new(aid: Aid) -> Self {
        Self { aid }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("oid", self.aid.to_string()), ("oid_type", "0".to_string())]
    }
}

/// `/x/note/list` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoteUserPrivateListParams {
    pagination: NotePagination,
}

impl NoteUserPrivateListParams {
    pub fn new() -> Self {
        Self {
            pagination: NotePagination::default(),
        }
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.pagination.page = validate_positive("pn", page)?;
        Ok(self)
    }

    pub fn with_page_size(mut self, page_size: u32) -> BpiResult<Self> {
        self.pagination.page_size = validate_positive("ps", page_size)?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        self.pagination.query_pairs()
    }
}

impl Default for NoteUserPrivateListParams {
    fn default() -> Self {
        Self::new()
    }
}

/// `/x/note/publish/list/archive` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotePublicArchiveListParams {
    aid: Aid,
    pagination: NotePagination,
}

impl NotePublicArchiveListParams {
    pub fn new(aid: Aid) -> Self {
        Self {
            aid,
            pagination: NotePagination::default(),
        }
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.pagination.page = validate_positive("pn", page)?;
        Ok(self)
    }

    pub fn with_page_size(mut self, page_size: u32) -> BpiResult<Self> {
        self.pagination.page_size = validate_positive("ps", page_size)?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![("oid", self.aid.to_string()), ("oid_type", "0".to_string())];
        params.extend(self.pagination.query_pairs());
        params
    }
}

/// `/x/note/publish/list/user` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoteUserPublicListParams {
    pagination: NotePagination,
}

impl NoteUserPublicListParams {
    pub fn new() -> Self {
        Self {
            pagination: NotePagination::default(),
        }
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.pagination.page = validate_positive("pn", page)?;
        Ok(self)
    }

    pub fn with_page_size(mut self, page_size: u32) -> BpiResult<Self> {
        self.pagination.page_size = validate_positive("ps", page_size)?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        self.pagination.query_pairs()
    }
}

impl Default for NoteUserPublicListParams {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct NotePagination {
    page: u32,
    page_size: u32,
}

impl NotePagination {
    fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("pn", self.page.to_string()),
            ("ps", self.page_size.to_string()),
        ]
    }
}

impl Default for NotePagination {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 10,
        }
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
    fn note_public_info_params_serializes_cvid() -> BpiResult<()> {
        let params = NotePublicInfoParams::new(Cvid::new(15_160_286)?);

        assert_eq!(params.query_pairs(), vec![("cvid", "15160286".to_string())]);
        Ok(())
    }

    #[test]
    fn note_user_public_list_params_serializes_default_pagination() {
        let params = NoteUserPublicListParams::new();

        assert_eq!(
            params.query_pairs(),
            vec![("pn", "1".to_string()), ("ps", "10".to_string())]
        );
    }

    #[test]
    fn note_public_archive_list_params_rejects_zero_page_size() -> BpiResult<()> {
        let err = NotePublicArchiveListParams::new(Aid::new(338_677_252)?)
            .with_page_size(0)
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "ps", .. }
        ));
        Ok(())
    }
}
