use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    pub download_dir: Option<String>,
    pub quality: String,
    pub archive_mode: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            download_dir: None,
            quality: "best".to_owned(),
            archive_mode: "fast".to_owned(),
        }
    }
}
