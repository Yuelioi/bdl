pub mod commands;
mod diagnostic_export;
pub mod events;
mod media_finalize;
mod parse_session;
mod queue_coordinator;
pub mod secure_store;
pub mod state;
mod task_failure;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
