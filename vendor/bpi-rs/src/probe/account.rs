use std::path::Path;

use config::{Config, File};
use serde::Deserialize;

use crate::{Account, BpiError, BpiResult};

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawProbeConfig {
    pub vip: Option<ProbeAccountProfile>,
    pub normal: Option<ProbeAccountProfile>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProbeAccountProfile {
    pub bili_jct: String,
    pub dede_user_id: String,
    #[serde(default)]
    pub dede_user_id_ckmd5: String,
    pub sessdata: String,
    pub buvid3: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeAccountProfileName {
    Vip,
    Normal,
}

impl ProbeAccountProfileName {
    fn parse(value: &str) -> BpiResult<Self> {
        match value {
            "vip" => Ok(Self::Vip),
            "normal" => Ok(Self::Normal),
            _ => Err(BpiError::invalid_parameter(
                "profile",
                "supported probe account profiles are vip and normal",
            )),
        }
    }
}

impl RawProbeConfig {
    pub fn load(path: impl AsRef<Path>) -> BpiResult<Self> {
        let path = path.as_ref();
        let file = path.to_string_lossy();
        Config::builder()
            .add_source(File::with_name(&file))
            .build()
            .and_then(Config::try_deserialize)
            .map_err(|err| BpiError::parse(format!("failed to load probe account config: {err}")))
    }

    pub fn account(&self, profile: &str) -> BpiResult<Option<Account>> {
        let profile = ProbeAccountProfileName::parse(profile)?;
        Ok(self.profile(profile).map(|profile| profile.to_account()))
    }

    fn profile(&self, profile: ProbeAccountProfileName) -> Option<ProbeAccountProfile> {
        match profile {
            ProbeAccountProfileName::Vip => self.vip.clone(),
            ProbeAccountProfileName::Normal => self.normal.clone(),
        }
    }
}

impl ProbeAccountProfile {
    fn to_account(&self) -> Account {
        Account::new(
            self.dede_user_id.clone(),
            self.sessdata.clone(),
            self.bili_jct.clone(),
            self.buvid3.clone(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_profile_maps_vip_fields() {
        let raw = RawProbeConfig {
            vip: Some(ProbeAccountProfile {
                bili_jct: "csrf".to_string(),
                dede_user_id: "42".to_string(),
                dede_user_id_ckmd5: "ck".to_string(),
                sessdata: "session".to_string(),
                buvid3: "buvid".to_string(),
            }),
            normal: None,
        };

        let account = raw
            .account("vip")
            .expect("profile name should be valid")
            .expect("vip account should exist");

        assert_eq!(account.dede_user_id, "42");
        assert_eq!(account.bili_jct, "csrf");
    }

    #[test]
    fn account_profile_maps_normal_fields() {
        let raw = RawProbeConfig {
            vip: None,
            normal: Some(ProbeAccountProfile {
                bili_jct: "csrf2".to_string(),
                dede_user_id: "43".to_string(),
                dede_user_id_ckmd5: "ck2".to_string(),
                sessdata: "session2".to_string(),
                buvid3: "buvid2".to_string(),
            }),
        };

        let account = raw
            .account("normal")
            .expect("profile name should be valid")
            .expect("normal account should exist");

        assert_eq!(account.dede_user_id, "43");
        assert_eq!(account.bili_jct, "csrf2");
        assert_eq!(account.buvid3, "buvid2");
    }

    #[test]
    fn account_profile_returns_none_for_unknown_profile() {
        let raw = RawProbeConfig {
            vip: None,
            normal: None,
        };

        assert!(
            raw.account("normal")
                .expect("profile name should be valid")
                .is_none()
        );
    }

    #[test]
    fn account_profile_rejects_arbitrary_names() {
        let raw = RawProbeConfig::default();

        let err = raw.account("account2").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "profile",
                ..
            }
        ));
    }

    #[test]
    fn account_config_deserializes_root_profile_sections() -> Result<(), BpiError> {
        let raw: RawProbeConfig = Config::builder()
            .add_source(File::from_str(
                r#"
                [vip]
                bili_jct = "vip-csrf"
                dede_user_id = "42"
                dede_user_id_ckmd5 = "vip-ck"
                sessdata = "vip-session"
                buvid3 = "vip-buvid"

                [normal]
                bili_jct = "normal-csrf"
                dede_user_id = "43"
                dede_user_id_ckmd5 = "normal-ck"
                sessdata = "normal-session"
                buvid3 = "normal-buvid"
                "#,
                config::FileFormat::Toml,
            ))
            .build()
            .and_then(Config::try_deserialize)
            .map_err(|err| BpiError::parse(err.to_string()))?;

        assert_eq!(
            raw.account("vip")?
                .expect("vip profile should exist")
                .dede_user_id,
            "42"
        );
        assert_eq!(
            raw.account("normal")?
                .expect("normal profile should exist")
                .dede_user_id,
            "43"
        );
        Ok(())
    }

    #[test]
    fn account_config_rejects_legacy_flat_suffixes() -> Result<(), BpiError> {
        let raw: RawProbeConfig = Config::builder()
            .add_source(File::from_str(
                r#"
                bili_jct_vip = "vip-csrf"
                dede_user_id_vip = 42
                dede_user_id_ckmd5_vip = "vip-ck"
                sessdata_vip = "vip-session"
                buvid3_vip = "vip-buvid"

                bili_jct_normal = "normal-csrf"
                dede_user_id_normal = 43
                sessdata_normal = "normal-session"
                buvid3_normal = "normal-buvid"
                "#,
                config::FileFormat::Toml,
            ))
            .build()
            .and_then(Config::try_deserialize)
            .map_err(|err| BpiError::parse(err.to_string()))?;

        assert!(raw.account("vip")?.is_none());
        assert!(raw.account("normal")?.is_none());
        Ok(())
    }

    #[test]
    fn account_config_rejects_legacy_flat_fields_as_vip_fallback() -> Result<(), BpiError> {
        let raw: RawProbeConfig = Config::builder()
            .add_source(File::from_str(
                r#"
                bili_jct = "legacy-csrf"
                dede_user_id = 42
                dede_user_id_ckmd5 = "legacy-ck"
                sessdata = "legacy-session"
                buvid3 = "legacy-buvid"
                "#,
                config::FileFormat::Toml,
            ))
            .build()
            .and_then(Config::try_deserialize)
            .map_err(|err| BpiError::parse(err.to_string()))?;

        assert!(raw.account("vip")?.is_none());
        Ok(())
    }

    #[test]
    fn account_config_does_not_treat_numbered_fields_as_normal_profile() -> Result<(), BpiError> {
        let raw: RawProbeConfig = Config::builder()
            .add_source(File::from_str(
                r#"
                bili_jct2 = "normal-csrf"
                dede_user_id2 = "43"
                sessdata2 = "normal-session"
                buvid3_2 = "normal-buvid"
                "#,
                config::FileFormat::Toml,
            ))
            .build()
            .and_then(Config::try_deserialize)
            .map_err(|err| BpiError::parse(err.to_string()))?;

        assert!(raw.account("normal")?.is_none());
        Ok(())
    }
}
