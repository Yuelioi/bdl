//! Exact, previewed file deletion. Queue records outlive their files, never the reverse.
use crate::{mobile_storage::DocumentSnapshot, state::AppState};
use bdl_core::{
    BdlError, BdlResult,
    fetcher::{segment_path_for, state_path_for, state_temp_path_for},
    queue::{DownloadExportTarget, DownloadTask, TaskStatus},
};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::ErrorKind,
    path::{Component, Path, PathBuf},
    time::{Duration, Instant, SystemTime},
};

fn error(message: impl Into<String>) -> BdlError {
    BdlError::Planning {
        message: message.into(),
    }
}
fn lock_error() -> BdlError {
    error("文件清理状态锁不可用。")
}

fn file_io_error(operation: &str, path: &Path, cause: std::io::Error) -> BdlError {
    let code = if cause.kind() == ErrorKind::PermissionDenied {
        "file_access_denied"
    } else {
        "file_io_error"
    };
    #[cfg(windows)]
    let code = match cause.raw_os_error() {
        Some(32 | 33) => "file_in_use",
        Some(19) => "file_read_only",
        Some(21 | 53 | 67 | 1231) => "storage_unavailable",
        Some(112) => "disk_full",
        _ => code,
    };
    error(format!(
        "[stage=file_cleanup code={code}] {operation}失败：{}；os_error={}；{cause}",
        path.display(),
        cause
            .raw_os_error()
            .map_or_else(|| "none".to_owned(), |code| code.to_string())
    ))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileStamp {
    size: u64,
    modified: SystemTime,
    identity: (u64, u64),
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct LocalFile {
    path: PathBuf,
    stamp: FileStamp,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct DocumentFile {
    tree_uri: String,
    snapshot: DocumentSnapshot,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Plan {
    task_ids: Vec<String>,
    files: Vec<LocalFile>,
    documents: Vec<DocumentFile>,
    preserved: Vec<String>,
    roots: Vec<PathBuf>,
}
#[derive(Default)]
pub(crate) struct PreviewStore {
    entries: HashMap<String, (Instant, Plan)>,
}
#[derive(Serialize, Clone, Debug)]
pub struct RemovalPreview {
    pub token: String,
    pub task_ids: Vec<String>,
    pub files: Vec<String>,
    pub file_count: usize,
    pub total_bytes: u64,
    pub preserved: Vec<String>,
    pub roots: Vec<String>,
}
impl RemovalPreview {
    fn from_plan(token: String, plan: &Plan) -> Self {
        let files = plan
            .files
            .iter()
            .map(|f| f.path.to_string_lossy().into_owned())
            .chain(
                plan.documents
                    .iter()
                    .map(|f| f.snapshot.document_uri.clone()),
            )
            .collect::<Vec<_>>();
        Self {
            token,
            task_ids: plan.task_ids.clone(),
            file_count: files.len(),
            files,
            total_bytes: plan.files.iter().map(|f| f.stamp.size).sum::<u64>()
                + plan.documents.iter().map(|f| f.snapshot.size).sum::<u64>(),
            preserved: plan.preserved.clone(),
            roots: plan
                .roots
                .iter()
                .map(|p| p.to_string_lossy().into_owned())
                .collect(),
        }
    }
}

fn absolute(path: &Path) -> BdlResult<PathBuf> {
    if path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(error("拒绝清理包含上级目录的路径。"));
    }
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    Ok(path
        .components()
        .filter(|c| !matches!(c, Component::CurDir))
        .collect())
}
fn key(path: &Path) -> BdlResult<String> {
    let path = absolute(path)?;
    let path = fs::canonicalize(&path).unwrap_or(path);
    let value = path.to_string_lossy().into_owned();
    #[cfg(windows)]
    {
        Ok(value.to_lowercase())
    }
    #[cfg(not(windows))]
    {
        Ok(value)
    }
}
fn is_link(metadata: &fs::Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 != 0
    }
    #[cfg(not(windows))]
    {
        false
    }
}
fn validate_ancestors(path: &Path) -> BdlResult<()> {
    for ancestor in path.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if is_link(&metadata) => {
                // macOS installs these root-owned aliases itself. Temporary
                // paths commonly use /var, so rejecting them blocks ordinary
                // file cleanup. Only the exact platform alias is permitted.
                #[cfg(target_os = "macos")]
                {
                    use std::os::unix::fs::MetadataExt;
                    let expected = match ancestor.to_str() {
                        Some("/var") => Some(Path::new("/private/var")),
                        Some("/tmp") => Some(Path::new("/private/tmp")),
                        _ => None,
                    };
                    if let Some(expected) = expected {
                        let destination = fs::read_link(ancestor)?;
                        let destination = if destination.is_absolute() {
                            destination
                        } else {
                            Path::new("/").join(destination)
                        };
                        if metadata.uid() == 0 && destination == expected {
                            continue;
                        }
                    }
                }
                // Android's root-owned, platform-managed user-zero alias is part
                // of app_data_dir. Do not generalize this to user-created links.
                #[cfg(target_os = "android")]
                if ancestor == Path::new("/data/user/0")
                    && fs::read_link(ancestor)? == Path::new("/data/data")
                {
                    use std::os::unix::fs::MetadataExt;
                    if metadata.uid() == 0 {
                        continue;
                    }
                }
                return Err(error(format!(
                    "拒绝清理符号链接或目录联接：{}",
                    ancestor.display()
                )));
            }
            Ok(_) => {}
            Err(e) if e.kind() == ErrorKind::NotFound =>
            {
                #[cfg(windows)]
                if ancestor.parent().is_none() {
                    return Err(error(format!(
                        "输出磁盘或共享目录不可访问，尚未清理文件：{}",
                        ancestor.display()
                    )));
                }
            }
            Err(e) => return Err(file_io_error("读取路径信息", ancestor, e)),
        }
    }
    Ok(())
}
fn stamp(path: &Path) -> BdlResult<Option<FileStamp>> {
    validate_ancestors(path)?;
    let metadata = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(file_io_error("读取文件信息", path, e)),
    };
    if !metadata.is_file() {
        return Err(error(format!("只允许清理普通文件：{}", path.display())));
    }
    #[cfg(unix)]
    let identity = {
        use std::os::unix::fs::MetadataExt;
        (metadata.dev(), metadata.ino())
    };
    #[cfg(windows)]
    let identity = {
        use std::os::windows::fs::OpenOptionsExt;
        use windows_sys::Win32::Storage::FileSystem::FILE_READ_ATTRIBUTES;
        // Preview needs identity/metadata, never permission to read media data.
        let file = fs::OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES)
            .open(path)
            .map_err(|e| file_io_error("读取文件信息", path, e))?;
        windows_identity(&file)?
    };
    #[cfg(not(any(unix, windows)))]
    let identity = (0, 0);
    Ok(Some(FileStamp {
        size: metadata.len(),
        modified: metadata
            .modified()
            .map_err(|e| file_io_error("读取文件时间", path, e))?,
        identity,
    }))
}

/// Validate the entire cache tree before deleting anything. File deletion uses
/// the same identity and ancestor checks as task removal.
pub(crate) fn clear_cache_directory(root: &Path) -> BdlResult<usize> {
    let root = absolute(root)?;
    validate_ancestors(&root)?;
    match fs::symlink_metadata(&root) {
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(0),
        Err(e) => return Err(file_io_error("读取缓存目录", &root, e)),
        Ok(metadata) if !metadata.is_dir() => return Err(error("缓存入口不是目录。")),
        Ok(_) => {}
    }
    let mut pending = vec![(root.clone(), 0usize)];
    let mut directories = Vec::new();
    let mut files = Vec::new();
    while let Some((directory, depth)) = pending.pop() {
        validate_ancestors(&directory)?;
        if depth > 32 || directories.len() + files.len() > 100_000 {
            return Err(error("缓存目录过大或层级过深，尚未清理文件。"));
        }
        directories.push(directory.clone());
        for entry in fs::read_dir(&directory)? {
            let path = entry?.path();
            validate_ancestors(&path)?;
            if fs::symlink_metadata(&path)?.is_dir() {
                pending.push((path, depth + 1));
            } else if let Some(stamp) = stamp(&path)? {
                files.push(LocalFile { path, stamp });
            }
        }
    }
    let mut removed = 0;
    for file in files {
        removed += usize::from(delete_file(&file)?);
    }
    for directory in directories.into_iter().rev().filter(|path| path != &root) {
        validate_ancestors(&directory)?;
        fs::remove_dir(&directory).map_err(|e| file_io_error("清理缓存子目录", &directory, e))?;
    }
    Ok(removed)
}
#[cfg(windows)]
fn windows_identity(file: &fs::File) -> BdlResult<(u64, u64)> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{
        BY_HANDLE_FILE_INFORMATION, GetFileInformationByHandle,
    };
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) } == 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    Ok((
        u64::from(info.dwVolumeSerialNumber),
        (u64::from(info.nFileIndexHigh) << 32) | u64::from(info.nFileIndexLow),
    ))
}
fn delete_file(file: &LocalFile) -> BdlResult<bool> {
    let Some(current) = stamp(&file.path)? else {
        return Ok(false);
    };
    if current != file.stamp {
        return Err(error(format!(
            "文件已变化，请重新确认：{}",
            file.path.display()
        )));
    }
    #[cfg(windows)]
    {
        use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
        use windows_sys::Win32::Storage::FileSystem::{
            DELETE, FILE_DISPOSITION_INFO, FILE_FLAG_OPEN_REPARSE_POINT, FILE_READ_ATTRIBUTES,
            FileDispositionInfo, SetFileInformationByHandle,
        };
        let handle = fs::OpenOptions::new()
            .access_mode(FILE_READ_ATTRIBUTES | DELETE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&file.path)
            .map_err(|e| file_io_error("打开待删除文件", &file.path, e))?;
        let metadata = handle
            .metadata()
            .map_err(|e| file_io_error("读取文件信息", &file.path, e))?;
        if metadata.permissions().readonly() {
            return Err(error(format!(
                "[stage=file_cleanup code=file_read_only] 文件为只读，未删除：{}",
                file.path.display()
            )));
        }
        if is_link(&metadata)
            || windows_identity(&handle)? != file.stamp.identity
            || metadata.len() != file.stamp.size
            || metadata.modified()? != file.stamp.modified
        {
            return Err(error("文件已变化，请重新确认。"));
        }
        let info = FILE_DISPOSITION_INFO { DeleteFile: true };
        if unsafe {
            SetFileInformationByHandle(
                handle.as_raw_handle(),
                FileDispositionInfo,
                &info as *const _ as _,
                std::mem::size_of_val(&info) as u32,
            )
        } == 0
        {
            return Err(file_io_error(
                "删除文件",
                &file.path,
                std::io::Error::last_os_error(),
            ));
        }
    }
    #[cfg(not(windows))]
    {
        fs::remove_file(&file.path).map_err(|e| file_io_error("删除文件", &file.path, e))?;
    }
    Ok(true)
}

