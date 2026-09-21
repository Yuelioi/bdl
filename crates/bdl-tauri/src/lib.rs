pub mod commands;
mod diagnostic_export;
pub mod events;
mod media_finalize;
pub mod media_mux;
pub mod mobile_storage;
mod parse_control;
mod parse_pacing;
mod parse_session;
mod queue_coordinator;
mod queue_worker;
pub mod secure_store;
pub mod state;
pub mod task_execution;
mod task_failure;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
