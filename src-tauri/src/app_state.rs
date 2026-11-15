use std::{path::PathBuf, sync::Arc};

use sage::{RefreshDebouncer, Result, Sage, api_sync_event};
use sage_api::ErrorKind;
#[cfg(not(mobile))]
use sage_apps::{AppsHostState, emit_selected_wallet_changed, process_sage_network_change};
use sage_wallet::SyncEvent;
#[cfg(not(mobile))]
use tauri::Manager;
use tauri::{AppHandle, Emitter};
use tokio::{sync::Mutex, task::JoinHandle};

use crate::error;

/// The outcome of initialization, once it has been attempted.
pub struct Initialized(pub Mutex<Option<error::Result<()>>>);

/// Set instead of the apps host state when the apps database couldn't be opened.
#[cfg_attr(mobile, allow(dead_code))] // Mobile builds have no apps database
pub struct AppsDbFailure {
    pub root: PathBuf,
    pub reason: String,
    /// The database was migrated by a newer version of Sage, which can still open it.
    pub newer_version: bool,
}

impl AppsDbFailure {
    /// Whether deleting the database is offered. It isn't when a newer version of Sage can still
    /// open it, since installing that version again recovers the apps and agents.
    pub fn resettable(&self) -> bool {
        !self.newer_version
    }

    pub fn to_error(&self) -> error::Error {
        if self.newer_version {
            error::Error {
                kind: ErrorKind::Internal,
                reason: format!(
                    "The apps database was created by a newer version of Sage. Install that \
                     version again to keep your apps and agents.\n\n{}",
                    self.reason
                ),
                handle_error_code: None,
                recipients: None,
            }
        } else {
            error::Error {
                kind: ErrorKind::AppsDatabase,
                reason: self.reason.clone(),
                handle_error_code: None,
                recipients: None,
            }
        }
    }
}

pub struct RpcTask(pub Mutex<Option<JoinHandle<anyhow::Result<()>>>>);

pub type AppState = Arc<Mutex<Sage>>;

pub async fn initialize(app_handle: AppHandle, sage: &mut Sage) -> Result<()> {
    let mut receiver = sage.initialize().await?;

    tokio::spawn(async move {
        let mut debouncer = RefreshDebouncer::default();

        loop {
            tokio::select! {
                received = receiver.recv() => {
                    let Some(event) = received else {
                        break;
                    };

                    #[cfg(not(mobile))]
                    if let SyncEvent::NetworkChanged { .. } = &event {
                        let apps_state = app_handle.state::<AppsHostState>();

                        process_sage_network_change(&app_handle, &apps_state).await;
                    }
                    #[cfg(not(mobile))]
                    if let SyncEvent::WalletChanged {
                        fingerprint: Some(fingerprint),
                    } = &event
                    {
                        let apps_state = app_handle.state::<AppsHostState>();

                        emit_selected_wallet_changed(&app_handle, &apps_state, *fingerprint).await;
                    }
                    let Some(event) = api_sync_event(event) else {
                        continue;
                    };

                    if let Some(event) = debouncer.push(event)
                        && app_handle.emit("sync-event", event).is_err()
                    {
                        break;
                    }
                }
                events = debouncer.flushed() => {
                    for event in events {
                        if app_handle.emit("sync-event", event).is_err() {
                            return Result::Ok(());
                        }
                    }
                }
            }
        }

        Result::Ok(())
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failure(newer_version: bool) -> AppsDbFailure {
        AppsDbFailure {
            root: PathBuf::new(),
            reason: "migration failed".to_string(),
            newer_version,
        }
    }

    #[test]
    fn unopenable_database_offers_the_reset() {
        let failure = failure(false);
        assert!(failure.resettable());
        let error = failure.to_error();
        assert!(matches!(error.kind, ErrorKind::AppsDatabase));
        assert_eq!(error.reason, "migration failed");
    }

    #[test]
    fn database_from_a_newer_version_offers_no_reset() {
        let failure = failure(true);
        assert!(!failure.resettable());
        let error = failure.to_error();
        assert!(!matches!(error.kind, ErrorKind::AppsDatabase));
        assert!(error.reason.contains("newer version"), "{}", error.reason);
    }
}
