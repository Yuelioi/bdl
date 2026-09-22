// --- 直播间管理 API 结构体 ---

use crate::BilibiliRequest;
use crate::BpiResult;
use crate::live::LiveClient;
use reqwest::multipart::Form;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const FETCH_WEB_UP_STREAM_ADDR_ENDPOINT: &str =
    "https://api.live.bilibili.com/xlive/app-blink/v1/live/FetchWebUpStreamAddr";
const WEB_LIVE_CENTER_START_ENDPOINT: &str =
    "https://api.live.bilibili.com/xlive/app-blink/v1/streaming/WebLiveCenterStartLive";

/// 开通直播间响应数据

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateRoomData {
    #[serde(rename = "roomID")]
    pub room_id: Option<String>,
}

/// 直播间信息更新响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateRoomData {
    pub sub_session_key: String,
    pub audit_info: Option<AuditInfo>,
}

/// 审核信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuditInfo {
    pub audit_title_reason: String,
    pub audit_title_status: u8,
    pub audit_title: Option<String>,
    pub update_title: Option<String>,
}

/// RTMP 推流地址信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RtmpInfo {
    pub addr: String,
    pub code: String,
}

/// 网页 link 中心推流地址（FetchWebUpStreamAddr）
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebUpStreamAddrData {
    pub addr: RtmpInfo,
    pub line: Value,
    #[serde(default)]
    pub srt_addr: Value,
}

/// 开始直播响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StartLiveData {
    pub change: u8,
    pub status: String,
    #[serde(default)]
    pub rtmp: Option<RtmpInfo>,
    pub live_key: String,
    pub sub_session_key: String,
    pub need_face_auth: bool,
    // 其他不明确的字段都使用 Value
    pub room_type: Value,
    pub protocols: Value,
    pub notice: Value,
    pub qr: Value,
    pub service_source: String,
    pub rtmp_backup: Value,
    pub up_stream_extra: Value,
}

/// 关闭直播响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StopLiveData {
    pub change: u8,
    pub status: String,
}

/// 预更新直播间信息响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdatePreLiveInfoData {
    pub audit_info: Option<AuditInfo>,
}

/// PC直播姬版本号响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PcLiveVersionData {
    pub curr_version: String,
    pub build: u64,
    pub instruction: String,
    pub file_size: String,
    pub file_md5: String,
    pub content: String,
    pub download_url: String,
    pub hdiffpatch_switch: u8,
}

impl<'a> LiveClient<'a> {
    /// 开通直播间
    pub async fn live_create_room(&self) -> BpiResult<CreateRoomData> {
        let csrf = self.client.csrf()?;
        let form = Form::new()
            .text("platform", "web")
            .text("visit_id", "")
            .text("csrf", csrf.clone())
            .text("csrf_token", csrf);

        self.client
            .post("https://api.live.bilibili.com/xlive/app-blink/v1/preLive/CreateRoom")
            .multipart(form)
            .send_bpi_payload("live.room.create")
            .await
    }

    /// 更新直播间信息
    ///
    /// # 参数
    /// * `room_id` - 直播间 ID
    /// * `title` - 标题，可选
    /// * `area_id` - 分区 ID，可选
    /// * `add_tag` - 要添加的标签，可选
    /// * `del_tag` - 要删除的标签，可选
    pub async fn live_update_room_info(
        &self,
        room_id: u64,
        title: Option<&str>,
        area_id: Option<u64>,
        add_tag: Option<&str>,
        del_tag: Option<&str>,
    ) -> BpiResult<UpdateRoomData> {
        let csrf = self.client.csrf()?;
        let mut form = Form::new()
            .text("room_id", room_id.to_string())
            .text("csrf", csrf.clone())
            .text("csrf_token", csrf);

        if let Some(t) = title {
            form = form.text("title", t.to_string());
        }
        if let Some(a) = area_id {
            form = form.text("area_id", a.to_string());
        }
        if let Some(a_tag) = add_tag {
            form = form.text("add_tag", a_tag.to_string());
        }
        if let Some(d_tag) = del_tag {
            form = form.text("del_tag", d_tag.to_string());
        }

        self.client
            .post("https://api.live.bilibili.com/room/v1/Room/update")
            .multipart(form)
            .send_bpi_payload("live.room.update")
            .await
    }

