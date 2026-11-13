use std::sync::Arc;

use sage::{RefreshDebouncer, Result, Sage, api_sync_event};
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
