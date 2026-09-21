//! Deterministic queue transition rules shared by startup recovery and commands.

use std::collections::HashSet;

use bdl_core::BdlResult;
use bdl_core::naming::{DuplicateNamingStrategy, resolve_duplicate_path, unique_path};
use bdl_core::queue::{DownloadResource, DownloadTask, ResourceStatus, TaskStatus};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateTaskMatch {
    pub proposed_task_id: String,
    pub title: String,
    pub existing_task_id: String,
    pub existing_status: TaskStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateTaskEnqueueResult {
    pub inserted: Vec<DownloadTask>,
    pub duplicates: Vec<DuplicateTaskMatch>,
    pub requires_confirmation: bool,
    pub skipped_existing: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct StartupRecoverySnapshot {
    pub task_ids: Vec<String>,
    pub auto_recovery_enabled: bool,
}

pub(crate) fn ignores_status_transition(current: TaskStatus, next: TaskStatus) -> bool {
    matches!(
        (current, next),
        (
            TaskStatus::Muxing | TaskStatus::Completed,
            TaskStatus::Paused | TaskStatus::Cancelled
        ) | (
            TaskStatus::Paused | TaskStatus::Cancelled,
            TaskStatus::Muxing
        )
    )
}

pub(crate) fn dedupe_tasks_by_id(queue: &mut Vec<DownloadTask>) -> bool {
    let original_len = queue.len();
    let mut seen = HashSet::new();
    queue.retain(|task| seen.insert(task.id.clone()));
    queue.len() != original_len
}

pub(crate) fn duplicate_task_matches(
    planned: &[DownloadTask],
    existing: &[DownloadTask],
) -> Vec<DuplicateTaskMatch> {
    planned
        .iter()
        .filter_map(|candidate| {
            existing
                .iter()
                .find(|task| task.logical_id() == candidate.id)
                .map(|task| DuplicateTaskMatch {
                    proposed_task_id: candidate.id.clone(),
                    title: candidate.title.clone(),
                    existing_task_id: task.id.clone(),
                    existing_status: task.status,
                })
        })
        .collect()
}

pub(crate) fn prepare_duplicate_copies(
    tasks: Vec<DownloadTask>,
    existing: &[DownloadTask],
) -> BdlResult<Vec<DownloadTask>> {
    let mut used_ids = existing
        .iter()
        .map(|task| task.id.clone())
        .chain(tasks.iter().map(|task| task.id.clone()))
        .collect::<HashSet<_>>();
    let mut reserved_paths = existing
        .iter()
        .map(|task| task.output_path.clone())
        .collect::<HashSet<_>>();
    let mut prepared = Vec::with_capacity(tasks.len());
    for task in tasks {
        if !existing
            .iter()
            .any(|candidate| candidate.logical_id() == task.id)
        {
            reserved_paths.insert(task.output_path.clone());
            prepared.push(task);
            continue;
        }
        let mut copy_index = 1;
        while !used_ids.insert(format!("{}:copy:{copy_index}", task.logical_id())) {
            copy_index += 1;
        }
        let output_path = unique_path(task.output_path.clone(), &mut reserved_paths);
        prepared.push(task.into_duplicate_copy(copy_index, output_path)?);
    }
    Ok(prepared)
}

/// A separate planning call must not reuse an unfinished task's files. As with
/// collisions within one plan, suffix live reservations even in overwrite mode;
/// completed outputs remain subject to the planner's selected naming policy.
pub(crate) fn reserve_queued_paths(
    tasks: Vec<DownloadTask>,
    existing: &[DownloadTask],
    strategy: DuplicateNamingStrategy,
) -> BdlResult<Vec<DownloadTask>> {
    let mut reserved = existing
        .iter()
        .filter(|task| task.status != TaskStatus::Completed)
        .map(|task| task.output_path.clone())
        .collect::<HashSet<_>>();
    let mut prepared = Vec::with_capacity(tasks.len());
    for task in tasks {
        let path = if reserved.contains(&task.output_path) {
            Some(unique_path(task.output_path.clone(), &mut reserved))
        } else {
            // Recheck disk under the enqueue lock: an earlier download may have
            // completed since the planner inspected its destination.
            resolve_duplicate_path(task.output_path.clone(), &mut reserved, strategy)
        };
        if let Some(path) = path {
            prepared.push(if path == task.output_path {
                task
            } else {
                task.with_output_path(path)?
            });
        }
    }
    Ok(prepared)
}

pub(crate) fn append_new_tasks(
    queue: &mut Vec<DownloadTask>,
    tasks: Vec<DownloadTask>,
) -> Vec<DownloadTask> {
    let mut known_ids = queue
        .iter()
        .map(|task| task.id.clone())
        .collect::<HashSet<_>>();
    let mut inserted = Vec::new();
    for task in tasks {
        if known_ids.insert(task.id.clone()) {
            inserted.push(task.clone());
            queue.push(task);
        }
    }
    inserted
}

pub(crate) fn prepare_startup_recovery(
    queue: &mut [DownloadTask],
    auto_recovery_enabled: bool,
) -> StartupRecoverySnapshot {
    let mut task_ids = Vec::new();
    for task in queue {
        if task.status == TaskStatus::Waiting && task.scheduled_at.is_some() {
            continue;
        }
        if !matches!(
            task.status,
            TaskStatus::Waiting
                | TaskStatus::Parsing
                | TaskStatus::Downloading
                | TaskStatus::Muxing
        ) {
            continue;
        }
        task_ids.push(task.id.clone());
        task.status = if auto_recovery_enabled {
            TaskStatus::Waiting
        } else {
            TaskStatus::Paused
        };
        reset_interrupted_resources(&mut task.resources);
    }
    StartupRecoverySnapshot {
        task_ids,
        auto_recovery_enabled,
    }
}

pub(crate) fn reset_interrupted_resources(resources: &mut [DownloadResource]) {
    for resource in resources {
        if matches!(
            resource.status,
            ResourceStatus::Downloading | ResourceStatus::Paused
        ) {
            resource.status = ResourceStatus::Pending;
        }
    }
}

pub(crate) fn reset_task_for_retry(task: &mut DownloadTask) {
    let redownload_all = task.status == TaskStatus::Completed;
    task.status = TaskStatus::Waiting;
    task.scheduled_at = None;
    for resource in &mut task.resources {
        if redownload_all || resource.status != ResourceStatus::Completed {
            resource.status = ResourceStatus::Pending;
        }
    }
}