    /// 获取网页 link 中心推流地址
    pub async fn live_fetch_web_up_stream_addr(&self) -> BpiResult<WebUpStreamAddrData> {
        let csrf = self.client.csrf()?;
        let form = web_up_stream_addr_form(&csrf);

        self.client
            .post(FETCH_WEB_UP_STREAM_ADDR_ENDPOINT)
            .form(&form)
            .send_bpi_payload("live.web_up_stream_addr")
            .await
    }

    /// 网页 link 中心开始直播（第三方软件开播，WBI 签名）
    pub async fn live_web_center_start(
        &self,
        room_id: u64,
        area_v2: u64,
    ) -> BpiResult<StartLiveData> {
        let csrf = self.client.csrf()?;
        let form = web_live_center_start_query(room_id, area_v2, &csrf);
        let signed = self.client.get_wbi_sign2(form).await?;

        // 浏览器 link 中心：WBI 参数在 query string，POST body 为空
        self.client
            .post(WEB_LIVE_CENTER_START_ENDPOINT)
            .query(&signed)
            .send_bpi_payload("live.web_center_start")
            .await
    }

    /// 开始直播（直播姬旧接口）
    #[allow(dead_code)]
    async fn live_start_legacy(
        &self,
        room_id: u64,
        area_v2: u64,
        platform: &str,
    ) -> BpiResult<StartLiveData> {
        let csrf = self.client.csrf()?;
        let form = Form::new()
            .text("room_id", room_id.to_string())
            .text("area_v2", area_v2.to_string())
            .text("platform", platform.to_string())
            .text("csrf", csrf.clone())
            .text("csrf_token", csrf);

        self.client
            .post("https://api.live.bilibili.com/room/v1/Room/startLive")
            .multipart(form)
            .send_bpi_payload("live.start")
            .await
    }

    /// 关闭直播
    ///
    /// # 参数
    /// * `room_id` - 直播间 ID
    /// * `platform` - 直播平台，如 "pc_link"
    pub async fn live_stop(&self, room_id: u64, platform: &str) -> BpiResult<StopLiveData> {
        let csrf = self.client.csrf()?;
        let form = Form::new()
            .text("platform", platform.to_string())
            .text("room_id", room_id.to_string())
            .text("csrf", csrf.clone())
            .text("csrf_token", csrf);

        self.client
            .post("https://api.live.bilibili.com/room/v1/Room/stopLive")
            .multipart(form)
            .send_bpi_payload("live.stop")
            .await
    }

    /// 预更新直播间信息
    ///
    /// # 参数
    /// * `title` - 标题，可选
    /// * `cover` - 封面 URL，可选
    pub async fn live_update_pre_live_info(
        &self,
        title: Option<&str>,
        cover: Option<&str>,
    ) -> BpiResult<UpdatePreLiveInfoData> {
        let csrf = self.client.csrf()?;
        let mut form = Form::new()
            .text("platform", "web")
            .text("mobi_app", "web")
            .text("build", "1")
            .text("csrf", csrf.clone())
            .text("csrf_token", csrf);

        if let Some(t) = title {
            form = form.text("title", t.to_string());
        }
        if let Some(c) = cover {
            form = form.text("cover", c.to_string());
        }

        self.client
            .post("https://api.live.bilibili.com/xlive/app-blink/v1/preLive/UpdatePreLiveInfo")
            .multipart(form)
            .send_bpi_payload("live.pre_live_info.update")
            .await
    }

    /// 更新直播间公告
    ///
    /// # 参数
    /// * `room_id` - 直播间 ID
    /// * `uid` - 用户ID
    /// * `content` - 公告内容
    pub async fn live_update_room_news(
        &self,
        room_id: u64,
        uid: u64,
        content: &str,
    ) -> BpiResult<Value> {
        let csrf = self.client.csrf()?;
        let form = Form::new()
            .text("room_id", room_id.to_string())
            .text("uid", uid.to_string())
            .text("content", content.to_string())
            .text("csrf", csrf.clone())
            .text("csrf_token", csrf);

        self.client
            .post("https://api.live.bilibili.com/xlive/app-blink/v1/index/updateRoomNews")
            .multipart(form)
            .send_bpi_payload("live.room_news.update")
            .await
    }
}

