//! B站用户搜索相关接口
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/user)
use serde::{Deserialize, Serialize};

// --- 响应数据结构体 ---

/// 合集或课堂统计数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CollectionStat {
    pub coin: u64,
    pub danmaku: u64,
    pub favorite: u64,
    pub like: u64,
    pub mtime: u64,
    pub reply: u64,
    pub season_id: u64,
    pub share: u64,
    pub view: u64,
    pub vt: u64,
    pub vv: u64,
}

/// 所属合集或课堂元数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VideoMeta {
    pub attribute: u64,
    pub cover: String,
    pub ep_count: u64,
    pub ep_num: u64,
    pub first_aid: u64,
    pub id: u64,
    pub intro: String,
    pub mid: u64,
    pub ptime: u64,
    pub sign_state: u64,
    pub stat: Option<CollectionStat>,
    pub title: String,
}

/// 投稿视频列表项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContributedVideo {
    pub aid: u64,
    pub attribute: u64,
    pub author: String,
    pub bvid: String,
    pub comment: u64,
    pub copyright: String,
    pub created: u64,
    pub description: String,
    pub elec_arc_type: u8,
    pub enable_vt: u8,
    pub hide_click: bool,
    pub is_avoided: u8,
    pub is_charging_arc: bool,
    pub is_lesson_video: u8,
    pub is_lesson_finished: u8,
    pub is_live_playback: u8,
    pub is_pay: u8,
    pub is_self_view: bool,
    pub is_steins_gate: u8,
    pub is_union_video: u8,
    pub jump_url: Option<String>,
    pub length: String,
    pub mid: u64,
    pub meta: Option<VideoMeta>,
    pub pic: String,
    pub play: u64,
    pub playback_position: u64,
    pub review: u64,
    pub season_id: u64,
    pub subtitle: String,
    pub title: String,
    pub typeid: u64,
    pub video_review: u64,
    pub vt: u64,
    pub vt_display: String,
}

/// 投稿视频列表
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContributedVideoList {
    #[serde(default)]
    pub slist: Vec<serde_json::Value>,
    #[serde(default)]
    pub tlist: serde_json::Value,
    #[serde(default)]
    pub vlist: Vec<ContributedVideo>,
}

/// 页面信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PageInfo {
    pub count: u64,
    pub pn: u32,
    pub ps: u32,
}

/// 播放全部按钮
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EpisodicButton {
    pub text: String,
    pub uri: String,
}

/// 用户投稿视频明细响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContributedVideosResponseData {
    /// 列表信息
    pub list: ContributedVideoList,
    /// 页面信息
    pub page: PageInfo,
    /// “播放全部”按钮
    #[serde(default)]
    pub episodic_button: Option<EpisodicButton>,
    #[serde(default)]
    pub is_risk: bool,
    #[serde(default)]
    pub gaia_res_type: u8,
    #[serde(default)]
    pub gaia_data: Option<serde_json::Value>,
}

// --- 测试模块 ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::Mid;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::user::params::UserUploadedVideosParams;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use tracing::info;

    // 假设这是一个已知用户
    const TEST_MID: u64 = 53456;
    const TEST_KEYWORD: &str = "科技";

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_user_contributed_videos_default() -> Result<(), BpiError> {
        if std::env::var_os("BPI_LIVE_TEST").is_none() {
            return Ok(());
        }

        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .user()
            .uploaded_videos(
                UserUploadedVideosParams::new(Mid::new(TEST_MID)?)
                    .with_page(1)?
                    .with_page_size(2)?,
            )
            .await?;

        info!("用户投稿视频明细: {:?}", data);
        assert_eq!(data.page.pn, 1);
        assert_eq!(data.page.ps, 2);
        assert_eq!(data.list.videos.len(), 2);
        assert!(data.page.count > 0);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_user_contributed_videos_with_keyword() -> Result<(), BpiError> {
        if std::env::var_os("BPI_LIVE_TEST").is_none() {
            return Ok(());
        }

        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .user()
            .uploaded_videos(
                UserUploadedVideosParams::new(Mid::new(TEST_MID)?)
                    .with_keyword(TEST_KEYWORD)
                    .with_page(1)?
                    .with_page_size(10)?,
            )
            .await?;

        info!("用户投稿视频明细（关键词）: {:?}", data);
        assert!(data.page.count > 0);

        Ok(())
    }

    fn uploaded_videos_contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/uploaded-videos/contract.json"
        ))
    }

    #[test]
    fn legacy_user_contributed_videos_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = uploaded_videos_contract()?;

        assert_eq!(contract.name, "user.uploaded_videos");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/space/wbi/arc/search"
        );
        assert!(contract.request.auth.requires_wbi());
        assert_eq!(
            contract.request.query.get("mid").map(String::as_str),
            Some("2")
        );
        assert_eq!(
            contract.request.query.get("order").map(String::as_str),
            Some("pubdate")
        );
        assert_eq!(
            contract.request.query.get("pn").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            contract.request.query.get("ps").map(String::as_str),
            Some("30")
        );
        Ok(())
    }

    #[test]
    fn legacy_user_contributed_videos_fixture_parses_promoted_contract_model() -> BpiResult<()> {
        let videos = ApiEnvelope::<ContributedVideosResponseData>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/uploaded-videos/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(videos.page.pn, 1);
        assert_eq!(videos.page.ps, 30);
        Ok(())
    }
}
