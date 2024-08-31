use chia::protocol::Bytes32;
use serde::{Deserialize, Serialize};
use serde_with::{hex::Hex, serde_as};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DidData {
    pub encoded_id: String,
    #[serde_as(as = "Hex")]
    pub launcher_id: Bytes32,
    pub address: String,
}
