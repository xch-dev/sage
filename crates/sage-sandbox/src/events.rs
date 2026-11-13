//! Forwards a wallet's sync events to the pages listening on the bridge.
//!
//! This is the loop in `src-tauri/src/app_state.rs`, sending to a broadcast channel instead of the
//! Tauri window.

use sage::{RefreshDebouncer, api_sync_event};
use sage_api::SyncEvent as ApiEvent;
use sage_wallet::SyncEvent;
use tokio::sync::{broadcast, mpsc};

/// Forwards events until the wallet's sync manager stops.
pub fn forward_events(
    mut receiver: mpsc::Receiver<SyncEvent>,
    sender: broadcast::Sender<ApiEvent>,
) {
    tokio::spawn(async move {
        let mut debouncer = RefreshDebouncer::default();

        loop {
            // Nobody listening isn't an error, since every page can be closed.
            tokio::select! {
                received = receiver.recv() => {
                    let Some(event) = received else {
                        break;
                    };

                    if let Some(event) = api_sync_event(event).and_then(|event| debouncer.push(event)) {
                        let _ = sender.send(event);
                    }
                }
                events = debouncer.flushed() => {
                    for event in events {
                        let _ = sender.send(event);
                    }
                }
            }
        }
    });
}
