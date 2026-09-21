use std::fmt;
use std::path::Path;
use std::sync::Arc;

use bdl_core::queue::DownloadExportTarget;
use bdl_core::settings::DocumentTreeDirectory;
use bdl_core::{BdlError, BdlResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportOutcome {
    Exported,
    SkippedExisting,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportResult {
    pub outcome: ExportOutcome,
    pub relative_path: String,
    pub document_uri: String,
}

#[derive(Clone)]
pub struct MobileStorage {
    backend: Arc<dyn MobileStorageBackend>,
}

impl fmt::Debug for MobileStorage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MobileStorage")
            .finish_non_exhaustive()
    }
}

impl MobileStorage {
    pub fn unsupported() -> Self {
        Self {
            backend: Arc::new(UnsupportedMobileStorageBackend),
        }
    }

    pub fn from_backend<B>(backend: B) -> Self
    where
        B: MobileStorageBackend + 'static,
    {
        Self {
            backend: Arc::new(backend),
        }
    }

    pub fn pick_document_tree(&self) -> BdlResult<DocumentTreeDirectory> {
        self.backend.pick_document_tree()
    }

    pub fn read_clipboard_text(&self) -> BdlResult<String> {
        self.backend.read_clipboard_text()
    }

    pub fn save_image_to_gallery(
        &self,
        file_name: &str,
        mime_type: &str,
        base64_data: &str,
    ) -> BdlResult<String> {
        self.backend
            .save_image_to_gallery(file_name, mime_type, base64_data)
    }

    pub fn export_file(
        &self,
        source_path: &Path,
        target: &DownloadExportTarget,
    ) -> BdlResult<ExportResult> {
        self.backend.export_file(source_path, target)
    }

    pub fn open_exported_file(&self, target: &DownloadExportTarget) -> BdlResult<()> {
        self.backend.open_exported_file(target)
    }

    pub fn open_export_directory(&self, target: &DownloadExportTarget) -> BdlResult<()> {
        self.backend.open_export_directory(target)
    }
}

pub trait MobileStorageBackend: Send + Sync {
    fn pick_document_tree(&self) -> BdlResult<DocumentTreeDirectory>;
    fn read_clipboard_text(&self) -> BdlResult<String>;
    fn save_image_to_gallery(
        &self,
        file_name: &str,
        mime_type: &str,
        base64_data: &str,
    ) -> BdlResult<String>;
    fn export_file(
        &self,
        source_path: &Path,
        target: &DownloadExportTarget,
    ) -> BdlResult<ExportResult>;
    fn open_exported_file(&self, target: &DownloadExportTarget) -> BdlResult<()>;
    fn open_export_directory(&self, target: &DownloadExportTarget) -> BdlResult<()>;
}

#[derive(Debug)]
struct UnsupportedMobileStorageBackend;

impl MobileStorageBackend for UnsupportedMobileStorageBackend {
    fn pick_document_tree(&self) -> BdlResult<DocumentTreeDirectory> {
        Err(unsupported_error())
    }

    fn read_clipboard_text(&self) -> BdlResult<String> {
        Err(BdlError::Platform {
            message: "当前平台未接入原生剪贴板读取。".to_owned(),
        })
    }

    fn save_image_to_gallery(
        &self,
        _file_name: &str,
        _mime_type: &str,
        _base64_data: &str,
    ) -> BdlResult<String> {
        Err(BdlError::Platform {
            message: "当前平台未接入系统相册保存。".to_owned(),
        })
    }

    fn export_file(
        &self,
        _source_path: &Path,
        _target: &DownloadExportTarget,
    ) -> BdlResult<ExportResult> {
        Err(unsupported_error())
    }

    fn open_exported_file(&self, _target: &DownloadExportTarget) -> BdlResult<()> {
        Err(unsupported_error())
    }

    fn open_export_directory(&self, _target: &DownloadExportTarget) -> BdlResult<()> {
        Err(unsupported_error())
    }
}

fn unsupported_error() -> BdlError {
    BdlError::Platform {
        message: "当前平台未接入系统文档目录导出。".to_owned(),
    }
}
