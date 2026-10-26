use bdl_core::{
    BdlResult,
    naming::DuplicateNamingStrategy,
    queue::{
        DownloadResource, DownloadResourceIntent, DownloadResourceKind, DownloadTask,
        DownloadTaskMediaSelection, DuplicateTaskPolicy, ResourceStatus, TaskStatus,
    },
    storage::TaskStorage,
};
use bdl_tauri::{
    secure_store::{CredentialBackend, SecureStore},
    state::AppState,
};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Condvar, Mutex},
};

#[derive(Clone, Default)]
struct Credentials(Arc<CredentialGate>);
type CredentialGate = (Mutex<(Option<String>, bool, bool)>, Condvar);
impl CredentialBackend for Credentials {
    fn load_cookie(&self) -> BdlResult<Option<String>> {
        Ok(self.0.0.lock().unwrap().0.clone())
    }
    fn save_cookie(&self, cookie: &str) -> BdlResult<()> {
        let (lock, changed) = &*self.0;
        let mut state = lock.lock().unwrap();
        state.0 = Some(cookie.into());
        if cookie.contains("DedeUserID=101") {
            state.1 = true;
            changed.notify_all();
            while !state.2 {
                state = changed.wait(state).unwrap();
            }
        }
        Ok(())
    }
    fn clear_cookie(&self) -> BdlResult<()> {
        self.0.0.lock().unwrap().0 = None;
        Ok(())
    }
}

