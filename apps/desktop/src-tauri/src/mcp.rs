use bdl_core::public_error::ErrorCode;
use bdl_tauri::commands::{CommandError, CommandResult};
use std::convert::Infallible;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::net::{Ipv4Addr, TcpListener};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bdl_core::account::AccountLibraryFolderKind;
use bdl_core::model::NormalizedSourceTree;
use bdl_core::queue::{DownloadTask, DuplicateTaskPolicy, TaskStatus};
use bdl_core::settings::{ParseRules, validate_speed_limit};
use bdl_core::{BdlError, BdlResult};
use bdl_tauri::commands::{
    SelectionCreateTasksRequest, queue_pause, queue_resume, queue_retry, selection_create_tasks,
};
use bdl_tauri::state::{AppState, SettingsSnapshot};
use http_body_util::{BodyExt, Full, Limited};
use hyper::body::{Bytes, Incoming};
use hyper::header::{AUTHORIZATION, CONTENT_TYPE, HOST, ORIGIN};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode};
use hyper_util::rt::{TokioIo, TokioTimer};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use subtle::ConstantTimeEq;
use tauri::{AppHandle, Manager, State};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use uuid::Uuid;

const PORT: u16 = 48105;
const AUTHORITY: &str = "127.0.0.1:48105";
const ORIGIN_URL: &str = "http://127.0.0.1:48105";
const MCP_URL: &str = "http://127.0.0.1:48105/mcp";
const VERSION: &str = "2025-11-25";
const SUPPORTED_VERSIONS: &[&str] = &[VERSION, "2025-06-18", "2025-03-26"];

#[derive(Default)]
struct ServiceState {
    running: Option<Running>,
    startup_error: Option<CommandError>,
}

pub(crate) struct McpService {
    token_path: PathBuf,
    state: Mutex<ServiceState>,
}

struct Running {
    token: String,
    task: tauri::async_runtime::JoinHandle<()>,
}

impl Drop for Running {
    fn drop(&mut self) {
        self.task.abort();
    }
}

#[derive(Serialize)]
pub(crate) struct McpStatus {
    enabled: bool,
    url: &'static str,
    error: Option<CommandError>,
}

impl McpService {
    pub(crate) fn new(app: &AppHandle, config_dir: &Path) -> Self {
        let service = Self {
            token_path: config_dir.join("mcp-token"),
            state: Mutex::default(),
        };
        let result = service.read_token().and_then(|token| {
            token
                .map(|token| service.start(app, token, false))
                .transpose()
        });
        {
            let mut state = service.state.lock().expect("new MCP state");
            match result {
                Ok(running) => state.running = running,
                Err(error) => state.startup_error = Some(error),
            }
        }
        service
    }

    fn read_token(&self) -> CommandResult<Option<String>> {
        match fs::read_to_string(&self.token_path) {
            Ok(token) if Uuid::try_parse(&token).is_ok() => Ok(Some(token)),
            Ok(_) => Err(CommandError::coded(
                ErrorCode::IntegrationTokenInvalid,
                "invalid token file",
            )),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(CommandError::from(bdl_core::BdlError::Io(e))),
        }
    }

    fn start(&self, app: &AppHandle, token: String, save: bool) -> CommandResult<Running> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, PORT)).map_err(|error| {
            if error.kind() == std::io::ErrorKind::AddrInUse {
                CommandError::coded(ErrorCode::IntegrationPortBusy, error)
            } else {
                CommandError::coded(ErrorCode::IntegrationFailed, error)
            }
        })?;
        listener
            .set_nonblocking(true)
            .map_err(|e| CommandError::from(bdl_core::BdlError::Io(e)))?;
        let runtime = tauri::async_runtime::handle();
        let listener = {
            let _runtime = runtime.inner().enter();
            tokio::net::TcpListener::from_std(listener)
                .map_err(|e| CommandError::from(bdl_core::BdlError::Io(e)))?
        };
        if save {
            self.save_token(&token)
                .map_err(|e| CommandError::from(bdl_core::BdlError::Io(e)))?;
        }
        let app = app.clone();
        let authorization = Arc::<[u8]>::from(format!("Bearer {token}").into_bytes());
        let task = runtime.spawn(serve(listener, app, authorization));
        Ok(Running { token, task })
    }

    fn save_token(&self, token: &str) -> std::io::Result<()> {
        fs::create_dir_all(self.token_path.parent().expect("MCP config directory"))?;
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&self.token_path)?;
        if let Err(error) = file.write_all(token.as_bytes()) {
            drop(file);
            let _ = fs::remove_file(&self.token_path);
            return Err(error);
        }
        Ok(())
    }
}

