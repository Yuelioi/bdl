// B站视频合集相关接口实现
//
// [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/video)

// --- 响应数据结构体 ---

use crate::BilibiliRequest;
use crate::BpiError;
use crate::BpiResult;
use crate::video::VideoClient;
use serde::{Deserialize, Serialize};

const CREATE_AND_ADD_ARCHIVES_ENDPOINT: &str =
    "https://api.bilibili.com/x/series/series/createAndAddArchives";
const DELETE_SERIES_ENDPOINT: &str = "https://api.bilibili.com/x/series/series/delete";
const DELETE_ARCHIVES_ENDPOINT: &str = "https://api.bilibili.com/x/series/series/delArchives";
const ADD_ARCHIVES_ENDPOINT: &str = "https://api.bilibili.com/x/series/series/addArchives";
const UPDATE_SERIES_ENDPOINT: &str = "https://api.bilibili.com/x/series/series/update";

/// 创建视频列表响应数据

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateSeriesResponseData {
    /// 视频列表 ID
    pub series_id: u64,
}

/// 创建视频系列并向其中添加稿件的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionCreateAndAddArchivesParams {
    mid: u64,
    name: String,
    keywords: Option<String>,
    description: Option<String>,
    aids: Option<String>,
}

impl CollectionCreateAndAddArchivesParams {
    pub fn new(mid: u64, name: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            mid: validate_nonzero_u64("mid", mid)?,
            name: normalize_non_blank("name", name.into())?,
            keywords: None,
            description: None,
            aids: None,
        })
    }

    pub fn keywords(mut self, keywords: impl Into<String>) -> BpiResult<Self> {
        self.keywords = Some(normalize_non_blank("keywords", keywords.into())?);
        Ok(self)
    }

    pub fn description(mut self, description: impl Into<String>) -> BpiResult<Self> {
        self.description = Some(normalize_non_blank("description", description.into())?);
        Ok(self)
    }

    pub fn aids(mut self, aids: impl Into<String>) -> BpiResult<Self> {
        self.aids = Some(normalize_non_blank("aids", aids.into())?);
        Ok(self)
    }

    fn into_multipart(self) -> reqwest::multipart::Form {
        let mut form = reqwest::multipart::Form::new()
            .text("mid", self.mid.to_string())
            .text("name", self.name);

        if let Some(keywords) = self.keywords {
            form = form.text("keywords", keywords);
        }
        if let Some(description) = self.description {
            form = form.text("description", description);
        }
        if let Some(aids) = self.aids {
            form = form.text("aids", aids);
        }

        form
    }
}

/// 删除视频系列的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CollectionDeleteSeriesParams {
    mid: u64,
    series_id: u64,
}

impl CollectionDeleteSeriesParams {
    pub fn new(mid: u64, series_id: u64) -> BpiResult<Self> {
        Ok(Self {
            mid: validate_nonzero_u64("mid", mid)?,
            series_id: validate_nonzero_u64("series_id", series_id)?,
        })
    }

    fn query_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("csrf", csrf.to_string()),
            ("mid", self.mid.to_string()),
            ("series_id", self.series_id.to_string()),
            ("aids", String::new()),
        ]
    }
}

/// 在视频系列中添加或删除稿件的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionArchivesMutationParams {
    mid: u64,
    series_id: u64,
    aids: String,
}

impl CollectionArchivesMutationParams {
    pub fn new(mid: u64, series_id: u64, aids: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            mid: validate_nonzero_u64("mid", mid)?,
            series_id: validate_nonzero_u64("series_id", series_id)?,
            aids: normalize_non_blank("aids", aids.into())?,
        })
    }

    fn form_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("mid", self.mid.to_string()),
            ("series_id", self.series_id.to_string()),
            ("aids", self.aids.clone()),
        ]
    }
}

/// 编辑现有视频系列的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionUpdateSeriesParams {
    mid: u64,
    series_id: u64,
    name: String,
    keywords: Option<String>,
    description: Option<String>,
    add_aids: Option<String>,
    del_aids: Option<String>,
}

