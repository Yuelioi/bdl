use serde::{Deserialize, Serialize};

// --- 获取收藏夹元数据 ---

/// 收藏夹元数据的创建者信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FavFolderUpper {
    pub mid: u64,
    pub name: String,
    pub face: String,
    pub followed: bool,
    pub vip_type: u8,
    /// 阿b拼写错误
    #[serde(rename = "vip_statue")]
    pub vip_status: u8,
}

/// 收藏夹元数据的状态数
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FavFolderCntInfo {
    pub collect: u64,
    pub play: u64,
    pub thumb_up: u64,
    pub share: u64,
}

/// 收藏夹元数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FavFolderInfo {
    pub id: u64,
    pub fid: u64,
    pub mid: u64,
    pub attr: u32,
    pub title: String,
    pub cover: String,
    pub upper: FavFolderUpper,
    pub cover_type: u8,
    pub cnt_info: FavFolderCntInfo,
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

// --- 获取指定用户创建的所有收藏夹信息 ---

/// 用户创建的收藏夹列表项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreatedFolderItem {
    pub id: u64,
    pub fid: u64,
    pub mid: u64,
    pub attr: u32,
    pub title: String,
    pub fav_state: u8,
    pub media_count: u32,
}

/// 用户创建的收藏夹信息数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreatedFolderListData {
    pub count: u32,
    pub list: Vec<CreatedFolderItem>,
}

// --- 查询用户收藏的视频收藏夹 ---

/// 用户收藏的视频收藏夹列表项的创建人信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CollectedFolderUpper {
    pub mid: u64,
    pub name: String,
    pub face: String,
}

/// 用户收藏的视频收藏夹列表项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CollectedFolderItem {
    pub id: u64,
    pub fid: u64,
    pub mid: u64,
    pub attr: u32,
    pub title: String,
    pub cover: String,
    pub upper: CollectedFolderUpper,
    pub cover_type: u8,
    pub intro: String,
    pub ctime: u64,
    pub mtime: u64,
    pub state: u8,
    pub fav_state: u8,
    pub media_count: u32,
}

/// 用户收藏的视频收藏夹列表数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CollectedFolderListData {
    pub count: u32,
    pub list: Vec<CollectedFolderItem>,
}

// --- 批量获取指定收藏id的内容 ---

/// 内容信息列表中的UP主信息
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceInfoUpper {
    pub mid: u64,
    pub name: String,
    pub face: String,
}

/// 内容信息列表中的状态数
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceInfoCntInfo {
    pub collect: u64,
    pub play: u64,
    pub danmaku: u64,
}

