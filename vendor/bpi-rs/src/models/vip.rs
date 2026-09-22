use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;

/// 会员信息
#[derive(Default, Debug, Clone, Serialize)]
pub struct Vip {
    /// 会员类型 0：无 1：月大会员 2：年度及以上大会员
    /// 别名：vipType | vip_type | type
    pub vip_type: u8,
    /// 会员状态 0：无 1：有
    /// 别名：vipStatus | vip_status | status
    pub vip_status: u8,
    /// 会员过期时间 毫秒时间戳
    /// 别名：vipDueDate | due_date | vip_due_date
    pub vip_due_date: u64,
    /// 会员标签
    pub label: VipLabel,
    /// 会员昵称颜色 颜色码，一般为#FB7299
    pub nickname_color: String,

    /// 支付类型 0：未开启自动续费 1：已开启自动续费
    pub vip_pay_type: Option<u8>,

    /// 大角色类型 1：月度大会员 3：年度大会员 7：十年大会员 15：百年大会员
    pub role: Option<u8>,

    /// 是否为tv会员
    pub is_tv_vip: Option<bool>,
    /// 电视大会员状态 0：未开通
    pub tv_vip_status: Option<u8>,
    /// 电视大会员支付类型
    pub tv_vip_pay_type: Option<u8>,
    /// 电视大会员过期时间 秒级时间戳
    pub tv_due_date: Option<u64>,

    /// 用户mid
    pub mid: Option<u64>,
    /// 昵称
    pub name: Option<String>,
}

/// 会员标签结构体
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct VipLabel {
    /// 会员类型文案（大会员/年度大会员/十年大会员/百年大会员/最强绿鲤鱼）
    #[serde(default)]
    pub text: String,
    /// 会员标签（vip/annual_vip/ten_annual_vip/hundred_annual_vip/fools_day_hundred_annual_vip）
    #[serde(default)]
    pub label_theme: String,
    /// 会员标签文本颜色
    #[serde(default)]
    pub text_color: String,
    /// 样式
    #[serde(default)]
    pub bg_style: u32,
    /// 会员标签背景颜色（颜色码，一般为#FB7299）
    #[serde(default)]
    pub bg_color: String,
}

impl<'de> Deserialize<'de> for Vip {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct VipVisitor;

        impl<'de> Visitor<'de> for VipVisitor {
            type Value = Vip;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a Vip object")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Vip, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut vip_type = None;
                let mut vip_status = None;
                let mut vip_due_date = None;
                let mut label = None;
                let mut nickname_color = None;
                let mut vip_pay_type = None;
                let mut role = None;
                let mut is_tv_vip = None;
                let mut tv_vip_status = None;
                let mut tv_vip_pay_type = None;
                let mut tv_due_date = None;
                let mut mid = None;
                let mut name = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "vipType" | "vip_type" | "type" => {
                            if vip_type.is_some() {
                                return Err(de::Error::duplicate_field("vip_type"));
                            }
                            vip_type = Some(next_u8_value(&mut map, "vip_type")?);
                        }
                        "vip_status" | "vipStatus" | "status" => {
                            if vip_status.is_none() {
                                vip_status = Some(next_u8_value(&mut map, "vip_status")?);
                            } else {
                                let _: serde_json::Value = map.next_value()?;
                            }
                        }
                        "vipDueDate" | "due_date" | "vip_due_date" => {
                            if vip_due_date.is_none() {
                                vip_due_date = Some(next_u64_value(&mut map, "vip_due_date")?);
                            } else {
                                let _: serde_json::Value = map.next_value()?;
                            }
                        }
                        "label" => {
                            if label.is_some() {
                                return Err(de::Error::duplicate_field("label"));
                            }
                            label = Some(map.next_value()?);
                        }
                        "nickname_color" => {
                            if nickname_color.is_some() {
                                return Err(de::Error::duplicate_field("nickname_color"));
                            }
                            nickname_color = Some(map.next_value()?);
                        }
                        "vip_pay_type" => {
                            if vip_pay_type.is_some() {
                                return Err(de::Error::duplicate_field("vip_pay_type"));
                            }
                            vip_pay_type = next_optional_u8_value(&mut map, "vip_pay_type")?;
                        }
                        "role" => {
                            if role.is_some() {
                                return Err(de::Error::duplicate_field("role"));
                            }
                            role = next_optional_u8_value(&mut map, "role")?;
                        }
                        "is_tv_vip" => {
                            if is_tv_vip.is_some() {
                                return Err(de::Error::duplicate_field("is_tv_vip"));
                            }
                            is_tv_vip = Some(map.next_value()?);
                        }
                        "tv_vip_status" => {
                            if tv_vip_status.is_some() {
                                return Err(de::Error::duplicate_field("tv_vip_status"));
                            }
                            tv_vip_status = next_optional_u8_value(&mut map, "tv_vip_status")?;
                        }
                        "tv_vip_pay_type" => {
                            if tv_vip_pay_type.is_some() {
                                return Err(de::Error::duplicate_field("tv_vip_pay_type"));
                            }
                            tv_vip_pay_type = next_optional_u8_value(&mut map, "tv_vip_pay_type")?;
                        }
                        "tv_due_date" => {
                            if tv_due_date.is_some() {
                                return Err(de::Error::duplicate_field("tv_due_date"));
                            }
                            tv_due_date = next_optional_u64_value(&mut map, "tv_due_date")?;
                        }
                        "mid" => {
                            if mid.is_some() {
                                return Err(de::Error::duplicate_field("mid"));
                            }
                            mid = next_optional_u64_value(&mut map, "mid")?;
                        }
                        "name" => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        _ => {
                            // 忽略未知字段
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }

