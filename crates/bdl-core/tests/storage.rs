use std::path::PathBuf;

use bdl_core::BdlResult;
use bdl_core::account::AccountSummary;
use bdl_core::model::HeaderPair;
use bdl_core::queue::{
    DownloadResource, DownloadResourceIntent, DownloadResourceKind, DownloadTask,
    DownloadTaskMediaSelection, DownloadTaskRefreshInput, DownloadTaskRefreshIntent, QueueLogEntry,
    QueueLogLevel, ResourceStatus, TaskStatus,
};
use bdl_core::storage::TaskStorage;
use chrono::{Duration, Utc};
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
    assert!(tables.contains(&"account_summary".to_owned()));
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

#[test]
fn task_storage_reloads_media_selection_after_reopen() -> BdlResult<()> {
    let fixture = StorageFixture::new()?;
    let task = sample_task();

    {
        let mut storage = TaskStorage::open(&fixture.db_path)?;
        storage.save_task(&task)?;
    }

    let storage = TaskStorage::open(&fixture.db_path)?;
    let tasks = storage.load_tasks()?;

    assert_eq!(tasks[0].media_selection, task.media_selection);
    Ok(())
}

#[test]
fn task_storage_reloads_speed_limit_after_reopen() -> BdlResult<()> {
    let fixture = StorageFixture::new()?;
    let task = sample_task();

    {
        let mut storage = TaskStorage::open(&fixture.db_path)?;
        storage.save_task(&task)?;
    }

    let storage = TaskStorage::open(&fixture.db_path)?;
    let tasks = storage.load_tasks()?;

    assert_eq!(tasks[0].speed_limit_bytes_per_second, Some(2 * 1024 * 1024));
    Ok(())
}

#[test]
fn task_storage_reloads_refresh_intent_after_reopen() -> BdlResult<()> {
    let fixture = StorageFixture::new()?;
    let task = sample_task();

    {
        let mut storage = TaskStorage::open(&fixture.db_path)?;
        storage.save_task(&task)?;
    }

    let storage = TaskStorage::open(&fixture.db_path)?;
    let tasks = storage.load_tasks()?;

    assert_eq!(tasks[0].refresh_intent, task.refresh_intent);
    Ok(())
}

#[test]
fn task_storage_reloads_task_logs_after_reopen() -> BdlResult<()> {
    let fixture = StorageFixture::new()?;
    let task = sample_task();
    let first = sample_log(
        &task.id,
        QueueLogLevel::Info,
        "开始下载任务",
        "2026-07-09T01:00:00Z",
    );
    let second = sample_log(
        &task.id,
        QueueLogLevel::Error,
        "fetch error: HTTP 403",
        "2026-07-09T01:00:01Z",
    );

    {
        let mut storage = TaskStorage::open(&fixture.db_path)?;
        storage.save_task(&task)?;
        storage.append_task_log(&first)?;
        storage.append_task_log(&second)?;
    }

    let storage = TaskStorage::open(&fixture.db_path)?;
    let logs = storage.load_task_logs(&task.id, 10)?;

    assert_eq!(logs, vec![second, first]);
    Ok(())
}

#[test]
fn task_storage_prunes_task_logs_older_than_retention_window() -> BdlResult<()> {
    let fixture = StorageFixture::new()?;
    let task = sample_task();
    let old_log = sample_log(
        &task.id,
        QueueLogLevel::Info,
        "old log",
        &(Utc::now() - Duration::days(31)).to_rfc3339(),
    );
    let fresh_log = sample_log(
        &task.id,
        QueueLogLevel::Info,
        "fresh log",
        &Utc::now().to_rfc3339(),
    );

    {
        let mut storage = TaskStorage::open(&fixture.db_path)?;
        storage.save_task(&task)?;
        storage.append_task_log(&old_log)?;
        storage.append_task_log(&fresh_log)?;
    }

    let storage = TaskStorage::open(&fixture.db_path)?;
    let logs = storage.load_task_logs(&task.id, 10)?;

    assert_eq!(logs, vec![fresh_log]);
    Ok(())
}

#[test]
fn task_storage_keeps_latest_thousand_logs_per_task() -> BdlResult<()> {
    let fixture = StorageFixture::new()?;
    let task = sample_task();

    {
        let mut storage = TaskStorage::open(&fixture.db_path)?;
        storage.save_task(&task)?;
        for index in 0..1002 {
            storage.append_task_log(&sample_log(
                &task.id,
                QueueLogLevel::Info,
                &format!("log {index}"),
                &Utc::now().to_rfc3339(),
            ))?;
        }
    }

    let storage = TaskStorage::open(&fixture.db_path)?;
    let logs = storage.load_task_logs(&task.id, 2000)?;

    assert_eq!(logs.len(), 1000);
    assert_eq!(logs[0].message, "log 1001");
    assert_eq!(logs[999].message, "log 2");
    Ok(())
}

