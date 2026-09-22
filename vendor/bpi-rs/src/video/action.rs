// B站视频交互接口(Web端)
//
// [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/video)

use crate::BilibiliRequest;
use crate::BpiError;
use crate::response::BpiResult;
use crate::video::VideoClient;
use serde::{Deserialize, Serialize};

const LIKE_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/archive/like";
const COIN_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/coin/add";
const COIN_STATUS_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/archive/coins";
const FAVORITE_ENDPOINT: &str = "https://api.bilibili.com/x/v3/fav/resource/deal";

/// 投币视频 - 响应结构体
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CoinData {
    /// 是否点赞成功
    pub like: bool,
}

/// 收藏视频 - 响应结构体
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct FavoriteData {
    /// 是否为未关注用户收藏
    pub prompt: bool,
    /// 作用不明确
    pub ga_data: Option<serde_json::Value>,
    /// 提示消息
    pub toast_msg: Option<String>,
    /// 成功数
    pub success_num: u32,
}

/// 当前账号对某个视频的投币状态。
#[derive(Debug, Serialize, Clone, Deserialize, PartialEq, Eq)]
pub struct VideoCoinStatusData {
    /// 当前账号已给此视频投出的硬币数量。
    pub multiply: u8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LegacyVideoIdForm {
    aid: String,
    bvid: String,
}

impl LegacyVideoIdForm {
    fn form_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        if self.aid != "0" {
            pairs.push(("aid", self.aid.clone()));
        }
        if !self.bvid.is_empty() {
            pairs.push(("bvid", self.bvid.clone()));
        }
        pairs
    }
}

/// 检查当前账号是否已给视频投币的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoCoinStatusParams {
    video_id: LegacyVideoIdForm,
}

impl VideoCoinStatusParams {
    pub fn from_aid(aid: u64) -> BpiResult<Self> {
        Self::from_ids(Some(aid), None)
    }

    pub fn from_bvid(bvid: impl Into<String>) -> BpiResult<Self> {
        Self::from_ids(None, Some(bvid.into()))
    }

    pub fn from_ids(aid: Option<u64>, bvid: Option<String>) -> BpiResult<Self> {
        Ok(Self {
            video_id: legacy_video_id_form(aid, bvid)?,
        })
    }

    fn query_pairs(&self) -> Vec<(&'static str, String)> {
        self.video_id.form_pairs()
    }
}

/// 视频点赞/取消点赞操作的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoLikeParams {
    video_id: LegacyVideoIdForm,
    like: u8,
}

impl VideoLikeParams {
    pub fn from_aid(aid: u64, like: u8) -> BpiResult<Self> {
        Self::from_ids(Some(aid), None, like)
    }

    pub fn from_bvid(bvid: impl Into<String>, like: u8) -> BpiResult<Self> {
        Self::from_ids(None, Some(bvid.into()), like)
    }

    pub fn from_ids(aid: Option<u64>, bvid: Option<String>, like: u8) -> BpiResult<Self> {
        let video_id = legacy_video_id_form(aid, bvid)?;
        validate_like_action(like)?;

        Ok(Self { video_id, like })
    }

    fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        let mut pairs = self.video_id.form_pairs();
        pairs.push(("like", self.like.to_string()));
        pairs.push(("csrf", csrf.to_string()));
        pairs
    }
}

/// 视频投币操作的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoCoinParams {
    video_id: LegacyVideoIdForm,
    multiply: u8,
    select_like: u8,
}

impl VideoCoinParams {
    pub fn from_aid(aid: u64, multiply: u8) -> BpiResult<Self> {
        Self::from_ids(Some(aid), None, multiply)
    }

    pub fn from_bvid(bvid: impl Into<String>, multiply: u8) -> BpiResult<Self> {
        Self::from_ids(None, Some(bvid.into()), multiply)
    }