impl CollectionUpdateSeriesParams {
    /// 使用必需的账号、系列和标题字段创建参数。
    pub fn new(mid: u64, series_id: u64, name: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            mid: validate_nonzero_u64("mid", mid)?,
            series_id: validate_nonzero_u64("series_id", series_id)?,
            name: normalize_non_blank("name", name.into())?,
            keywords: None,
            description: None,
            add_aids: None,
            del_aids: None,
        })
    }

    /// 设置逗号分隔的关键词。
    pub fn keywords(mut self, keywords: impl Into<String>) -> BpiResult<Self> {
        self.keywords = Some(normalize_non_blank("keywords", keywords.into())?);
        Ok(self)
    }

    /// 设置系列简介。
    pub fn description(mut self, description: impl Into<String>) -> BpiResult<Self> {
        self.description = Some(normalize_non_blank("description", description.into())?);
        Ok(self)
    }

    /// 设置要添加到系列中的逗号分隔 AID。
    pub fn add_aids(mut self, aids: impl Into<String>) -> BpiResult<Self> {
        self.add_aids = Some(normalize_non_blank("add_aids", aids.into())?);
        Ok(self)
    }

    /// 设置要从系列中移除的逗号分隔 AID。
    pub fn del_aids(mut self, aids: impl Into<String>) -> BpiResult<Self> {
        self.del_aids = Some(normalize_non_blank("del_aids", aids.into())?);
        Ok(self)
    }

    fn form_pairs(&self) -> Vec<(&'static str, String)> {
        let mut form = vec![
            ("mid", self.mid.to_string()),
            ("series_id", self.series_id.to_string()),
            ("name", self.name.clone()),
        ];

        if let Some(keywords) = self.keywords.as_ref() {
            form.push(("keywords", keywords.clone()));
        }
        if let Some(description) = self.description.as_ref() {
            form.push(("description", description.clone()));
        }
        if let Some(add_aids) = self.add_aids.as_ref() {
            form.push(("add_aids", add_aids.clone()));
        }
        if let Some(del_aids) = self.del_aids.as_ref() {
            form.push(("del_aids", del_aids.clone()));
        }

        form
    }

    fn into_multipart(self) -> reqwest::multipart::Form {
        self.form_pairs()
            .into_iter()
            .fold(reqwest::multipart::Form::new(), |form, (key, value)| {
                form.text(key, value)
            })
    }
}

// --- 测试模块 ---

impl<'a> VideoClient<'a> {
    /// 创建视频系列，并可选择向其中添加稿件。
    pub async fn create_collection_series(
        &self,
        params: CollectionCreateAndAddArchivesParams,
    ) -> BpiResult<CreateSeriesResponseData> {
        let csrf = self.client.csrf()?;
        let form = params.into_multipart();

        self.client
            .post(CREATE_AND_ADD_ARCHIVES_ENDPOINT)
            .query(&[("csrf", csrf)])
            .multipart(form)
            .send_bpi_payload("video.collection.series.create")
            .await
    }

    /// 删除视频系列。
    pub async fn delete_collection_series(
        &self,
        params: CollectionDeleteSeriesParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        self.client
            .post(DELETE_SERIES_ENDPOINT)
            .query(&params.query_pairs(&csrf))
            .send_bpi_optional_payload("video.collection.series.delete")
            .await
    }

    /// 从视频系列中删除稿件。
    pub async fn delete_collection_archives(
        &self,
        params: CollectionArchivesMutationParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        self.client
            .post(DELETE_ARCHIVES_ENDPOINT)
            .query(&[("csrf", csrf)])
            .form(&params.form_pairs())
            .send_bpi_optional_payload("video.collection.archives.delete")
            .await
    }

    /// 向视频系列中添加稿件。
    pub async fn add_collection_archives(
        &self,
        params: CollectionArchivesMutationParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        self.client
            .post(ADD_ARCHIVES_ENDPOINT)
            .query(&[("csrf", csrf)])
            .form(&params.form_pairs())
            .send_bpi_optional_payload("video.collection.archives.add")
            .await
    }

    /// 更新视频系列。
    pub async fn update_collection_series(
        &self,
        params: CollectionUpdateSeriesParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;
        let form = params.into_multipart();

        self.client
            .post(UPDATE_SERIES_ENDPOINT)
            .query(&[("csrf", csrf)])
            .multipart(form)
            .send_bpi_optional_payload("video.collection.series.update")
            .await
    }
}

fn validate_nonzero_u64(field: &'static str, value: u64) -> BpiResult<u64> {
    if value == 0 {
        return Err(BpiError::invalid_parameter(field, "value must be non-zero"));
    }

    Ok(value)
}

fn normalize_non_blank(field: &'static str, value: String) -> BpiResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BpiError::invalid_parameter(field, "value cannot be blank"));
    }

    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    // 请在运行测试前设置环境变量 `BPI_COOKIE`，以包含 SESSDATA 等登录信息
    // mid 和 series_id 根据实际情况修改

    // 测试用的 mid
    // 测试用的合集 ID

    #[test]
    fn collection_update_series_params_rejects_blank_name() {
        let err = CollectionUpdateSeriesParams::new(42, 100, "  ").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "name", .. }
        ));
    }
}
