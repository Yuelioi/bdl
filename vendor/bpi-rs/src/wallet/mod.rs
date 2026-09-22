//! 钱包模块
//!
//! [查看 API 文档](https://socialsisteryi.github.io/bilibili-API-collect/docs/wallet/info.html)
mod client;
pub mod info;
pub mod params;

pub use client::WalletClient;
pub use info::UserWallet;
pub use params::WalletInfoParams;
