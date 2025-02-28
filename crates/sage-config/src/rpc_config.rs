use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(default)]
pub struct RpcConfig {
    pub run_on_startup: bool,
    pub server_port: u16,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            run_on_startup: cfg!(all(not(target_os = "android"), not(target_os = "ios"))),
            server_port: 9257,
        }
    }
}
