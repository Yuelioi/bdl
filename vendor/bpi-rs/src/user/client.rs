use crate::{BilibiliRequest, BpiClient, BpiResult};

use super::model::{
    UserAlbumCount, UserBangumiFollowList, UserBatchCard, UserBatchInfo, UserCardProfile,
    UserFollowTag, UserFollowers, UserFollowings, UserMedalWall, UserNameToUid, UserNavStat,
    UserRelationStat, UserSpaceNotice, UserSpaceProfile, UserUpStat, UserUploadedVideos,
};
use super::params::{
    UserAlbumCountParams, UserBangumiFollowListParams, UserCardParams, UserCardsParams,
    UserFollowersParams, UserFollowingsParams, UserInfosParams, UserMedalWallParams,
    UserNameToUidParams, UserNavStatParams, UserRelationStatParams, UserSpaceNoticeParams,
    UserSpaceParams, UserUpStatParams, UserUploadedVideosParams,
};

const ALBUM_COUNT_ENDPOINT: &str = "https://api.vc.bilibili.com/link_draw/v1/doc/upload_count";
const BANGUMI_FOLLOW_LIST_ENDPOINT: &str = "https://api.bilibili.com/x/space/bangumi/follow/list";
const CARD_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/card";
const CARDS_ENDPOINT: &str = "https://api.vc.bilibili.com/account/v1/user/cards";
const FOLLOWERS_ENDPOINT: &str = "https://api.bilibili.com/x/relation/fans";
const FOLLOWINGS_ENDPOINT: &str = "https://api.bilibili.com/x/relation/followings";
const FOLLOW_TAGS_ENDPOINT: &str = "https://api.bilibili.com/x/relation/tags";
const INFOS_ENDPOINT: &str = "https://api.vc.bilibili.com/x/im/user_infos";
const MEDAL_WALL_ENDPOINT: &str = "https://api.live.bilibili.com/xlive/web-ucenter/user/MedalWall";
const NAME_TO_UID_ENDPOINT: &str = "https://api.bilibili.com/x/polymer/web-dynamic/v1/name-to-uid";
const NAV_STAT_ENDPOINT: &str = "https://api.bilibili.com/x/space/navnum";
const RELATION_STAT_ENDPOINT: &str = "https://api.bilibili.com/x/relation/stat";
const SPACE_INFO_ENDPOINT: &str = "https://api.bilibili.com/x/space/wbi/acc/info";
const SPACE_NOTICE_ENDPOINT: &str = "https://api.bilibili.com/x/space/notice";
const UP_STAT_ENDPOINT: &str = "https://api.bilibili.com/x/space/upstat";
const UPLOADED_VIDEOS_ENDPOINT: &str = "https://api.bilibili.com/x/space/wbi/arc/search";

