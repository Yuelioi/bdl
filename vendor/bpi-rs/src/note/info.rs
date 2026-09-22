use serde::{Deserialize, Serialize};

// --- 查询该稿件是否禁止笔记 ---

/// 稿件是否禁止笔记的响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NoteIsForbidData {
    /// 是否禁止笔记
    pub forbid_note_entrance: bool,
}

// --- 查询私有笔记内容 ---

/// 私有笔记的视频稿件信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrivateNoteArc {
    pub oid: u64,
    pub oid_type: u8,
    pub title: String,
    pub pic: String,
    pub status: u32,
    pub desc: String,
}

/// 私有笔记的标签
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrivateNoteTag {
    pub cid: u64,
    pub status: u8,
    pub index: u32,
    pub seconds: u32,
    pub pos: u32,
}

/// 私有笔记的响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrivateNoteInfoData {
    pub arc: PrivateNoteArc,
    pub audit_status: u8,
    pub cid_count: u32,
    pub content: String,
    pub forbid_note_entrance: bool,
    pub pub_reason: Option<String>,
    pub pub_status: u8,
    pub pub_version: u32,
    pub summary: String,
    pub tags: Vec<PrivateNoteTag>,
    pub title: String,
}

// --- 查询公开笔记内容 ---

/// 公开笔记的稿件信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicNoteArc {
    pub oid: u64,
    pub oid_type: u8,
    pub title: String,
    pub status: u32,
    pub pic: String,
    pub desc: String,
}

/// 公开笔记的作者信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicNoteAuthor {
    pub mid: u64,
    pub name: String,
    pub face: String,
    pub level: u8,
    pub vip_info: serde_json::Value,
    pub pendant: serde_json::Value,
}

