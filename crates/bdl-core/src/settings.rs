use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub download_dir: Option<String>,
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
            quality: "best".to_owned(),
            archive_mode: "fast".to_owned(),
            output_extension: "mp4".to_owned(),
            concurrent_tasks: 1,
            retry_count: 3,
            auto_refresh_expired_urls: true,
        }
    }
}
