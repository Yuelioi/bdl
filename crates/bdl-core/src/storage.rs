use std::path::{Path, PathBuf};

use chrono::Utc;
use rusqlite::{Connection, Transaction, params};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::account::redact_sensitive;
use crate::error::BdlResult;
use crate::queue::{DownloadResource, DownloadTask, QueueLogEntry, QueueLogLevel};

const MAX_COMPLETION_ERROR_SUMMARY_CHARS: usize = 2000;

pub struct TaskStorage {
    conn: Connection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletedRecord {
    pub id: String,
    pub task_id: String,
    pub title: String,
    pub source_id: String,
    pub output_path: PathBuf,
    pub selected_video_quality: String,
    pub selected_audio_quality: String,
    pub selected_video_codec: String,
    pub container: String,
    pub error_summary: Option<String>,
    pub completed_at: String,
}

impl TaskStorage {
    pub fn open(path: impl AsRef<Path>) -> BdlResult<Self> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        run_migrations(&conn)?;

        Ok(Self { conn })
    }

    pub fn table_names(&self) -> BdlResult<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn save_task(&mut self, task: &DownloadTask) -> BdlResult<()> {
        let tx = self.conn.transaction()?;
        save_task_in_tx(&tx, task, 0)?;
        tx.commit()?;
        Ok(())
    }

    pub fn save_tasks(&mut self, tasks: &[DownloadTask]) -> BdlResult<()> {
        let tx = self.conn.transaction()?;
        for (index, task) in tasks.iter().enumerate() {
            save_task_in_tx(&tx, task, index)?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn replace_tasks(&mut self, tasks: &[DownloadTask]) -> BdlResult<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "CREATE TEMP TABLE IF NOT EXISTS task_storage_keep_tasks (id TEXT PRIMARY KEY)",
            [],
        )?;
        tx.execute("DELETE FROM task_storage_keep_tasks", [])?;
        for task in tasks {
            tx.execute(
                "INSERT OR IGNORE INTO task_storage_keep_tasks (id) VALUES (?1)",
                [task.id.as_str()],
            )?;
        }

        tx.execute(
            "DELETE FROM tasks WHERE id NOT IN (SELECT id FROM task_storage_keep_tasks)",
            [],
        )?;
        tx.execute("DELETE FROM task_storage_keep_tasks", [])?;

        for (index, task) in tasks.iter().enumerate() {
            save_task_in_tx(&tx, task, index)?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn load_tasks(&self) -> BdlResult<Vec<DownloadTask>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, source_id, status, output_path, media_selection FROM tasks ORDER BY sort_order, rowid",
        )?;
        let task_rows = stmt.query_map([], |row| {
            Ok(TaskRow {
                id: row.get(0)?,
                title: row.get(1)?,
                source_id: row.get(2)?,
                status_json: row.get(3)?,
                output_path: row.get(4)?,
                media_selection_json: row.get(5)?,
            })
        })?;

        let mut tasks = Vec::new();
        for task_row in task_rows {
            let task_row = task_row?;
            let resources = self.load_resources(&task_row.id)?;
            tasks.push(DownloadTask {
                id: task_row.id,
                title: task_row.title,
                source_id: task_row.source_id,
                status: deserialize_json(&task_row.status_json)?,
                resources,
                output_path: PathBuf::from(task_row.output_path),
                media_selection: deserialize_json(&task_row.media_selection_json)?,
            });
        }

        Ok(tasks)
    }

