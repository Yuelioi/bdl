use crate::models::{Official, Pendant, Vip};
use serde::{Deserialize, Deserializer, Serialize, de};
// --- 动态详情 API 结构体 ---

/// 动态详情响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicDetailData {
    pub item: DynamicDetailItem,
}

/// 动态卡片内容，作为多个 API 的共享结构体
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicDetailItem {
    #[serde(deserialize_with = "deserialize_dynamic_id_text")]
    pub id_str: String,
    pub basic: DynamicBasic,

    pub modules: serde_json::Value,

    /// 当`type`字段的值为DYNAMIC_TYPE_FORWARD（转发动态）时，此字段不为null
    pub orig: Option<Box<DynamicDetailItem>>,

    pub r#type: String,

    pub visible: bool,
}

fn deserialize_dynamic_id_text<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::String(value) => Ok(value),
        serde_json::Value::Number(value) => Ok(value.to_string()),
        serde_json::Value::Null => Ok(String::new()),
        _ => Err(de::Error::custom(
            "dynamic id must be a string, number, or null",
        )),
    }
}

/// 动态卡片内容，作为多个 API 的共享结构体
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicForwardItem {
    pub desc: Desc,
    pub id_str: String,
    pub pub_time: String,
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Desc {
    pub rich_text_nodes: Vec<RichTextNode>,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichTextNode {
    pub orig_text: String,
    pub text: String,
    #[serde(rename = "type")]
    pub type_field: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub face: String,
    pub face_nft: bool,
    pub mid: i64,
    pub name: String,
    pub official: Official,
    pub pendant: Pendant,
    pub vip: Vip,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicBasic {
    pub comment_id_str: String,
    pub comment_type: i64,
    pub editable: Option<bool>,
    pub jump_url: Option<String>,
    pub like_icon: serde_json::Value,
    pub rid_str: String,
}

// --- 动态点赞与转发列表 API 结构体 ---

/// 点赞或转发的用户列表项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicReactionItem {
    pub action: String,
    /// 1: 对方仅关注了发送者    2: 发送者关注了对方
    pub attend: u8,
    pub desc: String,
    pub face: String,
    pub mid: String,
    pub name: String,
}

/// 动态点赞与转发列表响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicReactionData {
    pub has_more: bool,
    pub items: Vec<DynamicReactionItem>,
    pub offset: String,
    pub total: u64,
}

// --- 动态抽奖详情 API 结构体 ---

/// 抽奖中奖用户。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LotteryWinner {
    pub uid: u64,
    pub name: String,
    pub face: String,
    pub hongbao_money: Option<f64>,
}

/// 动态抽奖结果。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicLotteryResult {
    #[serde(default)]
    pub first_prize_result: Vec<LotteryWinner>,
    #[serde(default)]
    pub second_prize_result: Vec<LotteryWinner>,
    #[serde(default)]
    pub third_prize_result: Vec<LotteryWinner>,
}

/// 动态抽奖奖品类型。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicLotteryPrizeType {
    #[serde(rename = "type")]
    pub type_field: u8,
    pub value: DynamicLotteryPrizeTypeValue,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicLotteryPrizeTypeValue {
    pub count: u64,
    pub stype: u8,
}

/// 动态抽奖详情响应数据。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicLotteryData {
    pub lottery_id: u64,
    pub sender_uid: u64,
    pub business_type: u8,
    pub business_id: u64,
    pub status: u8,
    pub lottery_time: u64,
    pub participants: u64,
    pub first_prize: u32,
    pub first_prize_cmt: String,
    pub first_prize_pic: String,
    pub second_prize: u32,
    #[serde(default)]
    pub second_prize_cmt: Option<String>,
    pub second_prize_pic: String,
    pub third_prize: u32,
    #[serde(default)]
    pub third_prize_cmt: Option<String>,
    pub third_prize_pic: String,
    pub lottery_result: Option<DynamicLotteryResult>,
    pub followed: bool,
    pub has_charge_right: bool,
    pub lottery_at_num: u32,
    pub lottery_detail_url: String,
    pub lottery_feed_limit: u32,
    pub need_post: u8,
    pub participated: bool,
    #[serde(default)]
    pub prize_type_first: Option<DynamicLotteryPrizeType>,
    pub reposted: bool,
    pub ts: u64,
    pub upower_redirect_url: String,
    pub vip_batch_sign: String,
    pub vip_redirect_url: String,
}

