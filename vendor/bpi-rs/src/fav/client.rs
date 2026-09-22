use crate::fav::info::{
    CollectedFolderListData, CreatedFolderListData, FavFolderInfo, ResourceInfoItem,
};
use crate::fav::list::{FavListDetailData, FavResourceIdItem};
use crate::fav::{
    FavCollectedListParams, FavCreatedListParams, FavFolderInfoParams, FavListDetailParams,
    FavResourceIdsParams, FavResourceInfosParams,
};
use crate::{BilibiliRequest, BpiClient, BpiResult};

const FOLDER_INFO_ENDPOINT: &str = "https://api.bilibili.com/x/v3/fav/folder/info";
const CREATED_LIST_ENDPOINT: &str = "https://api.bilibili.com/x/v3/fav/folder/created/list-all";
const COLLECTED_LIST_ENDPOINT: &str = "https://api.bilibili.com/x/v3/fav/folder/collected/list";
const RESOURCE_INFOS_ENDPOINT: &str = "https://api.bilibili.com/x/v3/fav/resource/infos";
const LIST_DETAIL_ENDPOINT: &str = "https://api.bilibili.com/x/v3/fav/resource/list";
const RESOURCE_IDS_ENDPOINT: &str = "https://api.bilibili.com/x/v3/fav/resource/ids";

