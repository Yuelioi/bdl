use crate::BilibiliRequest;
use crate::BpiError;
use crate::BpiResult;
use crate::ids::{Mid, RoomId};
use crate::live::LiveClient;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct SilentUserInfo {
    /// 禁言者uid
    pub tuid: i64,
    /// 禁言者昵称
    pub tname: String,
    /// 发起者uid
    pub uid: i64,
    /// 发起者昵称
    pub name: String,
    /// 禁言时间
    pub ctime: String,
    /// 禁言记录Id
    pub id: i64,
    /// 是否是房主禁言的，0否，1是
    pub is_anchor: i8,
    /// 禁言者头像
    pub face: String,
    /// 禁言理由
    pub msg: String,
    /// 发起者权限
    pub admin_level: i8,
    /// 是否注销
    pub is_mystery: bool,
    /// 禁言结束时间，空代表永久或本场禁言
    pub block_end_time: String,
    /// 禁言模式，0代表永久，1代表正常，2代表本场禁言
    pub r#type: i8,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct SilentUserListData {
    /// 禁言列表
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub data: Vec<SilentUserInfo>,
    /// 禁言观众数量
    pub total: i32,
    /// 页码总数量，只有一页的时候没有
    #[serde(default)]
    pub total_page: i32,
    /// 页码，只有一页的时候没有
    #[serde(default)]
    pub pn: i32,
    /// 上限，只有一页的时候没有
    #[serde(default)]
    pub ps: i32,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct BannedUserInfo {
    /// 拉黑者uid
    pub uid: i64,
    /// 拉黑时间
    pub mtime: String,
    /// 拉黑者头像
    pub face: String,
    /// 拉黑者昵称
    pub name: String,
    /// 是否是房主拉黑的
    pub is_anchor: bool,
    /// 发起者昵称
    pub operator_name: String,
    /// 发起者权限
    pub admin_level: i8,
    /// 是否注销
    pub is_mystery: bool,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct BannedUserListData {
    /// 拉黑列表
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub data: Vec<BannedUserInfo>,
    /// 拉黑观众数量
    pub total: i32,
    /// 页码总数量，只有一页的时候没有，由于接口不返回，所以默认0
    #[serde(default)]
    pub total_page: i32,
    /// 上限，只有一页的时候没有，由于接口不返回，所以默认0
    #[serde(default)]
    pub pn: i32,
    /// 页码，只有一页的时候没有，由于接口不返回，所以默认0
    #[serde(default)]
    pub ps: i32,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ShieldKeywordInfo {
    /// 违禁词
    pub keyword: String,
    /// 添加者uid
    pub uid: i64,
    /// 添加者昵称
    pub name: String,
    /// 是否是房主添加的，0否，1是
    pub is_anchor: i8,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ShieldKeywordListData {
    /// 违禁词列表
    #[serde(default, deserialize_with = "deserialize_vec_or_default")]
    pub keyword_list: Vec<ShieldKeywordInfo>,
    /// 数量上限
    pub max_limit: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveSilentUserListParams {
    room_id: RoomId,
    page: u32,
    page_size: u32,
}

impl LiveSilentUserListParams {
    pub fn new(room_id: RoomId) -> Self {
        Self {
            room_id,
            page: 1,
            page_size: 10,
        }
    }

    pub fn page(mut self, page: u32) -> BpiResult<Self> {
        self.page = validate_positive_u32("pn", page)?;
        Ok(self)
    }

    pub fn page_size(mut self, page_size: u32) -> BpiResult<Self> {
        self.page_size = validate_positive_u32("ps", page_size)?;
        Ok(self)
    }

    pub(crate) fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("room_id", self.room_id.to_string()),
            ("pn", self.page.to_string()),
            ("ps", self.page_size.to_string()),
            ("csrf_token", csrf.to_string()),
            ("csrf", csrf.to_string()),
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveBannedUserListParams {
    anchor_id: Mid,
    page: u32,
    page_size: u32,
}

impl LiveBannedUserListParams {
    pub fn new(anchor_id: Mid) -> Self {
        Self {
            anchor_id,
            page: 1,
            page_size: 10,
        }
    }

    pub fn page(mut self, page: u32) -> BpiResult<Self> {
        self.page = validate_positive_u32("pn", page)?;
        Ok(self)
    }

    pub fn page_size(mut self, page_size: u32) -> BpiResult<Self> {
        self.page_size = validate_positive_u32("ps", page_size)?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("anchor_id", self.anchor_id.to_string()),
            ("pn", self.page.to_string()),
            ("ps", self.page_size.to_string()),
            ("mobi_app", "android".to_string()),
            ("platform", "android".to_string()),
            ("spmid", "444.8.0.0".to_string()),
            ("csrf_token", csrf.to_string()),
            ("csrf", csrf.to_string()),
            ("visit_id", String::new()),
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveShieldKeywordListParams {
    room_id: RoomId,
}

impl LiveShieldKeywordListParams {
    pub fn new(room_id: RoomId) -> Self {
        Self { room_id }
    }

    pub(crate) fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("room_id", self.room_id.to_string()),
            ("spmid", "444.8.0.0".to_string()),
            ("csrf_token", csrf.to_string()),
            ("csrf", csrf.to_string()),
            ("visit_id", String::new()),
            ("mobi_app", "android".to_string()),
            ("platform", "android".to_string()),
        ]
    }
}

fn validate_positive_u32(field: &'static str, value: u32) -> BpiResult<u32> {
    if value == 0 {
        return Err(BpiError::invalid_parameter(field, "value must be non-zero"));
    }

    Ok(value)
}

fn deserialize_vec_or_default<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::<Vec<T>>::deserialize(deserializer)?.unwrap_or_default())
}

impl<'a> LiveClient<'a> {
    /// 禁言观众
    /// tuid: 用户uid
    /// hour: -1永久 0本场直播
    /// msg: 禁言理由，一般为禁言的弹幕，选填
    pub async fn live_add_silent_user(
        &self,
        room_id: i64,
        tuid: i64,
        hour: i32,
        msg: Option<String>,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        let form = vec![
            ("room_id", room_id.to_string()),
            ("tuid", tuid.to_string()),
            ("msg", msg.unwrap_or_default()),
            ("mobile_app", "web".to_string()),
            (
                "type",
                if hour == 0 {
                    "2".to_string()
                } else {
                    "1".to_string()
                },
            ),
            ("hour", hour.to_string()),
            ("csrf_token", csrf.clone()),
            ("csrf", csrf),
        ];

        // if let Some(msg) = msg {
        //     form.push(("msg", msg.to_string()));
        // }

        self.client
            .post("https://api.live.bilibili.com/xlive/web-ucenter/v1/banned/AddSilentUser")
            .form(&form)
            .send_bpi_optional_payload("live.silent_user.add")
            .await
    }

    /// 解除禁言
    ///
    pub async fn live_del_block_user(
        &self,
        roomid: i64,
        tuid: i64,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        let form = vec![
            ("room_id", roomid.to_string()),
            ("tuid", tuid.to_string()),
            ("csrf_token", csrf.clone()),
            ("csrf", csrf),
        ];

        self.client
            .post("https://api.live.bilibili.com/xlive/web-ucenter/v1/banned/DelSilentUser")
            .form(&form)
            .send_bpi_optional_payload("live.silent_user.delete")
            .await
    }

    /// 拉黑观众
    /// anchor_id：主播uid
    pub async fn live_add_banned_user(
        &self,
        room_id: i64,
        anchor_id: i64,
        tuid: i64,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        let form = vec![
            ("tuid", tuid.to_string()),
            ("anchor_id", anchor_id.to_string()),
            ("spmid", "444.8.0.0".to_string()),
            ("csrf_token", csrf.clone()),
            ("csrf", csrf),
            ("visit_id", "".to_string()),
        ];

        self.client
            .post("https://api.live.bilibili.com/xlive/app-ucenter/v2/xbanned/banned/AddBlack")
            .header("Referer", format!("https://live.bilibili.com/{}", room_id))
            .form(&form)
            .send_bpi_optional_payload("live.banned_user.add")
            .await
    }

    /// 解除拉黑
    /// anchor_id：主播uid
    pub async fn live_del_banned_user(
        &self,
        room_id: i64,
        anchor_id: i64,
        tuid: i64,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        let form = vec![
            ("tuid", tuid.to_string()),
            ("anchor_id", anchor_id.to_string()),
            ("spmid", "444.8.0.0".to_string()),
            ("csrf_token", csrf.clone()),
            ("csrf", csrf),
            ("visit_id", "".to_string()),
            ("mobi_app", "android".to_string()),
            ("platform", "android".to_string()),
        ];

        self.client
            .post("https://api.live.bilibili.com/xlive/app-ucenter/v2/xbanned/banned/DelBlack")
            .header("Referer", format!("https://live.bilibili.com/{}", room_id))
            .form(&form)
            .send_bpi_optional_payload("live.banned_user.delete")
            .await
    }

    /// 添加屏蔽词
    ///
    pub async fn live_add_shield_keyword(
        &self,
        room_id: i64,
        keyword: String,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        let form = vec![
            ("keyword", keyword),
            ("room_id", room_id.to_string()),
            ("spmid", "444.8.0.0".to_string()),
            ("csrf_token", csrf.clone()),
            ("csrf", csrf),
            ("visit_id", "".to_string()),
            ("mobi_app", "android".to_string()),
            ("platform", "android".to_string()),
        ];

        self.client
            .post("https://api.live.bilibili.com/xlive/app-ucenter/v1/banned/AddShieldKeyword")
            .form(&form)
            .send_bpi_optional_payload("live.shield_keyword.add")
            .await
    }

    /// 删除屏蔽词
    ///
    pub async fn live_del_shield_keyword(
        &self,
        room_id: i64,
        keyword: String,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        let form = vec![
            ("keyword", keyword),
            ("room_id", room_id.to_string()),
            ("spmid", "444.8.0.0".to_string()),
            ("csrf_token", csrf.clone()),
            ("csrf", csrf),
            ("visit_id", "".to_string()),
            ("mobi_app", "android".to_string()),
            ("platform", "android".to_string()),
        ];

        self.client
            .post("https://api.live.bilibili.com/xlive/app-ucenter/v1/banned/DelShieldKeyword")
            .form(&form)
            .send_bpi_optional_payload("live.shield_keyword.delete")
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiResult};

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "silent-users" => include_bytes!(
                "../../tests/contracts/live/moderation-private-read/silent-users/contract.json"
            )
            .as_slice(),
            "banned-users" => include_bytes!(
                "../../tests/contracts/live/moderation-private-read/banned-users/contract.json"
            )
            .as_slice(),
            "shield-keywords" => include_bytes!(
                "../../tests/contracts/live/moderation-private-read/shield-keywords/contract.json"
            )
            .as_slice(),
            _ => unreachable!("unknown live moderation contract endpoint"),
        };

        EndpointContract::from_slice(bytes)
    }

    fn room_id() -> RoomId {
        RoomId::new(3_818_081).expect("test room id should be valid")
    }

    fn anchor_id() -> Mid {
        Mid::new(1_000_001).expect("test anchor id should be valid")
    }

    #[test]
    fn live_moderation_params_reject_zero_pagination() {
        let err = LiveSilentUserListParams::new(room_id())
            .page(0)
            .unwrap_err();
        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "pn", .. }
        ));

        let err = LiveBannedUserListParams::new(anchor_id())
            .page_size(0)
            .unwrap_err();
        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "ps", .. }
        ));
    }

    #[test]
    fn live_moderation_contracts_match_endpoint_requests() -> BpiResult<()> {
        let silent_users = contract("silent-users")?;
        let banned_users = contract("banned-users")?;
        let shield_keywords = contract("shield-keywords")?;

        assert_eq!(silent_users.name, "live.silent_users");
        assert_eq!(silent_users.request.method, HttpMethod::Post);
        assert_eq!(
            silent_users.request.url.as_str(),
            "https://api.live.bilibili.com/xlive/web-ucenter/v1/banned/GetSilentUserList"
        );
        let silent_params = LiveSilentUserListParams::new(room_id())
            .page(1)?
            .page_size(10)?;
        assert_eq!(
            silent_params.form_pairs("${csrf}"),
            vec![
                ("room_id", "3818081".to_string()),
                ("pn", "1".to_string()),
                ("ps", "10".to_string()),
                ("csrf_token", "${csrf}".to_string()),
                ("csrf", "${csrf}".to_string()),
            ]
        );

        assert_eq!(banned_users.name, "live.banned_users");
        assert_eq!(banned_users.request.method, HttpMethod::Get);
        assert_eq!(
            banned_users.request.url.as_str(),
            "https://api.live.bilibili.com/xlive/app-ucenter/v2/xbanned/banned/GetBlackList"
        );
        let banned_params = LiveBannedUserListParams::new(anchor_id())
            .page(1)?
            .page_size(10)?;
        assert_eq!(
            banned_params.query_pairs("${csrf}"),
            vec![
                ("anchor_id", "1000001".to_string()),
                ("pn", "1".to_string()),
                ("ps", "10".to_string()),
                ("mobi_app", "android".to_string()),
                ("platform", "android".to_string()),
                ("spmid", "444.8.0.0".to_string()),
                ("csrf_token", "${csrf}".to_string()),
                ("csrf", "${csrf}".to_string()),
                ("visit_id", String::new()),
            ]
        );

        assert_eq!(shield_keywords.name, "live.shield_keywords");
        assert_eq!(shield_keywords.request.method, HttpMethod::Post);
        assert_eq!(
            shield_keywords.request.url.as_str(),
            "https://api.live.bilibili.com/xlive/app-ucenter/v1/banned/GetShieldKeywordList"
        );
        assert_eq!(
            LiveShieldKeywordListParams::new(room_id()).form_pairs("${csrf}"),
            vec![
                ("room_id", "3818081".to_string()),
                ("spmid", "444.8.0.0".to_string()),
                ("csrf_token", "${csrf}".to_string()),
                ("csrf", "${csrf}".to_string()),
                ("visit_id", String::new()),
                ("mobi_app", "android".to_string()),
                ("platform", "android".to_string()),
            ]
        );

        assert_eq!(silent_users.cases.len(), 3);
        assert_eq!(banned_users.cases.len(), 3);
        assert_eq!(shield_keywords.cases.len(), 3);
        Ok(())
    }

    #[test]
    fn live_moderation_response_fixtures_parse_declared_models() -> BpiResult<()> {
        let anonymous = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/live/moderation-private-read/silent-users/responses/anonymous.requires_login.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert!(anonymous.requires_login());

        let not_admin = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/live/moderation-private-read/silent-users/responses/normal.not_admin.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert_eq!(not_admin.code(), Some(100_004));

        let silent_users = ApiEnvelope::<SilentUserListData>::from_slice(include_bytes!(
            "../../tests/contracts/live/moderation-private-read/silent-users/responses/vip.empty.success.json"
        ))?
        .into_payload()?;
        assert_eq!(silent_users.total, 0);

        let banned_empty = ApiEnvelope::<BannedUserListData>::from_slice(include_bytes!(
            "../../tests/contracts/live/moderation-private-read/banned-users/responses/normal.empty.success.json"
        ))?
        .into_payload()?;
        assert_eq!(banned_empty.total, 0);

        let banned_sample = ApiEnvelope::<BannedUserListData>::from_slice(include_bytes!(
            "../../tests/contracts/live/moderation-private-read/banned-users/responses/vip.sample.success.json"
        ))?
        .into_payload()?;
        assert_eq!(banned_sample.total, 1);
        assert_eq!(banned_sample.data[0].name, "<redacted-user>");

        let permission_denied = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/live/moderation-private-read/shield-keywords/responses/normal.permission_denied.json"
        ))?
        .ensure_success()
        .unwrap_err();
        assert_eq!(permission_denied.code(), Some(100_007));

        let shield_keywords = ApiEnvelope::<ShieldKeywordListData>::from_slice(include_bytes!(
            "../../tests/contracts/live/moderation-private-read/shield-keywords/responses/vip.empty.success.json"
        ))?
        .into_payload()?;
        assert_eq!(shield_keywords.max_limit, 1000);
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path = format!(
            "target/bpi-probe-runs/live/moderation-private-read/{endpoint}/{profile}.response.json"
        );
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn live_moderation_models_match_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            if let Some(body) = local_probe_body("silent-users", profile) {
                let envelope = serde_json::from_value::<ApiEnvelope<SilentUserListData>>(body)?;
                match profile {
                    "anonymous" => assert!(envelope.ensure_success().unwrap_err().requires_login()),
                    "normal" => {
                        assert_eq!(envelope.ensure_success().unwrap_err().code(), Some(100_004));
                    }
                    _ => {
                        let payload = envelope.into_payload()?;
                        assert!(payload.total >= 0);
                    }
                }
            }

            if let Some(body) = local_probe_body("banned-users", profile) {
                let envelope = serde_json::from_value::<ApiEnvelope<BannedUserListData>>(body)?;
                if profile == "anonymous" {
                    assert!(envelope.ensure_success().unwrap_err().requires_login());
                } else {
                    let payload = envelope.into_payload()?;
                    assert!(payload.total >= 0);
                }
            }

            if let Some(body) = local_probe_body("shield-keywords", profile) {
                let envelope = serde_json::from_value::<ApiEnvelope<ShieldKeywordListData>>(body)?;
                match profile {
                    "anonymous" => assert!(envelope.ensure_success().unwrap_err().requires_login()),
                    "normal" => {
                        assert_eq!(envelope.ensure_success().unwrap_err().code(), Some(100_007));
                    }
                    _ => {
                        let payload = envelope.into_payload()?;
                        assert!(payload.max_limit >= 0);
                    }
                }
            }
        }
        Ok(())
    }
}
