use std::{collections::HashMap, path::Path};

use bdl_core::naming::DuplicateNamingStrategy;
use bdl_core::queue::DownloadExportTarget;
use bdl_core::settings::DocumentTreeDirectory;
use bdl_core::{BdlError, BdlResult};
use bdl_tauri::mobile_storage::{ExportResult, MobileStorage, MobileStorageBackend};
use serde::{Deserialize, Serialize};
use tauri::{
    Manager, Runtime,
    plugin::{Builder, PermissionState as TauriPermissionState, PluginHandle, TauriPlugin},
};

const PLUGIN_IDENTIFIER: &str = "com.yueli.bdl.storage";
const LEGACY_GALLERY_PERMISSION_ALIAS: &str = "legacyGallery";

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("bdl-mobile-storage")
        .setup(|app, api| {
            let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "StoragePlugin")?;
            app.manage(MobileStorage::from_backend(AndroidStorageBackend {
                handle,
            }));
            Ok(())
        })
        .build()
}

struct AndroidStorageBackend<R: Runtime> {
    handle: PluginHandle<R>,
}

#[derive(Debug, Deserialize)]
struct PickDirectoryResponse {
    tree_uri: String,
    display_name: String,
}

#[derive(Debug, Deserialize)]
struct ClipboardTextResponse {
    text: String,
}

#[derive(Debug, Deserialize)]
struct LegacyGalleryPermissionResponse {
    required: bool,
}

#[derive(Debug, Deserialize)]
struct GallerySaveResponse {
    uri: String,
}

#[derive(Serialize)]
struct PermissionRequestPayload<'a> {
    permissions: [&'a str; 1],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveImagePayload<'a> {
    file_name: &'a str,
    mime_type: &'a str,
    base64_data: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportFilePayload<'a> {
    source_path: &'a str,
    tree_uri: &'a str,
    relative_path: &'a str,
    duplicate_naming_strategy: DuplicateNamingStrategy,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OpenExportPayload<'a> {
    tree_uri: &'a str,
    relative_path: &'a str,
    document_uri: Option<&'a str>,
}

impl<R: Runtime> MobileStorageBackend for AndroidStorageBackend<R> {
    fn pick_document_tree(&self) -> BdlResult<DocumentTreeDirectory> {
        self.handle
            .run_mobile_plugin::<PickDirectoryResponse>("pickDirectory", ())
            .map(|response| DocumentTreeDirectory {
                tree_uri: response.tree_uri,
                display_name: response.display_name,
            })
            .map_err(|error| platform_error("选择保存目录", error))
    }

    fn read_clipboard_text(&self) -> BdlResult<String> {
        self.handle
            .run_mobile_plugin::<ClipboardTextResponse>("readClipboardText", ())
            .map(|response| response.text)
            .map_err(|error| platform_error("读取剪贴板", error))
    }

    fn save_image_to_gallery(
        &self,
        file_name: &str,
        mime_type: &str,
        base64_data: &str,
    ) -> BdlResult<String> {
        let legacy_permission = self
            .handle
            .run_mobile_plugin::<LegacyGalleryPermissionResponse>(
                "legacyGalleryPermissionRequired",
                (),
            )
            .map_err(|error| platform_error("检查相册权限", error))?;
        if legacy_permission.required {
            let states = self
                .handle
                .run_mobile_plugin::<HashMap<String, TauriPermissionState>>(
                    "requestPermissions",
                    PermissionRequestPayload {
                        permissions: [LEGACY_GALLERY_PERMISSION_ALIAS],
                    },
                )
                .map_err(|error| platform_error("请求相册权限", error))?;
            if states.get(LEGACY_GALLERY_PERMISSION_ALIAS) != Some(&TauriPermissionState::Granted) {
                return Err(BdlError::Platform {
                    message: "保存二维码需要照片和媒体写入权限。".to_owned(),
                });
            }
        }

        self.handle
            .run_mobile_plugin::<GallerySaveResponse>(
                "saveImageToGallery",
                SaveImagePayload {
                    file_name,
                    mime_type,
                    base64_data,
                },
            )
            .map(|response| response.uri)
            .map_err(|error| platform_error("保存图片到相册", error))
    }

    fn export_file(
        &self,
        source_path: &Path,
        target: &DownloadExportTarget,
    ) -> BdlResult<ExportResult> {
        let DownloadExportTarget::DocumentTree {
            tree_uri,
            relative_path,
            duplicate_naming_strategy,
            ..
        } = target;
        let source_path = source_path.to_str().ok_or_else(|| BdlError::Platform {
            message: "Android 导出源文件路径不是有效 UTF-8。".to_owned(),
        })?;
        self.handle
            .run_mobile_plugin::<ExportResult>(
                "exportFile",
                ExportFilePayload {
                    source_path,
                    tree_uri,
                    relative_path,
                    duplicate_naming_strategy: *duplicate_naming_strategy,
                },
            )
            .map_err(|error| platform_error("导出文件", error))
    }

    fn open_exported_file(&self, target: &DownloadExportTarget) -> BdlResult<()> {
        let DownloadExportTarget::DocumentTree {
            tree_uri,
            relative_path,
            document_uri,
            ..
        } = target;
        self.handle
            .run_mobile_plugin::<serde_json::Value>(
                "openExportFile",
                OpenExportPayload {
                    tree_uri,
                    relative_path,
                    document_uri: document_uri.as_deref(),
                },
            )
            .map(|_| ())
            .map_err(|error| platform_error("打开导出文件", error))
    }

    fn open_export_directory(&self, target: &DownloadExportTarget) -> BdlResult<()> {
        let DownloadExportTarget::DocumentTree {
            tree_uri,
            relative_path,
            document_uri,
            ..
        } = target;
        self.handle
            .run_mobile_plugin::<serde_json::Value>(
                "openExportDirectory",
                OpenExportPayload {
                    tree_uri,
                    relative_path,
                    document_uri: document_uri.as_deref(),
                },
            )
            .map(|_| ())
            .map_err(|error| platform_error("打开导出目录", error))
    }
}

fn platform_error(action: &str, error: impl std::fmt::Display) -> BdlError {
    BdlError::Platform {
        message: format!("{action}失败：{error}"),
    }
}
