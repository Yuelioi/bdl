use std::collections::{BTreeMap, HashMap};
use std::sync::RwLock;

use crate::{BpiError, BpiResult};

const MIXIN_KEY_TAB: [usize; 64] = [
    46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35, 27, 43, 5, 49, 33, 9, 42, 19, 29,
    28, 14, 39, 12, 38, 41, 13, 37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4, 22, 25,
    54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WbiKeys {
    img_key: String,
    sub_key: String,
}

impl WbiKeys {
    pub fn new(img_key: impl Into<String>, sub_key: impl Into<String>) -> BpiResult<Self> {
        let img_key = img_key.into();
        if img_key.is_empty() {
            return Err(BpiError::invalid_parameter(
                "img_key",
                "img_key cannot be empty",
            ));
        }

        let sub_key = sub_key.into();
        if sub_key.is_empty() {
            return Err(BpiError::invalid_parameter(
                "sub_key",
                "sub_key cannot be empty",
            ));
        }

        Ok(Self { img_key, sub_key })
    }

    pub fn from_nav_urls(img_url: &str, sub_url: &str) -> BpiResult<Self> {
        Self::new(extract_key_stem(img_url)?, extract_key_stem(sub_url)?)
    }

    pub fn img_key(&self) -> &str {
        &self.img_key
    }

    pub fn sub_key(&self) -> &str {
        &self.sub_key
    }
}

#[derive(Debug, Default)]
pub struct WbiKeyCache {
    keys: RwLock<HashMap<String, WbiKeys>>,
}

impl WbiKeyCache {
    pub fn get(&self, bucket: &str) -> BpiResult<Option<WbiKeys>> {
        Ok(self
            .keys
            .read()
            .map_err(|_| BpiError::network("WBI key cache lock poisoned"))?
            .get(bucket)
            .cloned())
    }

    pub fn insert(&self, bucket: impl Into<String>, keys: WbiKeys) -> BpiResult<()> {
        self.keys
            .write()
            .map_err(|_| BpiError::network("WBI key cache lock poisoned"))?
            .insert(bucket.into(), keys);
        Ok(())
    }
}

pub fn mixin_key(img_key: &str, sub_key: &str) -> BpiResult<String> {
    let keys = WbiKeys::new(img_key, sub_key)?;
    let combined = format!("{}{}", keys.img_key, keys.sub_key);
    let bytes = combined.as_bytes();

    let key = MIXIN_KEY_TAB
        .iter()
        .filter_map(|&index| bytes.get(index).copied())
        .map(char::from)
        .take(32)
        .collect::<String>();

    if key.len() != 32 {
        return Err(BpiError::invalid_parameter(
            "wbi_keys",
            "combined WBI keys must produce a 32-byte mixin key",
        ));
    }

    Ok(key)
}

pub fn sign_params_at<I, K, V>(
    params: I,
    keys: &WbiKeys,
    timestamp: u64,
) -> BpiResult<Vec<(String, String)>>
where
    I: IntoIterator<Item = (K, V)>,
    K: ToString,
    V: ToString,
{
    let mixin_key = mixin_key(keys.img_key(), keys.sub_key())?;
    let mut params = params
        .into_iter()
        .map(|(key, value)| (key.to_string(), filter_value(&value.to_string())))
        .collect::<BTreeMap<_, _>>();

    params.insert("wts".to_string(), timestamp.to_string());

    let query = params
        .iter()
        .map(|(key, value)| format!("{}={}", url_encode(key), url_encode(value)))
        .collect::<Vec<_>>()
        .join("&");

    let digest = md5::compute(format!("{query}{mixin_key}"));
    params.insert("w_rid".to_string(), format!("{digest:x}"));

    Ok(params.into_iter().collect())
}

fn extract_key_stem(url: &str) -> BpiResult<String> {
    let file_name = url
        .rsplit('/')
        .next()
        .filter(|segment| !segment.is_empty())
        .ok_or_else(|| BpiError::unsupported_response("missing WBI key filename"))?;

    let Some((stem, extension)) = file_name.rsplit_once('.') else {
        return Err(BpiError::unsupported_response(
            "WBI key URL filename must include an extension",
        ));
    };

    if stem.is_empty() || extension.is_empty() {
        return Err(BpiError::unsupported_response("invalid WBI key filename"));
    }

    Ok(stem.to_string())
}

fn filter_value(value: &str) -> String {
    value.chars().filter(|c| !"!'()*".contains(*c)).collect()
}

fn url_encode(value: &str) -> String {
    let mut result = String::new();

    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(char::from(byte));
            }
            b' ' => result.push_str("%20"),
            _ => result.push_str(&format!("%{byte:02X}")),
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BpiError;

    const IMG_KEY: &str = "abcdefghijklmnopqrstuvwxyz123456";
    const SUB_KEY: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ654321";

    #[test]
    fn mixin_key_returns_stable_key_for_fixed_inputs() -> Result<(), BpiError> {
        let key = mixin_key(IMG_KEY, SUB_KEY)?;

        assert_eq!(key, "OPscVixApSk66dND2LfRBjKt43oHmGJn");
        Ok(())
    }

    #[test]
    fn sign_params_at_adds_wts_and_w_rid() -> Result<(), BpiError> {
        let keys = WbiKeys::new(IMG_KEY, SUB_KEY)?;
        let signed = sign_params_at([("foo", "bar")], &keys, 1_700_000_000)?;

        assert!(signed.contains(&("wts".to_string(), "1700000000".to_string())));
        assert!(signed.iter().any(|(key, _)| key == "w_rid"));
        Ok(())
    }

    #[test]
    fn sign_params_at_filters_reserved_value_characters() -> Result<(), BpiError> {
        let keys = WbiKeys::new(IMG_KEY, SUB_KEY)?;
        let signed = sign_params_at([("foo", "value!'()*")], &keys, 1_700_000_000)?;

        assert!(signed.contains(&("foo".to_string(), "value".to_string())));
        Ok(())
    }

    #[test]
    fn sign_params_at_returns_deterministic_sorted_output() -> Result<(), BpiError> {
        let keys = WbiKeys::new(IMG_KEY, SUB_KEY)?;
        let signed = sign_params_at(
            [("foo", "value!'()*"), ("bar", "space value")],
            &keys,
            1_700_000_000,
        )?;

        assert_eq!(
            signed,
            vec![
                ("bar".to_string(), "space value".to_string()),
                ("foo".to_string(), "value".to_string()),
                (
                    "w_rid".to_string(),
                    "e0bbf3c23838f46f9b7b2785d19dd01e".to_string()
                ),
                ("wts".to_string(), "1700000000".to_string()),
            ]
        );
        Ok(())
    }

    #[test]
    fn empty_img_key_returns_invalid_parameter() {
        let err = WbiKeys::new("", SUB_KEY).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "img_key",
                ..
            }
        ));
    }

    #[test]
    fn empty_sub_key_returns_invalid_parameter() {
        let err = WbiKeys::new(IMG_KEY, "").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "sub_key",
                ..
            }
        ));
    }

    #[test]
    fn malformed_nav_urls_return_unsupported_response() {
        let err = WbiKeys::from_nav_urls(
            "https://i0.hdslb.com/bfs/wbi/no-extension",
            "https://i0.hdslb.com/bfs/wbi/sub.png",
        )
        .unwrap_err();

        assert!(matches!(err, BpiError::UnsupportedResponse { .. }));
    }

    #[test]
    fn nav_urls_extract_key_stems() -> Result<(), BpiError> {
        let keys = WbiKeys::from_nav_urls(
            "https://i0.hdslb.com/bfs/wbi/abc123.png",
            "https://i0.hdslb.com/bfs/wbi/def456.png",
        )?;

        assert_eq!(keys.img_key(), "abc123");
        assert_eq!(keys.sub_key(), "def456");
        Ok(())
    }

    #[test]
    fn key_cache_returns_none_before_insert() -> Result<(), BpiError> {
        let cache = WbiKeyCache::default();

        assert!(cache.get("2026-07-02T10")?.is_none());
        Ok(())
    }

    #[test]
    fn key_cache_stores_keys_by_bucket() -> Result<(), BpiError> {
        let cache = WbiKeyCache::default();
        let keys = WbiKeys::new(IMG_KEY, SUB_KEY)?;

        cache.insert("2026-07-02T10", keys.clone())?;

        assert_eq!(cache.get("2026-07-02T10")?, Some(keys));
        Ok(())
    }
}