    pub fn save_completed_record(
        &mut self,
        task: &DownloadTask,
        logs: &[QueueLogEntry],
    ) -> BdlResult<CompletedRecord> {
        let record = completed_record_from_task(task, logs, Utc::now().to_rfc3339());
        self.conn.execute(
            r#"
            INSERT INTO history (
                id,
                task_id,
                title,
                source_id,
                output_path,
                selected_video_quality,
                selected_audio_quality,
                selected_video_codec,
                container,
                error_summary,
                completed_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(id) DO UPDATE SET
                task_id = excluded.task_id,
                title = excluded.title,
                source_id = excluded.source_id,
                output_path = excluded.output_path,
                selected_video_quality = excluded.selected_video_quality,
                selected_audio_quality = excluded.selected_audio_quality,
                selected_video_codec = excluded.selected_video_codec,
                container = excluded.container,
                error_summary = excluded.error_summary,
                completed_at = excluded.completed_at
            "#,
            params![
                record.id.as_str(),
                record.task_id.as_str(),
                record.title.as_str(),
                record.source_id.as_str(),
                path_to_string(&record.output_path),
                record.selected_video_quality.as_str(),
                record.selected_audio_quality.as_str(),
                record.selected_video_codec.as_str(),
                record.container.as_str(),
                record.error_summary.as_deref(),
                record.completed_at.as_str(),
            ],
        )?;

        Ok(record)
    }

    pub fn load_completed_records(&self) -> BdlResult<Vec<CompletedRecord>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT
                id,
                task_id,
                title,
                source_id,
                output_path,
                selected_video_quality,
                selected_audio_quality,
                selected_video_codec,
                container,
                error_summary,
                completed_at
            FROM history
            ORDER BY completed_at DESC, rowid DESC
            "#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(CompletedRecord {
                id: row.get(0)?,
                task_id: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                title: row.get(2)?,
                source_id: row.get(3)?,
                output_path: PathBuf::from(row.get::<_, String>(4)?),
                selected_video_quality: row.get(5)?,
                selected_audio_quality: row.get(6)?,
                selected_video_codec: row.get(7)?,
                container: row.get(8)?,
                error_summary: row.get(9)?,
                completed_at: row.get(10)?,
            })
        })?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn append_task_log(&mut self, entry: &QueueLogEntry) -> BdlResult<()> {
        self.conn.execute(
            r#"
            INSERT INTO task_logs (task_id, level, message, created_at)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![
                entry.task_id.as_str(),
                serialize_json(&entry.level)?,
                entry.message.as_str(),
                entry.created_at.as_str(),
            ],
        )?;
        Ok(())
    }

    pub fn load_task_logs(&self, task_id: &str, limit: usize) -> BdlResult<Vec<QueueLogEntry>> {
        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let mut stmt = self.conn.prepare(
            "SELECT task_id, level, message, created_at
             FROM task_logs
             WHERE task_id = ?1
             ORDER BY id DESC
             LIMIT ?2",
        )?;
        let log_rows = stmt.query_map(params![task_id, limit], |row| {
            Ok(TaskLogRow {
                task_id: row.get(0)?,
                level_json: row.get(1)?,
                message: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;

        let mut logs = Vec::new();
        for log_row in log_rows {
            let log_row = log_row?;
            logs.push(QueueLogEntry {
                task_id: log_row.task_id,
                level: deserialize_json(&log_row.level_json)?,
                message: log_row.message,
                created_at: log_row.created_at,
            });
        }

        Ok(logs)
    }

    fn load_resources(&self, task_id: &str) -> BdlResult<Vec<DownloadResource>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, intent, current_urls, headers, target_path, temp_path, status
             FROM resources
             WHERE task_id = ?1
             ORDER BY sort_order, rowid",
        )?;
        let resource_rows = stmt.query_map([task_id], |row| {
            Ok(ResourceRow {
                id: row.get(0)?,
                kind_json: row.get(1)?,
                intent_json: row.get(2)?,
                current_urls_json: row.get(3)?,
                headers_json: row.get(4)?,
                target_path: row.get(5)?,
                temp_path: row.get(6)?,
                status_json: row.get(7)?,
            })
        })?;

        let mut resources = Vec::new();
        for resource_row in resource_rows {
            let resource_row = resource_row?;
            resources.push(DownloadResource {
                id: resource_row.id,
                kind: deserialize_json(&resource_row.kind_json)?,
                intent: deserialize_json(&resource_row.intent_json)?,
                current_urls: deserialize_json(&resource_row.current_urls_json)?,
                headers: deserialize_json(&resource_row.headers_json)?,
                target_path: PathBuf::from(resource_row.target_path),
                temp_path: PathBuf::from(resource_row.temp_path),
                status: deserialize_json(&resource_row.status_json)?,
            });
        }

        Ok(resources)
    }
}

struct TaskRow {
    id: String,
    title: String,
    source_id: String,
    status_json: String,
    output_path: String,
    media_selection_json: String,
}

struct ResourceRow {
    id: String,
    kind_json: String,
    intent_json: String,
    current_urls_json: String,
    headers_json: String,
    target_path: String,
    temp_path: String,
    status_json: String,
}

struct TaskLogRow {
    task_id: String,
    level_json: String,
    message: String,
    created_at: String,
}

fn run_migrations(conn: &Connection) -> BdlResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            source_id TEXT NOT NULL,
            status TEXT NOT NULL,
            output_path TEXT NOT NULL,
            media_selection TEXT NOT NULL DEFAULT '{"video_quality":"unknown","audio_quality":"unknown","video_codec":"unknown","container":"unknown"}',
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS resources (
            id TEXT PRIMARY KEY,
            task_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            intent TEXT NOT NULL,
            current_urls TEXT NOT NULL,
            headers TEXT NOT NULL,
            target_path TEXT NOT NULL,
            temp_path TEXT NOT NULL,
            status TEXT NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS segments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            resource_id TEXT NOT NULL,
            offset INTEGER NOT NULL,
            length INTEGER NOT NULL,
            completed INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (resource_id) REFERENCES resources(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS history (
            id TEXT PRIMARY KEY,
            task_id TEXT,
            title TEXT NOT NULL,
            source_id TEXT NOT NULL DEFAULT '',
            output_path TEXT NOT NULL,
            selected_video_quality TEXT NOT NULL DEFAULT 'unknown',
            selected_audio_quality TEXT NOT NULL DEFAULT 'unknown',
            selected_video_codec TEXT NOT NULL DEFAULT 'unknown',
            container TEXT NOT NULL DEFAULT 'unknown',
            error_summary TEXT,
            completed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS task_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id TEXT NOT NULL,
            level TEXT NOT NULL,
            message TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
        );
        "#,
    )?;
    add_column_if_missing(
        conn,
        "tasks",
        "media_selection",
        "media_selection TEXT NOT NULL DEFAULT '{\"video_quality\":\"unknown\",\"audio_quality\":\"unknown\",\"video_codec\":\"unknown\",\"container\":\"unknown\"}'",
    )?;
    add_column_if_missing(
        conn,
        "history",
        "source_id",
        "source_id TEXT NOT NULL DEFAULT ''",
    )?;
    add_column_if_missing(
        conn,
        "history",
        "selected_video_quality",
        "selected_video_quality TEXT NOT NULL DEFAULT 'unknown'",
    )?;
    add_column_if_missing(
        conn,
        "history",
        "selected_audio_quality",
        "selected_audio_quality TEXT NOT NULL DEFAULT 'unknown'",
    )?;
    add_column_if_missing(
        conn,
        "history",
        "selected_video_codec",
        "selected_video_codec TEXT NOT NULL DEFAULT 'unknown'",
    )?;
    add_column_if_missing(
        conn,
        "history",
        "container",
        "container TEXT NOT NULL DEFAULT 'unknown'",
    )?;
    add_column_if_missing(conn, "history", "error_summary", "error_summary TEXT")?;
    Ok(())
}

fn save_task_in_tx(tx: &Transaction<'_>, task: &DownloadTask, sort_order: usize) -> BdlResult<()> {
    tx.execute(
        r#"
        INSERT INTO tasks (id, title, source_id, status, output_path, media_selection, sort_order, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            source_id = excluded.source_id,
            status = excluded.status,
            output_path = excluded.output_path,
            media_selection = excluded.media_selection,
            sort_order = excluded.sort_order,
            updated_at = CURRENT_TIMESTAMP
        "#,
        params![
            task.id,
            task.title,
            task.source_id,
            serialize_json(&task.status)?,
            path_to_string(&task.output_path),
            serialize_json(&task.media_selection)?,
            sort_order as i64,
        ],
    )?;

    tx.execute("DELETE FROM resources WHERE task_id = ?1", [&task.id])?;

    for (resource_order, resource) in task.resources.iter().enumerate() {
        save_resource_in_tx(tx, &task.id, resource, resource_order)?;
    }

    Ok(())
}

fn save_resource_in_tx(
    tx: &Transaction<'_>,
    task_id: &str,
    resource: &DownloadResource,
    sort_order: usize,
) -> BdlResult<()> {
    tx.execute(
        r#"
        INSERT INTO resources (
            id, task_id, kind, intent, current_urls, headers, target_path, temp_path, status, sort_order, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, CURRENT_TIMESTAMP)
        "#,
        params![
            resource.id,
            task_id,
            serialize_json(&resource.kind)?,
            serialize_json(&resource.intent)?,
            serialize_json(&resource.current_urls)?,
            serialize_json(&resource.headers)?,
            path_to_string(&resource.target_path),
            path_to_string(&resource.temp_path),
            serialize_json(&resource.status)?,
            sort_order as i64,
        ],
    )?;
    Ok(())
}

fn serialize_json(value: &impl Serialize) -> BdlResult<String> {
    serde_json::to_string(value).map_err(Into::into)
}

fn deserialize_json<T>(value: &str) -> BdlResult<T>
where
    T: DeserializeOwned,
{
    serde_json::from_str(value).map_err(Into::into)
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    column_definition: &str,
) -> BdlResult<()> {
    if column_exists(conn, table, column)? {
        return Ok(());
    }

    conn.execute(
        &format!("ALTER TABLE {table} ADD COLUMN {column_definition}"),
        [],
    )?;
    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> BdlResult<bool> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;

    for existing in columns {
        if existing? == column {
            return Ok(true);
        }
    }

    Ok(false)
}

fn completed_record_from_task(
    task: &DownloadTask,
    logs: &[QueueLogEntry],
    completed_at: String,
) -> CompletedRecord {
    CompletedRecord {
        id: task.id.clone(),
        task_id: task.id.clone(),
        title: task.title.clone(),
        source_id: task.source_id.clone(),
        output_path: task.output_path.clone(),
        selected_video_quality: task.media_selection.video_quality.clone(),
        selected_audio_quality: task.media_selection.audio_quality.clone(),
        selected_video_codec: task.media_selection.video_codec.clone(),
        container: task.media_selection.container.clone(),
        error_summary: completion_error_summary(logs),
        completed_at,
    }
}

fn completion_error_summary(logs: &[QueueLogEntry]) -> Option<String> {
    let mut relevant_logs = logs
        .iter()
        .filter(|log| matches!(log.level, QueueLogLevel::Warning | QueueLogLevel::Error))
        .collect::<Vec<_>>();
    relevant_logs.sort_by(|left, right| left.created_at.cmp(&right.created_at));

    let summary = relevant_logs
        .into_iter()
        .map(|log| {
            format!(
                "{}: {}",
                queue_log_level_label(log.level),
                redact_sensitive(&log.message)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let summary = truncate_chars(&summary, MAX_COMPLETION_ERROR_SUMMARY_CHARS);

    (!summary.is_empty()).then_some(summary)
}

fn queue_log_level_label(level: QueueLogLevel) -> &'static str {
    match level {
        QueueLogLevel::Info => "info",
        QueueLogLevel::Warning => "warning",
        QueueLogLevel::Error => "error",
    }
}

fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        return value.to_owned();
    }

    let keep_chars = max_chars.saturating_sub(3);
    let mut truncated = value.chars().take(keep_chars).collect::<String>();
    truncated.push_str("...");
    truncated
}
