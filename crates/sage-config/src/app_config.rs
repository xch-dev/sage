use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
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
