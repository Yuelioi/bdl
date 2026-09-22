//! 漫画用户信息
//!
//! [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/manga/User.md)

use serde::{Deserialize, Serialize};

/// 漫读券信息
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct UserCoupon {
    /// 漫读券id
    #[serde(rename = "ID")]
    pub id: i32,
    /// 漫读券剩余数
    pub remain_amount: i32,
    /// 漫读券总数
    pub total_amount: u32,
}

/// 漫读券信息数据
#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct CouponsData {
    /// 总剩余数量
    pub total_remain_amount: i32,
    /// 用户漫读券列表
    pub user_coupons: Vec<UserCoupon>,
    /// 漫读券信息
    pub coupon_info: CouponInfo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CouponInfo {
    /// 拥有的漫读券数量
    pub remain_coupon: i64,
    /// 拥有的通用券数量
    pub remain_silver: i64,
    /// 拥有的商城优惠券数量
    pub remain_shop_coupon: i64,
}

/// 获取漫读券列表请求参数
#[derive(Debug, Clone, Serialize)]
pub struct GetCouponsRequest {
    /// 页数
    #[serde(rename = "pageNum")]
    pub page_num: i32,

    /// 分页大小，默认20，取值范围`[1,100]`
    #[serde(rename = "pageSize")]
    pub page_size: i32,

    /// 是否未过期
    #[serde(rename = "notExpired", skip_serializing_if = "Option::is_none")]
    pub not_expired: Option<bool>,

    /// 标签类型
    #[serde(rename = "tabType", skip_serializing_if = "Option::is_none")]
    pub tab_type: Option<i32>,

    /// 类型
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiClient, BpiError, BpiResult};

    fn contract() -> BpiResult<EndpointContract> {
        EndpointContract::from_slice(include_bytes!(
            "../../tests/contracts/manga/read-core/coupons/contract.json"
        ))
    }

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_manga_coupons() -> Result<(), Box<BpiError>> {
        let bpi = BpiClient::new().expect("client should build");

        let data = bpi.manga().coupons(1, 20).await?;

        tracing::info!("{:#?}", data);

        Ok(())
    }

    #[test]
    fn manga_coupons_contract_matches_endpoint_request() -> BpiResult<()> {
        let contract = contract()?;

        assert_eq!(contract.name, "manga.coupons");
        assert_eq!(contract.request.method, HttpMethod::Post);
        assert_eq!(
            contract.request.url.as_str(),
            "https://manga.bilibili.com/twirp/user.v1.User/GetCoupons"
        );

        let body = contract
            .request
            .body
            .as_ref()
            .expect("contract should include json body");
        assert_eq!(body["pageNum"], 1);
        assert_eq!(body["pageSize"], 20);
        assert_eq!(body["notExpired"], true);
        assert_eq!(body["tabType"], 1);
        assert_eq!(body["type"], 0);

        let anonymous = &contract.cases[0].response;
        assert_eq!(anonymous.http_status, Some(401));
        assert_eq!(anonymous.api_code, None);
        assert_eq!(anonymous.api_code_text.as_deref(), Some("unauthenticated"));
        assert_eq!(anonymous.error.as_deref(), Some("requires_login"));

        for case in &contract.cases[1..] {
            assert_eq!(case.response.api_code, Some(0));
            assert_eq!(case.response.rust_model.as_deref(), Some("CouponsData"));
        }
        Ok(())
    }

    #[test]
    fn manga_coupons_response_fixtures_parse_declared_models() -> BpiResult<()> {
        let anonymous: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../tests/contracts/manga/read-core/coupons/responses/anonymous.requires_login.json"
        ))?;
        assert_eq!(anonymous["code"], "unauthenticated");
        assert_eq!(anonymous["msg"], "need login");

        let payload = ApiEnvelope::<CouponsData>::from_slice(include_bytes!(
            "../../tests/contracts/manga/read-core/coupons/responses/authenticated.success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.total_remain_amount, 0);
        assert!(payload.user_coupons.is_empty());
        assert_eq!(payload.coupon_info.remain_coupon, 0);
        assert_eq!(payload.coupon_info.remain_silver, 0);
        assert_eq!(payload.coupon_info.remain_shop_coupon, 0);
        Ok(())
    }

    fn local_probe_response(profile: &str) -> Option<serde_json::Value> {
        let path = format!("target/bpi-probe-runs/manga/read-core/coupons/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    #[test]
    fn manga_coupons_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(value) = local_probe_response(profile) else {
                continue;
            };
            let response = value
                .get("response")
                .ok_or_else(|| BpiError::unsupported_response("probe response missing response"))?;
            let body = response
                .get("body")
                .cloned()
                .ok_or_else(|| BpiError::unsupported_response("probe response missing body"))?;

            if profile == "anonymous" {
                assert_eq!(
                    response.get("status").and_then(serde_json::Value::as_u64),
                    Some(401)
                );
                assert_eq!(body["code"], "unauthenticated");
                continue;
            }

            let payload =
                serde_json::from_value::<ApiEnvelope<CouponsData>>(body)?.into_payload()?;
            assert!(payload.total_remain_amount >= 0);
        }
        Ok(())
    }
}
