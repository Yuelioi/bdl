use crate::manga::clockin::ClockInInfoData;
use crate::manga::point_shop::{Product, UserPointData};
use crate::manga::season::SeasonInfoData;
use crate::manga::user::{CouponsData, GetCouponsRequest};
use crate::{BilibiliRequest, BpiClient, BpiResult};

const SEASON_INFO_ENDPOINT: &str = "https://manga.bilibili.com/twirp/user.v1.Season/GetSeasonInfo";
const CLOCK_IN_INFO_ENDPOINT: &str =
    "https://manga.bilibili.com/twirp/activity.v1.Activity/GetClockInInfo";
const USER_POINT_ENDPOINT: &str =
    "https://manga.bilibili.com/twirp/pointshop.v1.Pointshop/GetUserPoint";
const POINT_PRODUCTS_ENDPOINT: &str =
    "https://manga.bilibili.com/twirp/pointshop.v1.Pointshop/ListProduct";
const COUPONS_ENDPOINT: &str = "https://manga.bilibili.com/twirp/user.v1.User/GetCoupons";

/// 漫画 API 客户端。
#[derive(Clone, Copy)]
pub struct MangaClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> MangaClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn season_info_endpoint(&self) -> &'static str {
        SEASON_INFO_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn clock_in_info_endpoint(&self) -> &'static str {
        CLOCK_IN_INFO_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn user_point_endpoint(&self) -> &'static str {
        USER_POINT_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn point_products_endpoint(&self) -> &'static str {
        POINT_PRODUCTS_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn coupons_endpoint(&self) -> &'static str {
        COUPONS_ENDPOINT
    }

    /// 获取漫画 season 信息。
    pub async fn season_info(&self) -> BpiResult<SeasonInfoData> {
        self.client
            .post(SEASON_INFO_ENDPOINT)
            .send_bpi_payload("manga.season_info")
            .await
    }

    /// 获取漫画签到状态信息。
    pub async fn clock_in_info(&self) -> BpiResult<ClockInInfoData> {
        self.client
            .post(CLOCK_IN_INFO_ENDPOINT)
            .send_bpi_payload("manga.clock_in_info")
            .await
    }

    /// 获取当前漫画积分余额。
    pub async fn user_point(&self) -> BpiResult<UserPointData> {
        self.client
            .post(USER_POINT_ENDPOINT)
            .send_bpi_payload("manga.user_point")
            .await
    }

    /// 列出漫画积分商城商品。
    pub async fn point_products(&self) -> BpiResult<Vec<Product>> {
        self.client
            .post(POINT_PRODUCTS_ENDPOINT)
            .send_bpi_payload("manga.point_products")
            .await
    }

    /// 获取当前账号的漫画券。
    pub async fn coupons(&self, page_num: i32, page_size: i32) -> BpiResult<CouponsData> {
        let params = GetCouponsRequest {
            page_num,
            page_size,
            not_expired: Some(true),
            tab_type: Some(1),
            r#type: Some(0),
        };

        self.client
            .post(COUPONS_ENDPOINT)
            .json(&params)
            .send_bpi_payload("manga.coupons")
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::manga::clockin::ClockInInfoData;
    use crate::manga::point_shop::{Product, UserPointData};
    use crate::manga::season::SeasonInfoData;
    use crate::manga::user::CouponsData;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiResult};

    fn assert_season_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<SeasonInfoData>>,
    {
    }

    fn assert_clock_in_info_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ClockInInfoData>>,
    {
    }

    fn assert_user_point_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<UserPointData>>,
    {
    }

    fn assert_point_products_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<Vec<Product>>>,
    {
    }

    fn assert_coupons_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<CouponsData>>,
    {
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "season-info" => {
                include_bytes!("../../tests/contracts/manga/read-core/season-info/contract.json")
                    .as_slice()
            }
            "clock-in-info" => {
                include_bytes!("../../tests/contracts/manga/read-core/clock-in-info/contract.json")
                    .as_slice()
            }
            "user-point" => {
                include_bytes!("../../tests/contracts/manga/read-core/user-point/contract.json")
                    .as_slice()
            }
            "point-products" => {
                include_bytes!("../../tests/contracts/manga/read-core/point-products/contract.json")
                    .as_slice()
            }
            "coupons" => {
                include_bytes!("../../tests/contracts/manga/read-core/coupons/contract.json")
                    .as_slice()
            }
            _ => unreachable!("unknown manga contract"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn manga_client_exposes_promoted_endpoint_urls() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let manga = client.manga();

        assert_eq!(
            manga.season_info_endpoint(),
            "https://manga.bilibili.com/twirp/user.v1.Season/GetSeasonInfo"
        );
        assert_eq!(
            manga.clock_in_info_endpoint(),
            "https://manga.bilibili.com/twirp/activity.v1.Activity/GetClockInInfo"
        );
        assert_eq!(
            manga.user_point_endpoint(),
            "https://manga.bilibili.com/twirp/pointshop.v1.Pointshop/GetUserPoint"
        );
        assert_eq!(
            manga.point_products_endpoint(),
            "https://manga.bilibili.com/twirp/pointshop.v1.Pointshop/ListProduct"
        );
        assert_eq!(
            manga.coupons_endpoint(),
            "https://manga.bilibili.com/twirp/user.v1.User/GetCoupons"
        );
        Ok(())
    }

    #[test]
    fn manga_methods_return_payload_futures() {
        let client = BpiClient::new().expect("client should build");
        let manga = client.manga();

        assert_season_info_future(manga.season_info());
        assert_clock_in_info_future(manga.clock_in_info());
        assert_user_point_future(manga.user_point());
        assert_point_products_future(manga.point_products());
        assert_coupons_future(manga.coupons(1, 20));
    }

    #[test]
    fn manga_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let manga = client.manga();

        let expectations = [
            (
                "season-info",
                "manga.season_info",
                manga.season_info_endpoint(),
            ),
            (
                "clock-in-info",
                "manga.clock_in_info",
                manga.clock_in_info_endpoint(),
            ),
            (
                "user-point",
                "manga.user_point",
                manga.user_point_endpoint(),
            ),
            (
                "point-products",
                "manga.point_products",
                manga.point_products_endpoint(),
            ),
            ("coupons", "manga.coupons", manga.coupons_endpoint()),
        ];

        for (endpoint, name, url) in expectations {
            let contract = contract(endpoint)?;

            assert_eq!(contract.name, name);
            assert_eq!(contract.request.method, HttpMethod::Post);
            assert_eq!(contract.request.url.as_str(), url);
        }

        let coupons = contract("coupons")?;
        let body = coupons
            .request
            .body
            .as_ref()
            .expect("coupons contract should include json body");
        assert_eq!(body["pageNum"], 1);
        assert_eq!(body["pageSize"], 20);
        assert_eq!(body["notExpired"], true);
        assert_eq!(body["tabType"], 1);
        assert_eq!(body["type"], 0);
        Ok(())
    }
}
