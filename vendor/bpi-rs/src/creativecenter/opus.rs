// 创作中心作品管理 API
//
// [参考文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/opus.md)

use crate::BilibiliRequest;
use crate::BpiResult;
use crate::creativecenter::CreativeCenterClient;
use serde_json::json;

impl<'a> CreativeCenterClient<'a> {
    /// 删除动态
    ///
    /// # 参数
    /// | 名称 | 类型 | 说明 |
    /// | ---- | ---- | ---- |
    /// | `dyn_id` | &str | 动态 ID |
    ///
    /// # 文档
    /// [删除动态](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/opus.md#删除动态)
    pub async fn dynamic_delete(&self, dyn_id: &str) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        self.client
            .post("https://api.bilibili.com/x/dynamic/feed/operate/remove")
            .query(&[("csrf", csrf)])
            .json(&json!({
              "dyn_id_str": dyn_id
            }))
            .send_bpi_optional_payload("creativecenter.dynamic.delete")
            .await
    }

    /// 删除专栏
    ///
    /// # 参数
    /// | 名称 | 类型 | 说明 |
    /// | ---- | ---- | ---- |
    /// | `aid` | u64 | 专栏文章 ID |
    ///
    /// # 文档
    /// [删除专栏](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/creativecenter/opus.md#删除专栏)
    pub async fn article_delete(&self, aid: u64) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        self.client
            .post("https://member.bilibili.com/x/web/article/delete")
            .form(&[("aid", aid.to_string()), ("csrf", csrf)])
            .send_bpi_optional_payload("creativecenter.article.delete")
            .await
    }
}
