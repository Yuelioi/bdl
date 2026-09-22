use crate::{BpiError, BpiResult};

/// `/x/vip/web/vip_center/combine` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VipCenterInfoParams {
    build: u32,
}

impl VipCenterInfoParams {
    pub fn new() -> Self {
        Self { build: 0 }
    }

    pub fn with_build(mut self, build: u32) -> Self {
        self.build = build;
        self
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("build", self.build.to_string())]
    }
}

impl Default for VipCenterInfoParams {
    fn default() -> Self {
        Self::new()
    }
}

/// `/x/vip/privilege/receive` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VipPrivilegeReceiveParams {
    typ: u8,
}

impl VipPrivilegeReceiveParams {
    pub fn new(typ: u8) -> BpiResult<Self> {
        if typ == 0 {
            return Err(BpiError::invalid_parameter(
                "type",
                "value must be non-zero",
            ));
        }

        Ok(Self { typ })
    }

    pub(crate) fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![("type", self.typ.to_string()), ("csrf", csrf.to_string())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vip_center_info_params_serializes_default_build() {
        let params = VipCenterInfoParams::new();

        assert_eq!(params.query_pairs(), vec![("build", "0".to_string())]);
    }

    #[test]
    fn vip_center_info_params_serializes_custom_build() {
        let params = VipCenterInfoParams::new().with_build(1);

        assert_eq!(params.query_pairs(), vec![("build", "1".to_string())]);
    }

    #[test]
    fn vip_center_info_params_allows_explicit_zero_build() {
        let params = VipCenterInfoParams::new().with_build(0);

        assert_eq!(params.query_pairs(), vec![("build", "0".to_string())]);
    }

    #[test]
    fn vip_privilege_receive_params_rejects_zero_type() {
        let err = VipPrivilegeReceiveParams::new(0).unwrap_err();

        assert!(matches!(
            err,
            crate::BpiError::InvalidParameter { field: "type", .. }
        ));
    }

    #[test]
    fn vip_privilege_receive_params_serializes_type() -> Result<(), crate::BpiError> {
        let params = VipPrivilegeReceiveParams::new(1)?;

        assert_eq!(
            params.form_pairs("csrf-token"),
            vec![
                ("type", "1".to_string()),
                ("csrf", "csrf-token".to_string()),
            ]
        );
        Ok(())
    }
}
