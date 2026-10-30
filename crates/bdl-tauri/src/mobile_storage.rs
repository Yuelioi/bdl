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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentSnapshot {
    pub document_uri: String,
    pub display_name: String,
    pub size: u64,
    pub last_modified: u64,
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

    pub fn write_clipboard_text(&self, text: &str) -> BdlResult<()> {
        self.backend.write_clipboard_text(text)
    }

    pub fn save_diagnostic_file(&self, path: &Path) -> BdlResult<bool> {
        self.backend.save_diagnostic_file(path)
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
        let DownloadExportTarget::DocumentTree {
            relative_path,
            duplicate_naming_strategy,
            ..
        } = target;
        self.backend.export_file(source_path, target).map_err(|error| error.with_context(format!(
            "[stage=export_file] source={} relative_path={} naming_strategy={duplicate_naming_strategy:?}",
            source_path.display(), relative_path,
        )))
    }

    pub fn open_exported_file(&self, target: &DownloadExportTarget) -> BdlResult<()> {
        self.backend.open_exported_file(target)
    }

    pub fn open_export_directory(&self, target: &DownloadExportTarget) -> BdlResult<()> {
        self.backend.open_export_directory(target)
    }

    pub fn inspect_document(
        &self,
        tree_uri: &str,
        document_uri: &str,
    ) -> BdlResult<Option<DocumentSnapshot>> {
        self.backend.inspect_document(tree_uri, document_uri)
    }
    pub fn resolve_document(
        &self,
        tree_uri: &str,
        relative_path: &str,
    ) -> BdlResult<Option<DocumentSnapshot>> {
        self.backend.resolve_document(tree_uri, relative_path)
    }
    pub fn delete_document(&self, tree_uri: &str, expected: &DocumentSnapshot) -> BdlResult<()> {
        self.backend.delete_document(tree_uri, expected)
    }
}

pub trait MobileStorageBackend: Send + Sync {
    fn resolve_document(
        &self,
        _tree_uri: &str,
        _relative_path: &str,
    ) -> BdlResult<Option<DocumentSnapshot>> {
        Err(unsupported_error())
    }
    fn inspect_document(
        &self,
        _tree_uri: &str,
        _document_uri: &str,
    ) -> BdlResult<Option<DocumentSnapshot>> {
        Err(unsupported_error())
    }
    fn delete_document(&self, _tree_uri: &str, _expected: &DocumentSnapshot) -> BdlResult<()> {
        Err(unsupported_error())
    }
    fn pick_document_tree(&self) -> BdlResult<DocumentTreeDirectory>;
    fn read_clipboard_text(&self) -> BdlResult<String>;
    fn write_clipboard_text(&self, text: &str) -> BdlResult<()>;
    fn save_diagnostic_file(&self, path: &Path) -> BdlResult<bool>;
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
    fn save_diagnostic_file(&self, _path: &Path) -> BdlResult<bool> {
        Err(unsupported_error())
    }
    fn write_clipboard_text(&self, _text: &str) -> BdlResult<()> {
        Err(unsupported_error())
    }
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
