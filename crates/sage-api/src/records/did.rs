use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct DidRecord {
    pub launcher_id: String,
    pub coin_id: String,
    pub address: String,
    pub name: Option<String>,
    pub visible: bool,
}