    pub fn from_ids(aid: Option<u64>, bvid: Option<String>, multiply: u8) -> BpiResult<Self> {
        let video_id = legacy_video_id_form(aid, bvid)?;
        validate_coin_multiply(multiply)?;

        Ok(Self {
            video_id,
            multiply,
            select_like: 0,
        })
    }

    pub fn select_like(mut self, select_like: bool) -> Self {
        self.select_like = u8::from(select_like);
        self
    }

    fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        let mut pairs = self.video_id.form_pairs();
        pairs.push(("multiply", self.multiply.to_string()));
        pairs.push(("select_like", self.select_like.to_string()));
        pairs.push(("csrf", csrf.to_string()));
        pairs
    }
}

/// 视频添加/移除收藏操作的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoFavoriteParams {
    rid: u64,
    add_media_ids: Vec<String>,
    del_media_ids: Vec<String>,
}

impl VideoFavoriteParams {
    pub fn new(
        rid: u64,
        add_media_ids: impl IntoIterator<Item = impl Into<String>>,
        del_media_ids: impl IntoIterator<Item = impl Into<String>>,
    ) -> BpiResult<Self> {
        if rid == 0 {
            return Err(BpiError::invalid_parameter("rid", "id must be non-zero"));
        }

        let add_media_ids = normalize_id_list("add_media_ids", add_media_ids)?;
        let del_media_ids = normalize_id_list("del_media_ids", del_media_ids)?;

        if add_media_ids.is_empty() && del_media_ids.is_empty() {
            return Err(BpiError::invalid_parameter(
                "media_ids",
                "at least one add or delete media id is required",
            ));
        }

        Ok(Self {
            rid,
            add_media_ids,
            del_media_ids,
        })
    }

    fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        let mut pairs = vec![
            ("rid", self.rid.to_string()),
            ("type", "2".to_string()),
            ("csrf", csrf.to_string()),
        ];

        if !self.add_media_ids.is_empty() {
            pairs.push(("add_media_ids", self.add_media_ids.join(",")));
        }
        if !self.del_media_ids.is_empty() {
            pairs.push(("del_media_ids", self.del_media_ids.join(",")));
        }

        pairs
    }
}

impl<'a> VideoClient<'a> {
    /// 检查当前账号已给视频投出多少硬币。
    ///
    /// [`Self::coin`] 成功后，上游 endpoint 可能会短暂滞后。
    pub async fn coin_status(
        &self,
        params: VideoCoinStatusParams,
    ) -> BpiResult<VideoCoinStatusData> {
        self.client
            .get(COIN_STATUS_ENDPOINT)
            .with_bilibili_headers()
            .query(&params.query_pairs())
            .send_bpi_payload("video.coin_status")
            .await
    }

    /// 点赞或取消点赞视频，并返回可选的标准 payload 结果。
    pub async fn like(&self, params: VideoLikeParams) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        self.client
            .post(LIKE_ENDPOINT)
            .with_bilibili_headers()
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("video.like")
            .await
    }

    /// 给视频投币，并返回标准 payload 结果。
    ///
    /// Bilibili 的 Web endpoint 需要包含 `buvid3` 的正常登录 cookie 集合；
    /// 即使存在 `SESSDATA` 和 `bili_jct`，缺少它的账号也可能触发风控。
    pub async fn coin(&self, params: VideoCoinParams) -> BpiResult<CoinData> {
        let csrf = self.client.csrf()?;

        self.client
            .post(COIN_ENDPOINT)
            .with_bilibili_headers()
            .form(&params.form_pairs(&csrf))
            .send_bpi_payload("video.coin")
            .await
    }

    /// 将视频添加到收藏夹，或从收藏夹中移除。
    pub async fn favorite(&self, params: VideoFavoriteParams) -> BpiResult<FavoriteData> {
        let csrf = self.client.csrf()?;

        self.client
            .post(FAVORITE_ENDPOINT)
            .with_bilibili_headers()
            .form(&params.form_pairs(&csrf))
            .send_bpi_payload("video.favorite")
            .await
    }
}

