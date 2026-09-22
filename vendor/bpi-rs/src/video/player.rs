//! B站 web 播放器相关接口
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/video)
use serde::{Deserialize, Serialize};

pub(crate) const PLAYER_INFO_V2_ENDPOINT: &str = "https://api.bilibili.com/x/player/wbi/v2";

// --- 响应数据结构体 ---

/// web 播放器信息响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayerInfoResponseData {
    /// 视频 aid
    pub aid: u64,
    /// 视频 bvid
    pub bvid: String,
    pub allow_bp: bool,
    pub no_share: bool,
    /// 视频 cid
    pub cid: u64,
    /// webmask 防挡字幕信息
    pub dm_mask: Option<DmMaskInfo>,
    /// 字幕信息
    pub subtitle: Option<SubtitleInfo>,
    /// 分段章节信息
    #[serde(default)]
    pub view_points: Vec<ViewPoint>,
    /// 请求 IP 信息
    pub ip_info: Option<serde_json::Value>,
    /// 登录用户 mid
    pub login_mid: u64,
    pub login_mid_hash: Option<String>,
    /// 是否为该视频 UP 主
    pub is_owner: bool,
    pub name: String,
    pub permission: String,
    /// 登录用户等级信息
    pub level_info: Option<serde_json::Value>,
    /// 登录用户 VIP 信息
    pub vip: Option<serde_json::Value>,
    /// 答题状态
    pub answer_status: u8,
    pub block_time: u64,
    pub role: String,
    /// 上次观看时间
    pub last_play_time: i64,
    /// 上次观看 cid
    pub last_play_cid: i64,
    /// 当前 UNIX 秒级时间戳
    pub now_time: u64,
    /// 在线人数
    pub online_count: Option<u64>,
    /// 是否必须登陆才能查看字幕
    pub need_login_subtitle: bool,
    /// 预告提示
    pub preview_toast: String,
    /// 互动视频资讯
    pub interaction: Option<InteractionInfo>,
    pub options: Option<PlayerOptions>,
    pub guide_attention: Option<serde_json::Value>,
    pub jump_card: Option<serde_json::Value>,
    pub operation_card: Option<serde_json::Value>,
    pub online_switch: Option<serde_json::Value>,
    pub fawkes: Option<serde_json::Value>,
    pub show_switch: Option<serde_json::Value>,
    /// 背景音乐信息
    pub bgm_info: Option<BgmInfo>,
    pub toast_block: bool,
    /// 是否为充电专属视频
    pub is_upower_exclusive: bool,
    pub is_upower_play: bool,
    pub is_ugc_pay_preview: bool,
    /// 充电专属视频信息
    pub elec_high_level: Option<ElecHighLevel>,
    pub disable_show_up_info: bool,
}

/// webmask 防挡字幕信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DmMaskInfo {
    /// 视频 cid
    pub cid: u64,
    pub plat: u8,
    /// webmask 取样 fps
    pub fps: u64,
    pub time: u64,
    /// webmask 资源 url
    pub mask_url: String,
}

/// 字幕信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubtitleInfo {
    pub allow_submit: bool,
    pub lan: String,
    pub lan_doc: String,
    #[serde(default)]
    pub subtitles: Vec<SubtitleItem>,
}

/// 单个字幕信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SubtitleItem {
    pub ai_status: u8,
    pub ai_type: u8,
    pub id: u64,
    pub id_str: String,
    pub is_lock: bool,
    /// 语言类型英文字母缩写
    pub lan: String,
    /// 语言类型中文名称
    pub lan_doc: String,
    /// 资源 url 地址
    pub subtitle_url: String,
    #[serde(rename = "type")]
    pub subtitle_type: u8,
}

/// 分段章节信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ViewPoint {
    /// 分段章节名
    pub content: String,
    /// 分段章节起始秒数
    pub from: u64,
    /// 分段章节结束秒数
    pub to: u64,
    #[serde(rename = "type")]
    pub point_type: u8,
    /// 图片资源地址
    #[serde(rename = "imgUrl")]
    pub img_url: String,
    #[serde(rename = "logoUrl")]
    pub logo_url: String,
    pub team_type: String,
    pub team_name: String,
}

/// 互动视频资讯
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InteractionInfo {
    /// 剧情图 id
    pub graph_version: u64,
    /// 未登录有机会返回
    pub msg: Option<String>,
    /// 错误信息
    pub error_toast: Option<String>,
    pub mark: Option<u8>,
    pub need_reload: Option<u8>,
}

