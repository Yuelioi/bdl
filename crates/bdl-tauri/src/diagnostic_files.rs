//! Own only diagnostic exports created by BDL; never traverse subdirectories.
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use bdl_core::error::IoResultExt;
use bdl_core::{BdlError, BdlResult};
use chrono::Utc;

pub(crate) const MAX_DIAGNOSTIC_FILES: usize = 10;
static FILES_LOCK: Mutex<()> = Mutex::new(());

fn lock_error() -> BdlError {
    BdlError::Planning {
        message: "诊断文件正在维护，请稍后重试。".into(),
    }
}

fn files(data_dir: &Path) -> BdlResult<Vec<(PathBuf, u64)>> {
    let directory = data_dir.join("diagnostics");
    crate::file_removal::validate_ancestors(&directory)?;
    let entries = match std::fs::read_dir(&directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error).path_context("read_diagnostics_directory", &directory),
    };
    let mut files = Vec::new();
    for entry in entries {
        let entry = entry.path_context("read_diagnostics_entry", &directory)?;
        let name = entry.file_name();
        let Some(stamp) = name
            .to_str()
            .and_then(|name| name.strip_prefix("bdl-diagnostics-"))
            .and_then(|name| name.strip_suffix(".json"))
        else {
            continue;
        };
        if stamp.len() < 15
            || !stamp
                .bytes()
                .all(|byte| byte.is_ascii_digit() || byte == b'-')
        {
            continue;
        }
        crate::file_removal::validate_ancestors(&entry.path())?;
        let metadata = entry
            .metadata()
            .path_context("stat_diagnostic_file", &entry.path())?;
        if metadata.is_file() {
            files.push((entry.path(), metadata.len()));
        }
    }
    files.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(files)
}

pub(crate) fn usage(data_dir: &Path) -> BdlResult<(usize, u64)> {
    let _guard = FILES_LOCK.lock().map_err(|_| lock_error())?;
    let files = files(data_dir)?;
    Ok((files.len(), files.iter().map(|(_, size)| size).sum()))
}

pub(crate) fn clear(data_dir: &Path) -> BdlResult<usize> {
    let _guard = FILES_LOCK.lock().map_err(|_| lock_error())?;
    let files = files(data_dir)?;
    for (path, _) in &files {
        crate::file_removal::validate_ancestors(path)?;
        std::fs::remove_file(path).path_context("clear_diagnostic_file", path)?;
    }
    Ok(files.len())
}

pub(crate) fn write(data_dir: &Path, report: &[u8]) -> BdlResult<PathBuf> {
    let _guard = FILES_LOCK.lock().map_err(|_| lock_error())?;
    let directory = data_dir.join("diagnostics");
    crate::file_removal::validate_ancestors(&directory)?;
    std::fs::create_dir_all(&directory).path_context("create_diagnostics_directory", &directory)?;
    let path = directory.join(format!(
        "bdl-diagnostics-{}.json",
        Utc::now().format("%Y%m%d-%H%M%S-%f")
    ));
    // A fresh name avoids overwriting exports from another click in the same second.
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .path_context("create_diagnostic_report", &path)?;
    std::io::Write::write_all(&mut file, report).path_context("write_diagnostic_report", &path)?;
    drop(file);
    for (expired, _) in files(data_dir)?.into_iter().skip(MAX_DIAGNOSTIC_FILES) {
        crate::file_removal::validate_ancestors(&expired)?;
        std::fs::remove_file(&expired).path_context("prune_diagnostic_report", &expired)?;
    }
    Ok(path)
}
