use crate::ids::{MediaId, Mid};
use crate::{BpiError, BpiResult};

/// `/x/v3/fav/folder/info` 的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FavFolderInfoParams {
    media_id: MediaId,
}

impl FavFolderInfoParams {
    pub fn new(media_id: MediaId) -> Self {
        Self { media_id }
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![("media_id", self.media_id.to_string())]
    }
}

/// `/x/v3/fav/folder/created/list-all` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavCreatedListParams {
    up_mid: Mid,
    typ: Option<u8>,
    rid: Option<u64>,
    web_location: String,
}

impl FavCreatedListParams {
    pub fn new(up_mid: Mid) -> Self {
        Self {
            up_mid,
            typ: None,
            rid: None,
            web_location: "333.1387".to_string(),
        }
    }

    pub fn with_type(mut self, typ: u8) -> Self {
        self.typ = Some(typ);
        self
    }

    pub fn with_resource_id(mut self, rid: u64) -> BpiResult<Self> {
        self.rid = Some(validate_positive_u64("rid", rid)?);
        Ok(self)
    }

    pub fn with_web_location(mut self, web_location: impl Into<String>) -> BpiResult<Self> {
        self.web_location = normalize_non_blank("web_location", web_location.into())?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = vec![("up_mid", self.up_mid.to_string())];

        if let Some(typ) = self.typ {
            pairs.push(("type", typ.to_string()));
        }
        if let Some(rid) = self.rid {
            pairs.push(("rid", rid.to_string()));
        }
        pairs.push(("web_location", self.web_location.clone()));

        pairs
    }
}

/// `/x/v3/fav/folder/collected/list` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavCollectedListParams {
    up_mid: Mid,
    page: u32,
    page_size: u32,
    platform: String,
}

impl FavCollectedListParams {
    pub fn new(up_mid: Mid) -> Self {
        Self {
            up_mid,
            page: 1,
            page_size: 20,
            platform: "web".to_string(),
        }
    }

    pub fn with_page(mut self, page: u32) -> BpiResult<Self> {
        self.page = validate_positive_u32("pn", page)?;
        Ok(self)
    }

    pub fn with_page_size(mut self, page_size: u32) -> BpiResult<Self> {
        self.page_size = validate_positive_u32("ps", page_size)?;
        Ok(self)
    }

    pub fn with_platform(mut self, platform: impl Into<String>) -> BpiResult<Self> {
        self.platform = normalize_non_blank("platform", platform.into())?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("up_mid", self.up_mid.to_string()),
            ("pn", self.page.to_string()),
            ("ps", self.page_size.to_string()),
            ("platform", self.platform.clone()),
        ]
    }
}

/// `/x/v3/fav/resource/infos` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavResourceInfosParams {
    resources: String,
    platform: String,
}

impl FavResourceInfosParams {
    pub fn new(resources: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            resources: normalize_non_blank("resources", resources.into())?,
            platform: "web".to_string(),
        })
    }

    pub fn with_platform(mut self, platform: impl Into<String>) -> BpiResult<Self> {
        self.platform = normalize_non_blank("platform", platform.into())?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("resources", self.resources.clone()),
            ("platform", self.platform.clone()),
        ]
    }
}

/// `/x/v3/fav/resource/ids` 的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavResourceIdsParams {
    media_id: MediaId,
    platform: String,
}

impl FavResourceIdsParams {
    pub fn new(media_id: MediaId) -> Self {
        Self {
            media_id,
            platform: "web".to_string(),
        }
    }

    pub fn with_platform(mut self, platform: impl Into<String>) -> BpiResult<Self> {
        self.platform = normalize_non_blank("platform", platform.into())?;
        Ok(self)
    }

    pub(crate) fn query_pairs(&self) -> Vec<(&'static str, String)> {
        vec![
            ("media_id", self.media_id.to_string()),
            ("platform", self.platform.clone()),
        ]
    }
}

/// 创建收藏夹的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavFolderAddParams {
    title: String,
    intro: Option<String>,
    privacy: Option<u8>,
    cover: Option<String>,
}

impl FavFolderAddParams {
    pub fn new(title: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            title: normalize_non_blank("title", title.into())?,
            intro: None,
            privacy: None,
            cover: None,
        })
    }

    pub fn intro(mut self, intro: impl Into<String>) -> BpiResult<Self> {
        self.intro = Some(normalize_non_blank("intro", intro.into())?);
        Ok(self)
    }

    pub fn privacy(mut self, privacy: u8) -> BpiResult<Self> {
        self.privacy = Some(validate_privacy(privacy)?);
        Ok(self)
    }

    pub fn cover(mut self, cover: impl Into<String>) -> BpiResult<Self> {
        self.cover = Some(normalize_non_blank("cover", cover.into())?);
        Ok(self)
    }

    pub(crate) fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        let mut pairs = vec![("title", self.title.clone()), ("csrf", csrf.to_string())];
        push_optional(&mut pairs, "intro", &self.intro);
        push_optional_value(&mut pairs, "privacy", self.privacy);
        push_optional(&mut pairs, "cover", &self.cover);
        pairs
    }
}

