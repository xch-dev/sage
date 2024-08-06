use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationInfo {
    pub derivation_index: u32,
    pub receive_address: String,
}
