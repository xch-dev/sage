use std::{collections::HashSet, future, sync::Arc, time::Duration};

use sage::{RefreshEvent, Result, Sage, api_sync_event};
#[cfg(not(mobile))]
use sage_apps::{AppsHostState, emit_selected_wallet_changed, process_sage_network_change};
use sage_wallet::SyncEvent;
#[cfg(not(mobile))]
use tauri::Manager;
use tauri::{AppHandle, Emitter};
use tokio::{
    sync::Mutex,
    task::JoinHandle,
    time::{Instant, sleep_until},
};

use crate::error;

/// How long a burst of refresh events is collected before being emitted.
const REFRESH_DEBOUNCE: Duration = Duration::from_millis(500);

/// The outcome of initialization, once it has been attempted.
pub struct Initialized(pub Mutex<Option<error::Result<()>>>);

pub struct RpcTask(pub Mutex<Option<JoinHandle<anyhow::Result<()>>>>);

pub type AppState = Arc<Mutex<Sage>>;

pub async fn initialize(app_handle: AppHandle, sage: &mut Sage) -> Result<()> {
    let mut receiver = sage.initialize().await?;

    tokio::spawn(async move {
        let mut pending = HashSet::new();
        let mut flush_at: Option<Instant> = None;

        loop {
            // Only wait on a deadline when a burst is actually in flight.
            let flush = async {
                match flush_at {
                    Some(deadline) => sleep_until(deadline).await,
                    None => future::pending().await,
                }
            };

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

                    if let Some(refresh) = RefreshEvent::from_api_event(&event) {
                        pending.insert(refresh);
                        flush_at.get_or_insert_with(|| Instant::now() + REFRESH_DEBOUNCE);
                    } else if app_handle.emit("sync-event", event).is_err() {
                        break;
                    }
                }
                () = flush => {
                    flush_at = None;

                    for refresh in pending.drain() {
                        if app_handle.emit("sync-event", refresh.into_api_event()).is_err() {
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
