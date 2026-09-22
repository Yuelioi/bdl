use crate::note::info::{NoteIsForbidData, PrivateNoteInfoData, PublicNoteInfoData};
use crate::note::list::{
    NoteListArchiveData, PrivateNoteListData, PublicNoteListArchiveData, PublicNoteListUserData,
};
use crate::note::{
    NoteArchiveListParams, NoteIsForbidParams, NotePrivateInfoParams, NotePublicArchiveListParams,
    NotePublicInfoParams, NoteUserPrivateListParams, NoteUserPublicListParams,
};
use crate::{BilibiliRequest, BpiClient, BpiResult};

const IS_FORBID_ENDPOINT: &str = "https://api.bilibili.com/x/note/is_forbid";
const PRIVATE_INFO_ENDPOINT: &str = "https://api.bilibili.com/x/note/info";
const PUBLIC_INFO_ENDPOINT: &str = "https://api.bilibili.com/x/note/publish/info";
const ARCHIVE_LIST_ENDPOINT: &str = "https://api.bilibili.com/x/note/list/archive";
const USER_PRIVATE_LIST_ENDPOINT: &str = "https://api.bilibili.com/x/note/list";
const PUBLIC_ARCHIVE_LIST_ENDPOINT: &str = "https://api.bilibili.com/x/note/publish/list/archive";
const USER_PUBLIC_LIST_ENDPOINT: &str = "https://api.bilibili.com/x/note/publish/list/user";