impl ServiceState {
    fn status(&self) -> McpStatus {
        McpStatus {
            enabled: self.running.is_some(),
            url: MCP_URL,
            error: self.startup_error.clone(),
        }
    }
}

#[tauri::command]
pub(crate) fn desktop_mcp_status(service: State<'_, McpService>) -> CommandResult<McpStatus> {
    Ok(service
        .state
        .lock()
        .map_err(|_| CommandError::coded(ErrorCode::IntegrationFailed, "无法读取 AI 接入状态。"))?
        .status())
}

#[tauri::command]
pub(crate) async fn desktop_mcp_set_enabled(
    app: AppHandle,
    service: State<'_, McpService>,
    enabled: bool,
) -> CommandResult<McpStatus> {
    let (status, stopped) = {
        let mut state = service.state.lock().map_err(|_| {
            CommandError::coded(ErrorCode::IntegrationFailed, "无法更改 AI 接入状态。")
        })?;
        let stopped = if enabled {
            if state.running.is_none() {
                let (token, save) = match service.read_token()? {
                    Some(token) => (token, false),
                    None => (Uuid::new_v4().to_string(), true),
                };
                state.running = Some(service.start(&app, token, save)?);
            }
            None
        } else {
            match fs::remove_file(&service.token_path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(CommandError::from(bdl_core::BdlError::Io(e))),
            }
            state.running.take()
        };
        state.startup_error = None;
        (state.status(), stopped)
    };
    if let Some(mut running) = stopped {
        running.task.abort();
        let _ = (&mut running.task).await;
    }
    Ok(status)
}

#[tauri::command]
pub(crate) fn desktop_mcp_token(service: State<'_, McpService>) -> CommandResult<String> {
    service
        .state
        .lock()
        .map_err(|_| CommandError::coded(ErrorCode::IntegrationFailed, "无法读取接入令牌。"))?
        .running
        .as_ref()
        .map(|running| running.token.clone())
        .ok_or_else(|| CommandError::coded(ErrorCode::IntegrationDisabled, "service disabled"))
}

async fn serve(listener: tokio::net::TcpListener, app: AppHandle, authorization: Arc<[u8]>) {
    let mut connections = JoinSet::new();
    let work = Arc::new(Semaphore::new(1));
    loop {
        tokio::select! {
            accepted = listener.accept(), if connections.len() < 32 => {
                let Ok((stream, _)) = accepted else {
                    tokio::time::sleep(Duration::from_millis(250)).await;
                    continue;
                };
                let app = app.clone();
                let authorization = authorization.clone();
                let work = work.clone();
                connections.spawn(async move {
                    let service = service_fn(|request| handle_http(request, &app, &authorization, &work));
                    let _ = http1::Builder::new()
                        .keep_alive(false)
                        .timer(TokioTimer::new())
                        .header_read_timeout(Duration::from_secs(5))
                        .serve_connection(TokioIo::new(stream), service)
                        .await;
                });
            }
            _ = connections.join_next(), if !connections.is_empty() => {}
        }
    }
}

