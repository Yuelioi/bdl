use serde::{Deserialize, Deserializer, Serialize};

/// 等级信息
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct LevelInfo {
    /// 当前等级 0-6
    pub current_level: i32,
    pub current_min: i32,
    pub current_exp: i32,

    /// 使用自定义反序列化函数处理 next_exp
    #[serde(deserialize_with = "deserialize_next_exp")]
    pub next_exp: NextExp,
}

/// 下一等级经验类型
#[derive(Debug, Clone, Serialize, Default)]
pub enum NextExp {
    Value(i32),
    #[default]
    Infinite,
}

/// 自定义反序列化函数处理 next_exp 字段
fn deserialize_next_exp<'de, D>(deserializer: D) -> Result<NextExp, D::Error>
where
    D: Deserializer<'de>,
{
    let s: serde_json::Value = Deserialize::deserialize(deserializer)?;
    match s {
        serde_json::Value::String(ref str_val) if str_val == "--" => Ok(NextExp::Infinite),
        serde_json::Value::Number(num) => Ok(NextExp::Value(num.as_i64().unwrap_or(0) as i32)),
        _ => Ok(NextExp::Infinite),
    }
}
