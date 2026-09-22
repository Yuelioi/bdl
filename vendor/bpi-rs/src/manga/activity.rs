// 漫画任务操作
//
// [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/manga/Activity.md)

// ================= 数据结构 =================

use crate::BpiError;
use crate::manga::MangaClient;
use crate::request::send_bpi_envelope;
use crate::response::BpiResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ShareComicData {
    /// 获取积分
    pub point: i32,
}

pub type ShareComicResponse = BpiResponse<ShareComicData>;

// ================= 实现 =================

impl<'a> MangaClient<'a> {
    /// 分享漫画获取积分
    ///
    /// # 文档
    /// [查看API文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/manga)
    pub async fn manga_share_comic(&self) -> Result<ShareComicResponse, BpiError> {
        let params = [("platform", "android")];
        let request = self
            .client
            .post("https://manga.bilibili.com/twirp/activity.v1.Activity/ShareComic")
            .form(&params);

        send_bpi_envelope(request, "分享漫画").await
    }
}

#[cfg(test)]
mod tests {}