pub(crate) fn temporary_paths(task: &DownloadTask) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for resource in &task.resources {
        let mut expected = resource.target_path.as_os_str().to_owned();
        expected.push(".bdlpart");
        // Never reinterpret a recorded arbitrary path as disposable data.
        if resource.temp_path.as_os_str() != expected {
            continue;
        }
        paths.push(resource.temp_path.clone());
        paths.push(state_path_for(&resource.temp_path));
        paths.push(state_temp_path_for(&resource.temp_path));
        for index in 0..8 {
            paths.push(segment_path_for(&resource.temp_path, index));
        }
        paths.push(resource.target_path.with_extension("m4s.bdl-copy.tmp"));
    }
    for output in task.file_paths() {
        let extension = output.extension().and_then(|s| s.to_str()).unwrap_or("bin");
        paths.push(output.with_extension(format!("{extension}.bdl-copy.tmp")));
    }
    paths
}
fn task_documents(task: &DownloadTask) -> Vec<(String, String)> {
    let Some(DownloadExportTarget::DocumentTree {
        tree_uri,
        document_uri,
        ..
    }) = &task.export_target
    else {
        return Vec::new();
    };
    document_uri
        .iter()
        .chain(
            task.media_selection
                .artifacts
                .iter()
                .filter_map(|a| a.document_uri.as_ref()),
        )
        .map(|uri| (tree_uri.clone(), uri.clone()))
        .collect()
}
fn document_key(uri: &str) -> BdlResult<String> {
    let uri = url::Url::parse(uri).map_err(|_| error("导出文件 URI 无效。"))?;
    let parts = uri
        .path_segments()
        .ok_or_else(|| error("导出文件 URI 无效。"))?
        .collect::<Vec<_>>();
    let document = parts
        .windows(2)
        .find(|pair| pair[0] == "document")
        .ok_or_else(|| error("导出文件缺少文档标识。"))?[1];
    let document = percent_encoding::percent_decode_str(document)
        .decode_utf8()
        .map_err(|_| error("导出文档标识无效。"))?;
    Ok(format!(
        "{}:{document}",
        uri.host_str()
            .ok_or_else(|| error("导出文件缺少提供程序。"))?
    ))
}
fn validate_task_paths(task: &DownloadTask) -> BdlResult<()> {
    let parent = task
        .output_path
        .parent()
        .ok_or_else(|| error("任务没有有效输出目录。"))?;
    let parent_key = key(parent)?;
    for path in finished_paths(task)
        .into_iter()
        .chain(temporary_paths(task))
    {
        // Existing records may use relative paths. Only a real file is a
        // deletion candidate; it must stay inside its recorded output folder.
        if fs::symlink_metadata(&path).is_ok()
            && key(path.parent().ok_or_else(|| error("文件没有有效目录。"))?)? != parent_key
        {
            return Err(error(format!(
                "文件不在任务输出目录中，拒绝自动清理：{}",
                path.display()
            )));
        }
    }
    for resource in &task.resources {
        let mut expected = resource.target_path.as_os_str().to_owned();
        expected.push(".bdlpart");
        if resource.temp_path.as_os_str() != expected
            && fs::symlink_metadata(&resource.temp_path).is_ok()
        {
            return Err(error(
                "任务的临时路径不是合法的下载临时文件，拒绝自动清理。",
            ));
        }
    }
    Ok(())
}
fn protected_keys(tasks: &[DownloadTask]) -> BdlResult<HashSet<String>> {
    tasks
        .iter()
        .flat_map(|task| {
            finished_paths(task)
                .into_iter()
                .chain(temporary_paths(task))
        })
        .map(|path| key(&path))
        .collect()
}
fn finished_paths(task: &DownloadTask) -> Vec<PathBuf> {
    let mut paths = task.file_paths();
    if task.media_selection.workflow.is_none() {
        paths.extend(
            task.resources
                .iter()
                .filter(|r| {
                    matches!(
                        r.intent,
                        bdl_core::queue::DownloadResourceIntent::Cover
                            | bdl_core::queue::DownloadResourceIntent::Subtitle
                            | bdl_core::queue::DownloadResourceIntent::Danmaku
                    )
                })
                .map(|r| crate::media_finalize::sidecar_output_path(task, r)),
        );
    }
    paths
}
fn add_local(
    plan: &mut Plan,
    path: &Path,
    protected: &HashSet<String>,
    seen: &mut HashSet<String>,
) -> BdlResult<()> {
    let path = absolute(path)?;
    let identity = key(&path)?;
    if !seen.insert(identity.clone()) {
        return Ok(());
    }
    if protected.contains(&identity) {
        plan.preserved
            .push(format!("其他任务仍在使用：{}", path.display()));
        return Ok(());
    }
    if let Some(stamp) = stamp(&path)? {
        plan.files.push(LocalFile { path, stamp });
    }
    Ok(())
}

struct RemovalGuard<'a> {
    state: &'a AppState,
    ids: Vec<String>,
}
impl Drop for RemovalGuard<'_> {
    fn drop(&mut self) {
        if let Ok(mut ids) = self.state.removals.lock() {
            for id in &self.ids {
                ids.remove(id);
            }
        }
        self.state.notify_queue_changed();
    }
}
struct FileCleanupGuard<'a>(&'a AppState);
impl Drop for FileCleanupGuard<'_> {
    fn drop(&mut self) {
        self.0
            .file_cleanup_active
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self.0.notify_queue_changed();
    }
}