/// 笔记 API 客户端。
#[derive(Clone, Copy)]
pub struct NoteClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> NoteClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn is_forbid_endpoint(&self) -> &'static str {
        IS_FORBID_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn private_info_endpoint(&self) -> &'static str {
        PRIVATE_INFO_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn public_info_endpoint(&self) -> &'static str {
        PUBLIC_INFO_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn archive_list_endpoint(&self) -> &'static str {
        ARCHIVE_LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn user_private_list_endpoint(&self) -> &'static str {
        USER_PRIVATE_LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn public_archive_list_endpoint(&self) -> &'static str {
        PUBLIC_ARCHIVE_LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn user_public_list_endpoint(&self) -> &'static str {
        USER_PUBLIC_LIST_ENDPOINT
    }

    /// 检查稿件是否禁止笔记。
    pub async fn is_forbid(&self, params: NoteIsForbidParams) -> BpiResult<NoteIsForbidData> {
        self.client
            .get(IS_FORBID_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("note.is_forbid")
            .await
    }

    /// 获取私有笔记内容。
    pub async fn private_info(
        &self,
        params: NotePrivateInfoParams,
    ) -> BpiResult<PrivateNoteInfoData> {
        self.client
            .get(PRIVATE_INFO_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("note.private_info")
            .await
    }

    /// 获取公开笔记内容。
    pub async fn public_info(&self, params: NotePublicInfoParams) -> BpiResult<PublicNoteInfoData> {
        self.client
            .get(PUBLIC_INFO_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("note.public_info")
            .await
    }

    /// 获取稿件的私有笔记 ID。
    pub async fn archive_list(
        &self,
        params: NoteArchiveListParams,
    ) -> BpiResult<NoteListArchiveData> {
        self.client
            .get(ARCHIVE_LIST_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("note.archive_list")
            .await
    }

    /// 获取当前用户拥有的私有笔记。
    pub async fn user_private_list(
        &self,
        params: NoteUserPrivateListParams,
    ) -> BpiResult<PrivateNoteListData> {
        self.client
            .get(USER_PRIVATE_LIST_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("note.user_private_list")
            .await
    }

    /// 获取稿件的公开笔记。
    pub async fn public_archive_list(
        &self,
        params: NotePublicArchiveListParams,
    ) -> BpiResult<PublicNoteListArchiveData> {
        self.client
            .get(PUBLIC_ARCHIVE_LIST_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("note.public_archive_list")
            .await
    }

    /// 获取某个用户创作的公开笔记。
    pub async fn user_public_list(
        &self,
        params: NoteUserPublicListParams,
    ) -> BpiResult<PublicNoteListUserData> {
        self.client
            .get(USER_PUBLIC_LIST_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("note.user_public_list")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::ids::{Aid, Cvid, NoteId};
    use crate::note::info::{NoteIsForbidData, PrivateNoteInfoData, PublicNoteInfoData};
    use crate::note::list::{
        NoteListArchiveData, PrivateNoteListData, PublicNoteListArchiveData, PublicNoteListUserData,
    };
    use crate::note::{
        NoteArchiveListParams, NoteIsForbidParams, NotePrivateInfoParams,
        NotePublicArchiveListParams, NotePublicInfoParams, NoteUserPrivateListParams,
        NoteUserPublicListParams,
    };
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiResult};

    const TEST_AID: u64 = 338_677_252;
    const TEST_PRIVATE_AID: u64 = 676_931_260;
    const TEST_NOTE_ID: u64 = 83_577_722_856_540_160;
    const TEST_CVID: u64 = 15_160_286;

    fn aid() -> BpiResult<Aid> {
        Aid::new(TEST_AID)
    }

    fn private_aid() -> BpiResult<Aid> {
        Aid::new(TEST_PRIVATE_AID)
    }

    fn note_id() -> BpiResult<NoteId> {
        NoteId::new(TEST_NOTE_ID)
    }

    fn cvid() -> BpiResult<Cvid> {
        Cvid::new(TEST_CVID)
    }

    fn assert_is_forbid_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<NoteIsForbidData>>,
    {
    }

    fn assert_private_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<PrivateNoteInfoData>>,
    {
    }

    fn assert_public_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<PublicNoteInfoData>>,
    {
    }

    fn assert_archive_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<NoteListArchiveData>>,
    {
    }

    fn assert_user_private_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<PrivateNoteListData>>,
    {
    }

    fn assert_public_archive_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<PublicNoteListArchiveData>>,
    {
    }

    fn assert_user_public_list_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<PublicNoteListUserData>>,
    {
    }

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
            _ => unreachable!("unknown note read contract"),
        };
        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn note_client_exposes_promoted_endpoint_urls() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let note = client.note();

        assert_eq!(
            note.is_forbid_endpoint(),
            "https://api.bilibili.com/x/note/is_forbid"
        );
        assert_eq!(
            note.private_info_endpoint(),
            "https://api.bilibili.com/x/note/info"
        );
        assert_eq!(
            note.public_info_endpoint(),
            "https://api.bilibili.com/x/note/publish/info"
        );
        assert_eq!(
            note.archive_list_endpoint(),
            "https://api.bilibili.com/x/note/list/archive"
        );
        assert_eq!(
            note.user_private_list_endpoint(),
            "https://api.bilibili.com/x/note/list"
        );
        assert_eq!(
            note.public_archive_list_endpoint(),
            "https://api.bilibili.com/x/note/publish/list/archive"
        );
        assert_eq!(
            note.user_public_list_endpoint(),
            "https://api.bilibili.com/x/note/publish/list/user"
        );
        Ok(())
    }

    #[test]
    fn note_methods_return_payload_futures() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let note = client.note();

        assert_is_forbid_future(note.is_forbid(NoteIsForbidParams::new(aid()?)));
        assert_private_info_future(
            note.private_info(NotePrivateInfoParams::new(private_aid()?, note_id()?)),
        );
        assert_public_info_future(note.public_info(NotePublicInfoParams::new(cvid()?)));
        assert_archive_list_future(note.archive_list(NoteArchiveListParams::new(private_aid()?)));
        assert_user_private_list_future(note.user_private_list(NoteUserPrivateListParams::new()));
        assert_public_archive_list_future(
            note.public_archive_list(NotePublicArchiveListParams::new(aid()?)),
        );
        assert_user_public_list_future(note.user_public_list(NoteUserPublicListParams::new()));
        Ok(())
    }

    #[test]
    fn note_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let note = client.note();
        let is_forbid = contract("is-forbid")?;
        let private_info = contract("private-info")?;
        let public_info = contract("public-info")?;
        let archive_list = contract("archive-list")?;
        let user_private_list = contract("user-private-list")?;
        let public_archive_list = contract("public-archive-list")?;
        let user_public_list = contract("user-public-list")?;

        assert_eq!(is_forbid.name, "note.is_forbid");
        assert_eq!(is_forbid.request.method, HttpMethod::Get);
        assert_eq!(is_forbid.request.url.as_str(), note.is_forbid_endpoint());
        assert_eq!(
            is_forbid.request.query.get("aid").map(String::as_str),
            Some("338677252")
        );

        assert_eq!(private_info.name, "note.private_info");
        assert_eq!(private_info.request.method, HttpMethod::Get);
        assert_eq!(
            private_info.request.url.as_str(),
            note.private_info_endpoint()
        );
        assert_eq!(
            private_info
                .request
                .query
                .get("note_id")
                .map(String::as_str),
            Some("83577722856540160")
        );

        assert_eq!(public_info.name, "note.public_info");
        assert_eq!(public_info.request.method, HttpMethod::Get);
        assert_eq!(
            public_info.request.url.as_str(),
            note.public_info_endpoint()
        );
        assert_eq!(
            public_info.request.query.get("cvid").map(String::as_str),
            Some("15160286")
        );

        assert_eq!(archive_list.name, "note.archive_list");
        assert_eq!(archive_list.request.method, HttpMethod::Get);
        assert_eq!(
            archive_list.request.url.as_str(),
            note.archive_list_endpoint()
        );
        assert_eq!(
            archive_list.request.query.get("oid").map(String::as_str),
            Some("676931260")
        );

        assert_eq!(user_private_list.name, "note.user_private_list");
        assert_eq!(user_private_list.request.method, HttpMethod::Get);
        assert_eq!(
            user_private_list.request.url.as_str(),
            note.user_private_list_endpoint()
        );
        assert_eq!(
            user_private_list
                .request
                .query
                .get("pn")
                .map(String::as_str),
            Some("1")
        );

        assert_eq!(public_archive_list.name, "note.public_archive_list");
        assert_eq!(public_archive_list.request.method, HttpMethod::Get);
        assert_eq!(
            public_archive_list.request.url.as_str(),
            note.public_archive_list_endpoint()
        );
        assert_eq!(
            public_archive_list
                .request
                .query
                .get("oid")
                .map(String::as_str),
            Some("338677252")
        );

        assert_eq!(user_public_list.name, "note.user_public_list");
        assert_eq!(user_public_list.request.method, HttpMethod::Get);
        assert_eq!(
            user_public_list.request.url.as_str(),
            note.user_public_list_endpoint()
        );
        assert_eq!(
            user_public_list.request.query.get("ps").map(String::as_str),
            Some("10")
        );
        Ok(())
    }
}