fn web_up_stream_addr_form(csrf: &str) -> Vec<(&'static str, String)> {
    vec![
        ("platform", "pc".to_string()),
        ("backup_stream", "0".to_string()),
        ("csrf", csrf.to_string()),
        ("csrf_token", csrf.to_string()),
    ]
}

fn web_live_center_start_query(
    room_id: u64,
    area_v2: u64,
    csrf: &str,
) -> Vec<(&'static str, String)> {
    vec![
        ("room_id", room_id.to_string()),
        ("platform", "pc".to_string()),
        ("area_v2", area_v2.to_string()),
        ("backup_stream", "0".to_string()),
        ("csrf", csrf.to_string()),
        ("csrf_token", csrf.to_string()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiResult};

    fn version_contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/live/public-core/version/contract.json"
        ))
    }

    #[test]
    fn live_version_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = version_contract()?;

        assert_eq!(contract.name, "live.version");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.live.bilibili.com/xlive/app-blink/v1/liveVersionInfo/getHomePageLiveVersion"
        );
        assert_eq!(
            contract
                .request
                .query
                .get("system_version")
                .map(String::as_str),
            Some("2")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("PcLiveVersionData")
        );
        Ok(())
    }

    #[test]
    fn live_version_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<PcLiveVersionData>::from_slice(include_bytes!(
            "../../tests/contracts/live/public-core/version/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.curr_version, "7.61.0.10694");
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/live/public-core/version/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn live_version_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body(profile) {
                let payload = serde_json::from_value::<ApiEnvelope<PcLiveVersionData>>(body)?
                    .into_payload()?;
                assert!(!payload.curr_version.is_empty());
            }
        }
        Ok(())
    }

    #[test]
    fn web_up_stream_addr_form_contains_web_defaults_and_csrf_aliases() {
        assert_eq!(
            web_up_stream_addr_form("csrf-token"),
            vec![
                ("platform", "pc".to_string()),
                ("backup_stream", "0".to_string()),
                ("csrf", "csrf-token".to_string()),
                ("csrf_token", "csrf-token".to_string()),
            ]
        );
    }

    #[test]
    fn web_center_start_query_contains_wbi_signed_inputs() {
        assert_eq!(
            web_live_center_start_query(4_354_019, 309, "csrf-token"),
            vec![
                ("room_id", "4354019".to_string()),
                ("platform", "pc".to_string()),
                ("area_v2", "309".to_string()),
                ("backup_stream", "0".to_string()),
                ("csrf", "csrf-token".to_string()),
                ("csrf_token", "csrf-token".to_string()),
            ]
        );
    }

    #[test]
    fn web_up_stream_addr_response_accepts_srt_addr_missing() -> BpiResult<()> {
        let payload = ApiEnvelope::<WebUpStreamAddrData>::from_slice(
            br#"{
                "code": 0,
                "message": "0",
                "data": {
                    "addr": { "addr": "rtmp://example/live/", "code": "stream-key" },
                    "line": []
                }
            }"#,
        )?
        .into_payload()?;

        assert_eq!(payload.addr.addr, "rtmp://example/live/");
        assert_eq!(payload.addr.code, "stream-key");
        assert!(payload.srt_addr.is_null());
        Ok(())
    }

    #[test]
    fn start_live_response_accepts_null_rtmp() -> BpiResult<()> {
        let payload = ApiEnvelope::<StartLiveData>::from_slice(
            br#"{
                "code": 0,
                "message": "0",
                "data": {
                    "change": 1,
                    "status": "LIVE",
                    "rtmp": null,
                    "live_key": "live-key",
                    "sub_session_key": "sub-session",
                    "need_face_auth": false,
                    "room_type": 0,
                    "protocols": [],
                    "notice": {},
                    "qr": "",
                    "service_source": "",
                    "rtmp_backup": [],
                    "up_stream_extra": {}
                }
            }"#,
        )?
        .into_payload()?;

        assert!(payload.rtmp.is_none());
        assert_eq!(payload.live_key, "live-key");
        Ok(())
    }
}
