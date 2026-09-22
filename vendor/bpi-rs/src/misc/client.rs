use std::time::{SystemTime, UNIX_EPOCH};

use crate::misc::MiscB23ShortLinkParams;
use crate::misc::b23tv::ShortLinkData;
use crate::misc::buvid::{Buvid3Data, BuvidData};
use crate::misc::sign::bili_ticket::TicketData;
use crate::sign::bili_ticket::ticket_request_params;
use crate::{BilibiliRequest, BpiClient, BpiError, BpiResult};

const BUVID3_ENDPOINT: &str = "https://api.bilibili.com/x/web-frontend/getbuvid";
const BUVID_ENDPOINT: &str = "https://api.bilibili.com/x/frontend/finger/spi";
const B23_SHORT_LINK_ENDPOINT: &str = "https://api.biliapi.net/x/share/click";
const BILI_TICKET_ENDPOINT: &str =
    "https://api.bilibili.com/bapis/bilibili.api.ticket.v1.Ticket/GenWebTicket";

/// 杂项 API 客户端。
#[derive(Clone, Copy)]
pub struct MiscClient<'a> {
    pub(crate) client: &'a BpiClient,
}

impl<'a> MiscClient<'a> {
    pub(crate) fn new(client: &'a BpiClient) -> Self {
        Self { client }
    }

