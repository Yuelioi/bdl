// -------------------
// 发送视频弹幕
// -------------------

use crate::BilibiliRequest;
use crate::BpiError;
use crate::BpiResult;
use crate::danmaku::DanmakuClient;
use crate::ids::{Aid, Bvid, Cid};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DanmakuPostData {
    pub colorful_src: Option<serde_json::Value>, // 当请求参数colorful=60001时有效
    pub dmid: u64,
    pub dmid_str: String,
}

/// 发送视频弹幕的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DanmakuSendParams {
    oid: Cid,
    msg: String,
    aid: Option<Aid>,
    bvid: Option<Bvid>,
    mode: u8,
    typ: u8,
    progress: u32,
    color: u32,
    font_size: u8,
    pool: u8,
}

impl DanmakuSendParams {
    /// 使用 Bilibili Web 默认值创建参数。
    pub fn new(oid: Cid, msg: impl Into<String>) -> BpiResult<Self> {
        let msg = msg.into();
        if msg.trim().is_empty() {
            return Err(BpiError::invalid_parameter(
                "msg",
                "danmaku message cannot be blank",
            ));
        }

        Ok(Self {
            oid,
            msg,
            aid: None,
            bvid: None,
            mode: 1,
            typ: 1,
            progress: 1878,
            color: 16_777_215,
            font_size: 25,
            pool: 0,
        })
    }

    /// 设置可选的 AV 数字视频 ID。
    pub fn aid(mut self, aid: Aid) -> Self {
        self.aid = Some(aid);
        self
    }

    /// 设置可选的 BV 字符串视频 ID。
    pub fn bvid(mut self, bvid: Bvid) -> Self {
        self.bvid = Some(bvid);
        self
    }

    /// 设置弹幕显示模式。
    pub fn mode(mut self, mode: u8) -> Self {
        self.mode = mode;
        self
    }

    /// 设置弹幕类型。
    pub fn danmaku_type(mut self, typ: u8) -> Self {
        self.typ = typ;
        self
    }

    /// 设置弹幕时间戳，单位为毫秒。
    pub fn progress(mut self, progress: u32) -> Self {
        self.progress = progress;
        self
    }

    /// 设置 RGB888 颜色值。
    pub fn color(mut self, color: u32) -> Self {
        self.color = color;
        self
    }

    /// 设置字号。
    pub fn font_size(mut self, font_size: u8) -> Self {
        self.font_size = font_size;
        self
    }

    /// 设置弹幕池。
    pub fn pool(mut self, pool: u8) -> Self {
        self.pool = pool;
        self
    }

    fn form_pairs(&self, csrf: impl Into<String>) -> Vec<(&'static str, String)> {
        let mut form = vec![
            ("type", self.typ.to_string()),
            ("oid", self.oid.to_string()),
            ("msg", self.msg.clone()),
            ("mode", self.mode.to_string()),
            ("fontsize", self.font_size.to_string()),
            ("color", self.color.to_string()),
            ("pool", self.pool.to_string()),
            ("progress", self.progress.to_string()),
            ("rnd", "2".to_string()),
            ("plat", "1".to_string()),
            ("csrf", csrf.into()),
            ("checkbox_type", "0".to_string()),
            ("colorful", "".to_string()),
            ("gaiasource", "main_web".to_string()),
            ("polaris_app_id", "100".to_string()),
            ("polaris_platform", "5".to_string()),
            ("spmid", "333.788.0.0".to_string()),
            ("from_spmid", "333.788.0.0".to_string()),
        ];

        if let Some(aid) = self.aid {
            form.push(("avid", aid.to_string()));
        }
        if let Some(bvid) = self.bvid.as_ref() {
            form.push(("bvid", bvid.to_string()));
        }

        form
    }
}

// -------------------
// 撤回弹幕
// -------------------

// -------------------
// 购买高级弹幕发送权限
// -------------------

// -------------------
// 检测高级弹幕发送权限
// -------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DanmakuAdvState {
    pub coins: u8,
    #[serde(default)]
    pub confirm: u8,
    pub accept: bool,
    #[serde(default)]
    pub has_buy: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DanmakuAdvStateParams {
    cid: Cid,
}

impl DanmakuAdvStateParams {
    pub fn new(cid: Cid) -> Self {
        Self { cid }
    }

    pub(crate) fn query_pairs(&self) -> [(&'static str, String); 2] {
        [("cid", self.cid.to_string()), ("mode", "sp".to_string())]
    }
}

// -------------------
// 点赞弹幕
// -------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DanmakuRecallParams {
    cid: Cid,
    dmid: u64,
}

