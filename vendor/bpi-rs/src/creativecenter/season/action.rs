// 创建合集 API
//
// [参考文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/season.md)

use crate::BilibiliRequest;
use crate::BpiResult;
use crate::creativecenter::CreativeCenterClient;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// 合集视频条目（添加用）

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EpisodeAdd {
    pub title: String,
    pub aid: u64,
    pub cid: u64,

    #[serde(default)]
    pub charging_pay: i64,
    #[serde(default)]
    pub member_first: i64,
    #[serde(default)]
    pub limited_free: bool,
}

impl<'a> CreativeCenterClient<'a> {
    /// 创建合集
    ///
    /// 创建一个新的视频合集，需要提供标题、封面等信息。
    ///
    /// # 参数
    /// | 名称 | 类型 | 说明 |
    /// | ---- | ---- | ---- |
    /// | `title` | &str | 合集标题 |
    /// | `desc` | `Option<&str>` | 合集简介，可选 |
    /// | `cover` | &str | 封面图 URL（从上传接口获取） |
    /// | `season_price` | `Option<u32>` | 合集价格，可选，默认 0 |
    ///
    /// # 文档
    /// [创建合集](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/season/create.md#创建合集)
    pub async fn season_create(
        &self,
        title: &str,
        desc: Option<&str>,
        cover: &str,
        season_price: Option<u32>,
    ) -> BpiResult<u64> {
        // 校验 csrf
        let csrf = self.client.csrf()?;

        let mut form = vec![
            ("title", title.to_string()),
            ("cover", cover.to_string()),
            ("csrf", csrf),
        ];

        if let Some(d) = desc {
            form.push(("desc", d.to_string()));
        }
        if let Some(price) = season_price {
            form.push(("season_price", price.to_string()));
        }

        self.client
            .post("https://member.bilibili.com/x2/creative/web/season/add")
            .form(&form)
            .send_bpi_payload("creativecenter.season.create")
            .await
    }

    /// 删除合集
    ///
    /// 删除指定的合集，需要提供合集 ID。
    ///
    /// # 参数
    /// | 名称 | 类型 | 说明 |
    /// | ---- | ---- | ---- |
    /// | `season_id` | u64 | 合集 ID |
    ///
    /// # 文档
    /// [删除合集](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/season/del.md#删除合集)
    pub async fn season_delete(&self, season_id: u64) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        let form = vec![("id", season_id.to_string()), ("csrf", csrf)];

        self.client
            .post("https://member.bilibili.com/x2/creative/web/season/del")
            .form(&form)
            .send_bpi_optional_payload("creativecenter.season.delete")
            .await
    }

    /// 添加视频到合集
    ///
    /// 将视频添加到指定的合集小节中。
    ///
    /// # 参数
    /// | 名称 | 类型 | 说明 |
    /// | ---- | ---- | ---- |
    /// | `section_id` | u64 | 合集小节 ID |
    /// | `episodes` | `Vec<EpisodeAdd>` | 视频列表 |
    ///
    /// # 文档
    /// [添加视频到合集](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/season/push.md#添加视频到合集)
    pub async fn season_episodes_add(
        &self,
        section_id: u64,
        episodes: Vec<EpisodeAdd>,
    ) -> BpiResult<Option<serde_json::Value>> {
        // 校验 csrf
        let csrf = self.client.csrf()?;

        let payload = json!({
            "sectionId": section_id,
            "episodes": episodes
        });

        self.client
            .post("https://member.bilibili.com/x2/creative/web/season/section/episodes/add")
            .with_bilibili_headers()
            .query(&[("csrf", csrf)])
            .json(&payload)
            .send_bpi_optional_payload("creativecenter.season.episodes.add")
            .await
    }
}

#[cfg(test)]
mod tests {}
