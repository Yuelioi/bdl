use serde::{Deserialize, Serialize};

/// 稿件私有笔记列表数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NoteListArchiveData {
    /// 笔记ID列表
    #[serde(rename = "noteIds")]
    pub note_ids: Option<Vec<String>>,
}

// --- 查询用户私有笔记 ---

/// 用户私有笔记的视频信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrivateNoteArc {
    pub oid: u64,
    pub status: u8,
    pub oid_type: u8,
    pub aid: u64,

    // 老笔记没有以下内容
    pub bvid: Option<String>,
    pub pic: Option<String>,
    pub desc: Option<String>,
}

/// 用户私有笔记列表项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrivateNoteItem {
    pub title: String,
    pub summary: String,
    pub mtime: String,
    pub arc: PrivateNoteArc,
    pub note_id: u64,
    pub audit_status: u8,
    pub web_url: String,
    pub note_id_str: String,
    pub message: String,
    pub forbid_note_entrance: Option<bool>,
    pub likes: u64,
    pub has_like: bool,
}

/// 用户私有笔记列表数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrivateNoteListData {
    pub list: Option<Vec<PrivateNoteItem>>,
    pub page: Option<NotePage>,
}

// --- 查询稿件公开笔记 ---

/// 稿件公开笔记列表项的作者信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicNoteAuthor {
    pub mid: u64,
    pub name: String,
    pub face: String,
    pub level: u8,
    pub vip_info: serde_json::Value,
    pub pendant: serde_json::Value,
}

/// 稿件公开笔记列表项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicNoteItem {
    pub cvid: u64,
    pub title: String,
    pub summary: String,
    pub pubtime: String,
    pub web_url: String,
    pub message: String,
    pub author: PublicNoteAuthor,
    pub likes: u64,
    pub has_like: bool,
}

/// 稿件公开笔记分页信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NotePage {
    pub total: u32,
    pub size: u32,
    pub num: u32,
}

/// 稿件公开笔记列表数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicNoteListArchiveData {
    pub list: Option<Vec<PublicNoteItem>>,
    pub page: Option<NotePage>,
    pub show_public_note: bool,
    pub message: String,
}

// --- 查询用户公开笔记 ---

