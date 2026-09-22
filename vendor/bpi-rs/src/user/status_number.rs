//! B站用户关系、UP主状态、导航栏等相关接口
//!
//! [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/user)
use serde::{Deserialize, Serialize};

// --- 响应数据结构体 ---

/// 用户关系状态数响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RelationStatResponseData {
    /// 目标用户 mid
    pub mid: u64,
    /// 关注数
    pub following: u64,
    /// 悄悄关注数
    pub whisper: u64,
    /// 黑名单数
    pub black: u64,
    /// 粉丝数
    pub follower: u64,
}

/// UP主状态数中的视频播放量
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstatArchive {
    /// 视频播放量
    pub view: u64,
}

/// UP主状态数中的专栏阅读量
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstatArticle {
    /// 专栏阅读量
    pub view: u64,
}

/// UP主状态数响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpstatResponseData {
    /// 视频播放量
    pub archive: UpstatArchive,
    /// 专栏阅读量
    pub article: UpstatArticle,
    /// 获赞次数
    pub likes: u64,
}

/// 用户导航栏状态数中的视频列表数
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NavnumChannel {
    /// 视频列表数
    pub master: u64,
    /// 视频列表数
    pub guest: u64,
}

/// 用户导航栏状态数中的收藏夹数
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NavnumFavourite {
    /// 全部收藏夹数
    pub master: u64,
    /// 公开收藏夹数
    pub guest: u64,
}

/// 用户导航栏状态数响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NavnumResponseData {
    /// 投稿视频数
    pub video: u64,
    /// 追番数
    pub bangumi: u64,
    /// 追剧数
    pub cinema: u64,
    /// 视频列表数
    pub channel: NavnumChannel,
    /// 收藏夹数
    pub favourite: NavnumFavourite,
    /// 关注 TAG 数
    pub tag: u64,
    /// 投稿专栏数
    pub article: u64,
    pub playlist: u64,
    /// 投稿图文数
    pub album: u64,
    /// 投稿音频数
    pub audio: u64,
    /// 投稿课程数
    pub pugv: u64,
    /// 动态数
    pub opus: u64,
    /// 视频合集数
    #[serde(rename = "season_num")]
    pub season_num: u64,
}

/// 相簿投稿数响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AlbumCountResponseData {
    /// 相簿总数
    pub all_count: u64,
    /// 发布绘画数
    pub draw_count: u64,
    /// 发布摄影数
    pub photo_count: u64,
    /// 发布日常（图片动态）数
    pub daily_count: u64,
}