    #[cfg(test)]
    pub(crate) fn buvid3_endpoint(&self) -> &'static str {
        BUVID3_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn buvid_endpoint(&self) -> &'static str {
        BUVID_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn b23_short_link_endpoint(&self) -> &'static str {
        B23_SHORT_LINK_ENDPOINT
    }

    #[cfg(test)]
    pub(crate) fn bili_ticket_endpoint(&self) -> &'static str {
        BILI_TICKET_ENDPOINT
    }

    /// 获取 Web buvid3 值。
    pub async fn buvid3(&self) -> BpiResult<Buvid3Data> {
        self.client
            .get(BUVID3_ENDPOINT)
            .send_bpi_payload("misc.buvid3")
            .await
    }

    /// 获取 Web buvid3 和 buvid4 值。
    pub async fn buvid(&self) -> BpiResult<BuvidData> {
        self.client
            .get(BUVID_ENDPOINT)
            .send_bpi_payload("misc.buvid")
            .await
    }

    /// 生成 b23.tv 短链接。
    pub async fn b23_short_link(&self, params: MiscB23ShortLinkParams) -> BpiResult<ShortLinkData> {
        let mut data = self
            .client
            .post(B23_SHORT_LINK_ENDPOINT)
            .form(&params.form_pairs())
            .send_bpi_payload::<ShortLinkData>("misc.b23tv.short_link")
            .await?;
        data.extract();
        Ok(data)
    }

    /// 生成 bili_ticket payload。
    pub async fn bili_ticket(&self) -> BpiResult<TicketData> {
        let csrf = self.client.csrf().unwrap_or_default();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|err| BpiError::network(format!("failed to get UNIX timestamp: {err}")))?
            .as_secs();
        let params = ticket_request_params(timestamp, csrf.as_str())?;

        self.client
            .post(BILI_TICKET_ENDPOINT)
            .query(&params)
            .send_bpi_payload("misc.bili_ticket")
            .await
    }

    /// 生成 bili_ticket，并只返回 ticket 字符串。
    pub async fn bili_ticket_string(&self) -> BpiResult<String> {
        self.bili_ticket().await.map(|data| data.ticket)
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;

    use crate::ids::Aid;
    use crate::misc::MiscB23ShortLinkParams;
    use crate::misc::b23tv::ShortLinkData;
    use crate::misc::buvid::{Buvid3Data, BuvidData};
    use crate::misc::sign::bili_ticket::TicketData;
    use crate::probe::contract::HttpMethod;
    use crate::probe::endpoint_contract::EndpointContract;
    use crate::{BpiClient, BpiResult};

    fn assert_buvid3_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<Buvid3Data>>,
    {
    }

    fn assert_buvid_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<BuvidData>>,
    {
    }

    fn assert_b23_short_link_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<ShortLinkData>>,
    {
    }

    fn assert_bili_ticket_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<TicketData>>,
    {
    }

    fn assert_bili_ticket_string_future<F>(_future: F)
    where
        F: Future<Output = BpiResult<String>>,
    {
    }

    fn contract(endpoint: &str) -> BpiResult<EndpointContract> {
        let bytes = match endpoint {
            "buvid3" => {
                include_bytes!("../../tests/contracts/misc/buvid3/contract.json").as_slice()
            }
            "buvid" => include_bytes!("../../tests/contracts/misc/buvid/contract.json").as_slice(),
            "b23-short-link" => {
                include_bytes!("../../tests/contracts/misc/b23tv/short-link/contract.json")
                    .as_slice()
            }
            "bili-ticket" => {
                include_bytes!("../../tests/contracts/misc/sign/bili-ticket/contract.json")
                    .as_slice()
            }
            _ => unreachable!("unknown misc endpoint fixture"),
        };

        EndpointContract::from_slice(bytes)
    }

    #[test]
    fn misc_client_exposes_promoted_endpoint_urls() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let misc = client.misc();

        assert_eq!(
            misc.buvid3_endpoint(),
            "https://api.bilibili.com/x/web-frontend/getbuvid"
        );
        assert_eq!(
            misc.buvid_endpoint(),
            "https://api.bilibili.com/x/frontend/finger/spi"
        );
        assert_eq!(
            misc.b23_short_link_endpoint(),
            "https://api.biliapi.net/x/share/click"
        );
        assert_eq!(
            misc.bili_ticket_endpoint(),
            "https://api.bilibili.com/bapis/bilibili.api.ticket.v1.Ticket/GenWebTicket"
        );
        Ok(())
    }

    #[test]
    fn misc_methods_return_payload_futures() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let misc = client.misc();

        assert_buvid3_future(misc.buvid3());
        assert_buvid_future(misc.buvid());
        assert_b23_short_link_future(
            misc.b23_short_link(MiscB23ShortLinkParams::new(Aid::new(10001)?)),
        );
        assert_bili_ticket_future(misc.bili_ticket());
        assert_bili_ticket_string_future(misc.bili_ticket_string());
        Ok(())
    }

    #[test]
    fn misc_contracts_match_module_client_endpoints() -> BpiResult<()> {
        let client = BpiClient::new()?;
        let misc = client.misc();
        let buvid3 = contract("buvid3")?;
        let buvid = contract("buvid")?;
        let b23_short_link = contract("b23-short-link")?;
        let bili_ticket = contract("bili-ticket")?;

        assert_eq!(buvid3.name, "misc.buvid3");
        assert_eq!(buvid3.request.method, HttpMethod::Get);
        assert_eq!(buvid3.request.url.as_str(), misc.buvid3_endpoint());
        assert!(buvid3.request.query.is_empty());

        assert_eq!(buvid.name, "misc.buvid");
        assert_eq!(buvid.request.method, HttpMethod::Get);
        assert_eq!(buvid.request.url.as_str(), misc.buvid_endpoint());
        assert!(buvid.request.query.is_empty());

        assert_eq!(b23_short_link.name, "misc.b23tv.short_link");
        assert_eq!(b23_short_link.request.method, HttpMethod::Post);
        assert_eq!(
            b23_short_link.request.url.as_str(),
            misc.b23_short_link_endpoint()
        );
        assert!(b23_short_link.request.query.is_empty());

        assert_eq!(bili_ticket.name, "misc.bili_ticket");
        assert_eq!(bili_ticket.request.method, HttpMethod::Post);
        assert_eq!(
            bili_ticket.request.url.as_str(),
            misc.bili_ticket_endpoint()
        );
        assert_eq!(
            bili_ticket.request.query.get("csrf").map(String::as_str),
            Some("${csrf}")
        );
        Ok(())
    }
}