impl AppState {
    fn invalidate_task_previews(&self, ids: &[String]) {
        // Removing a task ends its identity even if a later download reuses
        // the same ID and retained output. A poisoned store cannot issue tokens.
        if let Ok(mut store) = self.removal_previews.lock() {
            store
                .entries
                .retain(|_, (_, plan)| !plan.task_ids.iter().any(|id| ids.contains(id)));
        }
    }
    pub(crate) fn ensure_not_removing(&self, task_id: &str) -> BdlResult<()> {
        if self
            .file_cleanup_active
            .load(std::sync::atomic::Ordering::SeqCst)
        {
            return Err(error("正在清理文件，请稍后操作。"));
        }
        if self
            .removals
            .lock()
            .map_err(|_| lock_error())?
            .contains(task_id)
        {
            return Err(error("任务正在清理文件，请等待操作结束。"));
        }
        Ok(())
    }
    fn removal_guard(&self, ids: &[String]) -> BdlResult<RemovalGuard<'_>> {
        let mut queue = self.queue_edit()?;
        let mut removing = self.removals.lock().map_err(|_| lock_error())?;
        if ids.iter().any(|id| removing.contains(id)) {
            return Err(error("任务正在清理，请勿重复操作。"));
        }
        for id in ids {
            if !queue.iter().any(|t| &t.id == id) {
                return Err(error("任务不存在，请刷新列表。"));
            }
        }
        let guard = RemovalGuard {
            state: self,
            ids: ids.to_vec(),
        };
        removing.extend(ids.iter().cloned());
        // Pause durably before signalling cancellation. A crash cannot restart
        // a partly deleted task automatically. Mux/export may finish; wait below.
        for task in queue.iter_mut().filter(|t| ids.contains(&t.id)) {
            if task.status != TaskStatus::Completed {
                task.status = TaskStatus::Paused;
                task.scheduled_at = None;
            }
        }
        // Drop the lock before the guard can run on a persistence error.
        drop(removing);
        self.persist_queue_edit(&mut queue)?;
        for id in ids {
            self.cancel_running_task(id)?;
        }
        self.notify_queue_changed();
        Ok(guard)
    }
    async fn wait_idle(&self, ids: &[String]) -> BdlResult<()> {
        tokio::time::timeout(Duration::from_secs(30), async {
            loop {
                let active = self
                    .queue_cancellations
                    .lock()
                    .map_err(|_| lock_error())?
                    .keys()
                    .any(|id| ids.contains(id));
                if !active {
                    return Ok(());
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await
        .map_err(|_| error("任务仍在停止或导出，尚未清理任何文件。请稍后重试。"))?
    }
    fn record_cleanup_failure(&self, stage: &str, ids: &[String], failure: &BdlError) {
        let raw = failure.to_string();
        let code = bdl_core::diagnostics::classify_failure(&raw)
            .map_or("cleanup_failed", |rule| rule.code.as_str());
        let message = bdl_core::account::redact_diagnostic(&format!(
            "[stage={stage} code={code}] 文件清理失败：{raw}"
        ));
        tracing::warn!(stage, code, task_ids = ?ids, detail = %message, "File cleanup failed");
        for id in ids {
            // Keep the original failure even if recording it also fails.
            if let Err(error) = self.append_task_log(bdl_core::queue::QueueLogEntry {
                task_id: id.clone(),
                level: bdl_core::queue::QueueLogLevel::Warning,
                message: message.clone(),
                created_at: chrono::Utc::now().to_rfc3339(),
            }) {
                tracing::warn!(detail = %bdl_core::account::redact_sensitive(&error.to_string()), "Failed to record cleanup failure");
            }
        }
    }
    pub(crate) async fn stop_and_remove_task(&self, task_id: &str) -> BdlResult<bool> {
        let result = self.stop_and_remove_task_inner(task_id).await;
        if let Err(error) = &result {
            self.record_cleanup_failure("remove_task", &[task_id.to_owned()], error);
        }
        result
    }
    async fn stop_and_remove_task_inner(&self, task_id: &str) -> BdlResult<bool> {
        if !self.queue_snapshot()?.iter().any(|t| t.id == task_id) {
            return Ok(false);
        }
        let ids = vec![task_id.to_owned()];
        let _guard = self.removal_guard(&ids)?;
        self.wait_idle(&ids).await?;
        self.remove_idle_task(task_id)
    }
    pub(crate) fn remove_idle_task(&self, task_id: &str) -> BdlResult<bool> {
        let mut queue = self.queue.lock().map_err(|_| lock_error())?;
        if self
            .queue_cancellations
            .lock()
            .map_err(|_| lock_error())?
            .contains_key(task_id)
        {
            return Err(error("任务仍在执行，不能移除记录。"));
        }
        let Some(task) = queue.iter().find(|t| t.id == task_id) else {
            return Ok(false);
        };
        validate_task_paths(task)?;
        let others = queue
            .iter()
            .filter(|t| t.id != task_id)
            .cloned()
            .collect::<Vec<_>>();
        let protected = protected_keys(&others)?;
        let mut plan = Plan {
            task_ids: vec![],
            files: vec![],
            documents: vec![],
            preserved: vec![],
            roots: vec![],
        };
        let mut seen = HashSet::new();
        for path in temporary_paths(task) {
            add_local(&mut plan, &path, &protected, &mut seen)?;
        }
        for file in &plan.files {
            delete_file(file)?;
        }
        let updated = queue
            .iter()
            .filter(|t| t.id != task_id)
            .cloned()
            .collect::<Vec<_>>();
        self.persist_queue(&queue, &updated)?;
        self.invalidate_task_previews(&[task_id.to_owned()]);
        *queue = updated;
        Ok(true)
    }
    fn task_plan(&self, tasks: &[DownloadTask], ids: &[String]) -> BdlResult<Plan> {
        if ids.is_empty() || ids.len() > 1000 {
            return Err(error("请选择 1 至 1000 个任务。"));
        }
        let mut ids = ids.to_vec();
        ids.sort();
        ids.dedup();
        let others = tasks
            .iter()
            .filter(|t| !ids.contains(&t.id))
            .cloned()
            .collect::<Vec<_>>();
        let protected = protected_keys(&others)?;
        let mut protected_documents = others
            .iter()
            .flat_map(task_documents)
            .map(|(_, uri)| document_key(&uri))
            .collect::<BdlResult<HashSet<_>>>()?;
        let authorities = tasks
            .iter()
            .filter(|t| ids.contains(&t.id))
            .flat_map(task_documents)
            .filter_map(|(_, uri)| {
                url::Url::parse(&uri)
                    .ok()
                    .and_then(|u| u.host_str().map(str::to_owned))
            })
            .collect::<HashSet<_>>();
        // A queued export has no receipt yet. Resolve its reserved names only
        // to protect existing documents, never to choose a deletion target.
        for task in &others {
            let Some(DownloadExportTarget::DocumentTree {
                tree_uri,
                relative_path,
                ..
            }) = &task.export_target
            else {
                continue;
            };
            if !url::Url::parse(tree_uri)
                .ok()
                .and_then(|u| u.host_str().map(str::to_owned))
                .is_some_and(|a| authorities.contains(&a))
            {
                continue;
            }
            let mut seen_paths = HashSet::new();
            for path in finished_paths(task) {
                let relative = if path == task.output_path {
                    relative_path.clone()
                } else {
                    bdl_core::queue::portable_relative_path(&bdl_core::queue::retarget_task_path(
                        &path,
                        &task.output_path,
                        &PathBuf::from(relative_path),
                    )?)?
                };
                if seen_paths.insert(relative.clone())
                    && let Some(document) = self
                        .mobile_storage()
                        .resolve_document(tree_uri, &relative)?
                {
                    protected_documents.insert(document_key(&document.document_uri)?);
                }
            }
        }
        let mut plan = Plan {
            task_ids: ids.clone(),
            files: vec![],
            documents: vec![],
            preserved: vec![],
            roots: vec![],
        };
        let mut seen = HashSet::new();
        let mut seen_documents = HashSet::new();
        for id in &ids {
            let task = tasks
                .iter()
                .find(|t| &t.id == id)
                .ok_or_else(|| error("任务不存在，请刷新列表。"))?;
            validate_task_paths(task)?;
            // Only recorded file paths, never a folder or a glob based on title.
            for path in finished_paths(task) {
                add_local(&mut plan, &path, &protected, &mut seen)?;
            }
            for (tree_uri, uri) in task_documents(task) {
                let identity = document_key(&uri)?;
                if !seen_documents.insert(identity.clone()) {
                    continue;
                }
                let skipped = task.media_selection.preserved_document_uris.contains(&uri)
                    || matches!(
                        &task.export_target,
                        Some(DownloadExportTarget::DocumentTree {
                            duplicate_naming_strategy:
                                bdl_core::naming::DuplicateNamingStrategy::SkipExisting,
                            ..
                        })
                    ) && !task.media_selection.owned_document_uris.contains(&uri);
                if protected_documents.contains(&identity) || skipped {
                    plan.preserved
                        .push(format!("共享或跳过已有的导出文件：{uri}"));
                    continue;
                }
                if let Some(snapshot) = self.mobile_storage().inspect_document(&tree_uri, &uri)? {
                    plan.documents.push(DocumentFile { tree_uri, snapshot });
                }
            }
            if task.export_target.is_some() && task.media_selection.workflow.is_none() {
                plan.preserved
                    .push("旧任务未登记的 Android 导出附件需要在文件管理器中处理。".into());
            }
        }
        plan.files.sort_by(|a, b| a.path.cmp(&b.path));
        plan.documents
            .sort_by(|a, b| a.snapshot.document_uri.cmp(&b.snapshot.document_uri));
        plan.preserved.sort();
        plan.preserved.dedup();
        Ok(plan)
    }
    fn save_preview(&self, plan: Plan) -> BdlResult<RemovalPreview> {
        let mut store = self.removal_previews.lock().map_err(|_| lock_error())?;
        store
            .entries
            .retain(|_, (time, _)| time.elapsed() < Duration::from_secs(300));
        if store.entries.len() >= 16
            && let Some(oldest) = store
                .entries
                .iter()
                .min_by_key(|(_, (time, _))| *time)
                .map(|(key, _)| key.clone())
        {
            store.entries.remove(&oldest);
        }
        let token = uuid::Uuid::new_v4().to_string();
        let preview = RemovalPreview::from_plan(token.clone(), &plan);
        store.entries.insert(token, (Instant::now(), plan));
        Ok(preview)
    }
    fn consume_preview(&self, token: &str) -> BdlResult<Plan> {
        let (time, plan) = self
            .removal_previews
            .lock()
            .map_err(|_| lock_error())?
            .entries
            .remove(token)
            .ok_or_else(|| error("清理确认已失效，请重新查看文件列表。"))?;
        if time.elapsed() >= Duration::from_secs(300) {
            return Err(error("清理确认已过期，请重新确认。"));
        }
        Ok(plan)
    }
    pub(crate) fn preview_task_deletion(&self, ids: &[String]) -> BdlResult<RemovalPreview> {
        let queue = self.queue_snapshot()?;
        let result = self
            .task_plan(&queue, ids)
            .and_then(|plan| self.save_preview(plan));
        if let Err(error) = &result {
            self.record_cleanup_failure("delete_preview", ids, error);
        }
        result
    }
    pub(crate) async fn delete_task_files(&self, token: &str) -> BdlResult<Vec<String>> {
        let approved = self.consume_preview(token)?;
        let ids = approved.task_ids.clone();
        let result = self.delete_task_files_inner(approved).await;
        if let Err(error) = &result {
            self.record_cleanup_failure("delete_files", &ids, error);
        }
        result
    }
    async fn delete_task_files_inner(&self, approved: Plan) -> BdlResult<Vec<String>> {
        if approved.task_ids.is_empty() {
            return Err(error("确认类型不匹配。"));
        }
        let _guard = self.removal_guard(&approved.task_ids)?;
        self.wait_idle(&approved.task_ids).await?;
        // Reserve the deletion window under the enqueue lock, then release it.
        // NEVER hold a queue lock across an Android plugin call: the main
        // thread may be handling another command waiting for that lock.
        let queue = self.queue.lock().map_err(|_| lock_error())?;
        if self
            .file_cleanup_active
            .swap(true, std::sync::atomic::Ordering::SeqCst)
        {
            return Err(error("已有文件清理操作，请稍后重试。"));
        }
        let _file_guard = FileCleanupGuard(self);
        let queue = {
            let snapshot = queue.clone();
            drop(queue);
            snapshot
        };
        let current = self.task_plan(&queue, &approved.task_ids)?;
        // Missing files are idempotent; new or replaced files need fresh consent.
        if current.files.iter().any(|f| !approved.files.contains(f))
            || current
                .documents
                .iter()
                .any(|f| !approved.documents.contains(f))
        {
            return Err(error(
                "停止任务期间文件已变化，尚未删除文件。请重新查看并确认。",
            ));
        }
        let others = queue
            .iter()
            .filter(|t| !approved.task_ids.contains(&t.id))
            .cloned()
            .collect::<Vec<_>>();
        let protected = protected_keys(&others)?;
        let mut temps = Plan {
            task_ids: vec![],
            files: vec![],
            documents: vec![],
            preserved: vec![],
            roots: vec![],
        };
        let mut seen = HashSet::new();
        for task in queue.iter().filter(|t| approved.task_ids.contains(&t.id)) {
            for path in temporary_paths(task) {
                add_local(&mut temps, &path, &protected, &mut seen)?;
            }
        }
        for file in current.files.iter().chain(&temps.files) {
            delete_file(file)?;
        }
        for document in &current.documents {
            self.mobile_storage()
                .delete_document(&document.tree_uri, &document.snapshot)?;
        }
        let mut queue = self.queue.lock().map_err(|_| lock_error())?;
        let updated = queue
            .iter()
            .filter(|t| !approved.task_ids.contains(&t.id))
            .cloned()
            .collect::<Vec<_>>();
        self.persist_queue(&queue, &updated)?;
        self.invalidate_task_previews(&approved.task_ids);
        *queue = updated;
        Ok(approved.task_ids)
    }
    fn temp_roots(&self, tasks: &[DownloadTask]) -> BdlResult<Vec<PathBuf>> {
        let mut roots = vec![
            self.data_dir().join("temp"),
            self.data_dir().join("downloads"),
        ];
        if let Some(path) = self.settings()?.download_dir {
            roots.push(PathBuf::from(path));
        }
        roots.extend(tasks.iter().flat_map(|t| {
            t.resources
                .iter()
                .filter_map(|r| r.temp_path.parent().map(Path::to_path_buf))
        }));
        let mut seen = HashSet::new();
        let mut result = Vec::new();
        for root in roots {
            let root = absolute(&root)?;
            if root.parent().is_none() {
                return Err(error("不能扫描磁盘根目录，请先设置下载文件夹。"));
            }
            validate_ancestors(&root)?;
            if seen.insert(key(&root)?) {
                result.push(root);
            }
        }
        Ok(result)
    }
    pub(crate) fn preview_temp_cleanup(&self) -> BdlResult<RemovalPreview> {
        let queue = self.queue.lock().map_err(|_| lock_error())?;
        let roots = self.temp_roots(&queue)?;
        let protected = protected_keys(&queue)?;
        let mut plan = Plan {
            task_ids: vec![],
            files: vec![],
            documents: vec![],
            preserved: vec![],
            roots: roots.clone(),
        };
        let mut seen = HashSet::new();
        let mut visited = HashSet::new();
        let mut count = 0;
        for root in roots {
            scan_temp(&root, 0, &mut count, &mut visited, &mut |path| {
                add_local(&mut plan, path, &protected, &mut seen)
            })?;
        }
        self.save_preview(plan)
    }
    pub(crate) fn cleanup_temp_files(&self, token: &str) -> BdlResult<usize> {
        let plan = self.consume_preview(token)?;
        if !plan.task_ids.is_empty() {
            return Err(error("确认类型不匹配。"));
        }
        let queue = self.queue.lock().map_err(|_| lock_error())?;
        let protected = protected_keys(&queue)?;
        let candidates = plan
            .files
            .iter()
            .filter_map(|file| match key(&file.path) {
                Ok(k) if !protected.contains(&k) => Some(Ok(file)),
                Ok(_) => None,
                Err(e) => Some(Err(e)),
            })
            .collect::<BdlResult<Vec<_>>>()?;
        // Revalidate the entire preview before the first unlink.
        for file in &candidates {
            if let Some(current) = stamp(&file.path)?
                && current != file.stamp
            {
                return Err(error("临时文件已变化，请重新预览后清理。"));
            }
        }
        let mut removed = 0;
        for file in candidates {
            removed += usize::from(delete_file(file)?);
        }
        Ok(removed)
    }
}

fn is_download_temp(name: &str) -> bool {
    if name.ends_with(".bdlpart") || name.ends_with(".bdl-copy.tmp") {
        return true;
    }
    let Some((base, suffix)) = name.rsplit_once('.') else {
        return false;
    };
    let valid_suffix = suffix == "state"
        || suffix
            .strip_prefix("seg")
            .is_some_and(|s| matches!(s, "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7"));
    valid_suffix
        && (base.ends_with(".bdlpart")
            || base
                .strip_prefix(".bdl-")
                .is_some_and(|id| uuid::Uuid::parse_str(id).is_ok()))
}
fn scan_temp(
    path: &Path,
    depth: usize,
    count: &mut usize,
    visited: &mut HashSet<String>,
    found: &mut impl FnMut(&Path) -> BdlResult<()>,
) -> BdlResult<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };
    if is_link(&metadata) {
        return Ok(());
    }
    if metadata.is_file() {
        if path
            .file_name()
            .and_then(|s| s.to_str())
            .is_some_and(is_download_temp)
        {
            found(path)?;
        }
        return Ok(());
    }
    if !metadata.is_dir() || !visited.insert(key(path)?) {
        return Ok(());
    }
    if depth > 32 {
        return Err(error("下载目录层级过深，请选择更具体的目录。"));
    }
    for entry in fs::read_dir(path)? {
        *count += 1;
        if *count > 100_000 {
            return Err(error("下载目录过大，请选择更具体的目录。"));
        }
        scan_temp(&entry?.path(), depth + 1, count, visited, found)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bdl_core::queue::{
        DownloadResource, DownloadResourceIntent, DownloadResourceKind, ResourceStatus,
    };
    struct Fixture {
        root: PathBuf,
        state: std::sync::Arc<AppState>,
    }
    impl Fixture {
        fn new() -> Self {
            Self::with_storage(crate::mobile_storage::MobileStorage::unsupported())
        }
        fn with_storage(storage: crate::mobile_storage::MobileStorage) -> Self {
            let root =
                std::env::temp_dir().join(format!("bdl-removal-test-{}", uuid::Uuid::new_v4()));
            fs::create_dir_all(root.join("downloads")).unwrap();
            // Force this preferred root even when the developer has legacy
            // .bdl state in the current directory.
            drop(bdl_core::storage::TaskStorage::open(root.join("state/tasks.sqlite")).unwrap());
            let state = AppState::new_with_platform_backends(
                root.join("state"),
                root.join("downloads"),
                crate::secure_store::SecureStore::in_memory(),
                storage,
                crate::media_mux::MediaMuxBackend::unsupported(),
                crate::task_execution::TaskExecutionBackend::noop(),
            )
            .unwrap();
            Self {
                root,
                state: std::sync::Arc::new(state),
            }
        }
        fn task(&self, id: &str) -> DownloadTask {
            let target = self.root.join("downloads").join(format!("{id}.video.m4s"));
            let mut temp = target.as_os_str().to_owned();
            temp.push(".bdlpart");
            DownloadTask {
                id: id.into(),
                title: id.into(),
                source_id: "fixture".into(),
                status: TaskStatus::Paused,
                resources: vec![DownloadResource {
                    id: format!("{id}:video"),
                    kind: DownloadResourceKind::Video,
                    intent: DownloadResourceIntent::Video,
                    current_urls: vec![],
                    headers: vec![],
                    target_path: target,
                    temp_path: PathBuf::from(temp),
                    status: ResourceStatus::Pending,
                }],
                output_path: self.root.join("downloads").join(format!("{id}.mp4")),
                export_target: None,
                refresh_intent: None,
                media_selection: Default::default(),
                scheduled_at: None,
                speed_limit_bytes_per_second: None,
            }
        }
        fn insert(&self, tasks: Vec<DownloadTask>) {
            let mut queue = self.state.queue.lock().unwrap();
            self.state.persist_queue(&queue, &tasks).unwrap();
            *queue = tasks;
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            // All fixtures, including link destinations, are inside this unique test root.
            if self.root.parent() == Some(std::env::temp_dir().as_path())
                && self
                    .root
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("bdl-removal-test-")
            {
                let _ = fs::remove_dir_all(&self.root);
            }
        }
    }
    fn write(path: &Path) {
        fs::write(path, b"fixture payload").unwrap();
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn deletion_failure_records_operation_path_and_system_code_without_losing_task() {
        use std::os::windows::fs::OpenOptionsExt;
        let fixture = Fixture::new();
        let task = fixture.task("locked-delete");
        write(&task.output_path);
        fixture.insert(vec![task.clone()]);
        let preview = fixture
            .state
            .preview_task_deletion(std::slice::from_ref(&task.id))
            .unwrap();
        let held = fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&task.output_path)
            .unwrap();
        let failure = fixture
            .state
            .delete_task_files(&preview.token)
            .await
            .unwrap_err();
        assert!(failure.to_string().contains("code=file_in_use"));
        let logs = fixture.state.task_logs(&task.id, 10).unwrap();
        assert!(logs.iter().any(|log| {
            log.level == bdl_core::queue::QueueLogLevel::Warning
                && log.message.contains("stage=delete_files")
                && log.message.contains("os_error=32")
                && log
                    .message
                    .contains(&task.output_path.display().to_string())
        }));
        assert!(fixture.state.task_snapshot(&task.id).is_ok());
        assert!(task.output_path.exists());
        drop(held);
        fixture.state.stop_and_remove_task(&task.id).await.unwrap();
        assert!(task.output_path.exists());
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn readonly_deletion_failure_is_distinct_from_account_permissions() {
        let fixture = Fixture::new();
        let task = fixture.task("readonly-delete");
        write(&task.output_path);
        fixture.insert(vec![task.clone()]);
        let original_permissions = fs::metadata(&task.output_path).unwrap().permissions();
        let mut permissions = original_permissions.clone();
        permissions.set_readonly(true);
        fs::set_permissions(&task.output_path, permissions.clone()).unwrap();
        let preview = fixture
            .state
            .preview_task_deletion(std::slice::from_ref(&task.id))
            .unwrap();
        let result = fixture.state.delete_task_files(&preview.token).await;
        // Restore only the test-owned file before asserting, including on failure.
        fs::set_permissions(&task.output_path, original_permissions).unwrap();
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("code=file_read_only")
        );
        assert!(fixture.state.task_snapshot(&task.id).is_ok());
        assert!(
            fixture
                .state
                .task_logs(&task.id, 10)
                .unwrap()
                .iter()
                .any(|log| log.message.contains("code=file_read_only"))
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_system_temp_aliases_allow_files_but_user_links_are_rejected() {
        use std::os::unix::fs::symlink;
        for alias in ["/var/tmp", "/tmp"] {
            let root = Path::new(alias).join(format!("bdl-removal-alias-{}", uuid::Uuid::new_v4()));
            fs::create_dir(&root).unwrap();
            let file = root.join("owned.bdlpart");
            write(&file);
            let owned = LocalFile {
                path: file.clone(),
                stamp: stamp(&file).unwrap().unwrap(),
            };
            let link = root.join("user-link");
            symlink(&root, &link).unwrap();
            assert!(stamp(&link.join("owned.bdlpart")).is_err());
            assert!(file.exists());
            assert!(delete_file(&owned).unwrap());
            assert!(!file.exists());
            fs::remove_file(link).unwrap();
            fs::remove_dir(root).unwrap();
        }
    }

    #[derive(Clone, Default)]
    struct DocumentBackend {
        files: std::sync::Arc<std::sync::Mutex<HashMap<String, DocumentSnapshot>>>,
        refuse_delete: std::sync::Arc<std::sync::atomic::AtomicBool>,
        state: std::sync::Arc<std::sync::Mutex<Option<std::sync::Weak<AppState>>>>,
    }
    impl crate::mobile_storage::MobileStorageBackend for DocumentBackend {
        fn pick_document_tree(&self) -> BdlResult<bdl_core::settings::DocumentTreeDirectory> {
            Err(error("unused"))
        }
        fn read_clipboard_text(&self) -> BdlResult<String> {
            Err(error("unused"))
        }
        fn save_image_to_gallery(&self, _: &str, _: &str, _: &str) -> BdlResult<String> {
            Err(error("unused"))
        }
        fn export_file(
            &self,
            _: &Path,
            _: &DownloadExportTarget,
        ) -> BdlResult<crate::mobile_storage::ExportResult> {
            Err(error("unused"))
        }
        fn open_exported_file(&self, _: &DownloadExportTarget) -> BdlResult<()> {
            Err(error("unused"))
        }
        fn open_export_directory(&self, _: &DownloadExportTarget) -> BdlResult<()> {
            Err(error("unused"))
        }
        fn inspect_document(&self, _: &str, uri: &str) -> BdlResult<Option<DocumentSnapshot>> {
            if let Some(state) = self
                .state
                .lock()
                .unwrap()
                .as_ref()
                .and_then(std::sync::Weak::upgrade)
            {
                assert!(
                    state.queue.try_lock().is_ok(),
                    "native bridge cannot run under the queue lock"
                );
            }
            Ok(self.files.lock().unwrap().get(uri).cloned())
        }
        fn resolve_document(
            &self,
            _: &str,
            relative_path: &str,
        ) -> BdlResult<Option<DocumentSnapshot>> {
            Ok(self
                .files
                .lock()
                .unwrap()
                .values()
                .find(|d| {
                    Path::new(relative_path)
                        .file_name()
                        .is_some_and(|n| n == d.display_name.as_str())
                })
                .cloned())
        }
        fn delete_document(&self, _: &str, expected: &DocumentSnapshot) -> BdlResult<()> {
            if let Some(state) = self
                .state
                .lock()
                .unwrap()
                .as_ref()
                .and_then(std::sync::Weak::upgrade)
            {
                assert!(
                    state.queue.try_lock().is_ok(),
                    "native deletion cannot run under the queue lock"
                );
                assert!(
                    state
                        .file_cleanup_active
                        .load(std::sync::atomic::Ordering::SeqCst)
                );
                assert!(state.retry_task("document").is_err());
            }
            if self.refuse_delete.load(std::sync::atomic::Ordering::SeqCst) {
                return Err(error("没有删除权限"));
            }
            let mut files = self.files.lock().unwrap();
            if files
                .get(&expected.document_uri)
                .is_some_and(|file| file != expected)
            {
                return Err(error("文件已变化"));
            }
            files.remove(&expected.document_uri);
            Ok(())
        }
    }
    fn document_task(fixture: &Fixture, id: &str, uri: &str) -> DownloadTask {
        let mut task = fixture.task(id);
        task.export_target = Some(DownloadExportTarget::DocumentTree {
            tree_uri: "content://fixture/tree/root".into(),
            relative_path: "fixture.mp4".into(),
            duplicate_naming_strategy: bdl_core::naming::DuplicateNamingStrategy::SkipExisting,
            document_uri: Some(uri.into()),
        });
        task.media_selection.workflow = Some(Default::default());
        task.media_selection.owned_document_uris.push(uri.into());
        task
    }

    #[tokio::test]
    async fn android_receipts_protect_skips_replacements_and_permission_failures() {
        let backend = DocumentBackend::default();
        let fixture = Fixture::with_storage(crate::mobile_storage::MobileStorage::from_backend(
            backend.clone(),
        ));
        *backend.state.lock().unwrap() = Some(std::sync::Arc::downgrade(&fixture.state));
        let uri = "content://fixture/tree/root/document/root%3Afixture.mp4";
        let snapshot = DocumentSnapshot {
            document_uri: uri.into(),
            display_name: "fixture.mp4".into(),
            size: 10,
            last_modified: 1,
        };
        backend
            .files
            .lock()
            .unwrap()
            .insert(uri.into(), snapshot.clone());
        let mut task = document_task(&fixture, "document", uri);
        fixture.insert(vec![task.clone()]);
        let preview = fixture
            .state
            .preview_task_deletion(std::slice::from_ref(&task.id))
            .unwrap();
        assert_eq!(preview.file_count, 1);
        backend
            .files
            .lock()
            .unwrap()
            .get_mut(uri)
            .unwrap()
            .last_modified = 2;
        assert!(
            fixture
                .state
                .delete_task_files(&preview.token)
                .await
                .is_err()
        );
        assert!(backend.files.lock().unwrap().contains_key(uri));
        let preview = fixture
            .state
            .preview_task_deletion(std::slice::from_ref(&task.id))
            .unwrap();
        backend
            .refuse_delete
            .store(true, std::sync::atomic::Ordering::SeqCst);
        assert!(
            fixture
                .state
                .delete_task_files(&preview.token)
                .await
                .is_err()
        );
        assert!(
            bdl_core::storage::TaskStorage::open(fixture.root.join("state/tasks.sqlite"))
                .unwrap()
                .load_tasks()
                .unwrap()
                .iter()
                .any(|t| t.id == task.id)
        );
        task.media_selection.owned_document_uris.clear();
        task.media_selection
            .preserved_document_uris
            .push(uri.into());
        fixture.insert(vec![task.clone()]);
        let preview = fixture
            .state
            .preview_task_deletion(std::slice::from_ref(&task.id))
            .unwrap();
        assert_eq!(preview.file_count, 0);
        assert!(!preview.preserved.is_empty());
        fixture
            .state
            .delete_task_files(&preview.token)
            .await
            .unwrap();
        assert!(backend.files.lock().unwrap().contains_key(uri));
        backend
            .refuse_delete
            .store(false, std::sync::atomic::Ordering::SeqCst);
        let task = document_task(&fixture, "owned", uri);
        fixture.insert(vec![task.clone()]);
        let preview = fixture
            .state
            .preview_task_deletion(std::slice::from_ref(&task.id))
            .unwrap();
        fixture
            .state
            .delete_task_files(&preview.token)
            .await
            .unwrap();
        assert!(backend.files.lock().unwrap().is_empty());
    }

    #[test]
    fn document_aliases_with_different_tree_grants_have_the_same_identity() {
        assert_eq!(
            document_key("content://fixture/tree/root/document/root%3Afolder%2Ffile.mp4").unwrap(),
            document_key("content://fixture/tree/root%3Afolder/document/root%3Afolder%2Ffile.mp4")
                .unwrap()
        );
    }

    #[tokio::test]
    async fn queued_exports_without_receipts_protect_their_reserved_document_names() {
        let backend = DocumentBackend::default();
        let fixture = Fixture::with_storage(crate::mobile_storage::MobileStorage::from_backend(
            backend.clone(),
        ));
        let uri = "content://fixture/tree/root/document/root%3Afixture.mp4";
        backend.files.lock().unwrap().insert(
            uri.into(),
            DocumentSnapshot {
                document_uri: uri.into(),
                display_name: "fixture.mp4".into(),
                size: 10,
                last_modified: 1,
            },
        );
        let task = document_task(&fixture, "owner", uri);
        let mut queued = document_task(&fixture, "queued", uri);
        queued.export_target = Some(DownloadExportTarget::DocumentTree {
            tree_uri: "content://fixture/tree/root".into(),
            relative_path: "fixture.mp4".into(),
            duplicate_naming_strategy: bdl_core::naming::DuplicateNamingStrategy::OverwriteExisting,
            document_uri: None,
        });
        queued.media_selection.owned_document_uris.clear();
        fixture.insert(vec![task.clone(), queued]);
        let preview = fixture
            .state
            .preview_task_deletion(std::slice::from_ref(&task.id))
            .unwrap();
        assert_eq!(preview.file_count, 0);
        fixture
            .state
            .delete_task_files(&preview.token)
            .await
            .unwrap();
        assert!(backend.files.lock().unwrap().contains_key(uri));
    }

    #[test]
    fn expired_confirmation_never_cleans_files() {
        let fixture = Fixture::new();
        let path = fixture.root.join("downloads/expired.bdlpart");
        write(&path);
        let preview = fixture.state.preview_temp_cleanup().unwrap();
        fixture
            .state
            .removal_previews
            .lock()
            .unwrap()
            .entries
            .get_mut(&preview.token)
            .unwrap()
            .0 = Instant::now() - Duration::from_secs(301);
        assert!(fixture.state.cleanup_temp_files(&preview.token).is_err());
        assert!(path.exists());
    }

    #[cfg(windows)]
    #[tokio::test]
    #[ignore = "requires a bounded junction fixture supplied by local-test/validate-removal-junction.py"]
    async fn junction_ancestor_refuses_deletion_without_touching_its_target() {
        let path = PathBuf::from(
            std::env::var("BDL_REMOVAL_JUNCTION_FILE").expect("isolated junction fixture required"),
        );
        let fixture = Fixture::new();
        let mut task = fixture.task("junction");
        task.resources.clear();
        task.output_path = path.clone();
        fixture.insert(vec![task.clone()]);
        assert!(
            fixture
                .state
                .preview_task_deletion(std::slice::from_ref(&task.id))
                .is_err()
        );
        assert!(fixture.state.stop_and_remove_task(&task.id).await.is_err());
        assert!(clear_cache_directory(path.parent().unwrap()).is_err());
        assert!(path.exists());
        let cache = fixture.root.join("state/cache");
        fs::create_dir_all(cache.join("child")).unwrap();
        write(&cache.join("one"));
        write(&cache.join("child/two"));
        assert_eq!(clear_cache_directory(&cache).unwrap(), 2);
        assert!(cache.exists());
        assert!(!cache.join("child").exists());
        let cleanup = fixture.root.clone();
        drop(fixture);
        if cleanup.exists() {
            fs::remove_dir_all(cleanup).unwrap();
        }
    }

    #[tokio::test]
    async fn remove_waits_for_attempt_and_cleans_segments_but_keeps_finished_files() {
        let fixture = Fixture::new();
        let mut task = fixture.task("remove");
        task.status = TaskStatus::Waiting;
        fixture.insert(vec![task.clone()]);
        let picked = fixture.state.take_next_startable_task().unwrap().unwrap();
        let token = fixture
            .state
            .register_task_cancel_token(&picked.id)
            .unwrap();
        write(&task.output_path);
        write(&task.resources[0].target_path);
        for path in temporary_paths(&task) {
            write(&path);
        }
        let removal = fixture.state.stop_and_remove_task(&task.id);
        let writer = async {
            while !token.is_cancelled() {
                tokio::time::sleep(Duration::from_millis(2)).await;
            }
            assert!(fixture.state.task_snapshot(&task.id).is_ok());
            assert!(fixture.state.retry_task(&task.id).is_err());
            assert!(
                fixture
                    .state
                    .update_task_status(&task.id, TaskStatus::Waiting)
                    .is_err()
            );
            // A final write arriving after cancellation is cleaned too.
            write(&segment_path_for(&task.resources[0].temp_path, 7));
            fixture.state.clear_task_cancel_token(&task.id).unwrap();
        };
        let (removed, ()) = tokio::join!(removal, writer);
        assert!(removed.unwrap());
        assert!(task.output_path.is_file());
        assert!(task.resources[0].target_path.is_file());
        assert!(temporary_paths(&task).iter().all(|p| !p.exists()));
        assert!(fixture.state.queue_snapshot().unwrap().is_empty());
    }

    #[tokio::test]
    async fn remove_timeout_preserves_files_and_durable_record() {
        let fixture = Fixture::new();
        let task = fixture.task("timeout");
        fixture.insert(vec![task.clone()]);
        fixture.state.register_task_cancel_token(&task.id).unwrap();
        write(&task.resources[0].temp_path);
        tokio::time::pause();
        assert!(
            fixture
                .state
                .stop_and_remove_task(&task.id)
                .await
                .unwrap_err()
                .to_string()
                .contains("尚未清理")
        );
        assert!(task.resources[0].temp_path.exists());
        assert_eq!(
            fixture.state.queue_snapshot().unwrap()[0].status,
            TaskStatus::Paused
        );
        assert!(fixture.state.removals.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn confirmed_deletion_removes_exact_manifest_and_long_path_auxiliaries() {
        let fixture = Fixture::new();
        let mut task = fixture.task(&"long".repeat(35));
        let attachment = task.output_path.with_extension("srt");
        task.media_selection
            .artifacts
            .push(bdl_core::workflow::DownloadArtifact {
                path: attachment.clone(),
                intent: Some(DownloadResourceIntent::Subtitle),
                resource_id: None,
                original: false,
                document_uri: None,
            });
        fixture.insert(vec![task.clone()]);
        for path in task.file_paths().into_iter().chain(temporary_paths(&task)) {
            write(&path);
        }
        let other = fixture.root.join("downloads/unrelated.mp4");
        write(&other);
        let preview = fixture
            .state
            .preview_task_deletion(std::slice::from_ref(&task.id))
            .unwrap();
        assert!(
            preview
                .files
                .contains(&attachment.to_string_lossy().into_owned())
        );
        assert!(
            fixture
                .state
                .delete_task_files("forged-token")
                .await
                .is_err()
        );
        assert!(task.output_path.exists());
        assert_eq!(
            fixture
                .state
                .delete_task_files(&preview.token)
                .await
                .unwrap(),
            vec![task.id.clone()]
        );
        assert!(
            task.file_paths()
                .into_iter()
                .chain(temporary_paths(&task))
                .all(|p| !p.exists())
        );
        assert!(other.exists());
        assert!(
            fixture
                .state
                .delete_task_files(&preview.token)
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn replaced_or_new_outputs_need_fresh_consent_before_any_unlink() {
        let fixture = Fixture::new();
        let task = fixture.task("changed");
        fixture.insert(vec![task.clone()]);
        write(&task.output_path);
        let preview = fixture
            .state
            .preview_task_deletion(std::slice::from_ref(&task.id))
            .unwrap();
        fs::write(&task.output_path, b"replacement content").unwrap();
        write(&task.resources[0].temp_path);
        assert!(
            fixture
                .state
                .delete_task_files(&preview.token)
                .await
                .is_err()
        );
        assert!(task.output_path.exists());
        assert!(task.resources[0].temp_path.exists());
        let preview = fixture
            .state
            .preview_task_deletion(std::slice::from_ref(&task.id))
            .unwrap();
        write(&task.resources[0].target_path);
        assert!(
            fixture
                .state
                .delete_task_files(&preview.token)
                .await
                .is_err()
        );
        assert!(task.output_path.exists());
        let preview = fixture
            .state
            .preview_task_deletion(std::slice::from_ref(&task.id))
            .unwrap();
        fixture
            .state
            .delete_task_files(&preview.token)
            .await
            .unwrap();
        assert!(!task.output_path.exists());
    }

    #[tokio::test]
    async fn shared_paths_remain_until_all_owners_are_explicitly_selected() {
        let fixture = Fixture::new();
        let task = fixture.task("shared");
        let mut other = task.clone();
        other.id = "other".into();
        other.resources[0].id = "other:video".into();
        other.status = TaskStatus::Completed;
        fixture.insert(vec![task.clone(), other.clone()]);
        write(&task.output_path);
        write(&task.resources[0].temp_path);
        let preview = fixture
            .state
            .preview_task_deletion(std::slice::from_ref(&task.id))
            .unwrap();
        assert_eq!(preview.file_count, 0);
        assert!(!preview.preserved.is_empty());
        fixture
            .state
            .delete_task_files(&preview.token)
            .await
            .unwrap();
        assert!(task.output_path.exists());
        assert!(task.resources[0].temp_path.exists());
        let preview = fixture
            .state
            .preview_task_deletion(&[other.id.clone()])
            .unwrap();
        fixture
            .state
            .delete_task_files(&preview.token)
            .await
            .unwrap();
        assert!(!task.output_path.exists());
    }

    #[test]
    fn orphan_cleanup_covers_segments_and_hashes_and_protects_all_queue_states() {
        let fixture = Fixture::new();
        let task = fixture.task("paused");
        fixture.insert(vec![task.clone()]);
        for path in temporary_paths(&task) {
            write(&path);
        }
        let root = fixture.root.join("downloads");
        let names = [
            "orphan.video.m4s.bdlpart".to_string(),
            "orphan.video.m4s.bdlpart.state".into(),
            "orphan.video.m4s.bdlpart.seg0".into(),
            format!(".bdl-{}.seg7", uuid::Uuid::new_v4()),
        ];
        for name in &names {
            write(&root.join(name));
        }
        let unrelated = root.join("orphan.video.m4s.bdlpart.seg0.txt");
        write(&unrelated);
        let preview = fixture.state.preview_temp_cleanup().unwrap();
        assert_eq!(preview.file_count, names.len());
        assert_eq!(
            fixture.state.cleanup_temp_files(&preview.token).unwrap(),
            names.len()
        );
        assert!(unrelated.exists());
        assert!(temporary_paths(&task).iter().all(|p| p.exists()));
    }

    #[test]
    fn newly_enqueued_owner_and_modified_temp_are_rechecked_at_confirmation() {
        let fixture = Fixture::new();
        let task = fixture.task("new-owner");
        write(&task.resources[0].temp_path);
        let preview = fixture.state.preview_temp_cleanup().unwrap();
        assert_eq!(preview.file_count, 1);
        fixture.insert(vec![task.clone()]);
        assert_eq!(fixture.state.cleanup_temp_files(&preview.token).unwrap(), 0);
        assert!(task.resources[0].temp_path.exists());
        fixture.insert(vec![]);
        let preview = fixture.state.preview_temp_cleanup().unwrap();
        fs::write(&task.resources[0].temp_path, b"still growing").unwrap();
        assert!(fixture.state.cleanup_temp_files(&preview.token).is_err());
        assert!(task.resources[0].temp_path.exists());
    }

    #[tokio::test]
    async fn directories_outside_paths_and_invalid_temps_never_get_deleted() {
        let fixture = Fixture::new();
        let mut task = fixture.task("directory");
        fixture.insert(vec![task.clone()]);
        fs::create_dir(&task.output_path).unwrap();
        assert!(
            fixture
                .state
                .preview_task_deletion(std::slice::from_ref(&task.id))
                .is_err()
        );
        assert!(task.output_path.is_dir());
        task.output_path = fixture.root.join("downloads/valid.mp4");
        task.resources[0].temp_path = fixture.root.join("precious.txt");
        write(&task.resources[0].temp_path);
        fixture.insert(vec![task.clone()]);
        assert!(fixture.state.stop_and_remove_task(&task.id).await.is_err());
        assert!(task.resources[0].temp_path.exists());
        task.resources[0].target_path = fixture.root.join("outside.m4s");
        write(&task.resources[0].target_path);
        task.resources[0].temp_path = fixture.root.join("outside.m4s.bdlpart");
        fixture.insert(vec![task.clone()]);
        assert!(
            fixture
                .state
                .preview_task_deletion(std::slice::from_ref(&task.id))
                .is_err()
        );
        assert!(task.resources[0].target_path.exists());
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn locked_file_failure_keeps_record_for_retry() {
        use std::os::windows::fs::OpenOptionsExt;
        let fixture = Fixture::new();
        let task = fixture.task("locked");
        fixture.insert(vec![task.clone()]);
        write(&task.output_path);
        let handle = fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&task.output_path)
            .unwrap();
        let preview = fixture
            .state
            .preview_task_deletion(std::slice::from_ref(&task.id))
            .unwrap();
        assert!(
            fixture
                .state
                .delete_task_files(&preview.token)
                .await
                .is_err()
        );
        assert!(fixture.state.task_snapshot(&task.id).is_ok());
        assert!(task.output_path.exists());
        drop(handle);
        let preview = fixture
            .state
            .preview_task_deletion(std::slice::from_ref(&task.id))
            .unwrap();
        fixture
            .state
            .delete_task_files(&preview.token)
            .await
            .unwrap();
    }

    fn assert_durable_ids(fixture: &Fixture, expected: &[&str]) {
        let stored = bdl_core::storage::TaskStorage::open(fixture.root.join("state/tasks.sqlite"))
            .unwrap()
            .load_tasks()
            .unwrap();
        let mut ids = stored.iter().map(|t| t.id.as_str()).collect::<Vec<_>>();
        ids.sort();
        let mut expected = expected.to_vec();
        expected.sort();
        assert_eq!(ids, expected);
    }

    #[tokio::test]
    async fn audit_idle_state_matrix_preserves_or_deletes_exact_files_and_durable_records() {
        for status in [
            TaskStatus::Waiting,
            TaskStatus::Parsing,
            TaskStatus::Downloading,
            TaskStatus::Muxing,
            TaskStatus::Completed,
            TaskStatus::Failed,
            TaskStatus::Paused,
            TaskStatus::Cancelled,
        ] {
            for full in [false, true] {
                let fixture = Fixture::new();
                let mut task = fixture.task("selected");
                task.status = status;
                if status == TaskStatus::Waiting {
                    task.scheduled_at = Some(chrono::Utc::now() + chrono::Duration::hours(1));
                }
                let other = fixture.task("unselected");
                fixture.insert(vec![task.clone(), other.clone()]);
                for path in task
                    .file_paths()
                    .into_iter()
                    .chain(temporary_paths(&task))
                    .chain(other.file_paths())
                    .chain(temporary_paths(&other))
                {
                    write(&path);
                }
                let unrelated = fixture.root.join("downloads/unrelated.txt");
                write(&unrelated);
                if full {
                    let preview = fixture
                        .state
                        .preview_task_deletion(std::slice::from_ref(&task.id))
                        .unwrap();
                    fixture
                        .state
                        .delete_task_files(&preview.token)
                        .await
                        .unwrap();
                } else {
                    assert!(fixture.state.stop_and_remove_task(&task.id).await.unwrap());
                }
                assert!(temporary_paths(&task).iter().all(|p| !p.exists()));
                assert!(task.file_paths().iter().all(|p| p.exists() != full));
                assert!(
                    other
                        .file_paths()
                        .into_iter()
                        .chain(temporary_paths(&other))
                        .all(|p| p.exists())
                );
                assert!(unrelated.is_file());
                assert_durable_ids(&fixture, &["unselected"]);
                eprintln!(
                    "AUDIT idle {status:?}, full={full}: exact cleanup and durable queue verified"
                );
            }
        }
    }

    #[tokio::test]
    async fn audit_active_and_paused_unwinding_attempts_cannot_resume_or_recreate_parts() {
        for status in [
            TaskStatus::Downloading,
            TaskStatus::Paused,
            TaskStatus::Muxing,
            TaskStatus::Completed,
        ] {
            for full in [false, true] {
                let fixture = Fixture::new();
                let mut task = fixture.task("active");
                task.status = status;
                fixture.insert(vec![task.clone()]);
                let token = fixture.state.register_task_cancel_token(&task.id).unwrap();
                write(&task.output_path);
                write(&task.resources[0].temp_path);
                let preview = fixture
                    .state
                    .preview_task_deletion(std::slice::from_ref(&task.id))
                    .unwrap();
                let removal = async {
                    if full {
                        fixture
                            .state
                            .delete_task_files(&preview.token)
                            .await
                            .map(|_| true)
                    } else {
                        fixture.state.stop_and_remove_task(&task.id).await
                    }
                };
                let unwind = async {
                    token.cancelled().await;
                    assert!(task.output_path.is_file());
                    assert!(fixture.state.task_snapshot(&task.id).is_ok());
                    assert!(fixture.state.retry_task(&task.id).is_err());
                    assert!(
                        fixture
                            .state
                            .update_task_status(&task.id, TaskStatus::Waiting)
                            .is_err()
                    );
                    assert!(fixture.state.set_task_schedule(&task.id, None).is_err());
                    assert!(fixture.state.take_next_startable_task().unwrap().is_none());
                    tokio::time::sleep(Duration::from_millis(30)).await;
                    write(&segment_path_for(&task.resources[0].temp_path, 7));
                    fixture.state.clear_task_cancel_token(&task.id).unwrap();
                };
                let (removed, ()) = tokio::join!(removal, unwind);
                assert!(removed.unwrap());
                assert!(temporary_paths(&task).iter().all(|p| !p.exists()));
                assert_eq!(task.output_path.exists(), !full);
                assert_durable_ids(&fixture, &[]);
                eprintln!(
                    "AUDIT live {status:?}, full={full}: blocked resume and final segment write cleaned"
                );
            }
        }
    }

    #[tokio::test]
    async fn audit_real_http_resumed_download_stops_before_removal() {
        use bdl_core::fetcher::{FetchConfig, FetchState, ReqwestFetcher, write_fetch_state};
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        for full in [false, true] {
            let fixture = Fixture::new();
            let mut task = fixture.task("http-resume");
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            task.resources[0].current_urls =
                vec![format!("http://{}/fixture", listener.local_addr().unwrap())];
            fixture.insert(vec![task.clone()]);
            fs::write(&task.resources[0].temp_path, [42; 1024]).unwrap();
            write_fetch_state(
                &task.resources[0].temp_path,
                &FetchState {
                    total_bytes: Some(131072),
                    downloaded_bytes: 1024,
                    etag: Some("\"fixture\"".into()),
                    last_modified: None,
                    segment_ranges: None,
                },
            )
            .await
            .unwrap();
            fixture
                .state
                .update_task_status(&task.id, TaskStatus::Waiting)
                .unwrap();
            fixture.state.take_next_startable_task().unwrap().unwrap();
            let token = fixture.state.register_task_cancel_token(&task.id).unwrap();
            let server_token = token.clone();
            let server = tokio::spawn(async move {
                for request_index in 0..2 {
                    let (mut socket, _) = listener.accept().await.unwrap();
                    let mut request = Vec::new();
                    while !request.ends_with(b"\r\n\r\n") {
                        let mut byte = [0];
                        if socket.read(&mut byte).await.unwrap() == 0 {
                            break;
                        }
                        request.extend(byte);
                    }
                    assert!(request.starts_with(if request_index == 0 {
                        b"HEAD"
                    } else {
                        b"GET "
                    }));
                    if request_index == 0 {
                        socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 131072\r\nAccept-Ranges: bytes\r\nETag: \"fixture\"\r\nConnection: close\r\n\r\n").await.unwrap();
                    } else {
                        assert!(
                            String::from_utf8_lossy(&request)
                                .to_ascii_lowercase()
                                .contains("range: bytes=1024-")
                        );
                        socket.write_all(b"HTTP/1.1 206 Partial Content\r\nContent-Length: 130048\r\nContent-Range: bytes 1024-131071/131072\r\nETag: \"fixture\"\r\nConnection: close\r\n\r\n").await.unwrap();
                        socket.write_all(&[42; 1024]).await.unwrap();
                        server_token.cancelled().await;
                    }
                }
            });
            let download_state = fixture.state.clone();
            let resource = task.resources[0].clone();
            let download_id = task.id.clone();
            let writer = tokio::spawn(async move {
                let fetcher = ReqwestFetcher::with_config(FetchConfig {
                    max_retries: 0,
                    segment_count: 1,
                    ..Default::default()
                })
                .unwrap();
                let result = fetcher.fetch_cancelable(&resource, None, token).await;
                download_state
                    .clear_task_cancel_token(&download_id)
                    .unwrap();
                result
            });
            tokio::time::timeout(Duration::from_secs(5), async {
                loop {
                    if fs::metadata(&task.resources[0].temp_path).is_ok_and(|m| m.len() > 1024) {
                        break;
                    }
                    tokio::time::sleep(Duration::from_millis(5)).await;
                }
            })
            .await
            .unwrap();
            if full {
                let preview = fixture
                    .state
                    .preview_task_deletion(std::slice::from_ref(&task.id))
                    .unwrap();
                fixture
                    .state
                    .delete_task_files(&preview.token)
                    .await
                    .unwrap();
            } else {
                fixture.state.stop_and_remove_task(&task.id).await.unwrap();
            }
            assert!(writer.await.unwrap().is_err());
            server.await.unwrap();
            assert!(temporary_paths(&task).iter().all(|p| !p.exists()));
            assert!(!task.output_path.exists());
            assert_durable_ids(&fixture, &[]);
            tokio::time::sleep(Duration::from_millis(40)).await;
            assert!(temporary_paths(&task).iter().all(|p| !p.exists()));
            eprintln!(
                "AUDIT real HTTP resumed download, full={full}: cancellation finished and files stayed removed"
            );
        }
    }

    #[tokio::test]
    async fn audit_old_confirmation_must_not_delete_recreated_task_with_same_id() {
        for full in [false, true] {
            let fixture = Fixture::new();
            let task = fixture.task("same-id");
            write(&task.output_path);
            fixture.insert(vec![task.clone()]);
            let old = fixture
                .state
                .preview_task_deletion(std::slice::from_ref(&task.id))
                .unwrap();
            if full {
                let current = fixture
                    .state
                    .preview_task_deletion(std::slice::from_ref(&task.id))
                    .unwrap();
                fixture
                    .state
                    .delete_task_files(&current.token)
                    .await
                    .unwrap();
                write(&task.output_path);
            } else {
                fixture.state.stop_and_remove_task(&task.id).await.unwrap();
            }
            fixture.insert(vec![task.clone()]);
            assert!(fixture.state.delete_task_files(&old.token).await.is_err());
            assert!(task.output_path.is_file());
            assert_durable_ids(&fixture, &["same-id"]);
        }
    }

    #[tokio::test]
    async fn audit_mux_completion_after_cancel_requires_fresh_confirmation_before_unlink() {
        let fixture = Fixture::new();
        let mut task = fixture.task("muxing");
        task.status = TaskStatus::Muxing;
        fixture.insert(vec![task.clone()]);
        let token = fixture.state.register_task_cancel_token(&task.id).unwrap();
        write(&task.resources[0].target_path);
        write(&task.resources[0].temp_path);
        let preview = fixture
            .state
            .preview_task_deletion(std::slice::from_ref(&task.id))
            .unwrap();
        let mux = async {
            token.cancelled().await;
            write(&task.output_path);
            fixture
                .state
                .update_task_status(&task.id, TaskStatus::Completed)
                .unwrap();
            fixture.state.clear_task_cancel_token(&task.id).unwrap();
        };
        let (result, ()) = tokio::join!(fixture.state.delete_task_files(&preview.token), mux);
        assert!(result.unwrap_err().to_string().contains("文件已变化"));
        assert!(task.file_paths().iter().all(|p| p.exists()));
        assert!(task.resources[0].temp_path.exists());
        assert_durable_ids(&fixture, &["muxing"]);
        let preview = fixture
            .state
            .preview_task_deletion(std::slice::from_ref(&task.id))
            .unwrap();
        fixture
            .state
            .delete_task_files(&preview.token)
            .await
            .unwrap();
        assert_durable_ids(&fixture, &[]);
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn audit_windows_partial_cleanup_error_keeps_record_and_retry_recovers() {
        use std::os::windows::fs::OpenOptionsExt;
        for full in [false, true] {
            let fixture = Fixture::new();
            let mut settings = fixture.state.settings().unwrap();
            settings.startup_auto_recovery = true;
            fixture.state.update_settings(settings).unwrap();
            let task = fixture.task("partial");
            fixture.insert(vec![task.clone()]);
            write(&task.output_path);
            write(&task.resources[0].target_path);
            write(&task.resources[0].temp_path);
            let blocked = if full {
                task.resources[0].target_path.clone()
            } else {
                state_path_for(&task.resources[0].temp_path)
            };
            write(&blocked);
            let handle = fs::OpenOptions::new()
                .read(true)
                .share_mode(1)
                .open(&blocked)
                .unwrap();
            let result = if full {
                let preview = fixture
                    .state
                    .preview_task_deletion(std::slice::from_ref(&task.id))
                    .unwrap();
                fixture
                    .state
                    .delete_task_files(&preview.token)
                    .await
                    .map(|_| true)
            } else {
                fixture.state.stop_and_remove_task(&task.id).await
            };
            assert!(result.is_err());
            assert!(blocked.is_file());
            let deleted = if full {
                &task.output_path
            } else {
                &task.resources[0].temp_path
            };
            assert!(!deleted.exists());
            assert_durable_ids(&fixture, &["partial"]);
            let restarted = AppState::new_with_platform_backends(
                fixture.root.join("state"),
                fixture.root.join("downloads"),
                crate::secure_store::SecureStore::in_memory(),
                crate::mobile_storage::MobileStorage::unsupported(),
                crate::media_mux::MediaMuxBackend::unsupported(),
                crate::task_execution::TaskExecutionBackend::noop(),
            )
            .unwrap();
            assert_eq!(restarted.task_status(&task.id).unwrap(), TaskStatus::Paused);
            assert!(!restarted.has_startable_task().unwrap());
            drop(restarted);
            drop(handle);
            if full {
                let preview = fixture
                    .state
                    .preview_task_deletion(std::slice::from_ref(&task.id))
                    .unwrap();
                fixture
                    .state
                    .delete_task_files(&preview.token)
                    .await
                    .unwrap();
            } else {
                fixture.state.stop_and_remove_task(&task.id).await.unwrap();
            }
            assert_durable_ids(&fixture, &[]);
            assert!(temporary_paths(&task).iter().all(|p| !p.exists()));
        }
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn audit_windows_unavailable_drive_keeps_records_and_refuses_orphan_scan() {
        use windows_sys::Win32::Storage::FileSystem::GetLogicalDrives;
        let mask = unsafe { GetLogicalDrives() };
        assert_ne!(mask, 0);
        let letter = (0..26)
            .rev()
            .find(|index| mask & (1 << index) == 0)
            .expect("an unused drive letter for isolated path-only test");
        let fixture = Fixture::new();
        let mut task = fixture.task("offline");
        let folder = PathBuf::from(format!(
            "{}:\\bdl-unused-drive-{}",
            char::from(b'A' + letter as u8),
            uuid::Uuid::new_v4()
        ));
        task.output_path = folder.join("output.mp4");
        task.resources[0].target_path = folder.join("track.m4s");
        task.resources[0].temp_path = folder.join("track.m4s.bdlpart");
        fixture.insert(vec![task.clone()]);
        assert!(
            fixture
                .state
                .preview_task_deletion(std::slice::from_ref(&task.id))
                .is_err()
        );
        assert!(fixture.state.stop_and_remove_task(&task.id).await.is_err());
        assert!(fixture.state.preview_temp_cleanup().is_err());
        assert_durable_ids(&fixture, &["offline"]);
    }

    #[cfg(windows)]
    #[tokio::test]
    #[ignore = "requires isolated C-drive fixture and ACLs supplied by local-test/audit-removal-permissions.py"]
    async fn audit_windows_delete_permissions_and_readonly_keep_durable_records() {
        let root = PathBuf::from(
            std::env::var("BDL_REMOVAL_PERMISSION_ROOT").expect("isolated fixture required"),
        );
        assert!(root.is_absolute());
        for full in [false, true] {
            for case in ["allowed", "denied", "readonly", "unreadable"] {
                let fixture = Fixture::new();
                let mut task = fixture.task(case);
                let folder = root.join(format!("{case}-{}", if full { "full" } else { "remove" }));
                task.output_path = folder.join("output.mp4");
                task.resources[0].target_path = folder.join("track.m4s");
                task.resources[0].temp_path = folder.join("track.m4s.bdlpart");
                fixture.insert(vec![task.clone()]);
                let result = if full {
                    let preview = fixture
                        .state
                        .preview_task_deletion(std::slice::from_ref(&task.id))
                        .unwrap();
                    fixture
                        .state
                        .delete_task_files(&preview.token)
                        .await
                        .map(|_| true)
                } else {
                    fixture.state.stop_and_remove_task(&task.id).await
                };
                if matches!(case, "allowed" | "unreadable") {
                    assert!(result.unwrap());
                    assert_eq!(task.output_path.exists(), !full);
                    assert!(!task.resources[0].temp_path.exists());
                    assert_durable_ids(&fixture, &[]);
                } else {
                    let err = result.unwrap_err();
                    assert!(task.output_path.is_file());
                    assert!(task.resources[0].temp_path.is_file());
                    assert_durable_ids(&fixture, &[case]);
                    assert!(
                        !fixture
                            .state
                            .file_cleanup_active
                            .load(std::sync::atomic::Ordering::SeqCst)
                    );
                    assert!(fixture.state.removals.lock().unwrap().is_empty());
                    eprintln!(
                        "AUDIT C-drive {case}, full={full}: {err}; files and durable record retained"
                    );
                }
            }
        }
    }
}