/// 编辑收藏夹的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavFolderEditParams {
    media_id: MediaId,
    title: String,
    intro: Option<String>,
    privacy: Option<u8>,
    cover: Option<String>,
}

impl FavFolderEditParams {
    pub fn new(media_id: MediaId, title: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            media_id,
            title: normalize_non_blank("title", title.into())?,
            intro: None,
            privacy: None,
            cover: None,
        })
    }

    pub fn intro(mut self, intro: impl Into<String>) -> BpiResult<Self> {
        self.intro = Some(normalize_non_blank("intro", intro.into())?);
        Ok(self)
    }

    pub fn privacy(mut self, privacy: u8) -> BpiResult<Self> {
        self.privacy = Some(validate_privacy(privacy)?);
        Ok(self)
    }

    pub fn cover(mut self, cover: impl Into<String>) -> BpiResult<Self> {
        self.cover = Some(normalize_non_blank("cover", cover.into())?);
        Ok(self)
    }

    pub(crate) fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        let mut pairs = vec![
            ("media_id", self.media_id.to_string()),
            ("title", self.title.clone()),
            ("csrf", csrf.to_string()),
        ];
        push_optional(&mut pairs, "intro", &self.intro);
        push_optional_value(&mut pairs, "privacy", self.privacy);
        push_optional(&mut pairs, "cover", &self.cover);
        pairs
    }
}

/// 删除收藏夹的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavFolderDeleteParams {
    media_ids: Vec<MediaId>,
}

impl FavFolderDeleteParams {
    pub fn new(media_ids: impl IntoIterator<Item = MediaId>) -> BpiResult<Self> {
        let media_ids = media_ids.into_iter().collect::<Vec<_>>();
        if media_ids.is_empty() {
            return Err(BpiError::invalid_parameter(
                "media_ids",
                "at least one media id is required",
            ));
        }

        Ok(Self { media_ids })
    }

    pub(crate) fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            (
                "media_ids",
                self.media_ids
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            ),
            ("csrf", csrf.to_string()),
        ]
    }
}

/// 复制或移动收藏资源的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavResourceTransferParams {
    src_media_id: MediaId,
    tar_media_id: MediaId,
    mid: Mid,
    resources: String,
}

impl FavResourceTransferParams {
    pub fn new(
        src_media_id: MediaId,
        tar_media_id: MediaId,
        mid: Mid,
        resources: impl Into<String>,
    ) -> BpiResult<Self> {
        Ok(Self {
            src_media_id,
            tar_media_id,
            mid,
            resources: normalize_non_blank("resources", resources.into())?,
        })
    }

    pub(crate) fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("src_media_id", self.src_media_id.to_string()),
            ("tar_media_id", self.tar_media_id.to_string()),
            ("mid", self.mid.to_string()),
            ("resources", self.resources.clone()),
            ("platform", "web".to_string()),
            ("csrf", csrf.to_string()),
        ]
    }
}

/// 批量删除收藏资源的参数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FavResourceBatchDeleteParams {
    media_id: MediaId,
    resources: String,
}

impl FavResourceBatchDeleteParams {
    pub fn new(media_id: MediaId, resources: impl Into<String>) -> BpiResult<Self> {
        Ok(Self {
            media_id,
            resources: normalize_non_blank("resources", resources.into())?,
        })
    }

    pub(crate) fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("media_id", self.media_id.to_string()),
            ("resources", self.resources.clone()),
            ("platform", "web".to_string()),
            ("csrf", csrf.to_string()),
        ]
    }
}

/// 清理收藏夹中失效资源的参数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FavResourceCleanParams {
    media_id: MediaId,
}

impl FavResourceCleanParams {
    pub fn new(media_id: MediaId) -> Self {
        Self { media_id }
    }

    pub(crate) fn form_pairs(&self, csrf: &str) -> Vec<(&'static str, String)> {
        vec![
            ("media_id", self.media_id.to_string()),
            ("csrf", csrf.to_string()),
        ]
    }
}

fn validate_positive_u32(field: &'static str, value: u32) -> BpiResult<u32> {
    if value == 0 {
        return Err(BpiError::invalid_parameter(field, "value must be non-zero"));
    }

    Ok(value)
}

fn validate_positive_u64(field: &'static str, value: u64) -> BpiResult<u64> {
    if value == 0 {
        return Err(BpiError::invalid_parameter(field, "value must be non-zero"));
    }

    Ok(value)
}

