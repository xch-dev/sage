use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            log_level: "INFO".to_string(),
        }
    }
}