/// 用户领域 API 客户端。
#[derive(Clone, Copy)]
pub struct UserClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> UserClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn card_endpoint(&self) -> &'static str {
        CARD_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn cards_endpoint(&self) -> &'static str {
        CARDS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn infos_endpoint(&self) -> &'static str {
        INFOS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn bangumi_follow_list_endpoint(&self) -> &'static str {
        BANGUMI_FOLLOW_LIST_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn followings_endpoint(&self) -> &'static str {
        FOLLOWINGS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn followers_endpoint(&self) -> &'static str {
        FOLLOWERS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn follow_tags_endpoint(&self) -> &'static str {
        FOLLOW_TAGS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn medal_wall_endpoint(&self) -> &'static str {
        MEDAL_WALL_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn album_count_endpoint(&self) -> &'static str {
        ALBUM_COUNT_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn name_to_uid_endpoint(&self) -> &'static str {
        NAME_TO_UID_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn relation_stat_endpoint(&self) -> &'static str {
        RELATION_STAT_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn nav_stat_endpoint(&self) -> &'static str {
        NAV_STAT_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn space_info_endpoint(&self) -> &'static str {
        SPACE_INFO_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn space_notice_endpoint(&self) -> &'static str {
        SPACE_NOTICE_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn uploaded_videos_endpoint(&self) -> &'static str {
        UPLOADED_VIDEOS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn up_stat_endpoint(&self) -> &'static str {
        UP_STAT_ENDPOINT
    }

    /// 获取公开用户名片信息。
    pub async fn card(&self, params: UserCardParams) -> BpiResult<UserCardProfile> {
        self.client
            .get(CARD_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("user.card")
            .await
    }

    /// 获取一个或多个用户的精简公开名片信息。
    pub async fn cards(&self, params: UserCardsParams) -> BpiResult<Vec<UserBatchCard>> {
        self.client
            .get(CARDS_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("user.cards")
            .await
    }

    /// 获取一个或多个用户的详细公开批量信息。
    pub async fn infos(&self, params: UserInfosParams) -> BpiResult<Vec<UserBatchInfo>> {
        self.client
            .get(INFOS_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("user.infos")
            .await
    }

    /// 获取用户公开相簿投稿计数。
    pub async fn album_count(&self, params: UserAlbumCountParams) -> BpiResult<UserAlbumCount> {
        self.client
            .get(ALBUM_COUNT_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("user.album_count")
            .await
    }

    /// 获取公开用户追番或追剧的 season。
    pub async fn bangumi_follow_list(
        &self,
        params: UserBangumiFollowListParams,
    ) -> BpiResult<UserBangumiFollowList> {
        self.client
            .get(BANGUMI_FOLLOW_LIST_ENDPOINT)
            .with_bilibili_headers()
            .query(&params.query_pairs())
            .send_bpi_payload("user.bangumi_follow_list")
            .await
    }

    /// 获取公开会员关注的用户。
    pub async fn followings(&self, params: UserFollowingsParams) -> BpiResult<UserFollowings> {
        self.client
            .get(FOLLOWINGS_ENDPOINT)
            .with_bilibili_headers()
            .query(&params.query_pairs())
            .send_bpi_payload("user.followings")
            .await
    }

    /// 获取关注公开会员的用户。
    pub async fn followers(&self, params: UserFollowersParams) -> BpiResult<UserFollowers> {
        self.client
            .get(FOLLOWERS_ENDPOINT)
            .with_bilibili_headers()
            .query(&params.query_pairs())
            .send_bpi_payload("user.followers")
            .await
    }

    /// 获取当前已认证会话的关注分组。
    pub async fn follow_tags(&self) -> BpiResult<Vec<UserFollowTag>> {
        self.client
            .get(FOLLOW_TAGS_ENDPOINT)
            .with_bilibili_headers()
            .send_bpi_payload("user.follow_tags")
            .await
    }

    /// 获取用户的公开粉丝勋章墙。
    pub async fn medal_wall(&self, params: UserMedalWallParams) -> BpiResult<UserMedalWall> {
        self.client
            .get(MEDAL_WALL_ENDPOINT)
            .with_bilibili_headers()
            .query(&params.query_pairs())
            .send_bpi_payload("user.medal_wall")
            .await
    }

    /// 按公开显示名称查找会员 ID。
    pub async fn name_to_uid(&self, params: UserNameToUidParams) -> BpiResult<UserNameToUid> {
        self.client
            .get(NAME_TO_UID_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("user.name_to_uid")
            .await
    }

    /// 获取用户公开关系计数。
    pub async fn relation_stat(
        &self,
        params: UserRelationStatParams,
    ) -> BpiResult<UserRelationStat> {
        self.client
            .get(RELATION_STAT_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("user.relation_stat")
            .await
    }

    /// 获取用户公开空间导航计数。
    pub async fn nav_stat(&self, params: UserNavStatParams) -> BpiResult<UserNavStat> {
        self.client
            .get(NAV_STAT_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("user.nav_stat")
            .await
    }

    /// 获取用户公开创作者统计。
    pub async fn up_stat(&self, params: UserUpStatParams) -> BpiResult<UserUpStat> {
        self.client
            .get(UP_STAT_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("user.up_stat")
            .await
    }

    /// 获取公开用户空间信息。
    pub async fn space_info(&self, params: UserSpaceParams) -> BpiResult<UserSpaceProfile> {
        let signed_params = self.client.sign_wbi_params(params.query_pairs()).await?;

        self.client
            .get(SPACE_INFO_ENDPOINT)
            .query(&signed_params)
            .send_bpi_payload("user.space_info")
            .await
    }

    /// 获取用户公开空间公告。
    pub async fn space_notice(&self, params: UserSpaceNoticeParams) -> BpiResult<UserSpaceNotice> {
        self.client
            .get(SPACE_NOTICE_ENDPOINT)
            .query(&params.query_pairs())
            .send_bpi_payload("user.space_notice")
            .await
    }

    /// 获取用户公开空间中上传的视频。
    pub async fn uploaded_videos(
        &self,
        params: UserUploadedVideosParams,
    ) -> BpiResult<UserUploadedVideos> {
        let signed_params = self.client.sign_wbi_params(params.query_pairs()).await?;

        self.client
            .get(UPLOADED_VIDEOS_ENDPOINT)
            .query(&signed_params)
            .send_bpi_payload("user.uploaded_videos")
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ApiEnvelope, BpiClient, BpiError, BpiResult,
        probe::{contract::HttpMethod, endpoint_contract::EndpointContract},
    };
    use serde::de::DeserializeOwned;

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes: &[u8] = match endpoint {
            "album-count" => {
                include_bytes!("../../tests/contracts/user/public-read/album-count/contract.json")
            }
            "bangumi-follow-list" => include_bytes!(
                "../../tests/contracts/user/public-read/bangumi-follow-list/contract.json"
            ),
            "card" => include_bytes!("../../tests/contracts/user/public-read/card/contract.json"),
            "cards" => {
                include_bytes!("../../tests/contracts/user/public-read/cards/contract.json")
            }
            "follow-tags" => {
                include_bytes!("../../tests/contracts/user/relation-read/follow-tags/contract.json")
            }
            "followers" => {
                include_bytes!("../../tests/contracts/user/relation-read/followers/contract.json")
            }
            "followings" => {
                include_bytes!("../../tests/contracts/user/relation-read/followings/contract.json")
            }
            "infos" => {
                include_bytes!("../../tests/contracts/user/public-read/infos/contract.json")
            }
            "medal-wall" => {
                include_bytes!("../../tests/contracts/user/public-read/medal-wall/contract.json")
            }
            "name-to-uid" => {
                include_bytes!("../../tests/contracts/user/public-read/name-to-uid/contract.json")
            }
            "nav-stat" => {
                include_bytes!("../../tests/contracts/user/public-read/nav-stat/contract.json")
            }
            "relation-stat" => {
                include_bytes!("../../tests/contracts/user/public-read/relation-stat/contract.json")
            }
            "space-info" => {
                include_bytes!("../../tests/contracts/user/public-read/space-info/contract.json")
            }
            "space-notice" => {
                include_bytes!("../../tests/contracts/user/public-read/space-notice/contract.json")
            }
            "up-stat" => {
                include_bytes!("../../tests/contracts/user/public-read/up-stat/contract.json")
            }
            "uploaded-videos" => include_bytes!(
                "../../tests/contracts/user/public-read/uploaded-videos/contract.json"
            ),
            _ => {
                return Err(BpiError::invalid_parameter(
                    "endpoint",
                    "unknown user contract",
                ));
            }
        };

        EndpointContract::from_slice(bytes)
    }

    fn local_probe_body(batch: &str, endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/user/{batch}/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    fn parse_local_probe_outputs<T>(batch: &str, endpoint: &str, profiles: &[&str]) -> BpiResult<()>
    where
        T: DeserializeOwned,
    {
        for profile in profiles {
            let Some(body) = local_probe_body(batch, endpoint, profile) else {
                continue;
            };

            let _payload = serde_json::from_value::<ApiEnvelope<T>>(body)?.into_payload()?;
        }

        Ok(())
    }

    #[test]
    fn user_client_borrows_root_client() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let user = client.user();

        assert_eq!(
            user.card_endpoint(),
            "https://api.bilibili.com/x/web-interface/card"
        );
        Ok(())
    }

    #[test]
    fn user_client_exposes_cards_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let user = client.user();

        assert_eq!(
            user.cards_endpoint(),
            "https://api.vc.bilibili.com/account/v1/user/cards"
        );
        Ok(())
    }

    #[test]
    fn user_client_exposes_infos_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let user = client.user();

        assert_eq!(
            user.infos_endpoint(),
            "https://api.vc.bilibili.com/x/im/user_infos"
        );
        Ok(())
    }

    #[test]
    fn user_client_exposes_space_info_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let user = client.user();

        assert_eq!(
            user.space_info_endpoint(),
            "https://api.bilibili.com/x/space/wbi/acc/info"
        );
        Ok(())
    }

    #[test]
    fn user_client_exposes_space_notice_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let user = client.user();

        assert_eq!(
            user.space_notice_endpoint(),
            "https://api.bilibili.com/x/space/notice"
        );
        Ok(())
    }

    #[test]
    fn user_client_exposes_bangumi_follow_list_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let user = client.user();

        assert_eq!(
            user.bangumi_follow_list_endpoint(),
            "https://api.bilibili.com/x/space/bangumi/follow/list"
        );
        Ok(())
    }

    #[test]
    fn user_client_exposes_uploaded_videos_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let user = client.user();

        assert_eq!(
            user.uploaded_videos_endpoint(),
            "https://api.bilibili.com/x/space/wbi/arc/search"
        );
        Ok(())
    }

    #[test]
    fn user_client_exposes_relation_stat_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let user = client.user();

        assert_eq!(
            user.relation_stat_endpoint(),
            "https://api.bilibili.com/x/relation/stat"
        );
        Ok(())
    }

    #[test]
    fn user_client_exposes_followings_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let user = client.user();

        assert_eq!(
            user.followings_endpoint(),
            "https://api.bilibili.com/x/relation/followings"
        );
        Ok(())
    }

    #[test]
    fn user_client_exposes_followers_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let user = client.user();

        assert_eq!(
            user.followers_endpoint(),
            "https://api.bilibili.com/x/relation/fans"
        );
        Ok(())
    }

    #[test]
    fn user_client_exposes_follow_tags_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let user = client.user();

        assert_eq!(
            user.follow_tags_endpoint(),
            "https://api.bilibili.com/x/relation/tags"
        );
        Ok(())
    }

    #[test]
    fn user_client_exposes_medal_wall_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let user = client.user();

        assert_eq!(
            user.medal_wall_endpoint(),
            "https://api.live.bilibili.com/xlive/web-ucenter/user/MedalWall"
        );
        Ok(())
    }

    #[test]
    fn user_client_exposes_up_stat_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let user = client.user();

        assert_eq!(
            user.up_stat_endpoint(),
            "https://api.bilibili.com/x/space/upstat"
        );
        Ok(())
    }

    #[test]
    fn user_client_exposes_nav_stat_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let user = client.user();

        assert_eq!(
            user.nav_stat_endpoint(),
            "https://api.bilibili.com/x/space/navnum"
        );
        Ok(())
    }

    #[test]
    fn user_client_exposes_album_count_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let user = client.user();

        assert_eq!(
            user.album_count_endpoint(),
            "https://api.vc.bilibili.com/link_draw/v1/doc/upload_count"
        );
        Ok(())
    }

    #[test]
    fn user_client_exposes_name_to_uid_endpoint() -> Result<(), crate::BpiError> {
        let client = BpiClient::new()?;
        let user = client.user();

        assert_eq!(
            user.name_to_uid_endpoint(),
            "https://api.bilibili.com/x/polymer/web-dynamic/v1/name-to-uid"
        );
        Ok(())
    }

    #[test]
    fn user_public_read_contracts_match_endpoint_requests() -> BpiResult<()> {
        let expectations = [
            (
                "album-count",
                "user.album_count",
                ALBUM_COUNT_ENDPOINT,
                "UserAlbumCount",
                false,
            ),
            (
                "bangumi-follow-list",
                "user.bangumi_follow_list",
                BANGUMI_FOLLOW_LIST_ENDPOINT,
                "UserBangumiFollowList",
                false,
            ),
            ("card", "user.card", CARD_ENDPOINT, "UserCardProfile", false),
            (
                "cards",
                "user.cards",
                CARDS_ENDPOINT,
                "Vec<UserBatchCard>",
                false,
            ),
            (
                "infos",
                "user.infos",
                INFOS_ENDPOINT,
                "Vec<UserBatchInfo>",
                false,
            ),
            (
                "medal-wall",
                "user.medal_wall",
                MEDAL_WALL_ENDPOINT,
                "UserMedalWall",
                false,
            ),
            (
                "name-to-uid",
                "user.name_to_uid",
                NAME_TO_UID_ENDPOINT,
                "UserNameToUid",
                false,
            ),
            (
                "nav-stat",
                "user.nav_stat",
                NAV_STAT_ENDPOINT,
                "UserNavStat",
                false,
            ),
            (
                "relation-stat",
                "user.relation_stat",
                RELATION_STAT_ENDPOINT,
                "UserRelationStat",
                false,
            ),
            (
                "space-info",
                "user.space_info",
                SPACE_INFO_ENDPOINT,
                "UserSpaceProfile",
                true,
            ),
            (
                "space-notice",
                "user.space_notice",
                SPACE_NOTICE_ENDPOINT,
                "UserSpaceNotice",
                false,
            ),
            (
                "up-stat",
                "user.up_stat",
                UP_STAT_ENDPOINT,
                "UserUpStat",
                false,
            ),
            (
                "uploaded-videos",
                "user.uploaded_videos",
                UPLOADED_VIDEOS_ENDPOINT,
                "UserUploadedVideos",
                true,
            ),
        ];

        for (endpoint, name, url, rust_model, requires_wbi) in expectations {
            let contract = contract(endpoint)?;

            assert_eq!(contract.name, name);
            assert_eq!(contract.request.method, HttpMethod::Get);
            assert_eq!(contract.request.url.as_str(), url);
            assert_eq!(contract.request.auth.requires_wbi(), requires_wbi);
            assert_eq!(contract.cases.len(), 3);
            assert!(
                contract
                    .cases
                    .iter()
                    .any(|case| case.response.rust_model.as_deref() == Some(rust_model)),
                "{endpoint} should declare {rust_model} for at least one success case"
            );
        }

        Ok(())
    }

    #[test]
    fn user_relation_read_contracts_match_endpoint_requests() -> BpiResult<()> {
        let expectations = [
            (
                "followings",
                "user.followings",
                FOLLOWINGS_ENDPOINT,
                &[
                    ("order_type", "attention"),
                    ("pn", "1"),
                    ("ps", "20"),
                    ("vmid", "2"),
                ][..],
                "UserFollowings",
            ),
            (
                "followers",
                "user.followers",
                FOLLOWERS_ENDPOINT,
                &[("pn", "1"), ("ps", "20"), ("vmid", "2")][..],
                "UserFollowers",
            ),
            (
                "follow-tags",
                "user.follow_tags",
                FOLLOW_TAGS_ENDPOINT,
                &[][..],
                "Vec<UserFollowTag>",
            ),
        ];

        for (endpoint, name, url, query_pairs, rust_model) in expectations {
            let contract = contract(endpoint)?;

            assert_eq!(contract.name, name);
            assert_eq!(contract.request.method, HttpMethod::Get);
            assert_eq!(contract.request.url.as_str(), url);
            assert!(!contract.request.auth.requires_wbi());
            assert_eq!(contract.cases.len(), 3);
            assert!(
                contract
                    .cases
                    .iter()
                    .any(|case| case.response.rust_model.as_deref() == Some(rust_model)),
                "{endpoint} should declare {rust_model} for at least one success case"
            );

            for &(key, value) in query_pairs {
                if !key.is_empty() {
                    assert_eq!(
                        contract.request.query.get(key).map(String::as_str),
                        Some(value)
                    );
                }
            }

            let anonymous = contract
                .cases
                .iter()
                .find(|case| case.name == "anonymous")
                .ok_or_else(|| BpiError::unsupported_response("missing anonymous case"))?;
            assert_eq!(anonymous.response.http_status, Some(200));
            assert!(anonymous.response.api_code.is_some());
        }

        Ok(())
    }

    #[test]
    fn user_public_read_response_fixtures_parse_declared_models() -> BpiResult<()> {
        let album_count = ApiEnvelope::<UserAlbumCount>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/album-count/responses/success.json"
        ))?
        .into_payload()?;
        let bangumi_follow_list =
            ApiEnvelope::<UserBangumiFollowList>::from_slice(include_bytes!(
                "../../tests/contracts/user/public-read/bangumi-follow-list/responses/success.json"
            ))?
            .into_payload()?;
        let card = ApiEnvelope::<UserCardProfile>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/card/responses/success.json"
        ))?
        .into_payload()?;
        let cards = ApiEnvelope::<Vec<UserBatchCard>>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/cards/responses/success.json"
        ))?
        .into_payload()?;
        let infos = ApiEnvelope::<Vec<UserBatchInfo>>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/infos/responses/success.json"
        ))?
        .into_payload()?;
        let medal_wall = ApiEnvelope::<UserMedalWall>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/medal-wall/responses/success.json"
        ))?
        .into_payload()?;
        let name_to_uid = ApiEnvelope::<UserNameToUid>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/name-to-uid/responses/success.json"
        ))?
        .into_payload()?;
        let nav_stat = ApiEnvelope::<UserNavStat>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/nav-stat/responses/success.json"
        ))?
        .into_payload()?;
        let relation_stat = ApiEnvelope::<UserRelationStat>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/relation-stat/responses/success.json"
        ))?
        .into_payload()?;
        let space_info = ApiEnvelope::<UserSpaceProfile>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/space-info/responses/success.json"
        ))?
        .into_payload()?;
        let space_notice = ApiEnvelope::<UserSpaceNotice>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/space-notice/responses/success.json"
        ))?
        .into_payload()?;
        let up_stat = ApiEnvelope::<UserUpStat>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/up-stat/responses/success.json"
        ))?
        .into_payload()?;
        let uploaded_videos = ApiEnvelope::<UserUploadedVideos>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/uploaded-videos/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(album_count.all_count, 0);
        assert!(bangumi_follow_list.items.is_empty());
        assert_eq!(card.card.mid.get(), 2);
        assert_eq!(cards.len(), 1);
        assert_eq!(infos.len(), 1);
        assert_eq!(medal_wall.uid.get(), 2);
        assert_eq!(name_to_uid.uid_list.len(), 1);
        assert_eq!(nav_stat.channel.master, 0);
        assert_eq!(relation_stat.mid.get(), 2);
        assert_eq!(space_info.mid.get(), 2);
        assert_eq!(space_notice.content, "sanitized notice");
        assert_eq!(up_stat.likes, 1);
        assert!(uploaded_videos.list.videos.is_empty());
        Ok(())
    }

    #[test]
    fn user_relation_read_response_fixtures_parse_declared_models() -> BpiResult<()> {
        let followings = ApiEnvelope::<UserFollowings>::from_slice(include_bytes!(
            "../../tests/contracts/user/relation-read/followings/responses/success.json"
        ))?
        .into_payload()?;
        let followers = ApiEnvelope::<UserFollowers>::from_slice(include_bytes!(
            "../../tests/contracts/user/relation-read/followers/responses/success.json"
        ))?
        .into_payload()?;
        let follow_tags = ApiEnvelope::<Vec<UserFollowTag>>::from_slice(include_bytes!(
            "../../tests/contracts/user/relation-read/follow-tags/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(followings.list.len(), 1);
        assert_eq!(followers.list.len(), 1);
        assert_eq!(follow_tags.len(), 2);
        Ok(())
    }

    #[test]
    fn user_public_read_error_fixtures_preserve_observed_api_errors() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/user/public-read/cards/responses/anonymous.error.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/user/public-read/infos/responses/anonymous.error.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/user/public-read/medal-wall/responses/anonymous.error.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/user/public-read/name-to-uid/responses/anonymous.error.json"
            )
            .as_slice(),
        ] {
            let err = ApiEnvelope::<serde_json::Value>::from_slice(bytes)?
                .ensure_success()
                .unwrap_err();

            assert!(err.requires_login());
        }

        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/space-info/responses/anonymous.error.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert_eq!(err.code(), Some(-352));

        let anonymous_up_stat = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/up-stat/responses/anonymous.empty.json"
        ))?
        .into_payload()?;
        assert_eq!(anonymous_up_stat, serde_json::json!({}));

        Ok(())
    }

    #[test]
    fn user_relation_read_error_fixtures_preserve_observed_api_errors() -> BpiResult<()> {
        for bytes in [
            include_bytes!(
                "../../tests/contracts/user/relation-read/followings/responses/anonymous.error.json"
            )
            .as_slice(),
            include_bytes!(
                "../../tests/contracts/user/relation-read/follow-tags/responses/anonymous.error.json"
            )
            .as_slice(),
        ] {
            let err = ApiEnvelope::<serde_json::Value>::from_slice(bytes)?
                .ensure_success()
                .unwrap_err();

            assert!(err.requires_login());
        }

        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/user/relation-read/followers/responses/anonymous.error.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert_eq!(err.code(), Some(-352));

        Ok(())
    }

    #[test]
    fn user_public_read_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        parse_local_probe_outputs::<UserAlbumCount>(
            "public-read",
            "album-count",
            &["anonymous", "normal", "vip"],
        )?;
        parse_local_probe_outputs::<UserBangumiFollowList>(
            "public-read",
            "bangumi-follow-list",
            &["anonymous", "normal", "vip"],
        )?;
        parse_local_probe_outputs::<UserCardProfile>(
            "public-read",
            "card",
            &["anonymous", "normal", "vip"],
        )?;
        parse_local_probe_outputs::<Vec<UserBatchCard>>(
            "public-read",
            "cards",
            &["normal", "vip"],
        )?;
        parse_local_probe_outputs::<Vec<UserBatchInfo>>(
            "public-read",
            "infos",
            &["normal", "vip"],
        )?;
        parse_local_probe_outputs::<UserMedalWall>(
            "public-read",
            "medal-wall",
            &["normal", "vip"],
        )?;
        parse_local_probe_outputs::<UserNameToUid>(
            "public-read",
            "name-to-uid",
            &["normal", "vip"],
        )?;
        parse_local_probe_outputs::<UserNavStat>(
            "public-read",
            "nav-stat",
            &["anonymous", "normal", "vip"],
        )?;
        parse_local_probe_outputs::<UserRelationStat>(
            "public-read",
            "relation-stat",
            &["anonymous", "normal", "vip"],
        )?;
        parse_local_probe_outputs::<UserSpaceProfile>(
            "public-read",
            "space-info",
            &["normal", "vip"],
        )?;
        parse_local_probe_outputs::<UserSpaceNotice>(
            "public-read",
            "space-notice",
            &["anonymous", "normal", "vip"],
        )?;
        parse_local_probe_outputs::<UserUpStat>("public-read", "up-stat", &["normal", "vip"])?;
        parse_local_probe_outputs::<UserUploadedVideos>(
            "public-read",
            "uploaded-videos",
            &["anonymous", "normal", "vip"],
        )?;

        if let Some(body) = local_probe_body("public-read", "up-stat", "anonymous") {
            let payload =
                serde_json::from_value::<ApiEnvelope<serde_json::Value>>(body)?.into_payload()?;
            assert_eq!(payload, serde_json::json!({}));
        }

        Ok(())
    }

    #[test]
    fn user_relation_read_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        parse_local_probe_outputs::<UserFollowings>(
            "relation-read",
            "followings",
            &["normal", "vip"],
        )?;
        parse_local_probe_outputs::<UserFollowers>(
            "relation-read",
            "followers",
            &["normal", "vip"],
        )?;
        parse_local_probe_outputs::<Vec<UserFollowTag>>(
            "relation-read",
            "follow-tags",
            &["normal", "vip"],
        )?;

        for (endpoint, profile, code) in [
            ("followings", "anonymous", -101),
            ("followers", "anonymous", -352),
            ("follow-tags", "anonymous", -101),
        ] {
            let Some(body) = local_probe_body("relation-read", endpoint, profile) else {
                continue;
            };
            let err = serde_json::from_value::<ApiEnvelope<serde_json::Value>>(body)?
                .ensure_success()
                .unwrap_err();

            assert_eq!(err.code(), Some(code));
        }

        Ok(())
    }

    #[test]
    fn user_client_methods_use_payload_request_helpers() {
        let source = include_str!("client.rs");
        let payload_helper = concat!(".send_", "bpi_payload");
        let legacy_envelope_helper = concat!(".send_", "bpi::<");

        assert!(source.matches(payload_helper).count() >= 16);
        assert!(!source.contains(legacy_envelope_helper));
    }
}
