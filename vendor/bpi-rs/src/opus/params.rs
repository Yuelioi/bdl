use crate::ids::Mid;
use crate::{BpiError, BpiResult};

/// 空间 feed 请求中包含的 opus 条目。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpusSpaceFeedKind {
    All,
    Article,
    Dynamic,
}

impl OpusSpaceFeedKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Article => "article",
            Self::Dynamic => "dynamic",
        }
    }
}

/// `/x/polymer/web-dynamic/v1/opus/feed/space` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpusSpaceFeedParams {
    mid: Mid,
    page: u32,
    offset: Option<String>,
    kind: OpusSpaceFeedKind,
}

impl OpusSpaceFeedParams {
    pub fn new(mid: Mid) -> Self {
        Self {
            mid,
            page: 0,
            offset: None,
            kind: OpusSpaceFeedKind::All,
        }
    }

    pub fn with_page(mut self, page: u32) -> Self {
        self.page = page;
        self
    }

    pub fn with_offset(mut self, offset: impl Into<String>) -> BpiResult<Self> {
        let offset = offset.into();
        if offset.trim().is_empty() {
            return Err(BpiError::invalid_parameter(
                "offset",
                "offset cannot be blank",
            ));
        }

        self.offset = Some(offset);
        Ok(self)
    }

    pub fn with_kind(mut self, kind: OpusSpaceFeedKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut query = vec![
            ("host_mid", self.mid.to_string()),
            ("page", self.page.to_string()),
        ];

        if let Some(offset) = &self.offset {
            query.push(("offset", offset.clone()));
        }

        query.extend([
            ("type", self.kind.as_str().to_string()),
            ("web_location", "333.1387".to_string()),
        ]);

        query
    }
}
