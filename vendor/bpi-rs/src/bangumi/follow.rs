// 追番相关
//
// [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/bangumi/follow.md)

use crate::BilibiliRequest;
use crate::bangumi::BangumiClient;
use crate::ids::SeasonId;
use crate::response::BpiResult;
use serde::{Deserialize, Serialize};

const FOLLOW_ENDPOINT: &str = "https://api.bilibili.com/pgc/web/follow/add";
const UNFOLLOW_ENDPOINT: &str = "https://api.bilibili.com/pgc/web/follow/del";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BangumiFollowResult {
    pub fmid: i64,
    pub relation: bool,
    pub status: i32,
    pub toast: String,
}

/// 追番或取消追番 bangumi season 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BangumiFollowParams {
    season_id: SeasonId,
}

impl BangumiFollowParams {
    pub fn new(season_id: SeasonId) -> Self {
        Self { season_id }
    }

    pub(crate) fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("season_id", self.season_id.to_string()),
            ("csrf", csrf.to_string()),
        ]
    }
}

impl<'a> BangumiClient<'a> {
    /// 追番 bangumi season 并返回标准 payload 结果。
    pub async fn follow(&self, params: BangumiFollowParams) -> BpiResult<BangumiFollowResult> {
        let csrf = self.client.csrf()?;
        self.client
            .post(FOLLOW_ENDPOINT)
            .with_bilibili_headers()
            .form(&params.form_pairs(&csrf))
            .send_bpi_payload("bangumi.follow")
            .await
    }

    /// 取消追番 bangumi season 并返回标准 payload 结果。
    pub async fn unfollow(&self, params: BangumiFollowParams) -> BpiResult<BangumiFollowResult> {
        let csrf = self.client.csrf()?;
        self.client
            .post(UNFOLLOW_ENDPOINT)
            .with_bilibili_headers()
            .form(&params.form_pairs(&csrf))
            .send_bpi_payload("bangumi.unfollow")
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::BpiError;
    use crate::bangumi::BangumiFollowParams;
    use crate::ids::SeasonId;

    #[test]
    fn bangumi_follow_params_serializes_season_id() -> Result<(), BpiError> {
        let params = BangumiFollowParams::new(SeasonId::new(1172)?);

        assert_eq!(
            params.form_pairs("csrf-token"),
            vec![
                ("season_id", "1172".to_string()),
                ("csrf", "csrf-token".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn season_id_rejects_zero_before_follow_params() {
        let err = SeasonId::new(0).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "season_id",
                ..
            }
        ));
    }
}
