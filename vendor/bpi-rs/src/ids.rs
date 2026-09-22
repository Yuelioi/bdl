use std::fmt;
use std::str::FromStr;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{BpiError, BpiResult};

macro_rules! numeric_id {
    ($name:ident, $field:literal, $doc:literal) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u64);

        impl $name {
            /// 创建非零数字 ID。
            pub fn new(value: u64) -> BpiResult<Self> {
                if value == 0 {
                    return Err(BpiError::invalid_parameter($field, "id must be non-zero"));
                }

                Ok(Self(value))
            }

            /// 返回原始数字值。
            pub fn get(self) -> u64 {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl FromStr for $name {
            type Err = BpiError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                let parsed = value
                    .parse::<u64>()
                    .map_err(|_| BpiError::invalid_parameter($field, "id must be numeric"))?;
                Self::new(parsed)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_u64(self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct IdVisitor;

                impl Visitor<'_> for IdVisitor {
                    type Value = u64;

                    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                        formatter.write_str("a non-zero numeric id")
                    }

                    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        Ok(value)
                    }

                    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
                    where
                        E: de::Error,
                    {
                        u64::try_from(value).map_err(|_| E::custom("id must be non-negative"))
                    }
                }

                let value = deserializer.deserialize_any(IdVisitor)?;
                Self::new(value).map_err(de::Error::custom)
            }
        }
    };
}

numeric_id!(Aid, "aid", "Bilibili AV numeric video ID.");
numeric_id!(AudioId, "sid", "Bilibili audio song ID.");
numeric_id!(Cid, "cid", "Bilibili video page/content ID.");
numeric_id!(Mid, "mid", "Bilibili member/user ID.");
numeric_id!(RoomId, "room_id", "Bilibili live room ID.");
numeric_id!(MediaId, "media_id", "Bilibili media ID.");
numeric_id!(SeasonId, "season_id", "Bilibili season ID.");
numeric_id!(EpisodeId, "ep_id", "Bilibili episode ID.");
numeric_id!(NoteId, "note_id", "Bilibili note ID.");
numeric_id!(Cvid, "cvid", "Bilibili note/article CV ID.");

macro_rules! string_id {
    ($name:ident, $field:literal, $doc:literal, $validate:ident) => {
        #[doc = $doc]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            /// 创建已验证的字符串 ID。
            pub fn new(value: impl Into<String>) -> BpiResult<Self> {
                let value = value.into();
                $validate(&value)?;
                Ok(Self(value))
            }

            /// 返回原始字符串值。
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.0.fmt(f)
            }
        }

        impl FromStr for $name {
            type Err = BpiError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::new(value)
            }
        }

        impl Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_str(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(de::Error::custom)
            }
        }
    };
}

string_id!(Bvid, "bvid", "Bilibili BV string video ID.", validate_bvid);
string_id!(
    DynamicId,
    "dynamic_id",
    "Bilibili dynamic feed item ID.",
    validate_dynamic_id
);

fn validate_bvid(value: &str) -> BpiResult<()> {
    if !value.starts_with("BV") {
        return Err(BpiError::invalid_parameter(
            "bvid",
            "bvid must start with 'BV'",
        ));
    }

    if value.len() < 12 || !value.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
        return Err(BpiError::invalid_parameter(
            "bvid",
            "bvid must be at least 12 ASCII alphanumeric characters",
        ));
    }

    Ok(())
}

fn validate_dynamic_id(value: &str) -> BpiResult<()> {
    if value.trim().is_empty() {
        return Err(BpiError::invalid_parameter(
            "dynamic_id",
            "dynamic id cannot be blank",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aid_rejects_zero() {
        let err = Aid::new(0).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "aid", .. }
        ));
    }

    #[test]
    fn aid_displays_numeric_value() -> Result<(), BpiError> {
        let aid = Aid::new(170001)?;

        assert_eq!(aid.to_string(), "170001");
        Ok(())
    }

    #[test]
    fn audio_id_rejects_zero() {
        let err = AudioId::new(0).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "sid", .. }
        ));
    }

    #[test]
    fn audio_id_displays_numeric_value() -> Result<(), BpiError> {
        let sid = AudioId::new(13603)?;

        assert_eq!(sid.to_string(), "13603");
        Ok(())
    }

    #[test]
    fn season_id_rejects_zero() {
        let err = SeasonId::new(0).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "season_id",
                ..
            }
        ));
    }

    #[test]
    fn episode_id_displays_numeric_value() -> Result<(), BpiError> {
        let episode_id = EpisodeId::new(21265)?;

        assert_eq!(episode_id.to_string(), "21265");
        Ok(())
    }

    #[test]
    fn bvid_accepts_valid_value() -> Result<(), BpiError> {
        let bvid: Bvid = "BV1bx411c7ux".parse()?;

        assert_eq!(bvid.as_str(), "BV1bx411c7ux");
        Ok(())
    }

    #[test]
    fn bvid_rejects_invalid_prefix() {
        let err = "av170001".parse::<Bvid>().unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "bvid", .. }
        ));
    }

    #[test]
    fn dynamic_id_rejects_blank_value() {
        let err = "   ".parse::<DynamicId>().unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "dynamic_id",
                ..
            }
        ));
    }

    #[test]
    fn serde_round_trips_numeric_id() -> Result<(), Box<dyn std::error::Error>> {
        let mid = Mid::new(12345)?;

        let json = serde_json::to_string(&mid)?;
        assert_eq!(json, "12345");
        let decoded: Mid = serde_json::from_str(&json)?;
        assert_eq!(decoded, mid);
        Ok(())
    }

    #[test]
    fn serde_round_trips_string_id() -> Result<(), Box<dyn std::error::Error>> {
        let bvid: Bvid = "BV1bx411c7ux".parse()?;

        let json = serde_json::to_string(&bvid)?;
        assert_eq!(json, "\"BV1bx411c7ux\"");
        let decoded: Bvid = serde_json::from_str(&json)?;
        assert_eq!(decoded, bvid);
        Ok(())
    }
}
