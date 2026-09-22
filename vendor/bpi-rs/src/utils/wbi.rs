use crate::{BpiClient, BpiError};

impl BpiClient {
    pub(crate) async fn get_wbi_sign2<I, K, V>(
        &self,
        params: I,
    ) -> Result<Vec<(String, String)>, BpiError>
    where
        I: IntoIterator<Item = (K, V)>,
        K: ToString,
        V: ToString,
    {
        self.sign_wbi_params(params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ignore = "legacy live API test; requires explicit BPI_LIVE_TEST review"]
    #[tokio::test]
    async fn test_get_wts_and_rid2() -> Result<(), BpiError> {
        if std::env::var("BPI_LIVE_TEST").ok().as_deref() != Some("1") {
            return Ok(());
        }

        let bpi = BpiClient::new()?;

        let params = vec![
            ("bvid", "BV18x411c74j".to_string()),
            ("cid", "21448".to_string()),
            ("up_mid", "46473".to_string()),
            ("web_location", "0.0".to_string()),
        ];

        let wbi = bpi.get_wbi_sign2(params.clone()).await?;
        assert!(wbi.iter().any(|(key, _)| key == "w_rid"));

        let wbi = bpi.get_wbi_sign2(params).await?;
        assert!(wbi.iter().any(|(key, _)| key == "wts"));
        Ok(())
    }
}
