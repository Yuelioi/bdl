use std::path::{Path, PathBuf};

use chrono::{Duration, Utc};
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::account::{AccountSummary, redact_sensitive};
use crate::error::{BdlError, BdlResult, IoResultExt};
use crate::queue::{DownloadResource, DownloadTask, QueueLogEntry, QueueLogLevel};

const MAX_COMPLETION_ERROR_SUMMARY_CHARS: usize = 2000;
const TASK_LOG_RETENTION_DAYS: i64 = 30;
const MAX_TASK_LOGS_PER_TASK: usize = 1000;
const MAX_OPERATION_ERRORS: usize = 200;

#[derive(Debug, Clone, Serialize)]
pub struct OperationErrorRecord {
    pub id: i64,
    pub command: String,
    pub code: String,
    pub message: String,
    pub detail: Option<String>,
    pub environment: serde_json::Value,
    pub created_at: String,
}

pub struct TaskStorage {
    conn: Connection,
}

#[derive(Debug, Serialize)]
pub struct TaskLogCleanup {
    pub removed_logs: usize,
    pub task_ids: Vec<String>,
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
            std::fs::create_dir_all(parent).path_context("create_database_directory", parent)?;
        }

        let conn = Connection::open(path).map_err(|error| {
            BdlError::from(error)
                .with_context(format!("[stage=database_open] path={}", path.display()))
        })?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        run_migrations(&conn).map_err(|error| {
            error.with_context(format!("[stage=database_schema] path={}", path.display()))
        })?;

        let mut storage = Self { conn };
        storage.prune_expired_task_logs()?;
        prune_operation_errors(&storage.conn)?;
        prune_preparation_records(&storage.conn)?;
        Ok(storage)
    }

    pub fn table_names(&self) -> BdlResult<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn save_preparation_record(
        &mut self,
        id: &str,
        status: &str,
        record: &serde_json::Value,
    ) -> BdlResult<()> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO task_preparations (id, status, record, updated_at) VALUES (?1, ?2, ?3, ?4) ON CONFLICT(id) DO UPDATE SET status=excluded.status, record=excluded.record, updated_at=excluded.updated_at",
            params![id, status, serde_json::to_string(record)?, Utc::now().to_rfc3339()],
        )?;
        prune_preparation_records(&tx)?;
        tx.commit()?;
        Ok(())
    }

    pub fn preparation_records(&self) -> BdlResult<Vec<serde_json::Value>> {
        let mut statement = self
            .conn
            .prepare("SELECT record FROM task_preparations ORDER BY updated_at DESC")?;
        let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
        rows.map(|row| Ok(serde_json::from_str(&row?)?)).collect()
    }

    pub fn delete_preparation_record(&mut self, id: &str) -> BdlResult<()> {
        self.conn
            .execute("DELETE FROM task_preparations WHERE id=?1", [id])?;
        Ok(())
    }

    pub fn append_operation_error(
        &mut self,
        command: &str,
        code: &str,
        message: &str,
        detail: Option<&str>,
        environment: &serde_json::Value,
    ) -> BdlResult<i64> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO operation_errors (command, code, message, detail, environment, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![redact_sensitive(command), redact_sensitive(code), redact_sensitive(message),
                detail.map(crate::account::redact_diagnostic), serde_json::to_string(environment)?, Utc::now().to_rfc3339()],
        )?;
        let id = tx.last_insert_rowid();
        prune_operation_errors(&tx)?;
        tx.commit()?;
        Ok(id)
    }

    pub fn operation_errors(&self, limit: usize) -> BdlResult<Vec<OperationErrorRecord>> {
        let mut statement = self.conn.prepare(
            "SELECT id, command, code, message, detail, environment, created_at FROM operation_errors ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = statement.query_map([limit.min(MAX_OPERATION_ERRORS) as i64], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })?;
        rows.map(|row| {
            let (id, command, code, message, detail, environment, created_at) = row?;
            Ok(OperationErrorRecord {
                id,
                command,
                code,
                message,
                detail,
                environment: serde_json::from_str(&environment)?,
                created_at,
            })
        })
        .collect()
    }

    pub fn operation_error_usage(&self) -> BdlResult<(u64, u64)> {
        Ok(self.conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(length(CAST(command || code || message || COALESCE(detail, '') || environment AS BLOB))), 0) FROM operation_errors",
            [], |row| Ok((row.get(0)?, row.get(1)?)),
        )?)
    }

    pub fn clear_operation_errors(&mut self) -> BdlResult<usize> {
        Ok(self.conn.execute("DELETE FROM operation_errors", [])?)
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

    /// Commit a queue edit without rewriting unrelated tasks or their resources.
    pub fn apply_queue_changes(
        &mut self,
        previous: &[DownloadTask],
        next: &[DownloadTask],
    ) -> BdlResult<()> {
        let old: std::collections::HashMap<_, _> = previous
            .iter()
            .enumerate()
            .map(|(index, task)| (task.id.as_str(), (index, task)))
            .collect();
        let retained: std::collections::HashSet<_> =
            next.iter().map(|task| task.id.as_str()).collect();
        let mut inserted = false;
        let insertion_before_existing = next.iter().any(|task| {
            if old.contains_key(task.id.as_str()) {
                inserted
            } else {
                inserted = true;
                false
            }
        });
        let reordered = insertion_before_existing
            || !previous
                .iter()
                .filter(|task| retained.contains(task.id.as_str()))
                .map(|task| &task.id)
                .eq(next
                    .iter()
                    .filter(|task| old.contains_key(task.id.as_str()))
                    .map(|task| &task.id));
        let tx = self.conn.transaction()?;
        for task in previous
            .iter()
            .filter(|task| !retained.contains(task.id.as_str()))
        {
            tx.execute("DELETE FROM tasks WHERE id = ?1", [&task.id])?;
        }
        let mut append_order: usize = tx.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM tasks",
            [],
            |row| row.get(0),
        )?;
        for (index, task) in next.iter().enumerate() {
            if !reordered
                && old
                    .get(task.id.as_str())
                    .is_some_and(|(_, old_task)| *old_task == task)
            {
                continue;
            }
            let order = if reordered {
                index
            } else if old.contains_key(task.id.as_str()) {
                tx.query_row(
                    "SELECT sort_order FROM tasks WHERE id = ?1",
                    [&task.id],
                    |row| row.get(0),
                )?
            } else {
                let order = append_order;
                append_order += 1;
                order
            };
            if let Some((_, old_task)) = old.get(task.id.as_str()) {
                save_task_row_in_tx(&tx, task, order)?;
                if old_task.resources != task.resources {
                    let resources: std::collections::HashMap<_, _> = old_task
                        .resources
                        .iter()
                        .enumerate()
                        .map(|(index, resource)| (resource.id.as_str(), (index, resource)))
                        .collect();
                    let retained: std::collections::HashSet<_> = task
                        .resources
                        .iter()
                        .map(|resource| resource.id.as_str())
                        .collect();
                    for resource in old_task
                        .resources
                        .iter()
                        .filter(|resource| !retained.contains(resource.id.as_str()))
                    {
                        tx.execute(
                            "DELETE FROM resources WHERE task_id = ?1 AND id = ?2",
                            params![task.id, resource.id],
                        )?;
                    }
                    for (index, resource) in task.resources.iter().enumerate() {
                        let previous = resources.get(resource.id.as_str());
                        if previous
                            .is_some_and(|(old_index, old)| *old_index == index && *old == resource)
                        {
                            continue;
                        }
                        save_resource_in_tx(&tx, &task.id, resource, index, previous.is_some())?;
                    }
                }
            } else {
                save_task_in_tx(&tx, task, order)?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn load_tasks(&self) -> BdlResult<Vec<DownloadTask>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, source_id, status, output_path, export_target, refresh_intent, media_selection, scheduled_at, speed_limit_bytes_per_second FROM tasks ORDER BY sort_order, rowid",
        )?;
        let task_rows = stmt.query_map([], |row| {
            Ok(TaskRow {
                id: row.get(0)?,
                title: row.get(1)?,
                source_id: row.get(2)?,
                status_json: row.get(3)?,
                output_path: row.get(4)?,
                export_target_json: row.get(5)?,
                refresh_intent_json: row.get(6)?,
                media_selection_json: row.get(7)?,
                scheduled_at_json: row.get(8)?,
                speed_limit_bytes_per_second: row.get(9)?,
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
                export_target: task_row
                    .export_target_json
                    .as_deref()
                    .map(deserialize_json)
                    .transpose()?,
                refresh_intent: task_row
                    .refresh_intent_json
                    .as_deref()
                    .map(deserialize_json)
                    .transpose()?,
                media_selection: deserialize_json(&task_row.media_selection_json)?,
                scheduled_at: task_row
                    .scheduled_at_json
                    .as_deref()
                    .map(deserialize_json)
                    .transpose()?,
                speed_limit_bytes_per_second: task_row
                    .speed_limit_bytes_per_second
                    .and_then(|value| u64::try_from(value).ok())
                    .filter(|value| *value > 0),
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

    pub fn save_account_summary(&mut self, account: &AccountSummary) -> BdlResult<()> {
        self.conn.execute(
            r#"
            INSERT INTO account_summary (id, summary, updated_at)
            VALUES (1, ?1, CURRENT_TIMESTAMP)
            ON CONFLICT(id) DO UPDATE SET
                summary = excluded.summary,
                updated_at = CURRENT_TIMESTAMP
            "#,
            [serialize_json(account)?],
        )?;
        Ok(())
    }

    pub fn load_account_summary(&self) -> BdlResult<Option<AccountSummary>> {
        let summary_json = self
            .conn
            .query_row(
                "SELECT summary FROM account_summary WHERE id = 1",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()?;

        summary_json.as_deref().map(deserialize_json).transpose()
    }

    pub fn clear_account_summary(&mut self) -> BdlResult<()> {
        self.conn
            .execute("DELETE FROM account_summary WHERE id = 1", [])?;
        Ok(())
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
        self.prune_task_logs(entry.task_id.as_str())?;
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

    pub fn task_log_usage(&self) -> BdlResult<(u64, u64, u64)> {
        let (operations, operation_bytes) = self.operation_error_usage()?;
        let (count, bytes, cleanable): (u64, u64, u64) = self.conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(length(CAST(message AS BLOB))), 0),
                    COALESCE(SUM(CASE WHEN tasks.status IN ('\"completed\"', '\"failed\"', '\"cancelled\"') THEN 1 ELSE 0 END), 0)
             FROM task_logs JOIN tasks ON tasks.id = task_logs.task_id",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )?;
        Ok((
            count + operations,
            bytes + operation_bytes,
            cleanable + operations,
        ))
    }

    /// Keep active, queued and paused attempts intact: queue recovery reads their logs.
    pub fn clear_finished_task_logs(&mut self) -> BdlResult<TaskLogCleanup> {
        let transaction = self.conn.transaction()?;
        let task_ids = {
            let mut statement = transaction.prepare(
                "SELECT DISTINCT task_id FROM task_logs JOIN tasks ON tasks.id = task_logs.task_id
                 WHERE tasks.status IN ('\"completed\"', '\"failed\"', '\"cancelled\"')",
            )?;
            statement
                .query_map([], |row| row.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?
        };
        let removed_logs = transaction.execute(
            "DELETE FROM task_logs WHERE task_id IN
             (SELECT id FROM tasks WHERE status IN ('\"completed\"', '\"failed\"', '\"cancelled\"'))",
            [],
        )? + transaction.execute("DELETE FROM operation_errors", [])?;
        transaction.commit()?;
        Ok(TaskLogCleanup {
            removed_logs,
            task_ids,
        })
    }

    fn prune_expired_task_logs(&mut self) -> BdlResult<()> {
        let cutoff = (Utc::now() - Duration::days(TASK_LOG_RETENTION_DAYS)).to_rfc3339();
        self.conn.execute(
            "DELETE FROM task_logs WHERE datetime(created_at) < datetime(?1)",
            [cutoff],
        )?;
        Ok(())
    }

    fn prune_task_logs(&mut self, task_id: &str) -> BdlResult<()> {
        self.prune_expired_task_logs()?;

        let max_logs = i64::try_from(MAX_TASK_LOGS_PER_TASK).unwrap_or(i64::MAX);
        self.conn.execute(
            r#"
            DELETE FROM task_logs
            WHERE task_id = ?1
              AND id NOT IN (
                SELECT id
                FROM task_logs
                WHERE task_id = ?1
                ORDER BY id DESC
                LIMIT ?2
              )
            "#,
            params![task_id, max_logs],
        )?;
        Ok(())
    }
}

fn prune_preparation_records(conn: &Connection) -> BdlResult<()> {
    conn.execute("DELETE FROM task_preparations WHERE status IN ('completed','failed','cancelled','interrupted') AND (updated_at < ?1 OR id IN (SELECT id FROM task_preparations WHERE status IN ('completed','failed','cancelled','interrupted') ORDER BY updated_at DESC LIMIT -1 OFFSET 100))", [(Utc::now() - Duration::days(30)).to_rfc3339()])?;
    Ok(())
}

fn prune_operation_errors(conn: &Connection) -> BdlResult<()> {
    conn.execute(
        "DELETE FROM operation_errors WHERE datetime(created_at) < datetime(?1) OR id NOT IN (SELECT id FROM operation_errors ORDER BY id DESC LIMIT ?2)",
        params![(Utc::now() - Duration::days(TASK_LOG_RETENTION_DAYS)).to_rfc3339(), MAX_OPERATION_ERRORS as i64],
    )?;
    Ok(())
}

struct TaskRow {
    id: String,
    title: String,
    source_id: String,
    status_json: String,
    output_path: String,
    export_target_json: Option<String>,
    refresh_intent_json: Option<String>,
    media_selection_json: String,
    scheduled_at_json: Option<String>,
    speed_limit_bytes_per_second: Option<i64>,
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
            export_target TEXT,
            refresh_intent TEXT,
            media_selection TEXT NOT NULL DEFAULT '{"video_quality":"unknown","audio_quality":"unknown","video_codec":"unknown","container":"unknown"}',
            scheduled_at TEXT,
            speed_limit_bytes_per_second INTEGER,
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

        CREATE TABLE IF NOT EXISTS operation_errors (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            command TEXT NOT NULL,
            code TEXT NOT NULL,
            message TEXT NOT NULL,
            detail TEXT,
            environment TEXT NOT NULL,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS task_preparations (
            id TEXT PRIMARY KEY,
            status TEXT NOT NULL,
            record TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS account_summary (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            summary TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
    )?;
    add_column_if_missing(
        conn,
        "tasks",
        "media_selection",
        "media_selection TEXT NOT NULL DEFAULT '{\"video_quality\":\"unknown\",\"audio_quality\":\"unknown\",\"video_codec\":\"unknown\",\"container\":\"unknown\"}'",
    )?;
    add_column_if_missing(conn, "tasks", "export_target", "export_target TEXT")?;
    add_column_if_missing(conn, "tasks", "refresh_intent", "refresh_intent TEXT")?;
    add_column_if_missing(conn, "tasks", "scheduled_at", "scheduled_at TEXT")?;
    add_column_if_missing(
        conn,
        "tasks",
        "speed_limit_bytes_per_second",
        "speed_limit_bytes_per_second INTEGER",
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
    save_task_row_in_tx(tx, task, sort_order)?;
    tx.execute("DELETE FROM resources WHERE task_id = ?1", [&task.id])?;
    for (resource_order, resource) in task.resources.iter().enumerate() {
        save_resource_in_tx(tx, &task.id, resource, resource_order, false)?;
    }
    Ok(())
}

fn save_task_row_in_tx(
    tx: &Transaction<'_>,
    task: &DownloadTask,
    sort_order: usize,
) -> BdlResult<()> {
    let refresh_intent_json = task
        .refresh_intent
        .as_ref()
        .map(serialize_json)
        .transpose()?;
    let scheduled_at_json = task.scheduled_at.as_ref().map(serialize_json).transpose()?;
    let export_target_json = task
        .export_target
        .as_ref()
        .map(serialize_json)
        .transpose()?;

    tx.execute(
        r#"
        INSERT INTO tasks (id, title, source_id, status, output_path, export_target, refresh_intent, media_selection, scheduled_at, speed_limit_bytes_per_second, sort_order, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            source_id = excluded.source_id,
            status = excluded.status,
            output_path = excluded.output_path,
            export_target = excluded.export_target,
            refresh_intent = excluded.refresh_intent,
            media_selection = excluded.media_selection,
            scheduled_at = excluded.scheduled_at,
            speed_limit_bytes_per_second = excluded.speed_limit_bytes_per_second,
            sort_order = excluded.sort_order,
            updated_at = CURRENT_TIMESTAMP
        "#,
        params![
            task.id,
            task.title,
            task.source_id,
            serialize_json(&task.status)?,
            path_to_string(&task.output_path),
            export_target_json.as_deref(),
            refresh_intent_json.as_deref(),
            serialize_json(&task.media_selection)?,
            scheduled_at_json.as_deref(),
            task.speed_limit_bytes_per_second.and_then(|value| i64::try_from(value).ok()),
            sort_order as i64,
        ],
    )?;

    Ok(())
}

fn save_resource_in_tx(
    tx: &Transaction<'_>,
    task_id: &str,
    resource: &DownloadResource,
    sort_order: usize,
    update_existing: bool,
) -> BdlResult<()> {
    let query = if update_existing {
        "UPDATE resources SET kind=?3, intent=?4, current_urls=?5, headers=?6, target_path=?7, temp_path=?8, status=?9, sort_order=?10, updated_at=CURRENT_TIMESTAMP WHERE id=?1 AND task_id=?2"
    } else {
        r#"
        INSERT INTO resources (
            id, task_id, kind, intent, current_urls, headers, target_path, temp_path, status, sort_order, updated_at
        )
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, CURRENT_TIMESTAMP)
        "#
    };
    let changed = tx.execute(
        query,
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
    if changed != 1 {
        return Err(BdlError::Planning {
            message: "资源记录已变化，队列修改尚未提交。".into(),
        });
    }
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
