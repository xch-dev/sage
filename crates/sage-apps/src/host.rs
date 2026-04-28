use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use crate::bridge::state::BridgeState;
use crate::runtime::AppRuntimeState;
use crate::sandbox::SandboxStateStore;
use crate::types::SystemSageApp;
use sage::Sage;
use sage_api::ErrorKind;
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::Mutex;

pub type AppState = Arc<Mutex<Sage>>;

#[derive(Debug, Default)]
pub struct AppsHostState {
    pub system_apps: BTreeMap<String, SystemSageApp>,
    pub runtime: AppRuntimeState,
    pub bridge: BridgeState,
    pub sandbox: SandboxStateStore,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SageAppsError {
    pub kind: ErrorKind,
    pub reason: String,
}

impl From<sage::Error> for SageAppsError {
    fn from(error: sage::Error) -> Self {
        Self {
            kind: error.kind(),
            reason: error.to_string(),
        }
    }
}

impl From<reqwest::Error> for SageAppsError {
    fn from(error: reqwest::Error) -> Self {
        Self {
            kind: ErrorKind::Internal,
            reason: error.to_string(),
        }
    }
}

impl From<std::io::Error> for SageAppsError {
    fn from(error: std::io::Error) -> Self {
        Self {
            kind: ErrorKind::Internal,
            reason: error.to_string(),
        }
    }
}

impl fmt::Display for SageAppsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl std::error::Error for SageAppsError {}

pub type Result<T> = std::result::Result<T, SageAppsError>;
