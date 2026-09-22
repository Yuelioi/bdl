use serde::{Deserialize, Serialize};

/// 通用账户字段
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Account {
    /// 用户mid
    pub mid: u64,
    /// 昵称
    pub name: String,
    /// 性别（男/女/保密）
    pub sex: String,
    /// 头像url
    pub face: String,
    /// 签名
    pub sign: String,
    /// 等级
    pub rank: u32,
    /// 生日（秒时间戳）。部分接口会用负值表示 1970 年前的日期。
    pub birthday: i64,
    /// 未知字段
    pub is_fake_account: u32,
    /// 是否注销（0：正常，1：注销）
    pub is_deleted: u32,
    /// 是否注册审核（0：正常，1：审核）
    pub in_reg_audit: u32,
    /// 是否转正（0：未转正，1：正式会员）
    pub is_senior_member: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_deserializes_negative_birthday_timestamp() {
        let account: Account = serde_json::from_str(
            r#"{
                "mid": 1000001,
                "name": "sanitized",
                "sex": "保密",
                "face": "",
                "sign": "",
                "rank": 10000,
                "birthday": -1,
                "is_fake_account": 0,
                "is_deleted": 0,
                "in_reg_audit": 0,
                "is_senior_member": 0
            }"#,
        )
        .expect("account should parse negative birthday timestamps");

        assert_eq!(account.birthday, -1);
    }
}