/// 用户公开笔记列表数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PublicNoteListUserData {
    pub list: Option<Vec<PublicNoteItem>>,
    pub page: Option<NotePage>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::Aid;
    use crate::note::{
        NoteArchiveListParams, NotePublicArchiveListParams, NoteUserPrivateListParams,
        NoteUserPublicListParams,
    };
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use base64::{Engine as _, engine::general_purpose};
    use tracing::info;

    const TEST_PRIVATE_AID: u64 = 676_931_260;
    const TEST_PUBLIC_AID: u64 = 338_677_252;

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "archive-list" => {
                include_bytes!("../../tests/contracts/note/read/archive-list/contract.json")
                    .as_slice()
            }
            "user-private-list" => {
                include_bytes!("../../tests/contracts/note/read/user-private-list/contract.json")
                    .as_slice()
            }
            "public-archive-list" => {
                include_bytes!("../../tests/contracts/note/read/public-archive-list/contract.json")
                    .as_slice()
            }
            "user-public-list" => {
                include_bytes!("../../tests/contracts/note/read/user-public-list/contract.json")
                    .as_slice()
            }
            _ => {
                return Err(BpiError::invalid_parameter(
                    "endpoint",
                    "unknown note list contract",
                ));
            }
        };

        EndpointContract::from_slice(bytes)
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_note_list_archive() {
        let bpi = BpiClient::new().expect("client should build");
        let resp = bpi
            .note()
            .archive_list(NoteArchiveListParams::new(
                Aid::new(TEST_PRIVATE_AID).expect("test aid should be valid"),
            ))
            .await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("note ids: {:?}", data.note_ids);
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_note_list_user_private() {
        let bpi = BpiClient::new().expect("client should build");
        let resp = bpi
            .note()
            .user_private_list(
                NoteUserPrivateListParams::new()
                    .with_page(1)
                    .expect("test page should be valid")
                    .with_page_size(10)
                    .expect("test page size should be valid"),
            )
            .await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        if let Some(list) = data.list.as_ref() {
            info!("first note item: {:?}", list.first());
        }
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_note_list_public_archive() {
        let bpi = BpiClient::new().expect("client should build");
        let resp = bpi
            .note()
            .public_archive_list(
                NotePublicArchiveListParams::new(
                    Aid::new(TEST_PUBLIC_AID).expect("test aid should be valid"),
                )
                .with_page(1)
                .expect("test page should be valid")
                .with_page_size(10)
                .expect("test page size should be valid"),
            )
            .await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("show_public_note: {}", data.show_public_note);
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_note_list_public_user() {
        let bpi = BpiClient::new().expect("client should build");
        let resp = bpi
            .note()
            .user_public_list(
                NoteUserPublicListParams::new()
                    .with_page(1)
                    .expect("test page should be valid")
                    .with_page_size(10)
                    .expect("test page size should be valid"),
            )
            .await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("total public notes: {}", data.page.as_ref().unwrap().total);
    }

    #[test]
    fn note_archive_list_params_serializes_aid() -> Result<(), BpiError> {
        let params = NoteArchiveListParams::new(Aid::new(TEST_PRIVATE_AID)?);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("oid", TEST_PRIVATE_AID.to_string()),
                ("oid_type", "0".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn note_user_private_list_params_rejects_zero_page() {
        let err = NoteUserPrivateListParams::new().with_page(0).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "pn", .. }
        ));
    }

    #[test]
    fn note_public_archive_list_params_serializes_query() -> Result<(), BpiError> {
        let params = NotePublicArchiveListParams::new(Aid::new(TEST_PUBLIC_AID)?)
            .with_page(1)?
            .with_page_size(10)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("oid", TEST_PUBLIC_AID.to_string()),
                ("oid_type", "0".to_string()),
                ("pn", "1".to_string()),
                ("ps", "10".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn note_list_contracts_match_endpoint_requests() -> BpiResult<()> {
        let archive_list = contract("archive-list")?;
        assert_eq!(archive_list.name, "note.archive_list");
        assert_eq!(archive_list.request.method, HttpMethod::Get);
        assert_eq!(
            archive_list.request.url.as_str(),
            "https://api.bilibili.com/x/note/list/archive"
        );
        assert_eq!(
            archive_list.request.query.get("oid").map(String::as_str),
            Some("676931260")
        );
        assert_eq!(archive_list.cases[0].response.api_code, Some(-101));
        assert_eq!(
            archive_list.cases[1].response.rust_model.as_deref(),
            Some("NoteListArchiveData")
        );

        let user_private = contract("user-private-list")?;
        assert_eq!(user_private.name, "note.user_private_list");
        assert_eq!(
            user_private.request.url.as_str(),
            "https://api.bilibili.com/x/note/list"
        );
        assert_eq!(
            user_private.request.query.get("pn").map(String::as_str),
            Some("1")
        );
        assert_eq!(
            user_private.cases[1].response.rust_model.as_deref(),
            Some("PrivateNoteListData")
        );

        let public_archive = contract("public-archive-list")?;
        assert_eq!(public_archive.name, "note.public_archive_list");
        assert_eq!(
            public_archive.request.url.as_str(),
            "https://api.bilibili.com/x/note/publish/list/archive"
        );
        assert_eq!(
            public_archive.cases[0].response.rust_model.as_deref(),
            Some("PublicNoteListArchiveData")
        );

        let user_public = contract("user-public-list")?;
        assert_eq!(user_public.name, "note.user_public_list");
        assert_eq!(
            user_public.request.url.as_str(),
            "https://api.bilibili.com/x/note/publish/list/user"
        );
        assert_eq!(user_public.cases[0].response.http_status, Some(200));
        assert_eq!(
            user_public.cases[0].response.error.as_deref(),
            Some("requires_login")
        );
        assert_eq!(
            user_public.cases[1].response.rust_model.as_deref(),
            Some("PublicNoteListUserData")
        );
        Ok(())
    }

    #[test]
    fn note_list_response_fixtures_parse_declared_models() -> BpiResult<()> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/note/read/archive-list/responses/anonymous.requires_login.json"
        ))
        .and_then(ApiEnvelope::ensure_success)
        .unwrap_err();
        assert!(err.requires_login());

        let archive = ApiEnvelope::<NoteListArchiveData>::from_slice(include_bytes!(
            "../../tests/contracts/note/read/archive-list/responses/authenticated.success.json"
        ))?
        .into_payload()?;
        assert_eq!(
            archive
                .note_ids
                .as_ref()
                .and_then(|note_ids| note_ids.first())
                .map(String::as_str),
            Some("1")
        );

        let private_list = ApiEnvelope::<PrivateNoteListData>::from_slice(include_bytes!(
            "../../tests/contracts/note/read/user-private-list/responses/authenticated.success.json"
        ))?
        .into_payload()?;
        assert_eq!(
            private_list
                .list
                .as_ref()
                .and_then(|items| items.first())
                .map(|item| item.title.as_str()),
            Some("sanitized private note title")
        );

        let public_archive = ApiEnvelope::<PublicNoteListArchiveData>::from_slice(include_bytes!(
            "../../tests/contracts/note/read/public-archive-list/responses/closed.success.json"
        ))?
        .into_payload()?;
        assert!(!public_archive.show_public_note);

        let binary: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../tests/contracts/note/read/user-public-list/responses/anonymous.requires_login.binary.json"
        ))?;
        assert_eq!(binary["kind"], "binary");
        let decoded = general_purpose::STANDARD
            .decode(
                binary["body_base64"]
                    .as_str()
                    .ok_or_else(|| BpiError::unsupported_response("missing binary body"))?,
            )
            .map_err(|err| BpiError::parse(err.to_string()))?;
        let decoded_text =
            String::from_utf8(decoded).map_err(|err| BpiError::parse(err.to_string()))?;
        assert!(decoded_text.contains("\"code\":-101"));

        let public_user = ApiEnvelope::<PublicNoteListUserData>::from_slice(include_bytes!(
            "../../tests/contracts/note/read/user-public-list/responses/authenticated.success.json"
        ))?
        .into_payload()?;
        assert_eq!(public_user.page.as_ref().map(|page| page.total), Some(0));
        assert!(public_user.list.is_none());
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
    fn note_list_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body("archive-list", profile) else {
                continue;
            };
            if profile == "anonymous" {
                let err = serde_json::from_value::<ApiEnvelope<serde_json::Value>>(body)?
                    .ensure_success()
                    .unwrap_err();
                assert!(err.requires_login());
                continue;
            }
            serde_json::from_value::<ApiEnvelope<NoteListArchiveData>>(body)?.into_payload()?;
        }

        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body("user-private-list", profile) else {
                continue;
            };
            if profile == "anonymous" {
                let err = serde_json::from_value::<ApiEnvelope<serde_json::Value>>(body)?
                    .ensure_success()
                    .unwrap_err();
                assert!(err.requires_login());
                continue;
            }
            serde_json::from_value::<ApiEnvelope<PrivateNoteListData>>(body)?.into_payload()?;
        }

        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body("public-archive-list", profile) else {
                continue;
            };
            serde_json::from_value::<ApiEnvelope<PublicNoteListArchiveData>>(body)?
                .into_payload()?;
        }

        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body("user-public-list", profile) else {
                continue;
            };
            if profile == "anonymous" {
                assert_eq!(body["kind"], "binary");
                continue;
            }
            serde_json::from_value::<ApiEnvelope<PublicNoteListUserData>>(body)?.into_payload()?;
        }
        Ok(())
    }
}