impl DanmakuRecallParams {
    pub fn new(cid: Cid, dmid: u64) -> BpiResult<Self> {
        Ok(Self {
            cid,
            dmid: validate_nonzero_u64("dmid", dmid)?,
        })
    }

    fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("cid", self.cid.to_string()),
            ("dmid", self.dmid.to_string()),
            ("type", "1".to_string()),
            ("csrf", csrf.to_string()),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DanmakuBuyAdvParams {
    cid: Cid,
}

impl DanmakuBuyAdvParams {
    pub fn new(cid: Cid) -> Self {
        Self { cid }
    }

    fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("cid", self.cid.to_string()),
            ("mode", "sp".to_string()),
            ("csrf", csrf.to_string()),
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DanmakuThumbupParams {
    oid: Cid,
    dmid: u64,
    op: u8,
}

impl DanmakuThumbupParams {
    pub fn new(oid: Cid, dmid: u64, op: u8) -> BpiResult<Self> {
        if !matches!(op, 1 | 2) {
            return Err(BpiError::invalid_parameter("op", "value must be 1 or 2"));
        }

        Ok(Self {
            oid,
            dmid: validate_nonzero_u64("dmid", dmid)?,
            op,
        })
    }

    fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("oid", self.oid.to_string()),
            ("dmid", self.dmid.to_string()),
            ("op", self.op.to_string()),
            ("csrf", csrf.to_string()),
            ("platform", "web_player".to_string()),
        ]
    }
}

// -------------------
// 举报弹幕
// -------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DanmakuReportParams {
    cid: Cid,
    dmid: u64,
    reason: u8,
    content: Option<String>,
}

impl DanmakuReportParams {
    pub fn new(cid: Cid, dmid: u64, reason: u8) -> BpiResult<Self> {
        Ok(Self {
            cid,
            dmid: validate_nonzero_u64("dmid", dmid)?,
            reason,
            content: None,
        })
    }

    pub fn content(mut self, content: impl Into<String>) -> BpiResult<Self> {
        self.content = Some(normalize_non_blank("content", content.into())?);
        Ok(self)
    }

    fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        let mut form = vec![
            ("cid", self.cid.to_string()),
            ("dmid", self.dmid.to_string()),
            ("reason", self.reason.to_string()),
            ("csrf", csrf.to_string()),
        ];
        if let Some(content) = &self.content {
            form.push(("content", content.clone()));
        }
        form
    }
}

// -------------------
// 保护&删除弹幕
// -------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DanmakuEditStateParams {
    oid: Cid,
    dmids: String,
    state: u8,
}

impl DanmakuEditStateParams {
    pub fn new(oid: Cid, dmids: &[u64], state: u8) -> BpiResult<Self> {
        Ok(Self {
            oid,
            dmids: join_u64_ids("dmids", dmids)?,
            state,
        })
    }

    fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("type", "1".to_string()),
            ("oid", self.oid.to_string()),
            ("dmids", self.dmids.clone()),
            ("state", self.state.to_string()),
            ("csrf", csrf.to_string()),
        ]
    }
}

// -------------------
// 修改字幕池
// -------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DanmakuEditPoolParams {
    oid: Cid,
    dmids: String,
    pool: u8,
}

impl DanmakuEditPoolParams {
    pub fn new(oid: Cid, dmids: &[u64], pool: u8) -> BpiResult<Self> {
        Ok(Self {
            oid,
            dmids: join_u64_ids("dmids", dmids)?,
            pool,
        })
    }

    fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("type", "1".to_string()),
            ("oid", self.oid.to_string()),
            ("dmids", self.dmids.clone()),
            ("pool", self.pool.to_string()),
            ("csrf", csrf.to_string()),
        ]
    }
}

impl<'a> DanmakuClient<'a> {
    /// 发送视频弹幕
    ///
    /// 文档: [弹幕相关](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/danmaku)
    ///
    pub async fn send(&self, params: DanmakuSendParams) -> BpiResult<DanmakuPostData> {
        let csrf = self.client.csrf()?;

        let form = params.form_pairs(csrf);

        // 签名参数加入表单
        let signed_params = self.client.get_wbi_sign2(form.clone()).await?;

        self.client
            .post("https://api.bilibili.com/x/v2/dm/post")
            .form(&signed_params)
            .send_bpi_payload("danmaku.send")
            .await
    }

    /// 撤回弹幕。
    pub async fn recall(
        &self,
        params: DanmakuRecallParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;
        self.client
            .post("https://api.bilibili.com/x/dm/recall")
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("danmaku.recall")
            .await
    }

    /// 购买高级弹幕发送权限。
    pub async fn buy_adv(
        &self,
        params: DanmakuBuyAdvParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;
        self.client
            .post("https://api.bilibili.com/x/dm/adv/buy")
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("danmaku.adv.buy")
            .await
    }

