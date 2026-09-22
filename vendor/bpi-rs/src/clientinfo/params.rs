use std::net::IpAddr;

use crate::{BpiError, BpiResult};

/// `/ip_service/v1/ip_service/get_ip_addr` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ClientInfoIpParams {
    ip: Option<IpAddr>,
}

impl ClientInfoIpParams {
    /// 创建查询调用方当前 IP 地址的参数。
    pub fn new() -> Self {
        Self::default()
    }

    /// 为已验证的 IP 地址创建参数。
    pub fn for_ip(ip: IpAddr) -> Self {
        Self { ip: Some(ip) }
    }

    /// 解析并设置 IPv4 或 IPv6 地址。
    pub fn with_ip_str(mut self, ip: &str) -> BpiResult<Self> {
        self.ip = Some(parse_ip(ip)?);
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        self.ip
            .map(|ip| vec![("ip", ip.to_string())])
            .unwrap_or_default()
    }
}

fn parse_ip(ip: &str) -> BpiResult<IpAddr> {
    ip.trim().parse().map_err(|_| {
        BpiError::invalid_parameter("ip", "value must be a valid IPv4 or IPv6 address")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_info_ip_params_serializes_empty_defaults() {
        let params = ClientInfoIpParams::new();

        assert!(params.query_pairs().is_empty());
    }

    #[test]
    fn client_info_ip_params_serializes_ipv4() -> BpiResult<()> {
        let params = ClientInfoIpParams::new().with_ip_str("8.8.8.8")?;

        assert_eq!(params.query_pairs(), vec![("ip", "8.8.8.8".to_string())]);
        Ok(())
    }

    #[test]
    fn client_info_ip_params_serializes_ipv6() -> BpiResult<()> {
        let params = ClientInfoIpParams::new().with_ip_str("2001:4860:4860::8888")?;

        assert_eq!(
            params.query_pairs(),
            vec![("ip", "2001:4860:4860::8888".to_string())]
        );
        Ok(())
    }

    #[test]
    fn client_info_ip_params_rejects_invalid_ip() {
        let err = ClientInfoIpParams::new()
            .with_ip_str("not-an-ip")
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "ip", .. }
        ));
    }
}
