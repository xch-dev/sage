use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct AppConfig {
    pub log_level: String,
    pub active_fingerprint: Option<u32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            log_level: "INFO".to_string(),
            active_fingerprint: None,
        }
    }
}
