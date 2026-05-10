use crate::bridge::methods::user::environment::EnvironmentThemeView;
use crate::bridge::state::BridgeState;
use crate::runtime::AppRuntimeState;
use crate::sandbox::SandboxStateStore;
use parking_lot::RwLock;
use sage::Sage;
use sage_api::ErrorKind;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::sync::Arc;
use tokio::sync::Mutex;

pub type AppState = Arc<Mutex<Sage>>;

#[derive(Debug, Default)]
pub struct AppsHostState {
    pub app_operation_locks: RwLock<HashMap<String, Arc<Mutex<()>>>>,
    pub app_update_locks: RwLock<HashSet<String>>,
    pub runtime: AppRuntimeState,
    pub bridge: BridgeState,
    pub sandbox: SandboxStateStore,
    pub environment: AppsEnvironmentState,
}

impl AppsHostState {
    pub fn operation_lock_for_app(&self, app_id: &str) -> Arc<Mutex<()>> {
        if let Some(lock) = self.app_operation_locks.read().get(app_id) {
            return lock.clone();
        }

        self.app_operation_locks
            .write()
            .entry(app_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }

    pub fn try_begin_app_update(&self, app_id: &str) -> anyhow::Result<AppUpdateLockGuard<'_>> {
        let mut locks = self.app_update_locks.write();
        if !locks.insert(app_id.to_string()) {
            anyhow::bail!("app update already in progress for {app_id}");
        }
        Ok(AppUpdateLockGuard {
            state: self,
            app_id: app_id.to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SageAppsError {
    pub kind: ErrorKind,
    pub reason: String,
}

#[derive(Debug, Default)]
pub struct AppsEnvironmentState {
    pub theme: AppsEnvironmentThemeState,
}

#[derive(Debug, Default)]
pub struct AppsEnvironmentThemeState {
    pub current: Mutex<Option<EnvironmentThemeView>>,
}

#[derive(Debug)]
pub struct AppUpdateLockGuard<'a> {
    state: &'a AppsHostState,
    app_id: String,
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

impl From<anyhow::Error> for SageAppsError {
    fn from(error: anyhow::Error) -> Self {
        Self {
            kind: ErrorKind::Internal,
            reason: error.to_string(),
        }
    }
}

impl Drop for AppUpdateLockGuard<'_> {
    fn drop(&mut self) {
        self.state.app_update_locks.write().remove(&self.app_id);
    }
}

impl fmt::Display for SageAppsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl std::error::Error for SageAppsError {}

pub type Result<T> = std::result::Result<T, SageAppsError>;