/// 批量获取的内容信息列表项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceInfoItem {
    pub id: u64,
    #[serde(rename = "type")]
    pub type_name: u8,
    pub title: String,
    pub cover: String,
    pub intro: String,
    pub page: Option<u32>,
    pub duration: u32,
    pub upper: ResourceInfoUpper,
    pub attr: u8,
    pub cnt_info: ResourceInfoCntInfo,
    pub link: String,
    pub ctime: u64,
    pub pubtime: u64,
    pub fav_time: u64,
    pub bv_id: Option<String>,
    pub bvid: Option<String>,
    pub season: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fav::params::{
        FavCollectedListParams, FavCreatedListParams, FavFolderInfoParams, FavResourceInfosParams,
    };
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiResult};
    use tracing::info;

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "folder-info" => {
                include_bytes!("../../tests/contracts/fav/read/folder-info/contract.json")
                    .as_slice()
            }
            "created-list" => {
                include_bytes!("../../tests/contracts/fav/read/created-list/contract.json")
                    .as_slice()
            }
            "collected-list" => {
                include_bytes!("../../tests/contracts/fav/read/collected-list/contract.json")
                    .as_slice()
            }
            "resource-infos" => {
                include_bytes!("../../tests/contracts/fav/read/resource-infos/contract.json")
                    .as_slice()
            }
            _ => unreachable!("unknown fav info contract endpoint"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_fav_folder_info() {
        let bpi = BpiClient::new().expect("client should build");
        // 替换为一个公开收藏夹的media_id
        let params = FavFolderInfoParams::new(
            crate::ids::MediaId::new(1052622027).expect("fixture media id should be valid"),
        );
        let resp = bpi.fav().folder_info(params).await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("folder title: {}", data.title);
        info!("folder media_count: {}", data.media_count);
        info!("upper info: {:?}", data.upper);
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_fav_created_list() {
        let bpi = BpiClient::new().expect("client should build");

        let params = FavCreatedListParams::new(
            crate::ids::Mid::new(7792521).expect("fixture mid should be valid"),
        );
        let resp = bpi.fav().created_list(params).await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("created folders count: {}", data.count);
        info!("first folder info: {:?}", data.list.first());
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_fav_collected_list() {
        let bpi = BpiClient::new().expect("client should build");

        let params = FavCollectedListParams::new(
            crate::ids::Mid::new(7792521).expect("fixture mid should be valid"),
        )
        .with_page(1)
        .expect("fixture page should be valid")
        .with_page_size(20)
        .expect("fixture page size should be valid");
        let resp = bpi.fav().collected_list(params).await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("collected folders count: {}", data.count);
        info!("first collected folder info: {:?}", data.list.first());
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_fav_resource_infos() {
        let bpi = BpiClient::new().expect("client should build");
        let params =
            FavResourceInfosParams::new("371494037:2").expect("fixture resources should be valid");
        let resp = bpi.fav().resource_infos(params).await;

        info!("{:?}", resp);
        assert!(resp.is_ok());

        let data = resp.unwrap();
        info!("retrieved {} resources", data.len());
        info!("first resource info: {:?}", data.first());
    }

    #[test]
    fn fav_folder_info_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("folder-info")?;

        assert_eq!(contract.name, "fav.folder_info");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/v3/fav/folder/info"
        );
        assert_eq!(
            contract.request.query.get("media_id").map(String::as_str),
            Some("1052622027")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("FavFolderInfo")
        );
        Ok(())
    }

    #[test]
    fn fav_created_list_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("created-list")?;

        assert_eq!(contract.name, "fav.created_list");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/v3/fav/folder/created/list-all"
        );
        assert_eq!(
            contract.request.query.get("up_mid").map(String::as_str),
            Some("7792521")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("CreatedFolderListData")
        );
        Ok(())
    }

    #[test]
    fn fav_collected_list_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("collected-list")?;

        assert_eq!(contract.name, "fav.collected_list");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/v3/fav/folder/collected/list"
        );
        assert_eq!(
            contract.request.query.get("up_mid").map(String::as_str),
            Some("7792521")
        );
        assert_eq!(
            contract.request.query.get("platform").map(String::as_str),
            Some("web")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("CollectedFolderListData")
        );
        Ok(())
    }

    #[test]
    fn fav_resource_infos_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract("resource-infos")?;

        assert_eq!(contract.name, "fav.resource_infos");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/v3/fav/resource/infos"
        );
        assert_eq!(
            contract.request.query.get("resources").map(String::as_str),
            Some("371494037:2")
        );
        assert_eq!(contract.cases.len(), 3);
        assert_eq!(
            contract.cases[0].response.rust_model.as_deref(),
            Some("Vec<ResourceInfoItem>")
        );
        Ok(())
    }

    #[test]
    fn fav_info_response_fixtures_parse_declared_models() -> BpiResult<()> {
        let folder = ApiEnvelope::<FavFolderInfo>::from_slice(include_bytes!(
            "../../tests/contracts/fav/read/folder-info/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(folder.id, 1052622027);

        let created = ApiEnvelope::<CreatedFolderListData>::from_slice(include_bytes!(
            "../../tests/contracts/fav/read/created-list/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(created.list.len(), 1);

        let collected = ApiEnvelope::<CollectedFolderListData>::from_slice(include_bytes!(
            "../../tests/contracts/fav/read/collected-list/responses/success.json"
        ))?
        .into_payload()?;
        assert!(collected.list.is_empty());

        let resources = ApiEnvelope::<Vec<ResourceInfoItem>>::from_slice(include_bytes!(
            "../../tests/contracts/fav/read/resource-infos/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(resources.len(), 1);
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
    fn fav_info_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body("folder-info", profile) {
                let payload =
                    serde_json::from_value::<ApiEnvelope<FavFolderInfo>>(body)?.into_payload()?;
                assert_eq!(payload.id, 1052622027);
            }

            if let Some(body) = local_probe_body("created-list", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<CreatedFolderListData>>(body)?
                    .into_payload()?;
                assert!(payload.count >= payload.list.len() as u32);
            }

            if let Some(body) = local_probe_body("collected-list", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<CollectedFolderListData>>(body)?
                    .into_payload()?;
                assert!(payload.count >= payload.list.len() as u32);
            }

            if let Some(body) = local_probe_body("resource-infos", profile) {
                let payload = serde_json::from_value::<ApiEnvelope<Vec<ResourceInfoItem>>>(body)?
                    .into_payload()?;
                assert!(!payload.is_empty());
            }
        }
        Ok(())
    }
}
