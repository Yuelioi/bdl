// 视频观看进度上报相关接口
//
// [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/video)

// --- 测试模块 ---

use crate::BilibiliRequest;
use crate::BpiError;
use crate::response::BpiResult;
use crate::video::VideoClient;

const WATCH_PROGRESS_ENDPOINT: &str = "https://api.bilibili.com/x/v2/history/report";

/// 上报视频观看进度的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoWatchProgressParams {
    aid: u64,
    cid: u64,
    progress: u64,
}

impl VideoWatchProgressParams {
    pub fn new(aid: u64, cid: u64) -> BpiResult<Self> {
        if aid == 0 {
            return Err(BpiError::invalid_parameter("aid", "id must be non-zero"));
        }
        if cid == 0 {
            return Err(BpiError::invalid_parameter("cid", "id must be non-zero"));
        }

        Ok(Self {
            aid,
            cid,
            progress: 0,
        })
    }

    pub fn progress(mut self, progress: u64) -> Self {
        self.progress = progress;
        self
    }

    fn into_multipart(self, csrf: &str) -> reqwest::multipart::Form {
        reqwest::multipart::Form::new()
            .text("aid", self.aid.to_string())
            .text("cid", self.cid.to_string())
            .text("csrf", csrf.to_string())
            .text("progress", self.progress.to_string())
    }
}

impl<'a> VideoClient<'a> {
    /// 上报视频观看进度并返回标准 payload 结果。
    pub async fn report_watch_progress(
        &self,
        params: VideoWatchProgressParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;
        let form = params.into_multipart(&csrf);

        self.client
            .post(WATCH_PROGRESS_ENDPOINT)
            .multipart(form)
            .send_bpi_optional_payload("video.watch_progress.report")
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn watch_progress_params_rejects_zero_aid() {
        let err = VideoWatchProgressParams::new(0, 100).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "aid", .. }
        ));
    }
}