/// 公开笔记的响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicNoteInfoData {
    pub cvid: u64,
    pub note_id: u64,
    pub title: String,
    pub summary: String,
    pub content: String,
    pub cid_count: u32,
    pub pub_status: u8,
    pub tags: Vec<PrivateNoteTag>,
    pub arc: PublicNoteArc,
    pub author: PublicNoteAuthor,
    pub forbid_note_entrance: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::{Aid, Cvid, NoteId};
    use crate::note::{NoteIsForbidParams, NotePrivateInfoParams, NotePublicInfoParams};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use tracing::info;

    const TEST_AID: u64 = 338_677_252;
    const TEST_PRIVATE_AID: u64 = 676_931_260;
    const TEST_NOTE_ID: u64 = 83_577_722_856_540_160;
    const TEST_CVID: u64 = 15_160_286;

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "is-forbid" => {
                include_bytes!("../../tests/contracts/note/read/is-forbid/contract.json").as_slice()
            }
            "private-info" => {
                include_bytes!("../../tests/contracts/note/read/private-info/contract.json")
                    .as_slice()
            }
            "public-info" => {
                include_bytes!("../../tests/contracts/note/read/public-info/contract.json")
                    .as_slice()
            }
            _ => {
                return Err(BpiError::invalid_parameter(
                    "endpoint",
                    "unknown note info contract",
                ));
            }
        };

        EndpointContract::from_slice(bytes)
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_note_is_forbid() {
        let bpi = BpiClient::new().expect("client should build");
        let resp = bpi
            .note()
            .is_forbid(NoteIsForbidParams::new(
                Aid::new(TEST_AID).expect("test aid should be valid"),
            ))
            .await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("forbid_note_entrance: {}", data.forbid_note_entrance);
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_note_get_private_info() {
        let bpi = BpiClient::new().expect("client should build");
        let resp = bpi
            .note()
            .private_info(NotePrivateInfoParams::new(
                Aid::new(TEST_PRIVATE_AID).expect("test aid should be valid"),
                NoteId::new(TEST_NOTE_ID).expect("test note id should be valid"),
            ))
            .await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("note title: {}", data.title);
        info!("note content: {}", data.content);
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_note_get_public_info() {
        let bpi = BpiClient::new().expect("client should build");
        let resp = bpi
            .note()
            .public_info(NotePublicInfoParams::new(
                Cvid::new(TEST_CVID).expect("test cvid should be valid"),
            ))
            .await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("note title: {}", data.title);
        info!("note content: {}", data.content);
        info!("author name: {}", data.author.name);
    }

    #[test]
    fn note_is_forbid_params_serializes_aid() -> Result<(), BpiError> {
        let params = NoteIsForbidParams::new(Aid::new(TEST_AID)?);

        assert_eq!(params.query_pairs(), vec![("aid", TEST_AID.to_string())]);
        Ok(())
    }

    #[test]
    fn note_private_info_params_serializes_required_query() -> Result<(), BpiError> {
        let params =
            NotePrivateInfoParams::new(Aid::new(TEST_PRIVATE_AID)?, NoteId::new(TEST_NOTE_ID)?);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("oid", TEST_PRIVATE_AID.to_string()),
                ("oid_type", "0".to_string()),
                ("note_id", TEST_NOTE_ID.to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn cvid_rejects_zero() {
        let err = Cvid::new(0).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "cvid", .. }
        ));
    }

    #[test]
    fn note_info_contracts_match_endpoint_requests() -> BpiResult<()> {
        let is_forbid = contract("is-forbid")?;
        assert_eq!(is_forbid.name, "note.is_forbid");
        assert_eq!(is_forbid.request.method, HttpMethod::Get);
        assert_eq!(
            is_forbid.request.url.as_str(),
            "https://api.bilibili.com/x/note/is_forbid"
        );
        assert_eq!(
            is_forbid.request.query.get("aid").map(String::as_str),
            Some("338677252")
        );
        assert_eq!(
            is_forbid.cases[0].response.rust_model.as_deref(),
            Some("NoteIsForbidData")
        );

        let private_info = contract("private-info")?;
        assert_eq!(private_info.name, "note.private_info");
        assert_eq!(
            private_info.request.url.as_str(),
            "https://api.bilibili.com/x/note/info"
        );
        assert_eq!(
            private_info
                .request
                .query
                .get("note_id")
                .map(String::as_str),
            Some("83577722856540160")
        );
        assert_eq!(private_info.cases[0].response.api_code, Some(-101));
        assert_eq!(private_info.cases[1].response.api_code, Some(79511));
        assert_eq!(
            private_info.cases[2].response.rust_model.as_deref(),
            Some("PrivateNoteInfoData")
        );

        let public_info = contract("public-info")?;
        assert_eq!(public_info.name, "note.public_info");
        assert_eq!(
            public_info.request.url.as_str(),
            "https://api.bilibili.com/x/note/publish/info"
        );
        assert_eq!(
            public_info.request.query.get("cvid").map(String::as_str),
            Some("15160286")
        );
        assert_eq!(
            public_info.cases[0].response.rust_model.as_deref(),
            Some("PublicNoteInfoData")
        );
        Ok(())
    }

    #[test]
    fn note_info_response_fixtures_parse_declared_models() -> BpiResult<()> {
        let is_forbid = ApiEnvelope::<NoteIsForbidData>::from_slice(include_bytes!(
            "../../tests/contracts/note/read/is-forbid/responses/success.json"
        ))?
        .into_payload()?;
        assert!(!is_forbid.forbid_note_entrance);

        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/note/read/private-info/responses/anonymous.requires_login.json"
        ))
        .and_then(ApiEnvelope::ensure_success)
        .unwrap_err();
        assert!(err.requires_login());

        let not_owner: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../tests/contracts/note/read/private-info/responses/normal.not_owner.json"
        ))?;
        assert_eq!(not_owner["code"], 79511);

        let private_info = ApiEnvelope::<PrivateNoteInfoData>::from_slice(include_bytes!(
            "../../tests/contracts/note/read/private-info/responses/vip.success.json"
        ))?
        .into_payload()?;
        assert_eq!(private_info.title, "sanitized private note title");

        let public_info = ApiEnvelope::<PublicNoteInfoData>::from_slice(include_bytes!(
            "../../tests/contracts/note/read/public-info/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(public_info.cvid, TEST_CVID);
        assert_eq!(public_info.author.name, "sanitized author");
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/note/read/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn note_info_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body("is-forbid", profile) else {
                continue;
            };
            serde_json::from_value::<ApiEnvelope<NoteIsForbidData>>(body)?.into_payload()?;
        }

        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body("private-info", profile) else {
                continue;
            };
            match profile {
                "anonymous" => {
                    let err = serde_json::from_value::<ApiEnvelope<serde_json::Value>>(body)?
                        .ensure_success()
                        .unwrap_err();
                    assert!(err.requires_login());
                }
                "normal" => {
                    let value: serde_json::Value = serde_json::from_value(body)?;
                    assert_eq!(value["code"], 79511);
                }
                "vip" => {
                    serde_json::from_value::<ApiEnvelope<PrivateNoteInfoData>>(body)?
                        .into_payload()?;
                }
                _ => unreachable!(),
            }
        }

        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body("public-info", profile) else {
                continue;
            };
            serde_json::from_value::<ApiEnvelope<PublicNoteInfoData>>(body)?.into_payload()?;
        }
        Ok(())
    }
}