fn legacy_video_id_form(
    aid: Option<u64>,
    bvid: Option<String>,
) -> Result<LegacyVideoIdForm, BpiError> {
    let aid = match aid {
        Some(0) => {
            return Err(BpiError::invalid_parameter("aid", "id must be non-zero"));
        }
        Some(aid) => Some(aid.to_string()),
        None => None,
    };

    let bvid = match bvid {
        Some(bvid) if bvid.trim().is_empty() => {
            return Err(BpiError::invalid_parameter("bvid", "bvid cannot be blank"));
        }
        Some(bvid) => Some(bvid),
        None => None,
    };

    if aid.is_none() && bvid.is_none() {
        return Err(BpiError::invalid_parameter(
            "video_id",
            "aid or bvid is required",
        ));
    }

    Ok(LegacyVideoIdForm {
        aid: aid.unwrap_or_else(|| "0".to_string()),
        bvid: bvid.unwrap_or_default(),
    })
}

fn validate_like_action(like: u8) -> Result<(), BpiError> {
    if matches!(like, 1 | 2) {
        return Ok(());
    }

    Err(BpiError::invalid_parameter("like", "value must be 1 or 2"))
}

fn validate_coin_multiply(multiply: u8) -> Result<(), BpiError> {
    if matches!(multiply, 1 | 2) {
        return Ok(());
    }

    Err(BpiError::invalid_parameter(
        "multiply",
        "value must be 1 or 2",
    ))
}