async fn handle_http(
    request: Request<Incoming>,
    app: &AppHandle,
    authorization: &[u8],
    work: &Semaphore,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let headers = request.headers();
    let result = if request.uri().path() != "/mcp" {
        http_status(StatusCode::NOT_FOUND)
    } else if headers.get(HOST).is_none_or(|host| host != AUTHORITY)
        || headers
            .get(ORIGIN)
            .is_some_and(|origin| origin != ORIGIN_URL)
    {
        http_status(StatusCode::FORBIDDEN)
    } else if headers
        .get(AUTHORIZATION)
        .is_none_or(|header| !bool::from(header.as_bytes().ct_eq(authorization)))
    {
        http_status(StatusCode::UNAUTHORIZED)
    } else if headers.get("mcp-protocol-version").is_some_and(|header| {
        header.to_str().is_err() || !SUPPORTED_VERSIONS.iter().any(|version| header == *version)
    }) {
        http_status(StatusCode::BAD_REQUEST)
    } else if request.method() != Method::POST {
        let mut response = http_status(StatusCode::METHOD_NOT_ALLOWED);
        response.headers_mut().insert(
            hyper::header::ALLOW,
            hyper::header::HeaderValue::from_static("POST"),
        );
        response
    } else if headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_none_or(|value| {
            !value
                .split(';')
                .next()
                .unwrap_or("")
                .trim()
                .eq_ignore_ascii_case("application/json")
        })
    {
        http_status(StatusCode::UNSUPPORTED_MEDIA_TYPE)
    } else {
        match tokio::time::timeout(
            Duration::from_secs(5),
            Limited::new(request.into_body(), 64 * 1024).collect(),
        )
        .await
        {
            Ok(Ok(body)) => match serde_json::from_slice(&body.to_bytes()) {
                Ok(value) => match handle_rpc(value, app, work).await {
                    Some(result) => json_response(result),
                    None => http_status(StatusCode::ACCEPTED),
                },
                Err(_) => json_response(rpc_error(Value::Null, -32700, "Parse error")),
            },
            Ok(Err(_)) => http_status(StatusCode::PAYLOAD_TOO_LARGE),
            Err(_) => http_status(StatusCode::REQUEST_TIMEOUT),
        }
    };
    Ok(result)
}

async fn handle_rpc(message: Value, app: &AppHandle, work: &Semaphore) -> Option<Value> {
    let method = message.get("method").and_then(Value::as_str);
    let id = message.get("id");
    if message["jsonrpc"].as_str() != Some("2.0")
        || method.is_none()
        || id.is_some_and(|id| !id.is_string() && !id.is_i64() && !id.is_u64())
    {
        return Some(rpc_error(Value::Null, -32600, "Invalid request"));
    }
    let id = id?.clone();
    let params = &message["params"];
    let result = match method? {
        "initialize" => {
            let Some(version) = params["protocolVersion"].as_str() else {
                return Some(rpc_error(id, -32602, "Missing protocolVersion"));
            };
            json!({
                "protocolVersion": if SUPPORTED_VERSIONS.contains(&version) { version } else { VERSION },
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "bdl", "version": env!("CARGO_PKG_VERSION") }
            })
        }
        "ping" => json!({}),
        "tools/list" => json!({ "tools": tools() }),
        "tools/call" => {
            let args = &params["arguments"];
            let name = params["name"].as_str();
            let _permit = if matches!(name, Some("bdl_parse" | "bdl_download" | "bdl_library")) {
                Some(
                    work.acquire()
                        .await
                        .expect("MCP work semaphore is never closed"),
                )
            } else {
                None
            };
            let result = match name {
                Some("bdl_parse") => parse(app, args).await,
                Some("bdl_download") => download(app, args).await,
                Some("bdl_library") => library(app, args).await,
                Some("bdl_presets") => presets(app),
                Some("bdl_settings_get") => get_settings(app),
                Some("bdl_settings_update") => update_settings(app, args),
                Some("bdl_queue") => queue(app, args),
                Some("bdl_task_status") => task_status(app, args),
                Some("bdl_task_action") => task_action(app, args),
                _ => return Some(rpc_error(id, -32602, "Unknown tool")),
            };
            match result {
                Ok(result) => json!({
                    "content": [{ "type": "text", "text": result.to_string() }],
                    "structuredContent": result
                }),
                Err(error) => {
                    json!({ "content": [{ "type": "text", "text": bdl_core::account::redact_diagnostic(&error) }], "isError": true })
                }
            }
        }
        _ => return Some(rpc_error(id, -32601, "Method not found")),
    };
    Some(json!({ "jsonrpc": "2.0", "id": id, "result": result }))
}

