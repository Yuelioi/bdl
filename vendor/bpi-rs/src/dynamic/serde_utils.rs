use serde::{Deserialize, Deserializer, de};

pub(crate) fn deserialize_u64_from_string_or_number<'de, D>(
    deserializer: D,
) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;

    match value {
        serde_json::Value::Number(number) => number
            .as_u64()
            .ok_or_else(|| de::Error::custom("value must be a non-negative integer")),
        serde_json::Value::String(text) => text
            .parse::<u64>()
            .map_err(|_| de::Error::custom("value must be a numeric string")),
        _ => Err(de::Error::custom("value must be a string or number")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize)]
    struct Fixture {
        #[serde(deserialize_with = "deserialize_u64_from_string_or_number")]
        value: u64,
    }

    #[test]
    fn deserializes_number() {
        let fixture: Fixture = serde_json::from_str(r#"{ "value": 42 }"#).unwrap();

        assert_eq!(fixture.value, 42);
    }

    #[test]
    fn deserializes_numeric_string() {
        let fixture: Fixture = serde_json::from_str(r#"{ "value": "42" }"#).unwrap();

        assert_eq!(fixture.value, 42);
    }
}
