// B站用户空间相关接口
//
// [查看 API 文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/user)

// --- 响应数据结构体 ---

use crate::BilibiliRequest;
use crate::BpiError;
use crate::BpiResult;
use crate::user::UserClient;
use serde::{Deserialize, Serialize};

/// 用户空间公告响应数据

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SpaceNoticeResponseData(pub String);

/// 修改空间公告响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SetSpaceNoticeResponseData;

/// 追番/追剧列表项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BangumiFollowItem {
    pub season_id: i64,
    pub media_id: i64,
    pub season_type: i64,
    pub season_type_name: String,
    pub title: String,
    pub cover: String,
    pub total_count: i64,
    pub is_finish: i64,
    pub is_started: i64,
    pub is_play: i64,
    pub badge: String,
    pub badge_type: i64,
    pub rights: Rights,
    pub stat: Stat,
    pub new_ep: NewEp,
    pub rating: Option<Rating>,
    pub square_cover: String,
    pub season_status: i64,
    pub season_title: String,
    pub badge_ep: String,
    pub media_attr: i64,
    pub season_attr: i64,
    pub evaluate: String,
    pub areas: Vec<Area>,
    pub subtitle: String,
    pub first_ep: i64,
    pub can_watch: i64,
    pub series: Series,
    pub publish: Publish,
    pub mode: i64,
    pub section: Vec<Section>,
    pub url: String,
    pub badge_info: BadgeInfo,
    pub renewal_time: Option<String>,
    pub first_ep_info: FirstEpInfo,
    pub formal_ep_count: i64,
    pub short_url: String,
    pub badge_infos: BadgeInfos,
    pub season_version: String,
    pub subtitle_14: String,
    pub viewable_crowd_type: i64,
    pub summary: String,
    pub styles: Vec<String>,
    pub follow_status: i64,
    pub is_new: i64,
    pub progress: String,
    pub both_follow: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rights {
    pub allow_review: i64,
    pub is_selection: i64,
    pub selection_style: i64,
    pub is_rcmd: i64,

    pub demand_end_time: serde_json::Value,
    pub allow_preview: Option<i64>,
    pub allow_bp_rank: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stat {
    pub follow: i64,
    pub view: i64,
    pub danmaku: i64,
    pub reply: i64,
    pub coin: i64,
    pub series_follow: i64,
    pub series_view: i64,
    pub likes: i64,
    pub favorite: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewEp {
    pub id: i64,
    pub index_show: String,
    pub cover: String,
    pub title: String,
    pub long_title: Option<String>,
    pub pub_time: String,
    pub duration: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Rating {
    pub score: f64,
    pub count: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Area {
    pub id: i64,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Series {
    pub series_id: i64,
    pub title: String,
    pub season_count: i64,
    pub new_season_id: i64,
    pub series_ord: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Publish {
    pub pub_time: String,
    pub pub_time_show: String,
    pub release_date: String,
    pub release_date_show: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub section_id: i64,
    pub season_id: i64,
    pub limit_group: i64,
    pub watch_platform: i64,
    pub copyright: String,
    pub ban_area_show: i64,
    pub episode_ids: Vec<i64>,
    #[serde(rename = "type")]
    pub type_field: Option<i64>,
    pub title: Option<String>,
    pub attr: Option<i64>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BadgeInfo {
    pub text: String,
    pub bg_color: String,
    pub bg_color_night: String,
    pub img: String,
    pub multi_img: MultiImg,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiImg {
    pub color: String,
    pub medium_remind: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FirstEpInfo {
    pub id: i64,
    pub cover: String,
    pub title: String,
    pub long_title: Option<String>,
    pub pub_time: String,
    pub duration: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BadgeInfos {
    pub vip_or_pay: VipOrPay,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VipOrPay {
    pub text: String,
    pub bg_color: String,
    pub bg_color_night: String,
    pub img: String,
    pub multi_img: MultiImg2,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiImg2 {
    pub color: String,
    pub medium_remind: String,
}

/// 用户追番/追剧明细响应数据
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BangumiFollowListResponseData {
    /// 追番列表
    pub list: Vec<BangumiFollowItem>,
    /// 当前页码
    pub pn: u32,
    /// 每页项数
    pub ps: u32,
    /// 总计追番数
    pub total: u64,
}

/// 设置用户空间公告的参数。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UserSpaceNoticeSetParams {
    notice: Option<String>,
}

impl UserSpaceNoticeSetParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn notice(mut self, notice: impl Into<String>) -> BpiResult<Self> {
        let notice = notice.into();
        if notice.len() > 150 {
            return Err(BpiError::invalid_parameter(
                "notice",
                "length cannot exceed 150 bytes",
            ));
        }
        self.notice = Some(notice);
        Ok(self)
    }

    fn into_multipart(self, csrf: &str) -> reqwest::multipart::Form {
        let mut form = reqwest::multipart::Form::new().text("csrf", csrf.to_string());

        if let Some(notice) = self.notice {
            form = form.text("notice", notice);
        }

        form
    }
}

// --- API 实现 ---

// --- 测试模块 ---

impl<'a> UserClient<'a> {
    /// 设置用户空间公告并返回标准 payload 结果。
    pub async fn set_space_notice(
        &self,
        params: UserSpaceNoticeSetParams,
    ) -> BpiResult<Option<()>> {
        let csrf = self.client.csrf()?;
        let form = params.into_multipart(&csrf);

        self.client
            .post("https://api.bilibili.com/x/space/notice/set")
            .multipart(form)
            .send_bpi_optional_payload("user.space_notice.set")
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;

    use crate::{ApiEnvelope, BpiError, BpiResult};

    // 请在运行测试前设置环境变量 `BPI_COOKIE`，以包含 SESSDATA 等登录信息
    // mid 根据实际情况修改

    fn public_read_contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes: &[u8] = match endpoint {
            "bangumi-follow-list" => include_bytes!(
                "../../tests/contracts/user/public-read/bangumi-follow-list/contract.json"
            ),
            "space-notice" => {
                include_bytes!("../../tests/contracts/user/public-read/space-notice/contract.json")
            }
            _ => {
                return Err(BpiError::invalid_parameter(
                    "endpoint",
                    "unknown user space contract",
                ));
            }
        };

        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn legacy_user_space_contracts_match_endpoint_requests() -> BpiResult<()> {
        let notice = public_read_contract("space-notice")?;
        assert_eq!(notice.name, "user.space_notice");
        assert_eq!(notice.request.method, HttpMethod::Get);
        assert_eq!(
            notice.request.url.as_str(),
            "https://api.bilibili.com/x/space/notice"
        );
        assert_eq!(
            notice.request.query.get("mid").map(String::as_str),
            Some("2")
        );

        let bangumi = public_read_contract("bangumi-follow-list")?;
        assert_eq!(bangumi.name, "user.bangumi_follow_list");
        assert_eq!(
            bangumi.request.url.as_str(),
            "https://api.bilibili.com/x/space/bangumi/follow/list"
        );
        assert_eq!(
            bangumi.request.query.get("vmid").map(String::as_str),
            Some("1000001")
        );
        assert_eq!(
            bangumi.request.query.get("type").map(String::as_str),
            Some("1")
        );
        Ok(())
    }

    #[test]
    fn legacy_user_space_fixtures_parse_promoted_contract_models() -> BpiResult<()> {
        let notice = ApiEnvelope::<SpaceNoticeResponseData>::from_slice(include_bytes!(
            "../../tests/contracts/user/public-read/space-notice/responses/success.json"
        ))?
        .into_payload()?;
        let _notice_text = notice.0;

        let follow_list =
            ApiEnvelope::<BangumiFollowListResponseData>::from_slice(include_bytes!(
                "../../tests/contracts/user/public-read/bangumi-follow-list/responses/success.json"
            ))?
            .into_payload()?;
        assert_eq!(follow_list.pn, 1);
        Ok(())
    }
}
