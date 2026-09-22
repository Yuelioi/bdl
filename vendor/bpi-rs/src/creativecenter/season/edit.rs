// 编辑合集小节 API
//
// [参考文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/season.md)

use crate::BilibiliRequest;
use crate::BpiResult;
use crate::creativecenter::CreativeCenterClient;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// 合集信息编辑

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SeasonEdit {
    pub id: u64,       // 合集 ID
    pub title: String, // 合集标题
    pub cover: String, // 封面图 URL
    #[serde(default)]
    pub desc: Option<String>, // 合集简介
    #[serde(default)]
    pub season_price: Option<u32>, // 合集价格（默认 0）
    #[serde(default, rename = "isEnd")]
    pub is_end: Option<u32>, // 是否完结 0:未完结 1:完结
}

/// 合集小节信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeasonSectionEdit {
    pub id: u64,
    #[serde(rename = "type")]
    pub type_field: u64,
    #[serde(rename = "seasonId")]
    pub season_id: u64,
    pub title: String,
}

/// 合集内视频排序信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionSort {
    pub id: u64,    // 合集内视频 ID
    pub order: u32, // 排序位置
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpisodeEdit {
    pub id: u64,
    pub title: String,
    pub aid: u64,
    pub cid: u64,
    #[serde(rename = "seasonId")]
    pub season_id: u64,
    #[serde(rename = "sectionId")]
    pub section_id: u64,
    pub sorts: Vec<EpisodeSort>,
    pub order: u64,
}

#[derive(Serialize)]
struct EpisodeEditPayload {
    #[serde(flatten)]
    section: EpisodeEdit,
    sorts: Vec<EpisodeSort>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpisodeSort {
    pub id: u64,
    pub sort: u64,
}

/// 合集小节排序信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonSectionSort {
    pub id: u64,   // 小节 ID
    pub sort: u32, // 排序位置
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SectionAddEpisodesRequest {
    #[serde(rename = "sectionId")]
    pub section_id: u64,
    pub episodes: Vec<Episode>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Episode {
    pub title: String,
    pub aid: u64,
    pub cid: u64,
    pub charging_pay: i64,
    pub member_first: i64,
    pub limited_free: bool,
}

impl<'a> CreativeCenterClient<'a> {
    /// 编辑合集信息
    ///
    /// 编辑合集的基本信息，包括标题、封面、简介等。
    ///
    /// # 参数
    /// | 名称 | 类型 | 说明 |
    /// | ---- | ---- | ---- |
    /// | `season` | SeasonEdit | 合集信息 |
    /// | `sorts` | `Vec<SeasonSectionSort>` | 小节排序列表 |
    ///
    /// # 文档
    /// [编辑合集信息](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/season/edit.md#编辑合集信息)
    pub async fn season_edit(
        &self,
        season: SeasonEdit,
        sorts: Vec<SeasonSectionSort>,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        let payload = json!({
            "season": season,
            "sorts": sorts
        });

        self.client
            .post("https://member.bilibili.com/x2/creative/web/season/edit")
            .query(&[("csrf", csrf)])
            .json(&payload)
            .send_bpi_optional_payload("creativecenter.season.edit")
            .await
    }
    /// 编辑合集小节(需要开启小节功能)
    ///
    /// 编辑合集中的小节信息，包括小节标题和视频排序。
    ///
    /// # 参数
    /// | 名称 | 类型 | 说明 |
    /// | ---- | ---- | ---- |
    /// | `section` | SeasonSectionEdit | 小节信息 |
    /// | `sorts` | `Vec<SectionSort>` | 视频排序信息 |
    ///
    /// # 文档
    /// [编辑合集小节](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/season/edit.md#编辑合集小节)
    pub async fn season_section_edit(
        &self,
        section: SeasonSectionEdit,
        sorts: Vec<SectionSort>,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        let payload = json!({
            "section": section,
            "sorts": sorts
        });

        self.client
            .post("https://member.bilibili.com/x2/creative/web/season/section/edit")
            .query(&[("csrf", csrf)])
            .json(&payload)
            .send_bpi_optional_payload("creativecenter.season.section.edit")
            .await
    }

    /// 编辑小节中的章节
    ///
    /// 编辑合集中的小节信息，包括小节标题和视频排序。
    ///
    /// # 参数
    /// | 名称 | 类型 | 说明 |
    /// | ---- | ---- | ---- |
    /// | `section` | SeasonSectionEdit | 小节信息 |
    /// | `sorts` | `Vec<SectionSort>` | 视频排序信息 |
    ///
    /// # 文档
    /// [编辑合集小节](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/season/edit.md#编辑合集小节)
    pub async fn season_section_episode_edit(
        &self,
        section: EpisodeEdit,
        sorts: Vec<EpisodeSort>,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        let payload = EpisodeEditPayload { section, sorts };

        self.client
            .post("https://member.bilibili.com/x2/creative/web/season/section/episode/edit")
            .query(&[("csrf", csrf)])
            .json(&payload)
            .send_bpi_optional_payload("creativecenter.season.section.episode.edit")
            .await
    }

    /// 切换小节/正常显示
    ///
    /// # 参数
    /// * season_id 合集id
    pub async fn season_enable_section(
        &self,
        season_id: u64,
        enable: bool,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;
        let params = vec![
            ("csrf", csrf),
            ("season_id", season_id.to_string()),
            ("no_section", (if enable { "0" } else { "1" }).to_string()),
        ];

        self.client
            .post("https://member.bilibili.com/x2/creative/web/season/section/switch")
            .form(&params)
            .send_bpi_optional_payload("creativecenter.season.section.switch")
            .await
    }
    /// 添加视频到小节(需要开启小节功能)
    ///
    /// 将视频添加到指定的合集小节中。
    ///
    /// # 参数
    /// | 名称 | 类型 | 说明 |
    /// | ---- | ---- | ---- |
    /// | `aid` | u64 | 视频 aid |
    /// | `season_id` | u64 | 合集 ID |
    /// | `section_id` | u64 | 小节 ID |
    /// | `title` | &str | 视频标题 |
    ///
    /// # 文档
    /// [编辑投稿视频合集/小节](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/season/edit.md#编辑投稿视频合集小节)
    pub async fn season_section_add_episodes(
        &self,
        section_id: u64,
        episodes: Vec<Episode>,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        let payload = SectionAddEpisodesRequest {
            section_id,
            episodes,
        };

        self.client
            .post("https://member.bilibili.com/x2/creative/web/season/section/episodes/add")
            .json(&payload)
            .query(&[("csrf", csrf)])
            .send_bpi_optional_payload("creativecenter.season.section.episodes.add")
            .await
    }
}

#[cfg(test)]
mod tests {}