fn normalize_non_blank(field: &'static str, value: String) -> BpiResult<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(BpiError::invalid_parameter(field, "value cannot be blank"));
    }

    Ok(value)
}

fn validate_privacy(value: u8) -> BpiResult<u8> {
    if matches!(value, 0 | 1) {
        return Ok(value);
    }

    Err(BpiError::invalid_parameter(
        "privacy",
        "value must be 0 or 1",
    ))
}

fn push_optional(
    pairs: &mut Vec<(&'static str, String)>,
    field: &'static str,
    value: &Option<String>,
) {
    if let Some(value) = value {
        pairs.push((field, value.clone()));
    }
}

fn push_optional_value<T>(
    pairs: &mut Vec<(&'static str, String)>,
    field: &'static str,
    value: Option<T>,
) where
    T: ToString,
{
    if let Some(value) = value {
        pairs.push((field, value.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fav_folder_info_params_serializes_media_id() -> BpiResult<()> {
        let params = FavFolderInfoParams::new(MediaId::new(1052622027)?);

        assert_eq!(
            params.query_pairs(),
            vec![("media_id", "1052622027".to_string())]
        );
        Ok(())
    }

    #[test]
    fn fav_created_list_params_serializes_defaults() -> BpiResult<()> {
        let params = FavCreatedListParams::new(Mid::new(7792521)?);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("up_mid", "7792521".to_string()),
                ("web_location", "333.1387".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn fav_created_list_params_serializes_optional_filters() -> BpiResult<()> {
        let params = FavCreatedListParams::new(Mid::new(7792521)?)
            .with_type(2)
            .with_resource_id(170001)?
            .with_web_location("333.999")?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("up_mid", "7792521".to_string()),
                ("type", "2".to_string()),
                ("rid", "170001".to_string()),
                ("web_location", "333.999".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn fav_created_list_params_rejects_zero_resource_id() -> BpiResult<()> {
        let err = FavCreatedListParams::new(Mid::new(7792521)?)
            .with_resource_id(0)
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "rid", .. }
        ));
        Ok(())
    }

    #[test]
    fn fav_collected_list_params_serializes_defaults() -> BpiResult<()> {
        let params = FavCollectedListParams::new(Mid::new(7792521)?);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("up_mid", "7792521".to_string()),
                ("pn", "1".to_string()),
                ("ps", "20".to_string()),
                ("platform", "web".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn fav_collected_list_params_serializes_pagination() -> BpiResult<()> {
        let params = FavCollectedListParams::new(Mid::new(7792521)?)
            .with_page(2)?
            .with_page_size(30)?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("up_mid", "7792521".to_string()),
                ("pn", "2".to_string()),
                ("ps", "30".to_string()),
                ("platform", "web".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn fav_collected_list_params_rejects_zero_page() -> BpiResult<()> {
        let err = FavCollectedListParams::new(Mid::new(7792521)?)
            .with_page(0)
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "pn", .. }
        ));
        Ok(())
    }

    #[test]
    fn fav_resource_infos_params_serializes_defaults() -> BpiResult<()> {
        let params = FavResourceInfosParams::new("371494037:2")?;

        assert_eq!(
            params.query_pairs(),
            vec![
                ("resources", "371494037:2".to_string()),
                ("platform", "web".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn fav_resource_infos_params_rejects_blank_resources() {
        let err = FavResourceInfosParams::new("  ").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "resources",
                ..
            }
        ));
    }

    #[test]
    fn fav_resource_ids_params_serializes_defaults() -> BpiResult<()> {
        let params = FavResourceIdsParams::new(MediaId::new(1052622027)?);

        assert_eq!(
            params.query_pairs(),
            vec![
                ("media_id", "1052622027".to_string()),
                ("platform", "web".to_string())
            ]
        );
        Ok(())
    }

    #[test]
    fn fav_folder_add_params_rejects_blank_title() {
        let err = FavFolderAddParams::new(" ").unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter { field: "title", .. }
        ));
    }

    #[test]
    fn fav_folder_edit_params_rejects_invalid_privacy() -> BpiResult<()> {
        let err = FavFolderEditParams::new(MediaId::new(1052622027)?, "folder")?
            .privacy(2)
            .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "privacy",
                ..
            }
        ));
        Ok(())
    }

    #[test]
    fn fav_folder_delete_params_requires_media_ids() {
        let err = FavFolderDeleteParams::new(Vec::<MediaId>::new()).unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "media_ids",
                ..
            }
        ));
    }

    #[test]
    fn fav_resource_transfer_params_rejects_blank_resources() -> BpiResult<()> {
        let err =
            FavResourceTransferParams::new(MediaId::new(1)?, MediaId::new(2)?, Mid::new(3)?, " ")
                .unwrap_err();

        assert!(matches!(
            err,
            BpiError::InvalidParameter {
                field: "resources",
                ..
            }
        ));
        Ok(())
    }
}
