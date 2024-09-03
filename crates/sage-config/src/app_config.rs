use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AppConfig {
    #[serde(default = "default_log_level")]
    pub log_level: String,

    #[serde(default)]
    pub active_fingerprint: Option<u32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            active_fingerprint: None,
        }
    }
}

fn default_log_level() -> String {
    "INFO".to_string()
}
