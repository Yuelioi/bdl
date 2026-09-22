// 积分商城
//
// [查看 API 文档](https://github.com/Yuelioi/bilibili-API-collect/tree/cfc5fddcc8a94b74d91970bb5b4eaeb349addc47/docs/manga/point_shop.md)

// ================= 数据结构 =================

use crate::BpiError;
use crate::manga::MangaClient;
use crate::request::send_bpi_envelope;
use crate::response::BpiResponse;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct UserPointData {
    /// 用户当前持有的点数
    pub point: String,
}

pub type UserPointResponse = BpiResponse<UserPointData>;

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct ProductLimit {
    /// 限制类型
    #[serde(rename = "type")]
    pub limit_type: i32,
    /// 限制ID
    pub id: i64,
    /// 限制标题
    pub title: String,
}

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Product {
    /// 物品ID
    pub id: i64,
    /// 物品类型
    pub r#type: i32,
    /// 物品名称
    pub title: String,
    /// 显示的图像
    pub image: String,
    /// 库存总量
    pub amount: i32,
    /// 兑换所需点数（原价）
    pub cost: i32,
    /// 兑换所需点数（现价）
    pub real_cost: i32,
    /// 库存剩余数
    pub remain_amount: i32,
    /// 相关漫画ID
    pub comic_id: i64,
    /// 限定使用范围（漫画）
    pub limits: Vec<ProductLimit>,
    /// 折扣
    pub discount: i32,
    /// 产品类型
    pub product_type: i32,
    /// 挂件URL
    pub pendant_url: String,
    /// 挂件过期时间
    pub pendant_expire: i32,
    /// 兑换次数限制
    pub exchange_limit: i32,
    /// 地址截止时间
    pub address_deadline: String,
    /// 活动类型
    pub act_type: i32,
    /// 是否兑换过该物品
    pub has_exchanged: bool,
    /// 主优惠券截止时间
    pub main_coupon_deadline: String,
    /// 截止时间
    pub deadline: String,
    /// 点数
    pub point: String,
}

pub type ProductListResponse = BpiResponse<Vec<Product>>;

#[derive(Debug, Clone, Serialize)]
pub struct ExchangeRequest {
    /// 物品ID
    pub product_id: String,
    /// 兑换个数
    pub product_num: i32,
    /// 物品所需点数（现价）
    pub point: i32,
}

pub type ExchangeResponse = BpiResponse<serde_json::Value>;

// ================= 实现 =================

impl<'a> MangaClient<'a> {
    /// 兑换物品
    ///
    /// # 文档
    /// [查看API文档](https://github.com/SocialSisterYi/bilibili-API-collect/tree/master/docs/manga)
    ///
    /// # 参数
    ///
    /// | 名称 | 类型 | 说明 |
    /// | ---- | ---- | ---- |
    /// | `product_id` | i64 | 物品 ID |
    /// | `product_num` | i32 | 兑换数量 |
    /// | `point` | i32 | 现价所需点数 |
    pub async fn manga_point_exchange(
        &self,
        product_id: i64,
        product_num: i32,
        point: i32,
    ) -> Result<ExchangeResponse, BpiError> {
        let req = ExchangeRequest {
            product_id: product_id.to_string(),
            product_num,
            point,
        };

        let request = self
            .client
            .post("https://manga.bilibili.com/twirp/pointshop.v1.Pointshop/Exchange")
            .form(&req);

        send_bpi_envelope(request, "兑换物品").await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{ApiEnvelope, BpiError, BpiResult};

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "user-point" => {
                include_bytes!("../../tests/contracts/manga/read-core/user-point/contract.json")
                    .as_slice()
            }
            "point-products" => {
                include_bytes!("../../tests/contracts/manga/read-core/point-products/contract.json")
                    .as_slice()
            }
            _ => {
                return Err(BpiError::invalid_parameter(
                    "endpoint",
                    "unknown manga point shop contract",
                ));
            }
        };

        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn manga_point_shop_contracts_match_endpoint_requests() -> BpiResult<()> {
        let user_point = contract("user-point")?;
        let point_products = contract("point-products")?;

        assert_eq!(user_point.name, "manga.user_point");
        assert_eq!(user_point.request.method, HttpMethod::Post);
        assert_eq!(
            user_point.request.url.as_str(),
            "https://manga.bilibili.com/twirp/pointshop.v1.Pointshop/GetUserPoint"
        );
        assert_eq!(
            user_point.cases[0].response.rust_model.as_deref(),
            Some("UserPointData")
        );

        assert_eq!(point_products.name, "manga.point_products");
        assert_eq!(point_products.request.method, HttpMethod::Post);
        assert_eq!(
            point_products.request.url.as_str(),
            "https://manga.bilibili.com/twirp/pointshop.v1.Pointshop/ListProduct"
        );
        assert_eq!(
            point_products.cases[0].response.rust_model.as_deref(),
            Some("Vec<Product>")
        );
        Ok(())
    }

    #[test]
    fn manga_user_point_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<UserPointData>::from_slice(include_bytes!(
            "../../tests/contracts/manga/read-core/user-point/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.point, "0");
        Ok(())
    }

    #[test]
    fn manga_point_products_response_fixture_parses_declared_model() -> BpiResult<()> {
        let payload = ApiEnvelope::<Vec<Product>>::from_slice(include_bytes!(
            "../../tests/contracts/manga/read-core/point-products/responses/success.json"
        ))?
        .into_payload()?;

        assert_eq!(payload.len(), 1);
        assert_eq!(payload[0].id, 1938);
        assert_eq!(payload[0].point, "0");
        Ok(())
    }

    fn local_probe_body(endpoint: &str, profile: &str) -> Option<serde_json::Value> {
        let path =
            format!("target/bpi-probe-runs/manga/read-core/{endpoint}/{profile}.response.json");
        let bytes = std::fs::read(path).ok()?;
        let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        value
            .get("response")
            .and_then(|response| response.get("body"))
            .cloned()
    }

    #[test]
    fn manga_user_point_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body("user-point", profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<UserPointData>>(body)?.into_payload()?;

            assert!(payload.point.parse::<i64>().is_ok());
        }
        Ok(())
    }

    #[test]
    fn manga_point_products_model_matches_local_probe_outputs_when_available() -> BpiResult<()> {
        for profile in ["anonymous", "normal", "vip"] {
            let Some(body) = local_probe_body("point-products", profile) else {
                continue;
            };
            let payload =
                serde_json::from_value::<ApiEnvelope<Vec<Product>>>(body)?.into_payload()?;

            assert!(!payload.is_empty());
        }
        Ok(())
    }
}
