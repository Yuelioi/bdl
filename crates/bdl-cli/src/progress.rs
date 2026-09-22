use anyhow::{Context, bail};
use bdl_core::model::NormalizedSourceTree;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Default, Serialize, Deserialize)]
struct Saved {
    version: u32,
    signature: String,
    blocked_until: u64,
    sources: BTreeMap<String, NormalizedSourceTree>,
    completed: BTreeMap<String, Completed>,
}

#[derive(Serialize, Deserialize)]
struct Completed {
    path: PathBuf,
    size: u64,
}

pub struct Progress {
    path: Option<PathBuf>,
    saved: Saved,
    _lock: Option<std::fs::File>,
}

impl Progress {
    pub fn open(path: Option<PathBuf>, resume: bool, signature: String) -> anyhow::Result<Self> {
        let lock = if let Some(path) = &path {
            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent)?;
            }
            let lock = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(path.with_extension("lock"))?;
            fs2::FileExt::try_lock_exclusive(&lock)
                .context("progress file is in use by another process; wait for it to exit")?;
            Some(lock)
        } else {
            None
        };
        let saved = if resume {
            let path = path.as_ref().context("--resume requires --state")?;
            let saved: Saved =
                serde_json::from_slice(&std::fs::read(path).context("cannot read progress file")?)?;
            if saved.version != 1 || saved.signature != signature {
                bail!(
                    "progress file belongs to different inputs/options or an unsupported version"
                );
            }
            if saved.blocked_until > now() {
                bail!(
                    "source restriction cooldown remains active for {} seconds; do not automatically retry",
                    saved.blocked_until - now()
                );
            }
            saved
        } else {
            if path.as_ref().is_some_and(|path| path.exists()) {
                bail!("progress file already exists; use --resume or a different --state path");
            }
            Saved {
                version: 1,
                signature,
                ..Saved::default()
            }
        };
        let progress = Self {
            path,
            saved,
            _lock: lock,
        };
        progress.save()?;
        Ok(progress)
    }

    pub fn source(&self, input: &str) -> Option<NormalizedSourceTree> {
        self.saved.sources.get(input).cloned()
    }
    pub fn remember(&mut self, input: &str, tree: &NormalizedSourceTree) -> anyhow::Result<()> {
        let mut tree = tree.clone();
        for part in tree
            .groups
            .iter_mut()
            .flat_map(|g| &mut g.items)
            .flat_map(|i| &mut i.parts)
        {
            part.streams.clear();
            part.assets.clear();
        }
        self.saved.sources.insert(input.into(), tree);
        self.save()
    }
    pub fn completed(&self, key: &str) -> Option<String> {
        let done = self.saved.completed.get(key)?;
        let metadata = std::fs::metadata(&done.path).ok()?;
        (metadata.is_file() && metadata.len() > 0 && metadata.len() == done.size)
            .then(|| done.path.display().to_string())
    }
    pub fn finish(&mut self, key: String, path: &Path) -> anyhow::Result<()> {
        self.saved.completed.insert(
            key,
            Completed {
                path: path.to_owned(),
                size: std::fs::metadata(path)?.len(),
            },
        );
        self.save()
    }
    pub fn restrict(&mut self) -> anyhow::Result<()> {
        self.saved.blocked_until = now() + 300;
        self.save()
    }
    fn save(&self) -> anyhow::Result<()> {
        let Some(path) = &self.path else {
            return Ok(());
        };
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let temporary = path.with_extension(format!(
            "{}.{}.tmp",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos()
        ));
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        file.write_all(&serde_json::to_vec_pretty(&self.saved)?)?;
        file.sync_all()?;
        drop(file);
        std::fs::rename(temporary, path).context("cannot replace progress file")
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn temp(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "bdl-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }
    #[test]
    fn checkpoints_resume_metadata_without_assets_and_validate_completed_files() {
        let path = temp("state");
        let output = temp("media");
        std::fs::write(&output, b"media").unwrap();
        let mut progress = Progress::open(Some(path.clone()), false, "signature".into()).unwrap();
        let tree: NormalizedSourceTree = serde_json::from_value(serde_json::json!({
            "source":{"id":"video:one","kind":"video","input":"one","title":"title","loaded_count":1,"total_count":1,"has_more":false},
            "groups":[{"id":"group:one","kind":"video","title":"title","page":null,"items":[{
                "id":"item:one","title":"title","owner_name":null,"cover_url":null,"duration_seconds":1,
                "parts":[{"id":"part:one","title":"part","aid":1,"bvid":"one","cid":2,"duration_seconds":1,"streams":[],
                    "assets":[{"kind":"cover","format":null,"fetch_policy":"on_demand","urls":["signed-secret"],"headers":[{"name":"Cookie","value":"cookie-secret"}]}]}]
            }]}]
        })).unwrap();
        progress.remember("one", &tree).unwrap();
        progress.finish("task".into(), &output).unwrap();
        let stored = std::fs::read_to_string(&path).unwrap();
        assert!(!stored.contains("secret"));
        assert!(Progress::open(Some(path.clone()), true, "signature".into()).is_err());
        drop(progress);
        let resumed = Progress::open(Some(path.clone()), true, "signature".into()).unwrap();
        assert!(resumed.source("one").is_some());
        assert!(resumed.completed("task").is_some());
        std::fs::write(&output, b"changed").unwrap();
        assert!(resumed.completed("task").is_none());
        drop(resumed);
        assert!(Progress::open(Some(path.clone()), true, "different".into()).is_err());
        assert!(Progress::open(Some(path.clone()), false, "signature".into()).is_err());
        std::fs::remove_file(output).unwrap();
        std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn restriction_cooldown_survives_process_restart() {
        let path = temp("cooldown");
        let mut progress = Progress::open(Some(path.clone()), false, "signature".into()).unwrap();
        progress.restrict().unwrap();
        drop(progress);
        assert!(Progress::open(Some(path.clone()), true, "signature".into()).is_err());
        std::fs::remove_file(path).unwrap();
    }
}
