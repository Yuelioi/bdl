// --- 获取稍后再看视频列表 ---

use crate::BilibiliRequest;
use crate::historytoview::HistoryToViewClient;
use crate::historytoview::params::{ToViewAddParams, ToViewDeleteParams};
use crate::response::BpiResult;
use serde::{Deserialize, Serialize};

const TOVIEW_ADD_ENDPOINT: &str = "https://api.bilibili.com/x/v2/history/toview/add";
const TOVIEW_DELETE_ENDPOINT: &str = "https://api.bilibili.com/x/v2/history/toview/del";
const TOVIEW_CLEAR_ENDPOINT: &str = "https://api.bilibili.com/x/v2/history/toview/clear";

/// 稿件属性标志

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToViewRights {
    pub bp: u8,
    pub elec: u8,
    pub download: u8,
    pub movie: u8,
    pub pay: u8,
    pub arc_pay: Option<u8>,
    pub hd5: u8,
    pub no_reprint: u8,
    pub autoplay: u8,
    pub ugc_pay: u8,
    pub is_cooperation: u8,
    pub ugc_pay_preview: u8,
    pub pay_free_watch: Option<u8>,
    pub no_background: u8,
}

/// 稿件 UP 主信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToViewOwner {
    pub mid: u64,
    pub name: String,
    pub face: String,
}

/// 稿件状态数
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToViewStat {
    pub aid: u64,
    pub view: u64,
    pub danmaku: u64,
    pub reply: u64,
    pub favorite: u64,
    pub coin: u64,
    pub share: u64,
    pub now_rank: u64,
    pub his_rank: u32,
    pub like: u64,
    pub like_g: Option<u64>,
    pub dislike: u64,
    pub fav_g: Option<u64>,
    pub vt: i64,
    pub vv: i64,
}

/// 稿件1P分辨率
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToViewDimension {
    pub width: u32,
    pub height: u32,
    pub rotate: u8,
}

/// 稍后再看条目当前分 P 信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToViewPage {
    pub cid: u64,
    pub page: u32,
    pub from: String,
    pub part: String,
    pub duration: u32,
    pub vid: String,
    pub weblink: String,
    pub dimension: ToViewDimension,
    pub ctime: Option<u64>,
}

/// 稍后再看视频列表中的单个视频
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToViewVideoItem {
    pub aid: u64,
    pub videos: u32,
    pub tid: u32,
    pub tidv2: Option<u32>,
    pub tname: String,
    pub tnamev2: Option<String>,
    pub copyright: u8,
    pub pic: String,
    pub cover43: Option<String>,
    pub title: String,
    pub long_title: Option<String>,
    pub pubdate: u64,
    pub ctime: u64,
    pub desc: String,
    pub state: i32,
    pub arc_state: Option<i32>,
    pub attribute: Option<u32>, // 历史保留字段，可能为 null
    pub duration: u32,
    pub rights: ToViewRights,
    pub owner: ToViewOwner,
    pub stat: ToViewStat,
    pub dynamic: Option<String>,
    pub dimension: ToViewDimension,
    pub page: Option<ToViewPage>,
    pub count: Option<u32>,
    pub cid: u64,
    pub progress: i32,
    pub add_at: u64,
    pub bvid: String,
    pub uri: Option<String>,
    pub short_link_v2: Option<String>,
    pub season_title: Option<String>,
    pub pgc_label: Option<String>,
    pub c_source: Option<String>,
    pub card_type: Option<u8>,
    pub enable_vt: Option<u8>,
    pub forbid_fav: Option<bool>,
    pub forbid_sort: Option<bool>,
    pub show_up: Option<bool>,
    pub index_title: Option<String>,
    pub left_icon_type: Option<u8>,
    pub left_text: Option<String>,
    pub right_icon_type: Option<u8>,
    pub right_text: Option<String>,
    pub pid_v2: Option<u32>,
    pub pid_name_v2: Option<String>,
    pub up_from_v2: Option<u8>,
    pub view_text_1: Option<String>,
    pub translate_info: Option<serde_json::Value>,
}

/// 稍后再看视频列表的数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToViewListData {
    /// 稍后再看视频数
    pub count: u32,
    /// 稍后再看视频列表
    pub list: Vec<ToViewVideoItem>,
}

impl<'a> HistoryToViewClient<'a> {
    /// 将视频添加到稍后再看列表，并返回标准 payload 结果。
    pub async fn add_toview(
        &self,
        params: ToViewAddParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        self.client
            .post(TOVIEW_ADD_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("historytoview.toview.add")
            .await
    }

    /// 从稍后再看列表删除视频，并返回标准 payload 结果。
    pub async fn delete_toview(
        &self,
        params: ToViewDeleteParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        self.client
            .post(TOVIEW_DELETE_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("historytoview.toview.delete")
            .await
    }

    /// 清空稍后再看列表并返回标准 payload 结果。
    pub async fn clear_toview(&self) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        let form = [("csrf", csrf)];

        self.client
            .post(TOVIEW_CLEAR_ENDPOINT)
            .form(&form)
            .send_bpi_optional_payload("historytoview.toview.clear")
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/historytoview/read/toview-list/contract.json"
        ))
    }

    #[test]
    fn toview_list_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "historytoview.toview_list");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/v2/history/toview"
        );
        assert!(contract.request.query.is_empty());
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(contract.cases[0].response.api_code, Some(-101));
        assert_eq!(
            contract.cases[1].response.rust_model.as_deref(),
            Some("ToViewListData")
        );
        Ok(())
    }

    #[test]
    fn toview_list_response_fixtures_parse_declared_models() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/historytoview/read/toview-list/responses/anonymous.requires_login.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(err.requires_login());

        let payload = ApiEnvelope::<ToViewListData>::from_slice(include_bytes!(
            "../../tests/contracts/historytoview/read/toview-list/responses/authenticated.success.json"
        ))?
        .into_payload()?;
        assert_eq!(payload.count, 1);
        assert_eq!(payload.list.len(), 1);
        assert_eq!(payload.list[0].stat.vt, -1);
        Ok(())
    }

    fn local_probe_body(profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/historytoview/read/toview-list/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn toview_list_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body(profile) {
                let envelope = serde_json::from_value::<ApiEnvelope<ToViewListData>>(body)?;
                if profile == "anonymous" {
                    let err = envelope.ensure_success().unwrap_err();
                    assert!(err.requires_login());
                } else {
                    let payload = envelope.into_payload()?;
                    assert!(payload.count as usize >= payload.list.len());
                }
            }
        }
        Ok(())
    }
}
