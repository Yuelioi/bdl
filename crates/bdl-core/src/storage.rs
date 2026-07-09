use std::path::{Path, PathBuf};

use rusqlite::{Connection, Transaction, params};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::BdlResult;
use crate::queue::{DownloadResource, DownloadTask};

pub struct TaskStorage {
    conn: Connection,
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
        tx.execute("DELETE FROM tasks", [])?;
        for (index, task) in tasks.iter().enumerate() {
            save_task_in_tx(&tx, task, index)?;
        }
        tx.commit()?;
        Ok(())
    }

    pub fn load_tasks(&self) -> BdlResult<Vec<DownloadTask>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, source_id, status, output_path FROM tasks ORDER BY sort_order, rowid",
        )?;
        let task_rows = stmt.query_map([], |row| {
            Ok(TaskRow {
                id: row.get(0)?,
                title: row.get(1)?,
                source_id: row.get(2)?,
                status_json: row.get(3)?,
                output_path: row.get(4)?,
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
            });
        }

        Ok(tasks)
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

fn run_migrations(conn: &Connection) -> BdlResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            source_id TEXT NOT NULL,
            status TEXT NOT NULL,
            output_path TEXT NOT NULL,
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
            output_path TEXT NOT NULL,
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
    Ok(())
}

fn save_task_in_tx(tx: &Transaction<'_>, task: &DownloadTask, sort_order: usize) -> BdlResult<()> {
    tx.execute(
        r#"
        INSERT INTO tasks (id, title, source_id, status, output_path, sort_order, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, CURRENT_TIMESTAMP)
        ON CONFLICT(id) DO UPDATE SET
            title = excluded.title,
            source_id = excluded.source_id,
            status = excluded.status,
            output_path = excluded.output_path,
            sort_order = excluded.sort_order,
            updated_at = CURRENT_TIMESTAMP
        "#,
        params![
            task.id,
            task.title,
            task.source_id,
            serialize_json(&task.status)?,
            path_to_string(&task.output_path),
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
