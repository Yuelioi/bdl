use crate::BilibiliRequest;
use crate::BpiError;
use crate::BpiResult;
use crate::electric::ElectricClient;
use serde::{Deserialize, Serialize};

const BCOIN_QUICK_PAY_ENDPOINT: &str =
    "https://api.bilibili.com/x/ugcpay/web/v2/trade/elec/pay/quick";

#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct BcoinQuickPayData {
    /// 本用户 mid
    pub mid: i64,
    /// 目标用户 mid
    pub up_mid: i64,
    /// 留言 token（用于添加充电留言）
    pub order_no: String,
    /// 充电贝壳数（字符串）
    pub bp_num: String,
    /// 获得经验数
    pub exp: u32,
    /// 返回结果
    /// - `4`：成功
    /// - `-2`：低于 20 电池下限
    /// - `-4`：B 币不足
    pub status: i32,
    /// 错误信息（默认为空）
    pub msg: String,
}

/// B 币快速充电的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcoinQuickPayParams {
    bp_num: i32,
    is_bp_remains_prior: bool,
    up_mid: i64,
    otype: String,
    oid: i64,
}

impl BcoinQuickPayParams {
    pub fn new(
        bp_num: i32,
        is_bp_remains_prior: bool,
        up_mid: i64,
        otype: impl Into<String>,
        oid: i64,
    ) -> BpiResult<Self> {
        if !(2..=9999).contains(&bp_num) {
            return Err(BpiError::invalid_parameter(
                "bp_num",
                "value must be between 2 and 9999",
            ));
        }
        if up_mid <= 0 {
            return Err(BpiError::invalid_parameter("up_mid", "id must be positive"));
        }
        if oid <= 0 {
            return Err(BpiError::invalid_parameter("oid", "id must be positive"));
        }

        let otype = otype.into();
        if !matches!(otype.as_str(), "up" | "archive") {
            return Err(BpiError::invalid_parameter(
                "otype",
                "value must be 'up' or 'archive'",
            ));
        }

        Ok(Self {
            bp_num,
            is_bp_remains_prior,
            up_mid,
            otype,
            oid,
        })
    }

    fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("bp_num", self.bp_num.to_string()),
            ("is_bp_remains_prior", self.is_bp_remains_prior.to_string()),
            ("up_mid", self.up_mid.to_string()),
            ("otype", self.otype.clone()),
            ("oid", self.oid.to_string()),
            ("csrf", csrf.to_string()),
        ]
    }
}

impl<'a> ElectricClient<'a> {
    /// 执行 B 币快速充电并返回标准 payload 结果。
    pub async fn bcoin_quick_pay(
        &self,
        params: BcoinQuickPayParams,
    ) -> BpiResult<BcoinQuickPayData> {
        let csrf = self.client.csrf()?;

        self.client
            .post(BCOIN_QUICK_PAY_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_payload("electric.bcoin.quick_pay")
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bcoin_quick_pay_params_rejects_invalid_charge_amount() {
        let err = BcoinQuickPayParams::new(1, true, 42, "up", 42).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "bp_num",
                ..
            }
        ));
    }
}