fn normalize_id_list(
    field: &'static str,
    values: impl IntoIterator<Item = impl Into<String>>,
) -> BpiResult<Vec<String>> {
    values
        .into_iter()
        .map(|value| {
            let value = value.into();
            let value = value.trim();
            if value.is_empty() {
                return Err(BpiError::invalid_parameter(field, "value cannot be blank"));
            }

            Ok(value.to_string())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::{
        BpiClient, BpiError,
        session::{Account, AccountProfile},
    };

    use super::{
        VideoCoinParams, VideoCoinStatusParams, VideoFavoriteParams, VideoLikeParams,
        legacy_video_id_form, validate_coin_multiply,
    };

    #[test]
    fn legacy_video_id_form_rejects_missing_video_id() {
        let err = legacy_video_id_form(None, None).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "video_id",
                ..
            }
        ));
    }

    #[test]
    fn validate_coin_multiply_rejects_oversized_value() {
        let err = validate_coin_multiply(3).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "multiply",
                ..
            }
        ));
    }

    #[test]
    fn video_like_params_serializes_aid() -> Result<(), BpiError> {
        let params = VideoLikeParams::from_aid(170001, 1)?;

        assert_eq!(
            params.form_pairs("csrf-token"),
            vec![
                ("aid", "170001".to_string()),
                ("like", "1".to_string()),
                ("csrf", "csrf-token".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn video_coin_params_defaults_select_like_to_false() -> Result<(), BpiError> {
        let params = VideoCoinParams::from_bvid("BV1xx411c7mD", 2)?;

        assert_eq!(
            params.form_pairs("csrf-token"),
            vec![
                ("bvid", "BV1xx411c7mD".to_string()),
                ("multiply", "2".to_string()),
                ("select_like", "0".to_string()),
                ("csrf", "csrf-token".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn video_coin_status_params_serializes_bvid() -> Result<(), BpiError> {
        let params = VideoCoinStatusParams::from_bvid("BV1xx411c7mD")?;

        assert_eq!(
            params.query_pairs(),
            vec![("bvid", "BV1xx411c7mD".to_string())]
        );
        Ok(())
    }

    #[test]
    fn video_favorite_params_rejects_empty_operation() {
        let err = VideoFavoriteParams::new(170001, Vec::<String>::new(), Vec::<String>::new())
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "media_ids",
                ..
            }
        ));
    }

    #[test]
    fn video_favorite_params_rejects_blank_media_id() {
        let err = VideoFavoriteParams::new(170001, [" "], Vec::<String>::new()).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "add_media_ids",
                ..
            }
        ));
    }

    #[test]
    fn video_action_methods_use_documented_payload_shapes() {
        let source = include_str!("action.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source should precede tests");
        let optional_helper = concat!(".send_", "bpi_optional_payload");
        let payload_helper = concat!(".send_", "bpi_payload");

        assert!(source.contains(&format!("{payload_helper}(\"video.coin_status\")")));
        assert!(source.contains(&format!("{optional_helper}(\"video.like\")")));
        assert!(source.contains(&format!("{payload_helper}(\"video.coin\")")));
    }

    fn live_mutating_tests_enabled() -> bool {
        std::env::var("BPI_MUTATING_TEST").ok().as_deref() == Some("1")
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum LiveCoinTarget {
        Aid(u64),
        Bvid(String),
    }

    impl LiveCoinTarget {
        fn label(&self) -> String {
            match self {
                Self::Aid(aid) => format!("aid={aid}"),
                Self::Bvid(bvid) => format!("bvid={bvid}"),
            }
        }

        fn coin_status_params(&self) -> Result<VideoCoinStatusParams, BpiError> {
            match self {
                Self::Aid(aid) => VideoCoinStatusParams::from_aid(*aid),
                Self::Bvid(bvid) => VideoCoinStatusParams::from_bvid(bvid.as_str()),
            }
        }

        fn coin_params(&self, multiply: u8) -> Result<VideoCoinParams, BpiError> {
            match self {
                Self::Aid(aid) => VideoCoinParams::from_aid(*aid, multiply),
                Self::Bvid(bvid) => VideoCoinParams::from_bvid(bvid.as_str(), multiply),
            }
        }
    }

    fn live_coin_target_from_env() -> Result<LiveCoinTarget, BpiError> {
        parse_live_coin_target(
            std::env::var("BPI_VIDEO_AID").ok(),
            std::env::var("BPI_VIDEO_BVID").ok(),
        )
    }

    fn parse_live_coin_target(
        aid: Option<String>,
        bvid: Option<String>,
    ) -> Result<LiveCoinTarget, BpiError> {
        if let Some(aid) = aid {
            let aid = aid.trim();
            if !aid.is_empty() {
                let aid = aid.parse::<u64>().map_err(|_| {
                    BpiError::invalid_parameter("BPI_VIDEO_AID", "value must be a non-zero avid")
                })?;
                if aid == 0 {
                    return Err(BpiError::invalid_parameter(
                        "BPI_VIDEO_AID",
                        "value must be a non-zero avid",
                    ));
                }
                return Ok(LiveCoinTarget::Aid(aid));
            }
        }

        if let Some(bvid) = bvid {
            let bvid = bvid.trim();
            if !bvid.is_empty() {
                return Ok(LiveCoinTarget::Bvid(bvid.to_string()));
            }
        }

        Err(BpiError::invalid_parameter(
            "BPI_VIDEO_AID",
            "set BPI_VIDEO_AID or BPI_VIDEO_BVID",
        ))
    }

    fn live_coin_multiply_from_env() -> Result<u8, BpiError> {
        parse_live_coin_multiply(std::env::var("BPI_VIDEO_COIN_MULTIPLY").ok())
    }

    fn parse_live_coin_multiply(value: Option<String>) -> Result<u8, BpiError> {
        let Some(value) = value else {
            return Ok(1);
        };
        let value = value.trim();
        if value.is_empty() {
            return Ok(1);
        }

        let multiply = value.parse::<u8>().map_err(|_| {
            BpiError::invalid_parameter("BPI_VIDEO_COIN_MULTIPLY", "value must be 1 or 2")
        })?;
        if matches!(multiply, 1 | 2) {
            return Ok(multiply);
        }

        Err(BpiError::invalid_parameter(
            "BPI_VIDEO_COIN_MULTIPLY",
            "value must be 1 or 2",
        ))
    }

    #[test]
    fn live_coin_target_from_env_prefers_aid() -> Result<(), BpiError> {
        let target = parse_live_coin_target(Some("116856715220362".to_string()), None)?;

        assert_eq!(target.label(), "aid=116856715220362");
        Ok(())
    }

    #[test]
    fn live_coin_target_from_env_accepts_bvid() -> Result<(), BpiError> {
        let target = parse_live_coin_target(None, Some("BV11GTb6GEXX".to_string()))?;

        assert_eq!(target.label(), "bvid=BV11GTb6GEXX");
        Ok(())
    }

    #[test]
    fn live_coin_target_from_env_requires_a_video_id() {
        let err = parse_live_coin_target(None, None).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "BPI_VIDEO_AID",
                ..
            }
        ));
    }

    #[test]
    fn live_coin_multiply_from_env_defaults_to_one() -> Result<(), BpiError> {
        assert_eq!(parse_live_coin_multiply(None)?, 1);
        Ok(())
    }

    #[test]
    fn live_coin_multiply_from_env_rejects_invalid_value() {
        let err = parse_live_coin_multiply(Some("3".to_string())).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "BPI_VIDEO_COIN_MULTIPLY",
                ..
            }
        ));
    }

    #[ignore = "live mutating test; requires BPI_MUTATING_TEST=1 plus BPI_VIDEO_AID or BPI_VIDEO_BVID"]
    #[tokio::test]
    async fn live_vip_coin_from_env_spends_requested_coins() -> Result<(), BpiError> {
        if !live_mutating_tests_enabled() {
            eprintln!(
                "skipping live mutating test; set BPI_MUTATING_TEST=1, BPI_VIDEO_AID or BPI_VIDEO_BVID, and optional BPI_VIDEO_COIN_MULTIPLY"
            );
            return Ok(());
        }

        let target = live_coin_target_from_env()?;
        let multiply = live_coin_multiply_from_env()?;
        let account = Account::load_test_account_profile(AccountProfile::Vip)?;
        let client = BpiClient::builder().account(account).build()?;
        let video = client.video();

        match video.coin_status(target.coin_status_params()?).await {
            Ok(status) => eprintln!("coin status before {}: {status:?}", target.label()),
            Err(err) => eprintln!("coin status before {} failed: {err:?}", target.label()),
        }

        match video.coin(target.coin_params(multiply)?).await {
            Ok(payload) => eprintln!(
                "coin succeeded for {} with multiply={multiply}: {payload:?}",
                target.label()
            ),
            Err(err) => {
                eprintln!(
                    "coin failed for {} with multiply={multiply}: {err:?}",
                    target.label()
                );
                return Err(err);
            }
        }

        match video.coin_status(target.coin_status_params()?).await {
            Ok(status) => eprintln!("coin status after {}: {status:?}", target.label()),
            Err(err) => eprintln!("coin status after {} failed: {err:?}", target.label()),
        }

        Ok(())
    }

    #[ignore = "local account shape diagnostic; prints presence only, never secret values"]
    #[test]
    fn live_vip_account_shape_has_expected_cookie_fields() -> Result<(), BpiError> {
        let account = Account::load_test_account_profile(AccountProfile::Vip)?;

        eprintln!("vip has DedeUserID: {}", !account.dede_user_id.is_empty());
        eprintln!("vip has SESSDATA: {}", !account.sessdata.is_empty());
        eprintln!("vip has bili_jct: {}", !account.bili_jct.is_empty());
        eprintln!("vip has buvid3: {}", !account.buvid3.is_empty());

        Ok(())
    }

    #[ignore = "live read diagnostic; requires BPI_VIDEO_AID or BPI_VIDEO_BVID"]
    #[tokio::test]
    async fn live_vip_coin_status_from_env() -> Result<(), BpiError> {
        let target = live_coin_target_from_env()?;
        let account = Account::load_test_account_profile(AccountProfile::Vip)?;
        let client = BpiClient::builder().account(account).build()?;
        let video = client.video();

        let status = video.coin_status(target.coin_status_params()?).await?;
        eprintln!("coin status for {}: {status:?}", target.label());

        Ok(())
    }
}
