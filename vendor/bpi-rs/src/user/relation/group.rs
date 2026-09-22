// B站用户分组相关接口
//
// [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/user)

// --- 响应数据结构体 ---

use crate::BilibiliRequest;
use crate::BpiError;
use crate::BpiResult;
use crate::user::UserClient;
use serde::{Deserialize, Serialize};

/// 创建分组响应数据

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateTagResponseData {
    /// 创建的分组的 ID
    pub tagid: i64,
}

// --- API 实现 ---

// --- 测试模块 ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserGroupCreateParams {
    group_name: String,
}

impl UserGroupCreateParams {
    pub fn new(group_name: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            group_name: normalize_name("group_name", group_name.into())?,
        })
    }

    fn into_multipart(self, csrf: &str) -> reqwest::multipart::Form {
        reqwest::multipart::Form::new()
            .text("tag", self.group_name)
            .text("csrf", csrf.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserGroupUpdateParams {
    tag_id: i64,
    new_name: String,
}

impl UserGroupUpdateParams {
    pub fn new(tag_id: i64, new_name: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            tag_id: validate_positive_i64("tag_id", tag_id)?,
            new_name: normalize_name("new_name", new_name.into())?,
        })
    }

    fn into_multipart(self, csrf: &str) -> reqwest::multipart::Form {
        reqwest::multipart::Form::new()
            .text("tagid", self.tag_id.to_string())
            .text("name", self.new_name)
            .text("csrf", csrf.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UserGroupDeleteParams {
    tag_id: i64,
}

impl UserGroupDeleteParams {
    pub fn new(tag_id: i64) -> BpiResult<Self> {
        Ok(Self {
            tag_id: validate_positive_i64("tag_id", tag_id)?,
        })
    }

    fn into_multipart(self, csrf: &str) -> reqwest::multipart::Form {
        reqwest::multipart::Form::new()
            .text("tagid", self.tag_id.to_string())
            .text("csrf", csrf.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserGroupUsersParams {
    fids: String,
    tagids: String,
}

impl UserGroupUsersParams {
    pub fn new(fids: &[u64], tagids: &[i64]) -> BpiResult<Self> {
        Ok(Self {
            fids: join_u64_ids("fids", fids)?,
            tagids: join_i64_ids("tagids", tagids)?,
        })
    }

    pub fn remove_from_all(fids: &[u64]) -> BpiResult<Self> {
        Ok(Self {
            fids: join_u64_ids("fids", fids)?,
            tagids: "0".to_string(),
        })
    }

    fn into_multipart(self, csrf: &str) -> reqwest::multipart::Form {
        reqwest::multipart::Form::new()
            .text("fids", self.fids)
            .text("tagids", self.tagids)
            .text("csrf", csrf.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserGroupMoveUsersParams {
    fids: String,
    before_tag_ids: String,
    after_tag_ids: String,
}

impl UserGroupMoveUsersParams {
    pub fn new(fids: &[u64], before_tag_ids: &[i64], after_tag_ids: &[i64]) -> BpiResult<Self> {
        Ok(Self {
            fids: join_u64_ids("fids", fids)?,
            before_tag_ids: join_i64_ids("before_tag_ids", before_tag_ids)?,
            after_tag_ids: join_i64_ids("after_tag_ids", after_tag_ids)?,
        })
    }

    fn into_multipart(self, csrf: &str) -> reqwest::multipart::Form {
        reqwest::multipart::Form::new()
            .text("fids", self.fids)
            .text("beforeTagids", self.before_tag_ids)
            .text("afterTagids", self.after_tag_ids)
            .text("csrf", csrf.to_string())
    }
}

impl<'a> UserClient<'a> {
    pub async fn create_group_tag(
        &self,
        params: UserGroupCreateParams,
    ) -> BpiResult<CreateTagResponseData> {
        let csrf = self.client.csrf()?;
        let form = params.into_multipart(&csrf);

        self.client
            .post("https://api.bilibili.com/x/relation/tag/create")
            .multipart(form)
            .send_bpi_payload("user.group.create")
            .await
    }

    pub async fn update_group_tag(
        &self,
        params: UserGroupUpdateParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;
        let form = params.into_multipart(&csrf);

        self.client
            .post("https://api.bilibili.com/x/relation/tag/update")
            .multipart(form)
            .send_bpi_optional_payload("user.group.update")
            .await
    }

    pub async fn delete_group_tag(
        &self,
        params: UserGroupDeleteParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;
        let form = params.into_multipart(&csrf);

        self.client
            .post("https://api.bilibili.com/x/relation/tag/del")
            .multipart(form)
            .send_bpi_optional_payload("user.group.delete")
            .await
    }

    pub async fn add_group_users_to_tags(
        &self,
        params: UserGroupUsersParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;
        let form = params.into_multipart(&csrf);

        self.client
            .post("https://api.bilibili.com/x/relation/tags/addUsers")
            .multipart(form)
            .send_bpi_optional_payload("user.group.users.add")
            .await
    }

    pub async fn remove_group_users(
        &self,
        params: UserGroupUsersParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;
        let form = params.into_multipart(&csrf);

        self.client
            .post("https://api.bilibili.com/x/relation/tags/addUsers")
            .multipart(form)
            .send_bpi_optional_payload("user.group.users.remove")
            .await
    }

    pub async fn copy_group_users_to_tags(
        &self,
        params: UserGroupUsersParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;
        let form = params.into_multipart(&csrf);

        self.client
            .post("https://api.bilibili.com/x/relation/tags/copyUsers")
            .multipart(form)
            .send_bpi_optional_payload("user.group.users.copy")
            .await
    }

    pub async fn move_group_users_to_tags(
        &self,
        params: UserGroupMoveUsersParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;
        let form = params.into_multipart(&csrf);

        self.client
            .post("https://api.bilibili.com/x/relation/tags/moveUsers")
            .multipart(form)
            .send_bpi_optional_payload("user.group.users.move")
            .await
    }
}

fn normalize_name(field: &'static str, value: String) -> BpiResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BpiError::invalid_parameter(field, "value cannot be blank"));
    }
    if value.len() > 16 {
        return Err(BpiError::invalid_parameter(
            field,
            "length cannot exceed 16 bytes",
        ));
    }

    Ok(value)
}

fn validate_positive_i64(field: &'static str, value: i64) -> BpiResult<i64> {
    if value <= 0 {
        return Err(BpiError::invalid_parameter(field, "id must be positive"));
    }

    Ok(value)
}

fn join_u64_ids(field: &'static str, values: &[u64]) -> BpiResult<String> {
    if values.is_empty() || values.contains(&0) {
        return Err(BpiError::invalid_parameter(
            field,
            "ids must be non-empty and non-zero",
        ));
    }

    Ok(values
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(","))
}

fn join_i64_ids(field: &'static str, values: &[i64]) -> BpiResult<String> {
    if values.is_empty() || values.iter().any(|value| *value <= 0) {
        return Err(BpiError::invalid_parameter(
            field,
            "ids must be non-empty and positive",
        ));
    }

    Ok(values
        .iter()
        .map(i64::to_string)
        .collect::<Vec<_>>()
        .join(","))
}

#[cfg(test)]
mod tests {}
