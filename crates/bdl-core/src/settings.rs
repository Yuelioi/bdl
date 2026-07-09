use serde::{Deserialize, Serialize};

use crate::naming::DEFAULT_NAMING_TEMPLATE;

const LEGACY_DUPLICATE_TITLE_TEMPLATE: &str =
    "{title}/{title} - P{part_index} - {part_title}.{ext}";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub download_dir: Option<String>,
    pub naming_template: String,
    pub quality: String,
    pub archive_mode: String,
    pub output_extension: String,
    pub concurrent_tasks: usize,
    pub retry_count: usize,
    pub auto_refresh_expired_urls: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            download_dir: None,
            naming_template: DEFAULT_NAMING_TEMPLATE.to_owned(),
            quality: "best".to_owned(),
            archive_mode: "fast".to_owned(),
            output_extension: "mp4".to_owned(),
            concurrent_tasks: 1,
            retry_count: 3,
            auto_refresh_expired_urls: true,
        }
    }
}

impl AppSettings {
    pub fn normalized(mut self) -> Self {
        if self.naming_template.trim().is_empty()
            || self.naming_template.trim() == LEGACY_DUPLICATE_TITLE_TEMPLATE
        {
            self.naming_template = DEFAULT_NAMING_TEMPLATE.to_owned();
        }

        self
    }
}
