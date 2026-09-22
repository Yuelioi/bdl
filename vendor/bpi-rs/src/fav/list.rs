#[cfg(test)]
use crate::fav::params::FavResourceIdsParams;
use crate::ids::MediaId;
use crate::{BpiError, BpiResult};
use serde::{Deserialize, Deserializer, Serialize};

// --- 获取收藏夹内容明细列表 ---

/// 收藏夹内容明细列表中的 UP 主信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FavListUpper {
    pub mid: u64,
    pub name: String,
    pub face: String,
    pub followed: Option<bool>,
    pub vip_type: Option<u8>,
    #[serde(alias = "vip_statue")]
    pub vip_status: Option<u8>,
}

/// 收藏夹内容明细列表中的状态数
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FavListCntInfo {
    /// 收藏
    pub collect: u64,
    /// 播放
    pub play: u64,

    /// 分享 (仅info)
    pub share: Option<u64>,
    /// 点赞 (仅info)
    pub thumb_up: Option<u64>,
    ///弹幕 (仅media)
    pub danmaku: Option<u64>,
    /// 播放文本 (仅media)
    pub view_text_1: Option<String>,
}

/// 收藏夹元数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FavListInfo {
    pub id: u64,
    pub fid: u64,
    pub mid: u64,
    pub attr: u32,
    pub title: String,
    pub cover: String,
    pub upper: FavListUpper,
    pub cover_type: u8,
    pub cnt_info: FavListCntInfo,
    #[serde(rename = "type")]
    pub type_name: u32,
    pub intro: String,
    pub ctime: u64,
    pub mtime: u64,
    pub state: u8,
    pub fav_state: u8,
    pub like_state: u8,
    pub media_count: u32,
}

/// 收藏夹中的单个内容
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FavListMedia {
    pub id: u64,
    #[serde(rename = "type")]
    pub type_name: u8,
    pub title: String,
    pub cover: String,
    pub intro: String,
    pub page: Option<u32>,
    pub duration: u32,
    pub upper: FavListUpper,
    pub attr: u8,
    pub cnt_info: FavListCntInfo,
    pub link: String,
    pub ctime: u64,
    pub pubtime: u64,
    pub fav_time: u64,
    pub bv_id: Option<String>,
    pub bvid: Option<String>,
    pub season: Option<serde_json::Value>,
}

/// 收藏夹内容明细列表数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FavListDetailData {
    pub info: FavListInfo,
    #[serde(default, deserialize_with = "empty_vec_on_null")]
    pub medias: Vec<FavListMedia>,
    pub has_more: bool,
    pub ttl: u64,
}

// --- 获取收藏夹全部内容id ---

/// 收藏夹全部内容ID列表中的单个ID
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FavResourceIdItem {
    pub id: u64,
    #[serde(rename = "type")]
    pub type_name: u8,
    pub bv_id: Option<String>,
    pub bvid: Option<String>,
}

/// 获取收藏夹详细资源列表的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavListDetailParams {
    media_id: MediaId,
    tid: Option<u32>,
    keyword: Option<String>,
    order: Option<String>,
    typ: Option<u8>,
    ps: u32,
    pn: Option<u32>,
}

impl FavListDetailParams {
    /// 使用默认每页数量创建收藏列表详情参数。
    pub fn new(media_id: MediaId) -> Self {
        Self {
            media_id,
            tid: None,
            keyword: None,
            order: None,
            typ: None,
            ps: 20,
            pn: None,
        }
    }

    /// 设置可选分区过滤器。
    pub fn tid(mut self, tid: u32) -> Self {
        self.tid = Some(tid);
        self
    }

    /// 设置关键词过滤器。
    pub fn keyword(mut self, keyword: impl Into<String>) -> BpiResult<Self> {
        let keyword = keyword.into();
        validate_non_blank("keyword", &keyword)?;
        self.keyword = Some(keyword);
        Ok(self)
    }

    /// 设置排序 key，例如 `mtime`。
    pub fn order(mut self, order: impl Into<String>) -> BpiResult<Self> {
        let order = order.into();
        validate_non_blank("order", &order)?;
        self.order = Some(order);
        Ok(self)
    }

    /// 设置内容类型过滤器。
    pub fn content_type(mut self, typ: u8) -> Self {
        self.typ = Some(typ);
        self
    }

    /// 设置每页数量。
    pub fn page_size(mut self, ps: u32) -> BpiResult<Self> {
        if ps == 0 {
            return Err(BpiError::invalid_parameter(
                "ps",
                "page size must be non-zero",
            ));
        }
        self.ps = ps;
        Ok(self)
    }

    /// 设置页码。
    pub fn page(mut self, pn: u32) -> BpiResult<Self> {
        if pn == 0 {
            return Err(BpiError::invalid_parameter(
                "pn",
                "page number must be non-zero",
            ));
        }
        self.pn = Some(pn);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![
            ("media_id", self.media_id.to_string()),
            ("ps", self.ps.to_string()),
            ("platform", "web".to_string()),
        ];

        if let Some(tid) = self.tid {
            params.push(("tid", tid.to_string()));
        }
        if let Some(keyword) = self.keyword.as_ref() {
            params.push(("keyword", keyword.clone()));
        }
        if let Some(order) = self.order.as_ref() {
            params.push(("order", order.clone()));
        }
        if let Some(typ) = self.typ {
            params.push(("type", typ.to_string()));
        }
        if let Some(pn) = self.pn {
            params.push(("pn", pn.to_string()));
        }

        params
    }
}

