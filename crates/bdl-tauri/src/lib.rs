pub mod commands;
pub mod events;
mod parse_session;
pub mod secure_store;
pub mod state;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
