pub mod commands;
pub mod events;
pub mod state;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