fn validate_non_blank(field: &'static str, value: &str) -> BpiResult<()> {
    if value.trim().is_empty() {
        return Err(BpiError::invalid_parameter(field, "value cannot be blank"));
    }

    Ok(())
}

fn empty_vec_on_null<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::MediaId;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiResult};
    use tracing::info;

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "list-detail" => {
                include_bytes!("../../tests/contracts/fav/read/list-detail/contract.json")
                    .as_slice()
            }
            "resource-ids" => {
                include_bytes!("../../tests/contracts/fav/read/resource-ids/contract.json")
                    .as_slice()
            }
            _ => unreachable!("unknown fav list contract endpoint"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_fav_list_detail() {
        let bpi = BpiClient::new().expect("client should build");
        let media_id = 1052622027;
        let params = FavListDetailParams::new(MediaId::new(media_id).expect("media id is valid"))
            .order("mtime")
            .expect("order is valid")
            .content_type(0)
            .page_size(5)
            .expect("page size is valid")
            .page(1)
            .expect("page is valid");
        let resp = bpi.fav().list_detail(params).await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("has_more: {}", data.has_more);
        info!("total media count: {}", data.info.media_count);
        info!("retrieved media count: {}", data.medias.len());
        info!("first media item: {:?}", data.medias.first());
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_fav_resource_ids() {
        let bpi = BpiClient::new().expect("client should build");
        let params = FavResourceIdsParams::new(
            MediaId::new(1052622027).expect("fixture media id should be valid"),
        );
        let resp = bpi.fav().resource_ids(params).await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("total IDs retrieved: {}", data.len());
        info!("first ID item: {:?}", data.first());
    }

    #[test]
    fn fav_list_detail_params_serializes_required_query() -> Result<(), BpiError> {
        let params = FavListDetailParams::new(MediaId::new(1052622027)?);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("media_id", "1052622027".to_string()),
                ("ps", "20".to_string()),
                ("platform", "web".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn fav_list_detail_params_serializes_optional_query() -> Result<(), BpiError> {
        let params = FavListDetailParams::new(MediaId::new(1052622027)?)
            .tid(3)
            .keyword("rust")?
            .order("mtime")?
            .content_type(0)
            .page_size(5)?
            .page(1)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("media_id", "1052622027".to_string()),
                ("ps", "5".to_string()),
                ("platform", "web".to_string()),
                ("tid", "3".to_string()),
                ("keyword", "rust".to_string()),
                ("order", "mtime".to_string()),
                ("type", "0".to_string()),
                ("pn", "1".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn fav_list_detail_params_rejects_zero_page_size() {
        let err = FavListDetailParams::new(MediaId::new(1052622027).expect("media id is valid"))
            .page_size(0)
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "ps", .. }
        ));
    }

    #[test]
    fn fav_list_detail_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("list-detail")?;

        assert_eq!(contract.name, "fav.list_detail");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/v3/fav/resource/list"
        );
        assert_eq!(
            contract.request.query.get("media_id").map(String::as_str),
            Some("1052622027")
        );
        assert_eq!(
            contract.request.query.get("platform").map(String::as_str),
            Some("web")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("FavListDetailData")
        );
        Ok(())
    }

    #[test]
    fn fav_resource_ids_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("resource-ids")?;

        assert_eq!(contract.name, "fav.resource_ids");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/v3/fav/resource/ids"
        );
        assert_eq!(
            contract.request.query.get("media_id").map(String::as_str),
            Some("1052622027")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("Vec<FavResourceIdItem>")
        );
        Ok(())
    }

    #[test]
    fn fav_list_response_fixtures_parse_declared_models() -> BpiResult<()> {
        let detail = ApiEnvelope::<FavListDetailData>::from_slice(include_bytes!(
            "../../tests/contracts/fav/read/list-detail/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(detail.medias.len(), 1);

        let ids = ApiEnvelope::<Vec<FavResourceIdItem>>::from_slice(include_bytes!(
            "../../tests/contracts/fav/read/resource-ids/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(ids.len(), 1);
        Ok(())
    }

    #[test]
    fn fav_list_detail_accepts_null_medias_as_empty_list() -> BpiResult<()> {
        let detail = serde_json::from_value::<FavListDetailData>(serde_json::json!({
            "info": {
                "id": 1052622027,
                "fid": 1052622027,
                "mid": 7792521,
                "attr": 0,
                "title": "empty folder",
                "cover": "",
                "upper": {
                    "mid": 7792521,
                    "name": "owner",
                    "face": "",
                    "followed": false,
                    "vip_type": 0,
                    "vip_status": 0
                },
                "cover_type": 0,
                "cnt_info": {
                    "collect": 0,
                    "play": 0,
                    "share": 0,
                    "thumb_up": 0
                },
                "type": 2,
                "intro": "",
                "ctime": 0,
                "mtime": 0,
                "state": 0,
                "fav_state": 0,
                "like_state": 0,
                "media_count": 0
            },
            "medias": null,
            "has_more": false,
            "ttl": 1
        }))?;

        assert!(detail.medias.is_empty());
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/fav/read/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn fav_list_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body("list-detail", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<FavListDetailData>>(body)?
                    .into_payload()?;
                assert!(payload.info.media_count >= payload.medias.len() as u32);
            }

            if let Some(body) = local_probe_body("resource-ids", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<Vec<FavResourceIdItem>>>(body)?
                    .into_payload()?;
                assert!(!payload.is_empty());
            }
        }
        Ok(())
    }
}
