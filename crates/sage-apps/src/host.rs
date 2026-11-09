use std::{
    collections::{HashMap, HashSet},
    fmt,
    path::PathBuf,
    sync::Arc,
};

use parking_lot::RwLock;
use sage::Sage;
use sage_api::ErrorKind;
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::Mutex;

use crate::{AppRuntimeState, AppsDb, BridgeState, EnvironmentThemeView, SandboxStateStore};

pub type AppState = Arc<Mutex<Sage>>;

#[derive(Debug)]
pub struct AppsHostState {
    /// The Sage root, which holds the apps database and installed app files.
    pub(crate) root: PathBuf,
    /// The webview data directory of the main window. On Windows, app profiles are created inside
    /// it, and on Linux, app webviews share it. It's kept out of the Sage root by default, because
    /// on Windows that is in roaming app data, where webview data shouldn't be stored.
    pub webview_root: PathBuf,
    /// Distinguishes the deterministic data stores of builtin apps on macOS between Sage roots.
    /// It's `None` for the default root.
    pub(crate) builtin_storage_namespace: Option<String>,
    /// Whether app webviews must keep their storage in memory, because their persistent storage
    /// couldn't be isolated from other Sage roots.
    pub in_memory_webview_storage: bool,
    pub app_operation_locks: RwLock<HashMap<String, Arc<Mutex<()>>>>,
    pub app_update_locks: RwLock<HashSet<String>>,
    pub runtime: AppRuntimeState,
    pub bridge: BridgeState,
    pub sandbox: SandboxStateStore,
    pub environment: AppsEnvironmentState,
    pub db: AppsDb,
}

impl AppsHostState {
    pub fn new(root: PathBuf, webview_root: PathBuf, db: AppsDb) -> Self {
        Self {
            root,
            webview_root,
            builtin_storage_namespace: None,
            in_memory_webview_storage: false,
            app_operation_locks: RwLock::default(),
            app_update_locks: RwLock::default(),
            runtime: AppRuntimeState::default(),
            bridge: BridgeState::default(),
            sandbox: SandboxStateStore::default(),
            environment: AppsEnvironmentState::default(),
            db,
        }
    }

    #[must_use]
    pub fn with_builtin_storage_namespace(mut self, namespace: Option<String>) -> Self {
        self.builtin_storage_namespace = namespace;
        self
    }

    #[must_use]
    pub fn with_in_memory_webview_storage(mut self, in_memory: bool) -> Self {
        self.in_memory_webview_storage = in_memory;
        self
    }

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