    /// 点赞弹幕。
    pub async fn thumbup(
        &self,
        params: DanmakuThumbupParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        self.client
            .post("https://api.bilibili.com/x/v2/dm/thumbup/add")
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("danmaku.thumbup")
            .await
    }

    /// 举报弹幕。
    pub async fn report(
        &self,
        params: DanmakuReportParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        self.client
            .post("https://api.bilibili.com/x/dm/report/add")
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("danmaku.report")
            .await
    }

    /// 保护或删除弹幕。
    pub async fn edit_state(
        &self,
        params: DanmakuEditStateParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        self.client
            .post("https://api.bilibili.com/x/v2/dm/edit/state")
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("danmaku.edit.state")
            .await
    }

    /// 修改字幕池。
    pub async fn edit_pool(
        &self,
        params: DanmakuEditPoolParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        self.client
            .post("https://api.bilibili.com/x/v2/dm/edit/pool")
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("danmaku.edit.pool")
            .await
    }
}

fn validate_nonzero_u64(field: &'static str, value: u64) -> BpiResult<u64> {
    if value == 0 {
        return Err(BpiError::invalid_parameter(field, "id must be non-zero"));
    }

    Ok(value)
}

fn normalize_non_blank(field: &'static str, value: String) -> BpiResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BpiError::invalid_parameter(field, "value cannot be blank"));
    }

    Ok(value)
}

fn join_u64_ids(field: &'static str, values: &[u64]) -> BpiResult<String> {
    if values.is_empty() || values.contains(&0) {
        return Err(BpiError::invalid_parameter(
            field,
            "ids must be non-empty and non-zero",
        ));
    }

    Ok(values
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ids::Cid;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::response::ApiEnvelope;
    use std::collections::BTreeMap;

    const TEST_CID: u64 = 413195701;

    fn adv_state_contract() -> Result<EndpointContract, BpiError> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/danmaku/json-read/adv-state/contract.json"
        ))
    }

    fn query_map(params: [(&'static str, String); 2]) -> BTreeMap<String, String> {
        params
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect()
    }

    #[test]
    fn danmaku_send_params_rejects_blank_message() {
        let err =
            DanmakuSendParams::new(Cid::new(413195701).expect("cid is valid"), "   ").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "msg", .. }
        ));
    }

    #[test]
    fn danmaku_adv_state_params_serializes_query() -> Result<(), BpiError> {
        let params = DanmakuAdvStateParams::new(Cid::new(TEST_CID)?);

        assert_eq!(
            params.query_pairs(),
            [("cid", TEST_CID.to_string()), ("mode", "sp".to_string())]
        );
        Ok(())
    }

    #[test]
    fn danmaku_adv_state_contract_matches_endpoint_request() -> Result<(), BpiError> {
        let contract = adv_state_contract()?;
        let params = DanmakuAdvStateParams::new(Cid::new(TEST_CID)?);

        assert_eq!(contract.name, "danmaku.adv.state");
        assert_eq!(contract.request.method, HttpMethod::Get);
        assert_eq!(
            contract.request.url.as_str(),
            "https://api.bilibili.com/x/dm/adv/state"
        );
        assert_eq!(query_map(params.query_pairs()), contract.request.query);
        Ok(())
    }

    #[test]
    fn danmaku_adv_state_response_fixtures_parse_declared_model() -> Result<(), BpiError> {
        let normal = ApiEnvelope::<DanmakuAdvState>::from_slice(include_bytes!(
            "../../tests/contracts/danmaku/json-read/adv-state/responses/normal.success.json"
        ))?
        .into_payload()?;
        assert!(normal.accept);
        assert_eq!(normal.coins, 2);
        assert_eq!(normal.confirm, 0);
        assert!(!normal.has_buy);

        let vip = ApiEnvelope::<DanmakuAdvState>::from_slice(include_bytes!(
            "../../tests/contracts/danmaku/json-read/adv-state/responses/vip.success.json"
        ))?
        .into_payload()?;
        assert!(vip.accept);
        assert_eq!(vip.coins, 2);
        assert_eq!(vip.confirm, 1);
        assert!(vip.has_buy);
        Ok(())
    }

    #[test]
    fn danmaku_adv_state_anonymous_fixture_records_login_error() -> Result<(), BpiError> {
        let err = ApiEnvelope::<serde_json::Value>::from_slice(include_bytes!(
            "../../tests/contracts/danmaku/json-read/adv-state/responses/anonymous.requires_login.json"
        ))
        .and_then(ApiEnvelope::ensure_success)
        .unwrap_err();

        assert!(err.requires_login());
        Ok(())
    }
}
