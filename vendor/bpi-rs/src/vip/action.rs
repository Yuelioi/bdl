use crate::BilibiliRequest;
use crate::response::BpiResult;
use crate::vip::VipClient;
use crate::vip::params::VipPrivilegeReceiveParams;
use serde::{Deserialize, Serialize};

const PRIVILEGE_RECEIVE_ENDPOINT: &str = "https://api.bilibili.com/x/vip/privilege/receive";
const EXPERIENCE_ADD_ENDPOINT: &str = "https://api.bilibili.com/x/vip/experience/add";

/// 大会员每日经验返回数据

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VipExperienceData {
    pub r#type: u32,
    /// 是否领取成功
    pub is_grant: bool,
}

impl<'a> VipClient<'a> {
    /// 领取 VIP 特权券并返回标准 payload 结果。
    pub async fn receive_privilege(
        &self,
        params: VipPrivilegeReceiveParams,
    ) -> BpiResult<Option<serde_json::Value>> {
        let csrf = self.client.csrf()?;

        self.client
            .post(PRIVILEGE_RECEIVE_ENDPOINT)
            .form(&params.form_pairs(&csrf))
            .send_bpi_optional_payload("vip.privilege.receive")
            .await
    }

    /// 增加每日 VIP 经验并返回标准 payload 结果。
    pub async fn add_experience(&self) -> BpiResult<VipExperienceData> {
        let csrf = self.client.csrf()?;
        let params = [("csrf", csrf)];
        self.client
            .post(EXPERIENCE_ADD_ENDPOINT)
            .form(&params)
            .send_bpi_payload("vip.experience.add")
            .await
    }
}

#[cfg(test)]
mod tests {}
