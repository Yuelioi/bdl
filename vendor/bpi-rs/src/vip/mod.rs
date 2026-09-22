//! VIP

pub mod action;
pub mod center;
mod client;
pub mod params;

pub mod info;

pub use client::VipClient;
pub use params::{VipCenterInfoParams, VipPrivilegeReceiveParams};

// 用不了
// pub mod clockin;