#[test]
fn task_storage_saves_completed_record_with_media_metadata() -> BdlResult<()> {
    let fixture = StorageFixture::new()?;
    let mut task = sample_task();
    task.status = TaskStatus::Completed;
    let logs = vec![
        sample_log(
            &task.id,
            QueueLogLevel::Warning,
            "跳过字幕嵌入：当前字幕格式或封装格式不支持。",
            "2026-07-09T01:00:01Z",
        ),
        sample_log(
            &task.id,
            QueueLogLevel::Error,
            "Cookie: SESSDATA=secret; sign=abc",
            "2026-07-09T01:00:02Z",
        ),
    ];

    {
        let mut storage = TaskStorage::open(&fixture.db_path)?;
        storage.save_task(&task)?;
        storage.save_completed_record(&task, &logs)?;
    }

    let storage = TaskStorage::open(&fixture.db_path)?;
    let records = storage.load_completed_records()?;

    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record.task_id, task.id);
    assert_eq!(record.title, task.title);
    assert_eq!(record.source_id, task.source_id);
    assert_eq!(record.output_path, task.output_path);
    assert_eq!(record.selected_video_quality, "80");
    assert_eq!(record.selected_audio_quality, "30280");
    assert_eq!(record.selected_video_codec, "avc");
    assert_eq!(record.container, "mp4");
    assert!(record.completed_at.contains('T'));
    let summary = record
        .error_summary
        .as_deref()
        .expect("warning and error logs should be summarized");
    assert!(summary.contains("warning: 跳过字幕嵌入"));
    assert!(summary.contains("error: Cookie: <redacted>"));
    assert!(!summary.contains("secret"));
    assert!(!summary.contains("abc"));
    Ok(())
}

#[test]
fn task_storage_persists_account_summary_without_cookie_material() -> BdlResult<()> {
    let fixture = StorageFixture::new()?;
    let account = AccountSummary {
        logged_in: true,
        name: Some("fixture user".to_owned()),
        avatar_url: Some("https://example.test/avatar.jpg".to_owned()),
        mid: Some("42".to_owned()),
        vip_label: Some("年度大会员".to_owned()),
    };

    {
        let mut storage = TaskStorage::open(&fixture.db_path)?;
        storage.save_account_summary(&account)?;
    }

    let mut storage = TaskStorage::open(&fixture.db_path)?;
    assert_eq!(storage.load_account_summary()?, Some(account));

    storage.clear_account_summary()?;
    assert_eq!(storage.load_account_summary()?, None);
    Ok(())
}

#[test]
fn task_storage_keeps_task_logs_when_replacing_existing_task() -> BdlResult<()> {
    let fixture = StorageFixture::new()?;
    let mut task = sample_task();
    let log = sample_log(
        &task.id,
        QueueLogLevel::Info,
        "下载资源 视频",
        "2026-07-09T01:00:00Z",
    );

    {
        let mut storage = TaskStorage::open(&fixture.db_path)?;
        storage.save_task(&task)?;
        storage.append_task_log(&log)?;
        task.status = TaskStatus::Downloading;
        storage.replace_tasks(&[task])?;
    }

    let storage = TaskStorage::open(&fixture.db_path)?;
    let logs = storage.load_task_logs("task:source:part", 10)?;

    assert_eq!(logs, vec![log]);
    Ok(())
}

#[test]
fn task_storage_replace_tasks_tolerates_duplicate_task_ids() -> BdlResult<()> {
    let fixture = StorageFixture::new()?;
    let task = sample_task();
    let mut updated = sample_task();
    updated.title = "Example - P1 updated".to_owned();

    {
        let mut storage = TaskStorage::open(&fixture.db_path)?;
        storage.replace_tasks(&[task, updated.clone()])?;
    }

    let storage = TaskStorage::open(&fixture.db_path)?;
    let tasks = storage.load_tasks()?;

    assert_eq!(tasks, vec![updated]);
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
        refresh_intent: Some(DownloadTaskRefreshIntent {
            input: DownloadTaskRefreshInput::VideoBvid {
                bvid: "BV1xx411c7mD".to_owned(),
            },
            cid: 100,
        }),
        media_selection: DownloadTaskMediaSelection {
            video_quality: "80".to_owned(),
            audio_quality: "30280".to_owned(),
            video_codec: "avc".to_owned(),
            container: "mp4".to_owned(),
        },
        scheduled_at: Some(Utc::now() + Duration::minutes(30)),
        speed_limit_bytes_per_second: Some(2 * 1024 * 1024),
    }
}

fn sample_log(
    task_id: &str,
    level: QueueLogLevel,
    message: &str,
    created_at: &str,
) -> QueueLogEntry {
    QueueLogEntry {
        task_id: task_id.to_owned(),
        level,
        message: message.to_owned(),
        created_at: created_at.to_owned(),
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