/// 播放器选项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PlayerOptions {
    /// 是否 360 全景视频
    pub is_360: bool,
    pub without_vip: bool,
}

/// 背景音乐信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BgmInfo {
    /// 音乐 id
    pub music_id: String,
    /// 音乐标题
    pub music_title: String,
    /// 跳转 URL
    pub jump_url: String,
}

/// 充电专属视频信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ElecHighLevel {
    /// 解锁视频所需最低定价档位的代码
    pub privilege_type: u64,
    /// 提示标题
    pub title: String,
    /// 提示子标题
    pub sub_title: String,
    /// 是否显示按钮
    pub show_button: bool,
    /// 按钮文本
    pub button_text: String,
    /// 跳转url信息
    pub jump_url: Option<serde_json::Value>,
    /// 充电介绍语
    pub intro: String,
    #[serde(default)]
    pub open: bool,
    #[serde(default)]
    pub new: bool,
    #[serde(default)]
    pub question_text: String,
    #[serde(default)]
    pub qa_detail_link: String,
}

// --- 测试模块 ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{Aid, Cid};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::video::params::VideoPlayerInfoParams;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use tracing::info;

    const TEST_AID: u64 = 1906473802;
    const TEST_CID: u64 = 636329244;

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_player_info_v2_by_aid() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let params = VideoPlayerInfoParams::from_aid(Aid::new(TEST_AID)?, Cid::new(TEST_CID)?);
        let data = bpi.video().player_info_v2(params).await?;

        info!("播放器信息: {:?}", data);

        assert_eq!(data.aid, TEST_AID);
        assert_eq!(data.cid, TEST_CID);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_video_player_info_v2_by_bvid() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let params = VideoPlayerInfoParams::from_aid(Aid::new(TEST_AID)?, Cid::new(TEST_CID)?);
        let data = bpi.video().player_info_v2(params).await?;

        info!("播放器信息: {:?}", data);

        assert_eq!(data.aid, TEST_AID);
        assert_eq!(data.cid, TEST_CID);

        Ok(())
    }

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/video/player-read/player-info-v2/contract.json"
        ))
    }

    #[test]
    fn video_player_info_v2_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;
        let params = VideoPlayerInfoParams::from_bvid("BV1xx411c7mD".parse()?, Cid::new(62131)?);

        assert_eq!(contract.name, "video.player_info_v2");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(contract.request.url.as_str(), PLAYER_INFO_V2_ENDPOINT);
        assert!(contract.request.auth.requires_wbi());
        assert_eq!(
            contract.request.query.get("bvid").map(String::as_str),
            Some("BV1xx411c7mD")
        );
        assert_eq!(
            contract.request.query.get("cid").map(String::as_str),
            Some("62131")
        );
        assert_eq!(
            params.query_pairs(),
            vec![
                ("cid", "62131".to_string()),
                ("bvid", "BV1xx411c7mD".to_string())
            ]
        );
        assert_eq!(contract.cases.len(), 3);
        Ok(())
    }

    #[test]
    fn video_player_info_v2_response_fixtures_parse_declared_model() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/video/player-read/player-info-v2/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/video/player-read/player-info-v2/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/video/player-read/player-info-v2/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload =
                ApiEnvelope::<PlayerInfoResponseData>::from_slice(bytes)?.into_payload()?;

            assert_eq!(payload.bvid, "BV1xx411c7mD");
            assert_eq!(payload.cid, 62131);
        }
        Ok(())
    }

    #[test]
    fn video_player_info_v2_accepts_negative_resume_sentinels() -> BpiResult<()> {
        let mut response: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../tests/contracts/video/player-read/player-info-v2/responses/normal.success.json"
        ))?;
        response["data"]["last_play_time"] = serde_json::json!(-1000);
        response["data"]["last_play_cid"] = serde_json::json!(-1000);

        let payload = serde_json::from_value::<ApiEnvelope<PlayerInfoResponseData>>(response)?
            .into_payload()?;

        assert_eq!(
            (payload.last_play_time, payload.last_play_cid),
            (-1000, -1000)
        );
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path = format!(
            "target/bpi-probe-runs/video/player-read/player-info-v2/{profile}.response.json"
        );
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn video_player_info_v2_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body(profile) else {
                continue;
            };
            let payload = serde_json::from_value::<ApiEnvelope<PlayerInfoResponseData>>(body)?
                .into_payload()?;

            assert_eq!(payload.bvid, "BV1xx411c7mD");
        }
        Ok(())
    }
}
