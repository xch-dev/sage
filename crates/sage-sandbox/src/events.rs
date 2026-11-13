//! Turns a wallet's sync events into the `sync-event` payloads the frontend listens for.
//!
//! This mirrors the loop in `src-tauri/src/app_state.rs`, including the debounce of refresh events,
//! but sends to a broadcast channel instead of the Tauri window.

use std::{collections::HashSet, future, time::Duration};

use sage::{RefreshEvent, api_sync_event};
use sage_api::SyncEvent as ApiEvent;
use sage_wallet::SyncEvent;
use tokio::{
    sync::{broadcast, mpsc},
    time::{Instant, sleep_until},
};

/// How long a burst of refresh events is collected before being sent, as in the desktop app.
const REFRESH_DEBOUNCE: Duration = Duration::from_millis(500);

/// Forwards events until the wallet's sync manager stops.
pub fn forward_events(
    mut receiver: mpsc::Receiver<SyncEvent>,
    sender: broadcast::Sender<ApiEvent>,
) {
    tokio::spawn(async move {
        let mut pending = HashSet::new();
        let mut flush_at: Option<Instant> = None;

        loop {
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

                    let Some(event) = api_sync_event(event) else {
                        continue;
                    };

                    if let Some(refresh) = RefreshEvent::from_api_event(&event) {
                        pending.insert(refresh);
                        flush_at.get_or_insert_with(|| Instant::now() + REFRESH_DEBOUNCE);
                    } else {
                        // Nobody listening isn't an error, since every page can be closed.
                        let _ = sender.send(event);
                    }
                }
                () = flush => {
                    flush_at = None;

                    for refresh in pending.drain() {
                        let _ = sender.send(refresh.into_api_event());
                    }
                }
            }
        }
    });
}
