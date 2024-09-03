use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DidData {
    pub encoded_id: String,
    pub launcher_id: String,
    pub address: String,
}