struct TestDirectory(PathBuf);
impl Drop for TestDirectory {
    fn drop(&mut self) {
        assert!(self.0.is_absolute() && self.0.parent().unwrap().ends_with(".task"));
        assert!(
            !std::fs::symlink_metadata(&self.0)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
struct Fixture {
    root: PathBuf,
    state: Arc<AppState>,
    credentials: Credentials,
    _cleanup: TestDirectory,
}
impl Fixture {
    fn new() -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join(".task")
            .join(format!("core-contracts-{}", uuid::Uuid::new_v4()));
        let credentials = Credentials::default();
        let state = Arc::new(
            AppState::new_with_secure_store(
                root.join("data"),
                root.join("downloads"),
                SecureStore::from_backend(credentials.clone()),
            )
            .unwrap(),
        );
        Self {
            _cleanup: TestDirectory(root.clone()),
            root,
            state,
            credentials,
        }
    }
    fn db(&self) -> rusqlite::Connection {
        rusqlite::Connection::open(self.root.join("data/tasks.sqlite")).unwrap()
    }
    fn tasks(&self) -> Vec<DownloadTask> {
        TaskStorage::open(self.root.join("data/tasks.sqlite"))
            .unwrap()
            .load_tasks()
            .unwrap()
    }
    fn enqueue(&self, tasks: Vec<DownloadTask>) -> BdlResult<Vec<DownloadTask>> {
        Ok(self
            .state
            .enqueue_tasks_with_duplicate_policy(
                tasks,
                DuplicateTaskPolicy::Create,
                DuplicateNamingStrategy::AppendSuffix,
            )?
            .inserted)
    }
}
fn task(root: &Path, id: &str, file: &str) -> DownloadTask {
    DownloadTask {
        id: id.into(),
        title: id.into(),
        source_id: id.into(),
        status: TaskStatus::Waiting,
        output_path: root.join(file),
        export_target: None,
        refresh_intent: None,
        media_selection: DownloadTaskMediaSelection::default(),
        scheduled_at: None,
        speed_limit_bytes_per_second: None,
        resources: vec![DownloadResource {
            id: format!("{id}:video"),
            kind: DownloadResourceKind::Video,
            intent: DownloadResourceIntent::Video,
            current_urls: vec!["https://example.invalid/video".into()],
            headers: vec![],
            target_path: root.join(format!("{file}.m4s")),
            temp_path: root.join(format!("{file}.m4s.bdlpart")),
            status: ResourceStatus::Pending,
        }],
    }
}

#[test]
fn queue_database_failures_do_not_publish_memory_or_cancel_running_attempts() {
    let fixture = Fixture::new();
    let tasks = fixture
        .enqueue(vec![task(&fixture.root, "a", "a.mp4")])
        .unwrap();
    let db = fixture.db();
    db.execute_batch("CREATE TRIGGER reject_update BEFORE UPDATE ON tasks BEGIN SELECT RAISE(ABORT, 'injected failure'); END; CREATE TRIGGER reject_insert BEFORE INSERT ON tasks BEGIN SELECT RAISE(ABORT, 'injected failure'); END;").unwrap();
    assert!(fixture.state.take_next_startable_task().is_err());
    let token = fixture.state.register_task_cancel_token("a").unwrap();
    assert!(
        fixture
            .state
            .update_task_status("a", TaskStatus::Paused)
            .is_err()
    );
    assert!(!token.is_cancelled());
    assert!(
        fixture
            .enqueue(vec![task(&fixture.root, "b", "b.mp4")])
            .is_err()
    );
    assert_eq!(fixture.state.queue_snapshot().unwrap(), tasks);
    assert_eq!(fixture.tasks(), tasks);
    db.execute_batch("DROP TRIGGER reject_update; DROP TRIGGER reject_insert;")
        .unwrap();
    fixture.state.clear_task_cancel_token("a").unwrap();
    assert_eq!(
        fixture
            .state
            .take_next_startable_task()
            .unwrap()
            .unwrap()
            .id,
        "a"
    );
    assert_eq!(fixture.tasks()[0].status, TaskStatus::Downloading);
}

#[test]
fn queue_status_changes_preserve_unrelated_rows_and_resource_rows() {
    let fixture = Fixture::new();
    fixture
        .enqueue(vec![
            task(&fixture.root, "a", "a.mp4"),
            task(&fixture.root, "b", "b.mp4"),
        ])
        .unwrap();
    fixture.db().execute_batch("CREATE TRIGGER protect_resources BEFORE DELETE ON resources BEGIN SELECT RAISE(ABORT, 'unnecessary resource rewrite'); END; CREATE TRIGGER protect_b BEFORE UPDATE ON tasks WHEN OLD.id = 'b' BEGIN SELECT RAISE(ABORT, 'unrelated task rewrite'); END;").unwrap();
    fixture
        .state
        .update_task_status("a", TaskStatus::Paused)
        .unwrap();
    fixture.db().execute("INSERT INTO segments (resource_id, offset, length, completed) VALUES ('a:video', 0, 4, 1)", []).unwrap();
    fixture
        .state
        .update_resource_status("a", "a:video", ResourceStatus::Completed)
        .unwrap();
    assert_eq!(
        fixture
            .db()
            .query_row(
                "SELECT COUNT(*) FROM segments WHERE resource_id = 'a:video'",
                [],
                |row| row.get::<_, i64>(0)
            )
            .unwrap(),
        1
    );
    assert_eq!(fixture.tasks(), fixture.state.queue_snapshot().unwrap());
    assert_eq!(
        fixture
            .tasks()
            .iter()
            .map(|task| task.id.as_str())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
    fixture
        .db()
        .execute_batch("DROP TRIGGER protect_resources; DROP TRIGGER protect_b;")
        .unwrap();
    assert!(fixture.state.remove_task("a").unwrap());
    fixture
        .enqueue(vec![task(&fixture.root, "c", "c.mp4")])
        .unwrap();
    assert_eq!(
        fixture
            .tasks()
            .iter()
            .map(|task| task.id.as_str())
            .collect::<Vec<_>>(),
        ["b", "c"]
    );
    assert_eq!(fixture.tasks(), fixture.state.queue_snapshot().unwrap());
}

#[test]
fn overlapping_account_imports_publish_one_matching_summary_and_credential() {
    let fixture = Fixture::new();
    let a = fixture.state.clone();
    let first = std::thread::spawn(move || {
        a.import_cookie("DedeUserID=101; SESSDATA=test-a; bili_jct=csrf")
            .unwrap()
    });
    let (lock, changed) = &*fixture.credentials.0;
    let mut held = lock.lock().unwrap();
    while !held.1 {
        let (next, wait) = changed
            .wait_timeout(held, std::time::Duration::from_secs(10))
            .unwrap();
        held = next;
        assert!(!wait.timed_out(), "first credential write never started");
    }
    let b = fixture.state.clone();
    let (finished, completion) = std::sync::mpsc::channel();
    let second = std::thread::spawn(move || {
        let result = b.import_cookie("DedeUserID=202; SESSDATA=test-b; bili_jct=csrf");
        finished.send(()).unwrap();
        result.unwrap()
    });
    drop(held);
    let early_completion = completion.recv_timeout(std::time::Duration::from_millis(100));
    let mut held = lock.lock().unwrap();
    held.2 = true;
    changed.notify_all();
    drop(held);
    first.join().unwrap();
    second.join().unwrap();
    assert!(matches!(
        early_completion,
        Err(std::sync::mpsc::RecvTimeoutError::Timeout)
    ));
    assert_eq!(fixture.state.account().unwrap().mid.as_deref(), Some("202"));
    assert!(
        fixture
            .credentials
            .load_cookie()
            .unwrap()
            .unwrap()
            .contains("DedeUserID=202")
    );
    assert_eq!(
        TaskStorage::open(fixture.root.join("data/tasks.sqlite"))
            .unwrap()
            .load_account_summary()
            .unwrap()
            .unwrap()
            .mid
            .as_deref(),
        Some("202")
    );
}

#[test]
fn failed_account_summary_commit_restores_previous_credentials_and_profile() {
    let fixture = Fixture::new();
    let original = fixture
        .state
        .import_cookie("DedeUserID=202; SESSDATA=test-b; bili_jct=csrf")
        .unwrap();
    let credential = fixture.credentials.load_cookie().unwrap();
    fixture.db().execute_batch("CREATE TRIGGER reject_account BEFORE INSERT ON account_summary BEGIN SELECT RAISE(ABORT, 'injected account failure'); END; CREATE TRIGGER reject_logout BEFORE DELETE ON account_summary BEGIN SELECT RAISE(ABORT, 'injected logout failure'); END;").unwrap();
    assert!(
        fixture
            .state
            .import_cookie("DedeUserID=303; SESSDATA=test-c; bili_jct=csrf")
            .is_err()
    );
    assert_eq!(fixture.credentials.load_cookie().unwrap(), credential);
    assert_eq!(fixture.state.account().unwrap(), original);
    assert!(fixture.state.logout().is_err());
    assert_eq!(fixture.credentials.load_cookie().unwrap(), credential);
    assert_eq!(fixture.state.account().unwrap(), original);
}

#[test]
fn closed_qr_requests_cannot_register_or_import_and_old_cancel_preserves_new_login() {
    let fixture = Fixture::new();
    let old = fixture.state.begin_qr_login("old").unwrap();
    fixture.state.cancel_qr_login("old").unwrap();
    assert!(fixture.state.register_qr_login(old, "old-key").is_err());
    let current = fixture.state.begin_qr_login("new").unwrap();
    fixture.state.register_qr_login(current, "new-key").unwrap();
    fixture.state.cancel_qr_login("old").unwrap();
    assert_eq!(
        fixture.state.qr_login_generation("new-key").unwrap(),
        current
    );
    fixture.state.logout().unwrap();
    assert!(
        fixture
            .state
            .import_qr_cookie(
                "new-key",
                current,
                "DedeUserID=202; SESSDATA=late; bili_jct=csrf"
            )
            .is_err()
    );
    assert!(!fixture.state.account().unwrap().logged_in);
    assert!(fixture.credentials.load_cookie().unwrap().is_none());
}

#[cfg(windows)]
#[test]
fn windows_queue_reserves_case_aliases_as_distinct_outputs() {
    let fixture = Fixture::new();
    let inserted = fixture
        .enqueue(vec![
            task(&fixture.root, "a", "Video.mp4"),
            task(&fixture.root, "b", "video.mp4"),
        ])
        .unwrap();
    assert_ne!(
        inserted[0].output_path.to_string_lossy().to_lowercase(),
        inserted[1].output_path.to_string_lossy().to_lowercase()
    );
    std::fs::write(&inserted[0].output_path, b"first").unwrap();
    std::fs::write(&inserted[1].output_path, b"second").unwrap();
    assert_eq!(std::fs::read(&inserted[0].output_path).unwrap(), b"first");
}
