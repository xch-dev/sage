use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ImportKey {
    pub name: String,
    pub key: String,
    #[serde(default = "yes")]
    pub save_secrets: bool,
    #[serde(default = "yes")]
    pub login: bool,
}

fn yes() -> bool {
    true
}
