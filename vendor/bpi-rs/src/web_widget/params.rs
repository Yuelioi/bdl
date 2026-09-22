use crate::video::video_zone_v2::VideoPartitionV2;
use crate::{BpiError, BpiResult};

/// `/x/web-show/page/header` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebWidgetHeaderPageParams {
    resource_id: u32,
}

impl Default for WebWidgetHeaderPageParams {
    fn default() -> Self {
        Self { resource_id: 142 }
    }
}

impl WebWidgetHeaderPageParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_resource_id(mut self, resource_id: u32) -> BpiResult<Self> {
        if resource_id == 0 {
            return Err(BpiError::invalid_parameter(
                "resource_id",
                "value must be non-zero",
            ));
        }

        self.resource_id = resource_id;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> [(&'static str, String); 1] {
        [("resource_id", self.resource_id.to_string())]
    }
}

/// `/x/web-show/region/banner` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WebWidgetRegionBannerParams {
    region_id: u32,
}

impl WebWidgetRegionBannerParams {
    pub fn new(region_id: VideoPartitionV2) -> Self {
        Self {
            region_id: region_id.tid(),
        }
    }

    pub(crate) fn query_pairs(&self) -> [(&'static str, String); 1] {
        [("region_id", self.region_id.to_string())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video::video_zone_v2::Douga;

    #[test]
    fn header_page_params_serializes_default_resource() {
        let params = WebWidgetHeaderPageParams::new();

        assert_eq!(params.query_pairs(), [("resource_id", "142".to_string())]);
    }

    #[test]
    fn header_page_params_serializes_custom_resource() -> BpiResult<()> {
        let params = WebWidgetHeaderPageParams::new().with_resource_id(143)?;

        assert_eq!(params.query_pairs(), [("resource_id", "143".to_string())]);
        Ok(())
    }

    #[test]
    fn header_page_params_rejects_zero_resource() {
        let err = WebWidgetHeaderPageParams::new()
            .with_resource_id(0)
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "resource_id",
                ..
            }
        ));
    }

    #[test]
    fn region_banner_params_serializes_partition_id() {
        let partition = VideoPartitionV2::Douga(Douga::Douga);
        let expected_tid = partition.tid().to_string();
        let params = WebWidgetRegionBannerParams::new(partition);

        assert_eq!(params.query_pairs(), [("region_id", expected_tid)]);
    }
}