                let vip_type = vip_type.ok_or_else(|| de::Error::missing_field("vip_type"))?;
                let vip_status =
                    vip_status.ok_or_else(|| de::Error::missing_field("vip_status"))?;
                let vip_due_date = vip_due_date.unwrap_or_default();
                let label = label.unwrap_or_default();
                let nickname_color = nickname_color.unwrap_or_default();

                Ok(Vip {
                    vip_type,
                    vip_status,
                    vip_due_date,
                    label,
                    nickname_color,
                    vip_pay_type,
                    role,
                    is_tv_vip,
                    tv_vip_status,
                    tv_vip_pay_type,
                    tv_due_date,
                    mid,
                    name,
                })
            }
        }

        deserializer.deserialize_map(VipVisitor)
    }
}

fn next_u8_value<'de, M>(map: &mut M, field: &'static str) -> Result<u8, M::Error>
where
    M: MapAccess<'de>,
{
    parse_u8_value(map.next_value()?, field)
}

fn next_u64_value<'de, M>(map: &mut M, field: &'static str) -> Result<u64, M::Error>
where
    M: MapAccess<'de>,
{
    parse_u64_value(map.next_value()?, field)
}

fn next_optional_u8_value<'de, M>(map: &mut M, field: &'static str) -> Result<Option<u8>, M::Error>
where
    M: MapAccess<'de>,
{
    match map.next_value()? {
        serde_json::Value::Null => Ok(None),
        value => parse_u8_value(value, field).map(Some),
    }
}

fn next_optional_u64_value<'de, M>(
    map: &mut M,
    field: &'static str,
) -> Result<Option<u64>, M::Error>
where
    M: MapAccess<'de>,
{
    match map.next_value()? {
        serde_json::Value::Null => Ok(None),
        value => parse_u64_value(value, field).map(Some),
    }
}

fn parse_u8_value<E>(value: serde_json::Value, field: &'static str) -> Result<u8, E>
where
    E: de::Error,
{
    let raw = parse_u64_value::<E>(value, field)?;
    u8::try_from(raw).map_err(|_| E::custom(format!("{field} must fit in u8")))
}

fn parse_u64_value<E>(value: serde_json::Value, field: &'static str) -> Result<u64, E>
where
    E: de::Error,
{
    match value {
        serde_json::Value::Number(number) => number
            .as_u64()
            .ok_or_else(|| E::custom(format!("{field} must be a non-negative integer"))),
        serde_json::Value::String(text) => text
            .parse::<u64>()
            .map_err(|_| E::custom(format!("{field} must be a numeric string"))),
        _ => Err(E::custom(format!("{field} must be a string or number"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vip_deserializes_numeric_fields_from_strings() {
        let vip: Vip = serde_json::from_str(
            r##"{
                "type": "2",
                "status": "1",
                "due_date": "1813334400000",
                "label": {
                    "text": "年度大会员",
                    "label_theme": "annual_vip",
                    "text_color": "#FFFFFF",
                    "bg_style": 1,
                    "bg_color": "#FB7299"
                },
                "nickname_color": "#FB7299",
                "role": "3",
                "tv_due_date": "0",
                "tv_vip_pay_type": "0",
                "tv_vip_status": "0",
                "vip_pay_type": "0",
                "mid": "1000001"
            }"##,
        )
        .expect("vip should parse string numeric fields");

        assert_eq!(vip.vip_type, 2);
        assert_eq!(vip.vip_status, 1);
        assert_eq!(vip.vip_due_date, 1813334400000);
        assert_eq!(vip.role, Some(3));
        assert_eq!(vip.mid, Some(1000001));
    }

    #[test]
    fn vip_label_defaults_missing_display_fields() {
        let label: VipLabel = serde_json::from_str(
            r##"{
                "bg_style": 1,
                "text_color": ""
            }"##,
        )
        .expect("vip label should tolerate omitted display fields");

        assert_eq!(label.text, "");
        assert_eq!(label.label_theme, "");
        assert_eq!(label.text_color, "");
        assert_eq!(label.bg_style, 1);
        assert_eq!(label.bg_color, "");
    }

    #[test]
    fn vip_deserializes_compact_relation_payload() {
        let vip: Vip = serde_json::from_str(
            r#"{
                "vipType": 0,
                "vipStatus": 0
            }"#,
        )
        .expect("relation-list vip payload should parse");

        assert_eq!(vip.vip_type, 0);
        assert_eq!(vip.vip_status, 0);
        assert_eq!(vip.vip_due_date, 0);
        assert_eq!(vip.nickname_color, "");
        assert_eq!(vip.label.text, "");
    }
}