fn tools() -> Value {
    let input =
        json!({ "type": "string", "minLength": 1, "description": "Bilibili URL or BV/AV ID" });
    let ids =
        json!({ "type": "array", "minItems": 1, "items": { "type": "string", "minLength": 1 } });
    let page = json!({ "type": "integer", "minimum": 1, "maximum": u32::MAX, "default": 1 });
    let page_size = json!({ "type": "integer", "minimum": 1, "maximum": 50, "default": 20 });
    json!([
        {
            "name": "bdl_parse",
            "description": "Read metadata and selectable part IDs from one page of a Bilibili source. Use next_page when returned to browse more. Part IDs from listed pages can be downloaded.",
            "inputSchema": {
                "type": "object", "additionalProperties": false,
                "properties": { "input": input, "page": page }, "required": ["input"]
            },
            "annotations": { "readOnlyHint": true }
        },
        {
            "name": "bdl_download",
            "description": "Queue selected part IDs from one bdl_parse page (pass the same input and page). Use the default download preset or a preset_id returned by bdl_presets. Uses BDL's configured folder.",
            "inputSchema": {
                "type": "object", "additionalProperties": false,
                "properties": { "input": input, "part_ids": ids, "page": page, "preset_id": { "type": "string", "minLength": 1 } }, "required": ["input", "part_ids"]
            },
            "annotations": { "readOnlyHint": false }
        },
        {
            "name": "bdl_task_status",
            "description": "Read the status and available primary output path of task IDs returned by bdl_download or bdl_queue.",
            "inputSchema": {
                "type": "object", "additionalProperties": false,
                "properties": { "task_ids": ids }, "required": ["task_ids"]
            },
            "annotations": { "readOnlyHint": true }
        },
        {
            "name": "bdl_presets",
            "description": "List BDL's available download presets and the default preset ID.",
            "inputSchema": { "type": "object", "additionalProperties": false },
            "annotations": { "readOnlyHint": true }
        },
        {
            "name": "bdl_settings_get",
            "description": "Read BDL's total download limit in decimal MB/s, concurrent tasks, parse speed and default download preset ID.",
            "inputSchema": { "type": "object", "additionalProperties": false },
            "annotations": { "readOnlyHint": true }
        },
        {
            "name": "bdl_settings_update",
            "description": "Persist changes to the specified BDL settings. Confirm the requested changes with the user. Omitted fields retain their values.",
            "inputSchema": {
                "type": "object", "additionalProperties": false, "minProperties": 1,
                "properties": {
                    "download_limit_mb_s": { "type": "number", "minimum": 0, "description": "Total download bandwidth in decimal MB/s. Set zero for unlimited." },
                    "concurrent_tasks": { "type": "integer", "minimum": 1, "maximum": 5 },
                    "parse_speed": { "type": "string", "enum": ["slow", "standard", "fast"] },
                    "default_preset_id": { "type": "string", "minLength": 1, "description": "Use an existing ID from bdl_presets." }
                }
            },
            "annotations": { "readOnlyHint": false }
        },
        {
            "name": "bdl_library",
            "description": "Browse folders created or collected by the account logged in to BDL. Pass a returned source_url to bdl_parse to browse its items.",
            "inputSchema": {
                "type": "object", "additionalProperties": false,
                "properties": { "kind": { "type": "string", "enum": ["created_favorite", "collected_favorite"] }, "page": page, "page_size": page_size },
                "required": ["kind"]
            },
            "annotations": { "readOnlyHint": true }
        },
        {
            "name": "bdl_queue",
            "description": "Browse BDL's download queue, including tasks created in the desktop app. Optionally filter by status.",
            "inputSchema": {
                "type": "object", "additionalProperties": false,
                "properties": { "page": page, "page_size": page_size, "status": { "type": "string", "enum": ["waiting", "parsing", "downloading", "muxing", "completed", "failed", "paused", "cancelled"] } }
            },
            "annotations": { "readOnlyHint": true }
        },
        {
            "name": "bdl_task_action",
            "description": "Pause a queued or downloading task, resume a paused task, or retry a failed task. Use a task ID from bdl_queue or bdl_download.",
            "inputSchema": {
                "type": "object", "additionalProperties": false,
                "properties": { "task_id": { "type": "string", "minLength": 1 }, "action": { "type": "string", "enum": ["pause", "resume", "retry"] } },
                "required": ["task_id", "action"]
            },
            "annotations": { "readOnlyHint": false }
        }
    ])
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ParseArgs {
    input: String,
    page: Option<u32>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DownloadArgs {
    input: String,
    part_ids: Vec<String>,
    page: Option<u32>,
    preset_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StatusArgs {
    task_ids: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LibraryArgs {
    kind: AccountLibraryFolderKind,
    page: Option<u32>,
    page_size: Option<u32>,
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct QueueArgs {
    page: Option<u32>,
    page_size: Option<u32>,
    status: Option<TaskStatus>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum TaskAction {
    Pause,
    Resume,
    Retry,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ActionArgs {
    task_id: String,
    action: TaskAction,
}

struct SourcePage {
    tree: NormalizedSourceTree,
    // An explicit page read merges with the initial tree for the download planner.
    item_ids: Option<std::collections::HashSet<String>>,
    has_more: bool,
}

async fn source_page(state: &AppState, input: &str, page: u32) -> Result<SourcePage, String> {
    if page == 0 {
        return Err("页码必须大于 0。".into());
    }
    let tree = state
        .parse_source(input, false)
        .await
        .map_err(|error| error.to_string())?;
    if page == 1 {
        return Ok(SourcePage {
            has_more: tree.source.has_more,
            tree,
            item_ids: None,
        });
    }
    if !tree.source.has_more {
        return Err("这个来源没有更多分页。".into());
    }
    let (tree, ids, has_more) = state
        .load_page(tree, page, &Uuid::new_v4().to_string())
        .await
        .map_err(|error| error.to_string())?;
    Ok(SourcePage {
        tree,
        item_ids: Some(ids.into_iter().collect()),
        has_more,
    })
}

async fn parse(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let args = ParseArgs::deserialize(args).map_err(|error| error.to_string())?;
    let page = args.page.unwrap_or(1);
    let state = app.state::<AppState>();
    let SourcePage {
        tree,
        item_ids,
        has_more,
    } = source_page(&state, &args.input, page).await?;
    let items: Vec<_> = tree.groups.into_iter().flat_map(|group| group.items)
        .filter(|item| item_ids.as_ref().is_none_or(|ids| ids.contains(&item.id.0)))
        .enumerate().map(|(index, item)| {
        let parts: Vec<_> = item.parts.into_iter().enumerate().map(|(index, part)| json!({
            "number": index + 1, "id": part.id, "title": part.title, "duration_seconds": part.duration_seconds
        })).collect();
        json!({ "number": index + 1, "title": item.title, "parts": parts })
    }).collect();
    Ok(json!({
        "kind": tree.source.kind, "title": tree.source.title, "loaded_count": items.len(),
        "page": page, "total_count": tree.source.total_count, "has_more": has_more,
        "next_page": page.checked_add(1).filter(|_| has_more), "items": items
    }))
}

async fn download(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let args = DownloadArgs::deserialize(args).map_err(|error| error.to_string())?;
    if args.part_ids.is_empty() || args.part_ids.iter().any(String::is_empty) {
        return Err("请指定至少一个有效分 P。".into());
    }
    let state = app.state::<AppState>();
    let settings = state.settings().map_err(|error| error.to_string())?;
    let preset_id = args.preset_id.unwrap_or(settings.selected_download_preset);
    if !settings
        .download_presets
        .iter()
        .any(|preset| preset.id == preset_id)
    {
        return Err("下载预设不存在，请重新读取预设列表。".into());
    }
    let SourcePage { tree, item_ids, .. } =
        source_page(&state, &args.input, args.page.unwrap_or(1)).await?;
    let available: std::collections::HashSet<_> = tree
        .groups
        .iter()
        .flat_map(|group| &group.items)
        .filter(|item| item_ids.as_ref().is_none_or(|ids| ids.contains(&item.id.0)))
        .flat_map(|item| &item.parts)
        .map(|part| part.id.0.as_str())
        .collect();
    if args
        .part_ids
        .iter()
        .any(|id| !available.contains(id.as_str()))
    {
        return Err("所选分 P 已变化。请重新解析并选择。".into());
    }
    let result = selection_create_tasks(
        app.clone(),
        state,
        SelectionCreateTasksRequest {
            download_preset_id: Some(preset_id),
            source_id: tree.source.id.0.clone(),
            part_ids: args.part_ids,
            duplicate_policy: DuplicateTaskPolicy::Skip,
            ..Default::default()
        },
        tree,
        Uuid::new_v4().to_string(),
    )
    .await
    .map_err(|error| error.message)?;
    Ok(json!({
        "created": result.created.into_iter().map(task_summary).collect::<Vec<_>>(),
        "existing": result.duplicates.into_iter().map(|task| json!({ "id": task.existing_task_id, "title": task.title, "status": task.existing_status })).collect::<Vec<_>>(),
        "skipped_existing": result.skipped_existing
    }))
}

fn presets(app: &AppHandle) -> Result<Value, String> {
    let settings = app
        .state::<AppState>()
        .settings()
        .map_err(|error| error.to_string())?;
    let presets: Vec<_> = settings
        .download_presets
        .into_iter()
        .map(|preset| {
            json!({
                "id": preset.id, "name": preset.name, "workflow": preset.workflow
            })
        })
        .collect();
    Ok(json!({ "default_preset_id": settings.selected_download_preset, "presets": presets }))
}

#[derive(Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum ParseSpeed {
    Slow,
    Standard,
    Fast,
}

impl ParseSpeed {
    fn rules(self) -> ParseRules {
        match self {
            Self::Slow => ParseRules {
                pages_per_round: 1,
                interval_seconds: 3,
                rest_seconds: 8,
            },
            Self::Standard => ParseRules::default(),
            Self::Fast => ParseRules {
                pages_per_round: 4,
                ..ParseRules::default()
            },
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsPatch {
    download_limit_mb_s: Option<f64>,
    concurrent_tasks: Option<usize>,
    parse_speed: Option<ParseSpeed>,
    default_preset_id: Option<String>,
}

const BYTES_PER_MB: f64 = 1_000_000.0;

fn settings_summary(settings: &SettingsSnapshot) -> Value {
    let speed = [ParseSpeed::Slow, ParseSpeed::Standard, ParseSpeed::Fast]
        .into_iter()
        .find(|speed| speed.rules() == settings.parse_rules);
    json!({
        "download_limit_mb_s": settings.global_speed_limit_bytes_per_second.unwrap_or_default() as f64 / BYTES_PER_MB,
        "concurrent_tasks": settings.concurrent_tasks,
        "parse_speed": speed.map_or(json!("custom"), |speed| json!(speed)),
        "parse_rules": settings.parse_rules,
        "default_preset_id": settings.selected_download_preset
    })
}

fn get_settings(app: &AppHandle) -> Result<Value, String> {
    app.state::<AppState>()
        .settings()
        .map(|settings| settings_summary(&settings))
        .map_err(|error| error.to_string())
}

impl SettingsPatch {
    fn apply(self, settings: &mut SettingsSnapshot) -> BdlResult<()> {
        if let Some(mb_s) = self.download_limit_mb_s {
            if !mb_s.is_finite() || mb_s < 0.0 {
                return Err(invalid_setting("下载总限速必须为 0 或正数。"));
            }
            let limit = (mb_s != 0.0).then_some((mb_s * BYTES_PER_MB).round() as u64);
            validate_speed_limit(limit, bdl_core::settings::SpeedLimitScope::Global)?;
            settings.global_speed_limit_bytes_per_second = limit;
        }
        if let Some(count) = self.concurrent_tasks {
            if !(1..=5).contains(&count) {
                return Err(invalid_setting("并发任务数需为 1～5。"));
            }
            settings.concurrent_tasks = count;
        }
        if let Some(speed) = self.parse_speed {
            settings.parse_rules = speed.rules();
        }
        if let Some(id) = self.default_preset_id {
            if !settings
                .download_presets
                .iter()
                .any(|preset| preset.id == id)
            {
                return Err(invalid_setting("下载预设不存在，请重新读取预设列表。"));
            }
            settings.selected_download_preset = id;
        }
        Ok(())
    }
}

fn invalid_setting(message: &str) -> BdlError {
    BdlError::Planning {
        message: message.into(),
    }
}

fn update_settings(app: &AppHandle, args: &Value) -> Result<Value, String> {
    if args
        .as_object()
        .is_some_and(|fields| fields.values().any(Value::is_null))
    {
        return Err("需为要修改的设置提供有效值。".into());
    }
    let patch = SettingsPatch::deserialize(args).map_err(|error| error.to_string())?;
    if patch.download_limit_mb_s.is_none()
        && patch.concurrent_tasks.is_none()
        && patch.parse_speed.is_none()
        && patch.default_preset_id.is_none()
    {
        return Err("请指定至少一项要修改的设置。".into());
    }
    let settings = app
        .state::<AppState>()
        .edit_settings(|settings| patch.apply(settings))
        .map_err(|error| error.to_string())?;
    bdl_tauri::events::emit(app, bdl_tauri::events::SETTINGS_EXTERNAL_UPDATED, &())
        .map_err(|error| error.message)?;
    Ok(settings_summary(&settings))
}

async fn library(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let args = LibraryArgs::deserialize(args).map_err(|error| error.to_string())?;
    let page = app
        .state::<AppState>()
        .account_library(
            args.kind,
            args.page.unwrap_or(1),
            args.page_size.unwrap_or(20),
        )
        .await
        .map_err(|error| error.to_string())?;
    let items: Vec<_> = page
        .items
        .into_iter()
        .map(|item| {
            json!({
                "title": item.title, "source_url": item.source_url, "media_count": item.media_count
            })
        })
        .collect();
    Ok(
        json!({ "page": page.page, "page_size": page.page_size, "total": page.total, "has_more": page.has_more, "items": items }),
    )
}

fn queue(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let args = if args.is_null() {
        QueueArgs::default()
    } else {
        QueueArgs::deserialize(args).map_err(|error| error.to_string())?
    };
    let page = args.page.unwrap_or(1);
    let page_size = args.page_size.unwrap_or(20);
    if page == 0 || !(1..=50).contains(&page_size) {
        return Err("队列分页参数无效。".into());
    }
    let offset = ((page - 1) as usize).saturating_mul(page_size as usize);
    let tasks: Vec<_> = app
        .state::<AppState>()
        .queue_snapshot()
        .map_err(|error| error.to_string())?
        .into_iter()
        .filter(|task| args.status.is_none_or(|status| task.status == status))
        .collect();
    let total = tasks.len();
    let tasks: Vec<_> = tasks
        .into_iter()
        .skip(offset)
        .take(page_size as usize)
        .map(task_summary)
        .collect();
    Ok(
        json!({ "page": page, "page_size": page_size, "total": total, "has_more": total > offset.saturating_add(tasks.len()), "tasks": tasks }),
    )
}

fn task_status(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let args = StatusArgs::deserialize(args).map_err(|error| error.to_string())?;
    if args.task_ids.is_empty() || args.task_ids.iter().any(String::is_empty) {
        return Err("请指定至少一个有效任务。".into());
    }
    let ids: std::collections::HashSet<_> = args.task_ids.iter().collect();
    let tasks = app
        .state::<AppState>()
        .queue_snapshot()
        .map_err(|error| error.to_string())?;
    Ok(
        json!({ "tasks": tasks.into_iter().filter(|task| ids.contains(&task.id)).map(task_summary).collect::<Vec<_>>() }),
    )
}

fn task_action(app: &AppHandle, args: &Value) -> Result<Value, String> {
    let args = ActionArgs::deserialize(args).map_err(|error| error.to_string())?;
    let state = app.state::<AppState>();
    let status = state
        .task_status(&args.task_id)
        .map_err(|error| error.to_string())?;
    let result = match (args.action, status) {
        (
            TaskAction::Pause,
            TaskStatus::Waiting
            | TaskStatus::Parsing
            | TaskStatus::Downloading
            | TaskStatus::MuxQueued,
        ) => queue_pause(app.clone(), state, args.task_id),
        (TaskAction::Resume, TaskStatus::Paused) => queue_resume(app.clone(), state, args.task_id),
        (TaskAction::Retry, TaskStatus::Failed) => queue_retry(app.clone(), state, args.task_id),
        _ => return Err("任务当前状态不支持这项操作。".into()),
    };
    result.map(task_summary).map_err(|error| error.message)
}

fn task_summary(task: DownloadTask) -> Value {
    let output_path = (task.status == TaskStatus::Completed && task.output_path.is_file())
        .then_some(task.output_path);
    json!({ "id": task.id, "title": task.title, "status": task.status, "output_path": output_path })
}

fn http_status(status: StatusCode) -> Response<Full<Bytes>> {
    let mut response = Response::new(Full::default());
    *response.status_mut() = status;
    response
}

fn json_response(result: Value) -> Response<Full<Bytes>> {
    let mut response = Response::new(Full::new(Bytes::from(result.to_string())));
    response.headers_mut().insert(
        CONTENT_TYPE,
        hyper::header::HeaderValue::from_static("application/json"),
    );
    response
}

fn rpc_error(id: Value, code: i32, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}
