use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct WbiData {
    pub wts: u64,
    pub w_rid: String,
}
