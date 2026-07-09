use std::path::PathBuf;

use bdl_core::BdlResult;
use bdl_core::model::HeaderPair;
use bdl_core::queue::{
    DownloadResource, DownloadResourceIntent, DownloadResourceKind, DownloadTask, ResourceStatus,
    TaskStatus,
};
use bdl_core::storage::TaskStorage;
use uuid::Uuid;

#[test]
fn task_storage_creates_expected_tables() -> BdlResult<()> {
    let fixture = StorageFixture::new()?;
    let storage = TaskStorage::open(&fixture.db_path)?;

    let tables = storage.table_names()?;

    assert!(tables.contains(&"tasks".to_owned()));
    assert!(tables.contains(&"resources".to_owned()));
    assert!(tables.contains(&"segments".to_owned()));
    assert!(tables.contains(&"history".to_owned()));
    assert!(tables.contains(&"task_logs".to_owned()));
    Ok(())
}

#[test]
fn task_storage_reloads_task_with_resources_after_reopen() -> BdlResult<()> {
    let fixture = StorageFixture::new()?;
    let task = sample_task();

    {
        let mut storage = TaskStorage::open(&fixture.db_path)?;
        storage.save_task(&task)?;
    }

    let storage = TaskStorage::open(&fixture.db_path)?;
    let tasks = storage.load_tasks()?;

    assert_eq!(tasks, vec![task]);
    Ok(())
}

struct StorageFixture {
    dir: PathBuf,
    db_path: PathBuf,
}

impl StorageFixture {
    fn new() -> BdlResult<Self> {
        let dir = std::env::temp_dir().join(format!("bdl-storage-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&dir)?;
        let db_path = dir.join("tasks.sqlite");
        Ok(Self { dir, db_path })
    }
}

impl Drop for StorageFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn sample_task() -> DownloadTask {
    DownloadTask {
        id: "task:source:part".to_owned(),
        title: "Example - P1".to_owned(),
        source_id: "source".to_owned(),
        status: TaskStatus::Waiting,
        resources: vec![
            sample_resource(
                "task:source:part:resource:video",
                DownloadResourceKind::Video,
                DownloadResourceIntent::Video,
                ResourceStatus::Pending,
                "https://example.test/video.m4s",
            ),
            sample_resource(
                "task:source:part:resource:audio",
                DownloadResourceKind::Audio,
                DownloadResourceIntent::Audio,
                ResourceStatus::Completed,
                "https://example.test/audio.m4s",
            ),
        ],
        output_path: PathBuf::from("downloads/Example - P1.mp4"),
    }
}

fn sample_resource(
    id: &str,
    kind: DownloadResourceKind,
    intent: DownloadResourceIntent,
    status: ResourceStatus,
    url: &str,
) -> DownloadResource {
    DownloadResource {
        id: id.to_owned(),
        kind,
        intent,
        current_urls: vec![url.to_owned()],
        headers: vec![HeaderPair {
            name: "Referer".to_owned(),
            value: "https://www.bilibili.com".to_owned(),
        }],
        target_path: PathBuf::from(format!("downloads/{id}.m4s")),
        temp_path: PathBuf::from(format!("downloads/{id}.m4s.bdlpart")),
        status,
    }
}