// --- 测试模块 ---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::Mid;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::user::params::{
        UserAlbumCountParams, UserNavStatParams, UserRelationStatParams, UserUpStatParams,
    };
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};
    use tracing::info;

    // 请在运行测试前设置环境变量 `BPI_COOKIE`，以包含 SESSDATA 等登录信息
    // mid 根据实际情况修改
    const TEST_MID: u64 = 332704117;
    const TEST_UP_MID: u64 = 456664753;
    const TEST_NAV_MID: u64 = 645769214;

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_relation_stat() -> Result<(), BpiError> {
        if std::env::var_os("BPI_LIVE_TEST").is_none() {
            return Ok(());
        }

        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .user()
            .relation_stat(UserRelationStatParams::new(Mid::new(TEST_MID)?))
            .await?;

        info!("关系状态数: {:?}", data);
        assert_eq!(data.mid.get(), TEST_MID);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_up_stat() -> Result<(), BpiError> {
        if std::env::var_os("BPI_LIVE_TEST").is_none() {
            return Ok(());
        }

        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .user()
            .up_stat(UserUpStatParams::new(Mid::new(TEST_UP_MID)?))
            .await?;

        info!("UP主状态数: {:?}", data);
        assert!(data.likes > 0);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_nav_num() -> Result<(), BpiError> {
        if std::env::var_os("BPI_LIVE_TEST").is_none() {
            return Ok(());
        }

        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .user()
            .nav_stat(UserNavStatParams::new(Mid::new(TEST_NAV_MID)?))
            .await?;

        info!("用户导航栏状态数: {:?}", data);
        assert!(data.video > 0);

        Ok(())
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_album_count() -> Result<(), BpiError> {
        if std::env::var_os("BPI_LIVE_TEST").is_none() {
            return Ok(());
        }

        let bpi = BpiClient::new().expect("client should build");
        let data = bpi
            .user()
            .album_count(UserAlbumCountParams::new(Mid::new(TEST_NAV_MID)?))
            .await?;

        info!("相簿投稿数: {:?}", data);
        assert!(data.all_count > 0);

        Ok(())
    }

    fn public_read_contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes: &[u8] = match endpoint {
            "album-count" => {
                include_bytes!("../../tests/contracts/user/public-read/album-count/contract.json")
            }
            "nav-stat" => {
                include_bytes!("../../tests/contracts/user/public-read/nav-stat/contract.json")
            }
            "relation-stat" => {
                include_bytes!("../../tests/contracts/user/public-read/relation-stat/contract.json")
            }
            "up-stat" => {
                include_bytes!("../../tests/contracts/user/public-read/up-stat/contract.json")
            }
            _ => {
                return Err(BpiError::invalid_parameter(
                    "endpoint",
                    "unknown user status contract",
                ));
            }
        };

        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn legacy_user_status_contracts_match_endpoint_requests() -> BpiResult<()> {
        let relation = public_read_contract("relation-stat")?;
        assert_eq!(relation.name, "user.relation_stat");
        assert_eq!(relation.request.method, HttpMethod::Get);
        assert_eq!(
            relation.request.url.as_str(),
            "https://api.bilibili.com/x/relation/stat"
        );
        assert_eq!(
            relation.request.query.get("vmid").map(String::as_str),
            Some("2")
        );

        let up_stat = public_read_contract("up-stat")?;
        assert_eq!(up_stat.name, "user.up_stat");
        assert_eq!(up_stat.request.method, HttpMethod::Get);
        assert_eq!(
            up_stat.request.url.as_str(),
            "https://api.bilibili.com/x/space/upstat"
        );
        assert_eq!(
            up_stat.request.query.get("mid").map(String::as_str),
            Some("456664753")
        );

        let nav = public_read_contract("nav-stat")?;
        assert_eq!(nav.name, "user.nav_stat");
        assert_eq!(
            nav.request.url.as_str(),
            "https://api.bilibili.com/x/space/navnum"
        );
        assert_eq!(nav.request.query.get("mid").map(String::as_str), Some("2"));

        let album = public_read_contract("album-count")?;
        assert_eq!(album.name, "user.album_count");
        assert_eq!(
            album.request.url.as_str(),
            "https://api.vc.bilibili.com/link_draw/v1/doc/upload_count"
        );
        assert_eq!(
            album.request.query.get("uid").map(String::as_str),
            Some("2")
        );
        Ok(())
    }

    #[test]
    fn legacy_user_status_fixtures_parse_promoted_contract_models() -> BpiResult<()> {
        let relation = ApiEnvelope::<RelationStatResponseData>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/relation-stat/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(relation.mid, 2);

        let up_stat = ApiEnvelope::<UpstatResponseData>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/up-stat/responses/success.json"
        ))?
        .into_payload()?;
        assert!(up_stat.archive.view >= up_stat.article.view);

        let nav = ApiEnvelope::<NavnumResponseData>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/nav-stat/responses/success.json"
        ))?
        .into_payload()?;
        let _total_content = nav.video + nav.article + nav.album + nav.audio + nav.opus;

        let album = ApiEnvelope::<AlbumCountResponseData>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/album-count/responses/success.json"
        ))?
        .into_payload()?;
        assert_eq!(
            album.all_count,
            album.draw_count + album.photo_count + album.daily_count
        );
        Ok(())
    }
}
