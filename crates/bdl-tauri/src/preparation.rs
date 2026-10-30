use std::collections::HashMap;
use std::sync::Mutex;

use bdl_core::fetcher::FetchCancelToken;
use bdl_core::ids::PartId;
use bdl_core::model::NormalizedSourceTree;
use bdl_core::planner::{DownloadOptions, missing_required_media_parts, plan_selected_parts};
use bdl_core::queue::{
    DownloadExportTarget, DownloadTask, DuplicateTaskPolicy, portable_relative_path,
};
use bdl_core::settings::DocumentTreeDirectory;
use bdl_core::{BdlError, BdlResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::sync::Semaphore;

use crate::commands::{
    CommandError, CommandResult, DuplicateTaskMatch, SelectionCreateTasksRequest,
    SelectionCreateTasksResult,
};
use crate::parse_owned::PreparationContext;
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreparationStatus {
    Queued,
    Preparing,
    AwaitingConfirmation,
    Completed,
    Failed,
    Cancelled,
    Interrupted,
}

impl PreparationStatus {
    pub fn active(self) -> bool {
        matches!(
            self,
            Self::Queued | Self::Preparing | Self::AwaitingConfirmation
        )
    }
    fn name(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Preparing => "preparing",
            Self::AwaitingConfirmation => "awaiting_confirmation",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Interrupted => "interrupted",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreparationJob {
    pub id: String,
    pub title: String,
    pub source_id: String,
    pub input: String,
    pub origin: String,
    pub status: PreparationStatus,
    pub processed: usize,
    pub total: usize,
    pub created: usize,
    pub skipped_existing: usize,
    pub duplicates: Vec<DuplicateTaskMatch>,
    pub error: Option<CommandError>,
    pub created_at: String,
}

pub(crate) struct PreparationInput {
    pub tree: NormalizedSourceTree,
    pub selected: Vec<PartId>,
    pub options: DownloadOptions,
    pub context: PreparationContext,
    pub document_tree: Option<DocumentTreeDirectory>,
    pub work_dir: Option<std::path::PathBuf>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub speed_limit: Option<u64>,
    pub policy: DuplicateTaskPolicy,
}

struct PendingPlan {
    tasks: Vec<DownloadTask>,
    strategy: bdl_core::naming::DuplicateNamingStrategy,
    skipped: usize,
    account_revision: u64,
}

struct Entry {
    job: PreparationJob,
    token: FetchCancelToken,
    plan: Option<PendingPlan>,
}

pub(crate) struct PreparationRegistry {
    entries: Mutex<HashMap<String, Entry>>,
    gate: Semaphore,
}

impl Default for PreparationRegistry {
    fn default() -> Self {
        Self {
            entries: Mutex::new(HashMap::new()),
            gate: Semaphore::new(1),
        }
    }
}

impl PreparationRegistry {
    pub(crate) fn restore(storage: &mut bdl_core::storage::TaskStorage) -> BdlResult<Self> {
        let registry = Self::default();
        for value in storage.preparation_records()? {
            let mut job: PreparationJob = serde_json::from_value(value)?;
            if job.status.active() {
                job.status = PreparationStatus::Interrupted;
                job.duplicates.clear();
                job.error = Some(CommandError {
                    code: "preparation_interrupted".into(),
                    message: "上次任务准备因应用退出而中断，请重新提交。".into(),
                    detail: Some(format!("[stage=preparation_recovery] job_id={}", job.id)),
                });
                storage.save_preparation_record(
                    &job.id,
                    job.status.name(),
                    &serde_json::to_value(&job)?,
                )?;
            }
            registry.entries.lock().unwrap().insert(
                job.id.clone(),
                Entry {
                    job,
                    token: FetchCancelToken::new(),
                    plan: None,
                },
            );
        }
        Ok(registry)
    }
}

fn persist(state: &AppState, job: &PreparationJob) -> BdlResult<()> {
    state
        .storage
        .lock()
        .map_err(|_| BdlError::Planning {
            message: "任务准备记录不可用。".into(),
        })?
        .save_preparation_record(&job.id, job.status.name(), &serde_json::to_value(job)?)
}

pub(crate) fn build_input(
    state: &AppState,
    request: SelectionCreateTasksRequest,
    mut tree: NormalizedSourceTree,
    id: String,
) -> CommandResult<PreparationInput> {
    if tree.source.id.0 != request.source_id {
        return Err(BdlError::Planning {
            message: "提交的来源与所选内容不一致，请重新选择。".into(),
        }
        .into());
    }
    let settings = state.settings()?;
    let document_tree = request
        .document_tree_output
        .clone()
        .or_else(|| settings.document_tree_output.clone());
    let work_dir = document_tree
        .as_ref()
        .map(|_| state.data_dir().join("downloads/work").join(&id));
    let options =
        crate::commands::download_options_from_request(&request, &settings, work_dir.clone())?;
    let requested = request
        .part_ids
        .iter()
        .cloned()
        .map(PartId)
        .collect::<Vec<_>>();
    let selected = crate::parse_session::normalize_selected_part_ids(&tree, &requested);
    // Empty position slots preserve {index}/episode numbering without retaining other videos.
    for item in tree.groups.iter_mut().flat_map(|group| &mut group.items) {
        if !item.parts.iter().any(|part| selected.contains(&part.id)) {
            item.parts.clear();
            item.id.0.clear();
            item.title.clear();
            item.nfo_metadata = Default::default();
            item.owner_name = None;
            item.owner_mid = None;
            item.cover_url = None;
            item.duration_seconds = None;
            item.publish_date = None;
        }
    }
    if selected.is_empty() {
        return Err(BdlError::Planning {
            message: "请选择要下载的内容。".into(),
        }
        .into());
    }
    let scheduled_at =
        crate::commands::parse_future_schedule(request.scheduled_at.as_deref(), Utc::now())?;
    let speed_limit =
        crate::commands::parse_speed_limit(request.speed_limit_bytes_per_second, "单任务下载限速")?;
    Ok(PreparationInput {
        tree,
        selected,
        options,
        context: state.preparation_context(id)?,
        document_tree,
        work_dir,
        scheduled_at,
        speed_limit,
        policy: request.duplicate_policy,
    })
}

pub(crate) async fn plan_input(
    state: &AppState,
    input: PreparationInput,
    progress: impl Fn(usize, usize) -> BdlResult<()>,
) -> CommandResult<(Vec<DownloadTask>, usize, PreparationInput)> {
    let mut prepared = state
        .prepare_selection_owned(
            input.tree.clone(),
            &input.selected,
            &input.options,
            &input.context,
            false,
            &progress,
        )
        .await?;
    let missing = missing_required_media_parts(&prepared.tree, &prepared.part_ids, &input.options);
    if !missing.is_empty() {
        let refreshed = state
            .prepare_selection_owned(
                prepared.tree,
                &missing,
                &input.options,
                &input.context,
                true,
                |_, _| Ok(()),
            )
            .await?;
        prepared.tree = refreshed.tree;
    }
    let mut tasks = plan_selected_parts(&prepared.tree, &prepared.part_ids, &input.options)?;
    let skipped = prepared.part_ids.len().saturating_sub(tasks.len());
    for task in &mut tasks {
        task.scheduled_at = input.scheduled_at;
        task.speed_limit_bytes_per_second = input.speed_limit;
        if let (Some(directory), Some(work_dir)) = (&input.document_tree, &input.work_dir) {
            let relative =
                task.output_path
                    .strip_prefix(work_dir)
                    .map_err(|_| BdlError::Planning {
                        message: "Android 工作路径超出应用私有目录。".into(),
                    })?;
            task.export_target = Some(DownloadExportTarget::DocumentTree {
                tree_uri: directory.tree_uri.clone(),
                relative_path: portable_relative_path(relative)?,
                duplicate_naming_strategy: input.options.duplicate_naming_strategy,
                document_uri: None,
            });
        }
    }
    Ok((tasks, skipped, input))
}

pub(crate) fn commit(
    state: &AppState,
    tasks: Vec<DownloadTask>,
    skipped: usize,
    policy: DuplicateTaskPolicy,
    strategy: bdl_core::naming::DuplicateNamingStrategy,
) -> CommandResult<SelectionCreateTasksResult> {
    let outcome = state.enqueue_tasks_with_duplicate_policy(tasks, policy, strategy)?;
    Ok(SelectionCreateTasksResult {
        created: outcome.inserted,
        skipped_existing: skipped + outcome.skipped_existing,
        duplicates: outcome
            .duplicates
            .into_iter()
            .map(|duplicate| DuplicateTaskMatch {
                proposed_task_id: duplicate.proposed_task_id,
                title: duplicate.title,
                existing_task_id: duplicate.existing_task_id,
                existing_status: duplicate.existing_status,
            })
            .collect(),
        requires_confirmation: outcome.requires_confirmation,
    })
}

fn save_committed_result(state: &AppState, job: &mut PreparationJob) {
    job.error = None;
    if let Err(error) = persist(state, job) {
        let mut error = CommandError::from(error).with_context(&format!(
            "[stage=preparation_record_after_enqueue] job_id={} created={}",
            job.id, job.created
        ));
        error.message = "准备结果已保留，但记录保存失败，请查看错误详情。".into();
        job.error = Some(error);
    }
}

fn save_failure(state: &AppState, job: &mut PreparationJob) {
    if let Err(error) = persist(state, job) {
        let primary = job.error.take().expect("preparation failure has a cause");
        job.error = Some(primary.with_context(&format!(
            "[stage=preparation_failure_record] job_id={} {}",
            job.id,
            CommandError::from(error).diagnostic_message(),
        )));
    }
}

async fn report_record_error(app: &AppHandle, state: &AppState, id: &str) {
    if let Some(error) = state
        .preparation_jobs()
        .into_iter()
        .find(|job| job.id == id)
        .and_then(|job| job.error)
    {
        crate::commands::report_runtime_error(app, state, "preparation_record", error).await;
    }
}

pub(crate) async fn publish(
    app: &AppHandle,
    state: &AppState,
    result: &SelectionCreateTasksResult,
) {
    for task in &result.created {
        if let Err(error) = crate::events::emit(app, crate::events::QUEUE_TASK_UPDATED, task) {
            crate::commands::report_runtime_error(app, state, "publish_prepared_task", error).await;
        }
    }
    if !result.created.is_empty() {
        // Claim download protection before the preparation lease is released.
        // Queue workers also recheck the queue under this same lock on exit.
        if let Err(error) = state.sync_download_activity() {
            crate::commands::report_runtime_error(
                app,
                state,
                "handoff_prepared_tasks",
                error.into(),
            )
            .await;
        }
        crate::queue_worker::start(app);
    }
}

pub(crate) fn register(
    state: &AppState,
    input: &PreparationInput,
    origin: String,
) -> CommandResult<(PreparationJob, FetchCancelToken)> {
    let id = input.context.operation_id.clone();
    let job = PreparationJob {
        id: id.clone(),
        title: input.tree.source.title.clone(),
        source_id: input.tree.source.id.0.clone(),
        input: bdl_core::account::redact_sensitive(&input.tree.source.input),
        origin: bdl_core::account::redact_sensitive(&origin),
        status: PreparationStatus::Queued,
        processed: 0,
        total: input.selected.len(),
        created: 0,
        skipped_existing: 0,
        duplicates: vec![],
        error: None,
        created_at: Utc::now().to_rfc3339(),
    };
    let token = FetchCancelToken::new();
    {
        let mut entries = state.preparations.entries.lock().unwrap();
        if entries
            .values()
            .filter(|entry| entry.job.status.active())
            .count()
            >= 20
        {
            return Err(BdlError::Planning {
                message: "已有20批任务正在准备，请稍后再提交。".into(),
            }
            .into());
        }
        state.task_execution().set_preparation_active(&id, true)?;
        if let Err(error) = persist(state, &job) {
            let mut error = CommandError::from(error);
            if let Err(release) = state.task_execution().set_preparation_active(&id, false) {
                error = error.with_context(&format!("[stage=release_preparation] {release}"));
            }
            return Err(error);
        }
        entries.insert(
            id.clone(),
            Entry {
                job: job.clone(),
                token: token.clone(),
                plan: None,
            },
        );
    }
    Ok((job, token))
}

#[tauri::command]
pub async fn selection_submit_tasks(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    request: SelectionCreateTasksRequest,
    snapshot: NormalizedSourceTree,
    origin: String,
) -> CommandResult<PreparationJob> {
    let id = uuid::Uuid::new_v4().to_string();
    let input = build_input(&state, request, snapshot, id.clone())?;
    let (job, token) = register(&state, &input, origin)?;
    let selected_ids = input
        .selected
        .iter()
        .map(|id| id.0.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let result = run(&state, input, token).await;
        if let Ok(Some(outcome)) = &result {
            publish(&app, &state, outcome).await;
            report_record_error(&app, &state, &id).await;
        }
        if let Err(error) = result {
            let report = {
                let mut entries = state.preparations.entries.lock().unwrap();
                if let Some(entry) = entries.get_mut(&id) {
                    if entry.job.status != PreparationStatus::Cancelled {
                        entry.job.status = PreparationStatus::Failed;
                        entry.job.error = Some(error.with_context(&format!("[stage=task_preparation] job_id={id} origin={} source_id={} input={} requested_part_ids=[{selected_ids}] processed={}/{}", entry.job.origin, entry.job.source_id, entry.job.input, entry.job.processed, entry.job.total)));
                    }
                    entry.plan = None;
                    if entry.job.error.is_some() {
                        save_failure(&state, &mut entry.job);
                    }
                    entry.job.error.clone()
                } else {
                    None
                }
            };
            if let Some(error) = report {
                crate::commands::report_runtime_error(&app, &state, "task_preparation", error)
                    .await;
            }
        }
        if let Err(error) = state.task_execution().set_preparation_active(&id, false) {
            crate::commands::report_runtime_error(
                &app,
                &state,
                "stop_preparation_execution",
                error.into(),
            )
            .await;
        }
    });
    Ok(job)
}

pub(crate) async fn run(
    state: &AppState,
    input: PreparationInput,
    token: FetchCancelToken,
) -> CommandResult<Option<SelectionCreateTasksResult>> {
    let id = input.context.operation_id.clone();
    let _permit = tokio::select! {
        biased;
        _ = token.cancelled() => return Ok(None),
        permit = state.preparations.gate.acquire() => permit.map_err(|_| BdlError::Planning { message: "准备队列已关闭。".into() })?,
    };
    let progress = |processed: usize, total: usize| {
        let mut entries = state.preparations.entries.lock().unwrap();
        if token.is_cancelled() {
            return Err(BdlError::ParseCancelled {
                context: format!(
                    "[stage=task_preparation] job_id={id} cancel_reason=stop_requested"
                ),
            });
        }
        let entry = entries.get_mut(&id).expect("registered preparation");
        entry.job.status = PreparationStatus::Preparing;
        entry.job.processed = processed;
        entry.job.total = total.max(entry.job.total);
        persist(state, &entry.job)
    };
    let (tasks, skipped, input) = tokio::select! {
        biased;
        _ = token.cancelled() => return Ok(None),
        result = plan_input(state, input, progress) => result?,
    };
    let result = {
        let mut entries = state.preparations.entries.lock().unwrap();
        if token.is_cancelled() {
            return Ok(None);
        }
        let entry = entries.get_mut(&id).expect("registered preparation");
        state.check_account_revision(input.context.account_revision)?;
        let result = commit(
            state,
            tasks.clone(),
            skipped,
            input.policy,
            input.options.duplicate_naming_strategy,
        )?;
        entry.job.created = result.created.len();
        entry.job.skipped_existing = result.skipped_existing
            + if matches!(input.policy, DuplicateTaskPolicy::Skip) {
                result.duplicates.len()
            } else {
                0
            };
        entry.job.duplicates = result.duplicates.clone();
        entry.job.processed = entry.job.total;
        entry.job.status = if result.requires_confirmation {
            PreparationStatus::AwaitingConfirmation
        } else {
            PreparationStatus::Completed
        };
        if result.requires_confirmation {
            entry.plan = Some(PendingPlan {
                tasks,
                strategy: input.options.duplicate_naming_strategy,
                skipped,
                account_revision: input.context.account_revision,
            });
        }
        save_committed_result(state, &mut entry.job);
        result
    };
    Ok(Some(result))
}

#[tauri::command]
pub fn preparation_list(state: tauri::State<'_, AppState>) -> Vec<PreparationJob> {
    state.preparation_jobs()
}

#[tauri::command]
pub fn preparation_cancel(state: tauri::State<'_, AppState>, job_id: String) -> CommandResult<()> {
    state.cancel_preparation(&job_id)
}

impl AppState {
    pub(crate) fn preparation_jobs(&self) -> Vec<PreparationJob> {
        let mut entries = self.preparations.entries.lock().unwrap();
        let mut terminal = entries
            .values()
            .filter(|entry| !entry.job.status.active())
            .map(|entry| (entry.job.created_at.clone(), entry.job.id.clone()))
            .collect::<Vec<_>>();
        terminal.sort_by(|left, right| right.0.cmp(&left.0));
        for (_, id) in terminal.into_iter().skip(100) {
            entries.remove(&id);
        }
        let mut jobs = entries
            .values()
            .map(|entry| entry.job.clone())
            .collect::<Vec<_>>();
        jobs.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        jobs
    }
    pub(crate) fn cancel_preparation(&self, id: &str) -> CommandResult<()> {
        let mut entries = self.preparations.entries.lock().unwrap();
        if let Some(entry) = entries
            .get_mut(id)
            .filter(|entry| entry.job.status.active())
        {
            entry.token.cancel();
            self.cancel_parse(id, crate::parse_control::CancelReason::StopRequested);
            entry.plan = None;
            entry.job.status = PreparationStatus::Cancelled;
            persist(self, &entry.job)?;
        }
        Ok(())
    }
    pub(crate) fn cancel_account_preparations(&self) {
        let mut entries = self.preparations.entries.lock().unwrap();
        for entry in entries
            .values_mut()
            .filter(|entry| entry.job.status.active())
        {
            entry.token.cancel();
            self.cancel_parse(
                &entry.job.id,
                crate::parse_control::CancelReason::AccountChanged,
            );
            entry.plan = None;
            entry.job.status = PreparationStatus::Failed;
            entry.job.error = Some(CommandError {
                code: "account_changed".into(),
                message: "账号已变化，任务准备已停止，请重新提交。".into(),
                detail: Some(format!(
                    "job_id={} cancel_reason=account_changed",
                    entry.job.id
                )),
            });
            save_failure(self, &mut entry.job);
        }
    }
}

#[tauri::command]
pub async fn preparation_confirm(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    job_id: String,
    policy: DuplicateTaskPolicy,
) -> CommandResult<()> {
    if matches!(policy, DuplicateTaskPolicy::Ask) {
        return Err(BdlError::Planning {
            message: "请选择重复任务处理方式。".into(),
        }
        .into());
    }
    let result = {
        let mut entries = state.preparations.entries.lock().unwrap();
        let entry = entries.get_mut(&job_id).ok_or_else(|| BdlError::Planning {
            message: "未找到准备任务。".into(),
        })?;
        let plan = entry.plan.as_ref().ok_or_else(|| BdlError::Planning {
            message: "该任务当前无需确认。".into(),
        })?;
        state.check_account_revision(plan.account_revision)?;
        let result = commit(
            &state,
            plan.tasks.clone(),
            plan.skipped,
            policy,
            plan.strategy,
        )?;
        entry.job.status = PreparationStatus::Completed;
        entry.job.created = result.created.len();
        entry.job.skipped_existing = result.skipped_existing
            + if matches!(policy, DuplicateTaskPolicy::Skip) {
                result.duplicates.len()
            } else {
                0
            };
        entry.job.duplicates.clear();
        entry.plan = None;
        save_committed_result(&state, &mut entry.job);
        result
    };
    publish(&app, &state, &result).await;
    report_record_error(&app, &state, &job_id).await;
    Ok(())
}

#[tauri::command]
pub fn preparation_dismiss(state: tauri::State<'_, AppState>, job_id: String) -> CommandResult<()> {
    let mut entries = state.preparations.entries.lock().unwrap();
    if entries
        .get(&job_id)
        .is_some_and(|entry| entry.job.status.active())
    {
        return Err(BdlError::Planning {
            message: "请先停止任务准备。".into(),
        }
        .into());
    }
    state
        .storage
        .lock()
        .unwrap()
        .delete_preparation_record(&job_id)?;
    entries.remove(&job_id);
    Ok(())
}
