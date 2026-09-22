use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::sign::wbi::{WbiKeys, sign_params_at};
use crate::{ApiEnvelope, BpiClient, BpiError};

const WBI_NAV_ENDPOINT: &str = "https://api.bilibili.com/x/web-interface/nav";

#[derive(Deserialize, Serialize)]
struct WbiImgData {
    img_url: String,
    sub_url: String,
}

#[derive(Deserialize, Serialize)]
struct NavData {
    wbi_img: WbiImgData,
}

impl BpiClient {
    pub(crate) async fn sign_wbi_params<I, K, V>(
        &self,
        params: I,
    ) -> Result<Vec<(String, String)>, BpiError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: ToString,
        V: ToString,
    {
        let bucket = current_wbi_cache_bucket();
        let keys = if let Some(keys) = self.wbi_key_cache().get(&bucket)? {
            keys
        } else {
            let keys = self.fetch_wbi_keys().await?;
            self.wbi_key_cache().insert(bucket, keys.clone())?;
            keys
        };

        sign_params_at(params, &keys, current_unix_timestamp()?)
    }

    async fn fetch_wbi_keys(&self) -> Result<WbiKeys, BpiError> {
        let bytes = crate::transport::ReqwestTransport::send_request_builder(
            self.get(WBI_NAV_ENDPOINT),
            "wbi-nav",
        )
        .await?
        .body;

        wbi_keys_from_nav_bytes(&bytes)
    }
}

fn wbi_keys_from_nav_bytes(bytes: &[u8]) -> Result<WbiKeys, BpiError> {
    let resp = ApiEnvelope::<NavData>::from_slice(bytes)?;
    let data = resp
        .data
        .ok_or_else(|| BpiError::parse("获取 wbi 签名失败"))?;

    WbiKeys::from_nav_urls(&data.wbi_img.img_url, &data.wbi_img.sub_url)
}

pub(crate) fn current_wbi_cache_bucket() -> String {
    Local::now().format("%Y-%m-%d %H").to_string()
}

fn current_unix_timestamp() -> Result<u64, BpiError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| BpiError::network(format!("获取时间戳失败: {error}")))
        .map(|duration| duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::{current_wbi_cache_bucket, wbi_keys_from_nav_bytes};
    use crate::sign::wbi::WbiKeys;
    use crate::{BpiClient, BpiError};

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn sign_wbi_params_uses_cached_client_keys() -> Result<(), BpiError> {
        let client = BpiClient::new()?;
        client.wbi_key_cache().insert(
            current_wbi_cache_bucket(),
            WbiKeys::new(
                "abcdefghijklmnopqrstuvwxyz123456",
                "ABCDEFGHIJKLMNOPQRSTUVWXYZ654321",
            )?,
        )?;

        let signed = client.sign_wbi_params([("mid", "1001")]).await?;

        assert!(signed.contains(&("mid".to_string(), "1001".to_string())));
        assert!(signed.iter().any(|(key, _)| key == "wts"));
        assert!(signed.iter().any(|(key, _)| key == "w_rid"));
        Ok(())
    }

    #[test]
    fn wbi_keys_parse_from_anonymous_nav_error_envelope() -> Result<(), BpiError> {
        let keys = wbi_keys_from_nav_bytes(
            br#"{
                "code": -101,
                "message": "not logged in",
                "ttl": 1,
                "data": {
                    "isLogin": false,
                    "wbi_img": {
                        "img_url": "https://i0.hdslb.com/bfs/wbi/7cd084941338484aae1ad9425b84077c.png",
                        "sub_url": "https://i0.hdslb.com/bfs/wbi/4932caff0ff746eab6f01bf08b70ac45.png"
                    }
                }
            }"#,
        )?;

        assert_eq!(keys.img_key(), "7cd084941338484aae1ad9425b84077c");
        assert_eq!(keys.sub_key(), "4932caff0ff746eab6f01bf08b70ac45");
        Ok(())
    }
}