// --- 动态转发列表 API 结构体 ---

/// 动态转发列表响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicForwardData {
    pub has_more: bool,
    pub items: Vec<DynamicForwardItem>,
    pub offset: String,
    pub total: u64,
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicForwardInfoData {
    pub item: DynamicForwardItem,
}

// --- 获取动态图片 API 结构体 ---

/// 动态图片信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicPic {
    pub height: u64,
    pub size: f64,
    pub src: String,
    pub width: u64,
}

/// 动态图片列表响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DynamicPicsData {
    pub data: Vec<DynamicPic>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dynamic::{
        DynamicDetailParams, DynamicForwardItemParams, DynamicForwardsParams,
        DynamicLotteryNoticeParams, DynamicPicsParams, DynamicReactionsParams,
    };
    use crate::ids::DynamicId;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use std::collections::BTreeMap;
    use tracing::info;

    fn parse_dynamic_id(value: &str) -> Result<DynamicId, BpiError> {
        value.parse()
    }

    fn dynamic_item_json(id: serde_json::Value, orig: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "id_str": id,
            "basic": {
                "comment_id_str": "1",
                "comment_type": 1,
                "editable": null,
                "jump_url": null,
                "like_icon": {},
                "rid_str": "1"
            },
            "modules": {},
            "orig": orig,
            "type": "DYNAMIC_TYPE_AV",
            "visible": true
        })
    }

    #[test]
    fn dynamic_detail_item_accepts_legacy_numeric_id() {
        let item = serde_json::from_value::<DynamicDetailItem>(dynamic_item_json(
            serde_json::json!(188029375348892_u64),
            serde_json::Value::Null,
        ))
        .expect("legacy numeric dynamic id should normalize to text");

        assert_eq!(item.id_str, "188029375348892");
    }

    #[test]
    fn dynamic_detail_item_accepts_null_id_in_deleted_orig_placeholder() {
        let deleted_orig = dynamic_item_json(serde_json::Value::Null, serde_json::Value::Null);
        let item = serde_json::from_value::<DynamicDetailItem>(dynamic_item_json(
            serde_json::json!("944264795038679059"),
            deleted_orig,
        ))
        .expect("deleted orig placeholder should not prevent forward parsing");

        assert_eq!(
            item.orig
                .expect("orig placeholder should remain available")
                .id_str,
            ""
        );
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "detail" => include_bytes!("../../tests/contracts/dynamic/detail/detail/contract.json")
                .as_slice(),
            "reactions" => {
                include_bytes!("../../tests/contracts/dynamic/detail/reactions/contract.json")
                    .as_slice()
            }
            "forwards" => {
                include_bytes!("../../tests/contracts/dynamic/detail/forwards/contract.json")
                    .as_slice()
            }
            "pics" => {
                include_bytes!("../../tests/contracts/dynamic/detail/pics/contract.json").as_slice()
            }
            "forward-item" => {
                include_bytes!("../../tests/contracts/dynamic/detail/forward-item/contract.json")
                    .as_slice()
            }
            "lottery-notice" => include_bytes!(
                "../../tests/contracts/dynamic/lottery-notice-read/lottery-notice/contract.json"
            )
            .as_slice(),
            _ => unreachable!("unknown dynamic detail endpoint"),
        };

        EndpointContract::from_slice(bytes)
    }

    fn query_map<I>(query: I) -> BTreeMap<String, String>
    where
        I: IntoIterator<Item = (&'static str, String)>,
    {
        query
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect()
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_dynamic_detail() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let dynamic_id = "1099138163191840776";
        let data = bpi
            .dynamic()
            .detail(DynamicDetailParams::new(parse_dynamic_id(dynamic_id)?))
            .await?;

        info!("动态详情: {:?}", data.item);
        assert_eq!(data.item.id_str, dynamic_id);

        let dynamic_id = "1152614216889270274"; // 此动态为陈叔叔的一条转发动态
        let data = bpi
            .dynamic()
            .detail(DynamicDetailParams::new(parse_dynamic_id(dynamic_id)?))
            .await?;
        info!("动态详情: {:?}", data.item);
        assert_eq!(data.item.id_str, dynamic_id);
        assert!(data.item.orig.is_some());

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_dynamic_reactions() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let dynamic_id = "1099138163191840776";
        let data = bpi
            .dynamic()
            .reactions(DynamicReactionsParams::new(parse_dynamic_id(dynamic_id)?))
            .await?;

        info!("点赞/转发总数: {}", data.total);
        assert!(!data.items.is_empty());

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_lottery_notice() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let dynamic_id = "969916293954142214";
        let data = bpi
            .dynamic()
            .lottery_notice(DynamicLotteryNoticeParams::new(parse_dynamic_id(
                dynamic_id,
            )?))
            .await?;

        info!("抽奖状态: {}", data.status);
        assert_eq!(data.business_id.to_string(), dynamic_id);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_dynamic_forwards() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let dynamic_id = "1099138163191840776";
        let data = bpi
            .dynamic()
            .forwards(DynamicForwardsParams::new(parse_dynamic_id(dynamic_id)?))
            .await?;

        info!("转发总数: {}", data.total);
        assert!(!data.items.is_empty());

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_dynamic_pics() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let dynamic_id = "1099138163191840776";
        let data = bpi
            .dynamic()
            .pics(DynamicPicsParams::new(parse_dynamic_id(dynamic_id)?))
            .await?;

        info!("图片数量: {}", data.len());
        assert!(!data.is_empty());

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_forward_item() -> Result<(), BpiError> {
        let bpi = BpiClient::new().expect("client should build");
        let dynamic_id = "1110902525317349376";
        let data = bpi
            .dynamic()
            .forward_item(DynamicForwardItemParams::new(parse_dynamic_id(dynamic_id)?))
            .await?;

        info!("转发动态详情: {:?}", data.item);
        assert_eq!(data.item.id_str, dynamic_id);

        Ok(())
    }

    #[test]
    fn dynamic_detail_params_serializes_default_features() -> Result<(), BpiError> {
        let params = DynamicDetailParams::new(parse_dynamic_id("1099138163191840776")?);

        assert_eq!(
            params.query_pairs(),
            [
                ("id", "1099138163191840776".to_string()),
                (
                    "features",
                    "htmlNewStyle,itemOpusStyle,decorationCard".to_string()
                ),
            ]
        );
        Ok(())
    }

    #[test]
    fn dynamic_detail_params_serializes_custom_features() -> Result<(), BpiError> {
        let params = DynamicDetailParams::new(parse_dynamic_id("1099138163191840776")?)
            .with_features("itemOpusStyle,opusBigCover")?;

        assert_eq!(
            params.query_pairs(),
            [
                ("id", "1099138163191840776".to_string()),
                ("features", "itemOpusStyle,opusBigCover".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn dynamic_reactions_params_serializes_offset() -> Result<(), BpiError> {
        let params = DynamicReactionsParams::new(parse_dynamic_id("1099138163191840776")?)
            .with_offset("offset-token")?;

        assert_eq!(
            params.query_pairs(),
            [
                ("id", "1099138163191840776".to_string()),
                ("offset", "offset-token".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn dynamic_lottery_notice_params_serializes_csrf_query() -> Result<(), BpiError> {
        let params = DynamicLotteryNoticeParams::new(parse_dynamic_id("969916293954142214")?);

        assert_eq!(
            params.query_pairs("csrf-token"),
            [
                ("business_id", "969916293954142214".to_string()),
                ("business_type", "1".to_string()),
                ("csrf", "csrf-token".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn dynamic_pics_params_serializes_query() -> Result<(), BpiError> {
        let params = DynamicPicsParams::new(parse_dynamic_id("1099138163191840776")?);

        assert_eq!(
            params.query_pairs(),
            [("id", "1099138163191840776".to_string())]
        );
        Ok(())
    }

    #[test]
    fn dynamic_forward_item_params_serializes_query() -> Result<(), BpiError> {
        let params = DynamicForwardItemParams::new(parse_dynamic_id("1110902525317349376")?);

        assert_eq!(
            params.query_pairs(),
            [("id", "1110902525317349376".to_string())]
        );
        Ok(())
    }

    #[test]
    fn dynamic_forwards_params_rejects_blank_offset() -> Result<(), BpiError> {
        let err = DynamicForwardsParams::new(parse_dynamic_id("1099138163191840776")?)
            .with_offset("   ")
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "offset",
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn dynamic_detail_read_contracts_match_endpoint_requests() -> BpiResult<()> {
        let detail = contract("detail")?;
        assert_eq!(detail.name, "dynamic.detail");
        assert_eq!(detail.request.method, HttpMethod::Get);
        assert_eq!(
            detail.request.url.as_str(),
            "https://api.bilibili.com/x/polymer/web-dynamic/v1/detail"
        );
        assert_eq!(
            detail.request.query,
            query_map(
                DynamicDetailParams::new(parse_dynamic_id("1099138163191840776")?).query_pairs()
            )
        );
        assert_eq!(detail.cases.len(), 3);

        let reactions = contract("reactions")?;
        assert_eq!(reactions.name, "dynamic.detail_reaction");
        assert_eq!(
            reactions.request.url.as_str(),
            "https://api.bilibili.com/x/polymer/web-dynamic/v1/detail/reaction"
        );
        assert_eq!(
            reactions.request.query,
            query_map(
                DynamicReactionsParams::new(parse_dynamic_id("1099138163191840776")?).query_pairs()
            )
        );

        let forwards = contract("forwards")?;
        assert_eq!(forwards.name, "dynamic.detail_forward");
        assert_eq!(
            forwards.request.url.as_str(),
            "https://api.bilibili.com/x/polymer/web-dynamic/v1/detail/forward"
        );
        assert_eq!(
            forwards.request.query,
            query_map(
                DynamicForwardsParams::new(parse_dynamic_id("1099138163191840776")?).query_pairs()
            )
        );

        let pics = contract("pics")?;
        assert_eq!(pics.name, "dynamic.detail_pic");
        assert_eq!(
            pics.request.url.as_str(),
            "https://api.bilibili.com/x/polymer/web-dynamic/v1/detail/pic"
        );
        assert_eq!(
            pics.request.query,
            query_map(
                DynamicPicsParams::new(parse_dynamic_id("1099138163191840776")?).query_pairs()
            )
        );

        let forward_item = contract("forward-item")?;
        let forward_item_id = parse_dynamic_id("1110902525317349376")?;
        assert_eq!(forward_item.name, "dynamic.detail_forward_item");
        assert_eq!(
            forward_item.request.url.as_str(),
            "https://api.bilibili.com/x/polymer/web-dynamic/v1/detail/forward/item"
        );
        assert_eq!(
            forward_item.request.query,
            query_map(DynamicForwardItemParams::new(forward_item_id).query_pairs())
        );
        assert_eq!(
            forward_item.cases[0].response.error.as_deref(),
            Some("requires_login")
        );

        let lottery_notice = contract("lottery-notice")?;
        assert_eq!(lottery_notice.name, "dynamic.lottery_notice");
        assert_eq!(
            lottery_notice.request.url.as_str(),
            "https://api.vc.bilibili.com/lottery_svr/v1/lottery_svr/lottery_notice"
        );
        assert_eq!(
            lottery_notice.request.query,
            query_map(
                DynamicLotteryNoticeParams::new(parse_dynamic_id("969916293954142214")?)
                    .query_pairs("${csrf}")
            )
        );
        assert_eq!(lottery_notice.cases.len(), 3);
        Ok(())
    }

    #[test]
    fn dynamic_detail_read_response_fixtures_parse_declared_models() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/dynamic/detail/detail/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/dynamic/detail/detail/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/dynamic/detail/detail/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<DynamicDetailData>::from_slice(bytes)?.into_payload()?;
            assert_eq!(payload.item.id_str, "1099138163191840776");
        }

        for bytes in [
            include_bytes!(
                "../../tests/contracts/dynamic/detail/reactions/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/dynamic/detail/reactions/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/dynamic/detail/reactions/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<DynamicReactionData>::from_slice(bytes)?.into_payload()?;
            let _ = payload.total;
        }

        for bytes in [
            include_bytes!(
                "../../tests/contracts/dynamic/detail/forwards/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/dynamic/detail/forwards/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/dynamic/detail/forwards/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload = ApiEnvelope::<DynamicForwardData>::from_slice(bytes)?.into_payload()?;
            assert_eq!(payload.items.len(), 1);
        }

        for bytes in [
            include_bytes!(
                "../../tests/contracts/dynamic/detail/pics/responses/anonymous.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/dynamic/detail/pics/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!("../../tests/contracts/dynamic/detail/pics/responses/vip.success.json")
                .as_slice(),
        ] {
            let payload = ApiEnvelope::<Vec<DynamicPic>>::from_slice(bytes)?.into_payload()?;
            assert_eq!(payload.len(), 1);
        }

        for bytes in [
            include_bytes!(
                "../../tests/contracts/dynamic/detail/forward-item/responses/normal.success.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/dynamic/detail/forward-item/responses/vip.success.json"
            )
            .as_slice(),
        ] {
            let payload =
                ApiEnvelope::<DynamicForwardInfoData>::from_slice(bytes)?.into_payload()?;
            assert_eq!(payload.item.id_str, "1110902525317349376");
        }

        let payload = ApiEnvelope::<DynamicLotteryData>::from_slice(include_bytes!(
            "../../tests/contracts/dynamic/lottery-notice-read/lottery-notice/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(payload.business_id, 969916293954142214);
        assert_eq!(
            payload
                .lottery_result
                .as_ref()
                .map(|result| result.first_prize_result.len()),
            Some(1)
        );
        Ok(())
    }

    #[test]
    fn dynamic_forward_item_anonymous_fixture_records_login_error() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/dynamic/detail/forward-item/responses/anonymous.requires_login.json"
        ))?
        .ensure_success()
        .unwrap_err();

        assert_eq!(err.code(), Some(-101));
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let batch = if endpoint == "lottery-notice" {
            "lottery-notice-read"
        } else {
            "detail-readonly"
        };
        let path =
            format!("target/bpi-probe-runs/dynamic/{batch}/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn dynamic_detail_read_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body("detail", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<DynamicDetailData>>(body)?
                    .into_payload()?;
                assert_eq!(payload.item.id_str, "1099138163191840776");
            }

            if let Some(body) = local_probe_body("reactions", profile) {
                let _ = serde_json::from_value::<ApiEnvelope<DynamicReactionData>>(body)?
                    .into_payload()?;
            }

            if let Some(body) = local_probe_body("forwards", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<DynamicForwardData>>(body)?
                    .into_payload()?;
                assert!(!payload.items.is_empty());
            }

            if let Some(body) = local_probe_body("pics", profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<Vec<DynamicPic>>>(body)?.into_payload()?;
                assert!(!payload.is_empty());
            }
        }

        if let Some(body) = local_probe_body("forward-item", "anonymous") {
            let err = serde_json::from_value::<ApiEnvelope<serde_json::Value>>(body)?
                .ensure_success()
                .unwrap_err();
            assert_eq!(err.code(), Some(-101));
        }

        for profile in ["normal", "vip"] {
            if let Some(body) = local_probe_body("forward-item", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<DynamicForwardInfoData>>(body)?
                    .into_payload()?;
                assert_eq!(payload.item.id_str, "1110902525317349376");
            }
        }

        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body("lottery-notice", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<DynamicLotteryData>>(body)?
                    .into_payload()?;
                assert_eq!(payload.business_id, 969916293954142214);
            }
        }
        Ok(())
    }
}