/// 收藏 API 客户端。
#[derive(Clone, Copy)]
pub struct FavClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> FavClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn folder_info_endpoint(&self) -> &'static str {
        FOLDER_INFO_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn created_list_endpoint(&self) -> &'static str {
        CREATED_LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn collected_list_endpoint(&self) -> &'static str {
        COLLECTED_LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn resource_infos_endpoint(&self) -> &'static str {
        RESOURCE_INFOS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn list_detail_endpoint(&self) -> &'static str {
        LIST_DETAIL_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn resource_ids_endpoint(&self) -> &'static str {
        RESOURCE_IDS_ENDPOINT
    }

    /// 获取收藏夹元数据。
    pub async fn folder_info(&self, params: FavFolderInfoParams) -> BpiResult<FavFolderInfo> {
        self.client
            .get(FOLDER_INFO_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("fav.folder_info")
            .await
    }

    /// 获取用户创建的收藏夹。
    pub async fn created_list(
        &self,
        params: FavCreatedListParams,
    ) -> BpiResult<CreatedFolderListData> {
        self.client
            .get(CREATED_LIST_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("fav.created_list")
            .await
    }

    /// 获取用户收藏的收藏夹。
    pub async fn collected_list(
        &self,
        params: FavCollectedListParams,
    ) -> BpiResult<CollectedFolderListData> {
        self.client
            .get(COLLECTED_LIST_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("fav.collected_list")
            .await
    }

    /// 获取多个收藏资源的信息。
    pub async fn resource_infos(
        &self,
        params: FavResourceInfosParams,
    ) -> BpiResult<Vec<ResourceInfoItem>> {
        self.client
            .get(RESOURCE_INFOS_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("fav.resource_infos")
            .await
    }

    /// 获取收藏夹的详细资源列表。
    pub async fn list_detail(&self, params: FavListDetailParams) -> BpiResult<FavListDetailData> {
        self.client
            .get(LIST_DETAIL_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("fav.list_detail")
            .await
    }

    /// 获取收藏夹中的全部资源 ID。
    pub async fn resource_ids(
        &self,
        params: FavResourceIdsParams,
    ) -> BpiResult<Vec<FavResourceIdItem>> {
        self.client
            .get(RESOURCE_IDS_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("fav.resource_ids")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::fav::info::{
        CollectedFolderListData, CreatedFolderListData, FavFolderInfo, ResourceInfoItem,
    };
    use crate::fav::list::{FavListDetailData, FavResourceIdItem};
    use crate::fav::{
        FavCollectedListParams, FavCreatedListParams, FavFolderInfoParams, FavListDetailParams,
        FavResourceIdsParams, FavResourceInfosParams,
    };
    use crate::ids::{MediaId, Mid};
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiResult};

    const TEST_MEDIA_ID: u64 = 1_052_622_027;
    const TEST_MID: u64 = 7_792_521;

    fn media_id() -> BpiResult<MediaId> {
        MediaId::new(TEST_MEDIA_ID)
    }

    fn mid() -> BpiResult<Mid> {
        Mid::new(TEST_MID)
    }

    fn assert_folder_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<FavFolderInfo>>,
    {
    }

    fn assert_created_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<CreatedFolderListData>>,
    {
    }

    fn assert_collected_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<CollectedFolderListData>>,
    {
    }

    fn assert_resource_infos_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<Vec<ResourceInfoItem>>>,
    {
    }

    fn assert_list_detail_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<FavListDetailData>>,
    {
    }

    fn assert_resource_ids_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<Vec<FavResourceIdItem>>>,
    {
    }

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
            "list-detail" => {
                include_bytes!("../../tests/contracts/fav/read/list-detail/contract.json")
                    .as_slice()
            }
            "resource-ids" => {
                include_bytes!("../../tests/contracts/fav/read/resource-ids/contract.json")
                    .as_slice()
            }
            _ => unreachable!("unknown fav read contract"),
        };
        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn fav_client_exposes_promoted_endpoint_urls() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let fav = client.fav();

        assert_eq!(
            fav.folder_info_endpoint(),
            "https://api.bilibili.com/x/v3/fav/folder/info"
        );
        assert_eq!(
            fav.created_list_endpoint(),
            "https://api.bilibili.com/x/v3/fav/folder/created/list-all"
        );
        assert_eq!(
            fav.collected_list_endpoint(),
            "https://api.bilibili.com/x/v3/fav/folder/collected/list"
        );
        assert_eq!(
            fav.resource_infos_endpoint(),
            "https://api.bilibili.com/x/v3/fav/resource/infos"
        );
        assert_eq!(
            fav.list_detail_endpoint(),
            "https://api.bilibili.com/x/v3/fav/resource/list"
        );
        assert_eq!(
            fav.resource_ids_endpoint(),
            "https://api.bilibili.com/x/v3/fav/resource/ids"
        );
        Ok(())
    }

    #[test]
    fn fav_methods_return_payload_futures() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let fav = client.fav();

        assert_folder_info_future(fav.folder_info(FavFolderInfoParams::new(media_id()?)));
        assert_created_list_future(fav.created_list(FavCreatedListParams::new(mid()?)));
        assert_collected_list_future(fav.collected_list(FavCollectedListParams::new(mid()?)));
        assert_resource_infos_future(
            fav.resource_infos(FavResourceInfosParams::new("371494037:2")?),
        );
        assert_list_detail_future(
            fav.list_detail(
                FavListDetailParams::new(media_id()?)
                    .order("mtime")?
                    .content_type(0)
                    .page_size(5)?
                    .page(1)?,
            ),
        );
        assert_resource_ids_future(fav.resource_ids(FavResourceIdsParams::new(media_id()?)));
        Ok(())
    }

    #[test]
    fn fav_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let fav = client.fav();
        let folder_info = contract("folder-info")?;
        let created_list = contract("created-list")?;
        let collected_list = contract("collected-list")?;
        let resource_infos = contract("resource-infos")?;
        let list_detail = contract("list-detail")?;
        let resource_ids = contract("resource-ids")?;

        assert_eq!(folder_info.name, "fav.folder_info");
        assert_eq!(folder_info.request.method, HttpMethod::Get);
        assert_eq!(folder_info.request.url.as_str(), fav.folder_info_endpoint());

        assert_eq!(created_list.name, "fav.created_list");
        assert_eq!(created_list.request.method, HttpMethod::Get);
        assert_eq!(
            created_list.request.url.as_str(),
            fav.created_list_endpoint()
        );

        assert_eq!(collected_list.name, "fav.collected_list");
        assert_eq!(collected_list.request.method, HttpMethod::Get);
        assert_eq!(
            collected_list.request.url.as_str(),
            fav.collected_list_endpoint()
        );

        assert_eq!(resource_infos.name, "fav.resource_infos");
        assert_eq!(resource_infos.request.method, HttpMethod::Get);
        assert_eq!(
            resource_infos.request.url.as_str(),
            fav.resource_infos_endpoint()
        );

        assert_eq!(list_detail.name, "fav.list_detail");
        assert_eq!(list_detail.request.method, HttpMethod::Get);
        assert_eq!(list_detail.request.url.as_str(), fav.list_detail_endpoint());

        assert_eq!(resource_ids.name, "fav.resource_ids");
        assert_eq!(resource_ids.request.method, HttpMethod::Get);
        assert_eq!(
            resource_ids.request.url.as_str(),
            fav.resource_ids_endpoint()
        );
        assert_eq!(
            resource_ids
                .request
                .query
                .get("platform")
                .map(String::as_str),
            Some("web")
        );
        Ok(())
    }
}
