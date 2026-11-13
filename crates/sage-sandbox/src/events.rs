//! Turns a wallet's sync events into the `sync-event` payloads the frontend listens for.
//!
//! This mirrors `src-tauri/src/app_state.rs`, including the debounce of refresh events.

use std::{collections::HashSet, future, time::Duration};

use sage_api::SyncEvent as ApiEvent;
use sage_wallet::SyncEvent;
use tokio::{
    sync::{broadcast, mpsc},
    time::{Instant, sleep_until},
};

const REFRESH_DEBOUNCE: Duration = Duration::from_millis(500);

fn convert(event: SyncEvent) -> Option<ApiEvent> {
    Some(match event {
        SyncEvent::Start(ip) => ApiEvent::Start { ip: ip.to_string() },
        SyncEvent::Stop => ApiEvent::Stop,
        SyncEvent::Subscribed => ApiEvent::Subscribed,
        SyncEvent::DerivationIndex { .. } => ApiEvent::Derivation,
        SyncEvent::TransactionFailed {
            transaction_id,
            error,
        } => ApiEvent::TransactionFailed {
            transaction_id: transaction_id.to_string(),
            error,
        },
        SyncEvent::CoinsUpdated
        | SyncEvent::TransactionUpdated { .. }
        | SyncEvent::OfferUpdated { .. } => ApiEvent::CoinState,
        SyncEvent::PuzzleBatchSynced => ApiEvent::PuzzleBatchSynced,
        SyncEvent::CatInfo => ApiEvent::CatInfo,
        SyncEvent::DidInfo => ApiEvent::DidInfo,
        SyncEvent::NftData => ApiEvent::NftData,
        SyncEvent::WalletChanged { .. } | SyncEvent::NetworkChanged { .. } => return None,
    })
}

/// Whether the event only tells the frontend to refetch, so that a burst can be collapsed.
fn is_refresh(event: &ApiEvent) -> bool {
    matches!(
        event,
        ApiEvent::Derivation
            | ApiEvent::CoinState
            | ApiEvent::PuzzleBatchSynced
            | ApiEvent::CatInfo
            | ApiEvent::DidInfo
            | ApiEvent::NftData
    )
}

fn refresh_key(event: &ApiEvent) -> u8 {
    match event {
        ApiEvent::Derivation => 0,
        ApiEvent::CoinState => 1,
        ApiEvent::PuzzleBatchSynced => 2,
        ApiEvent::CatInfo => 3,
        ApiEvent::DidInfo => 4,
        _ => 5,
    }
}

/// Forwards events until the wallet's sync manager stops.
pub fn forward_events(
    mut receiver: mpsc::Receiver<SyncEvent>,
    sender: broadcast::Sender<ApiEvent>,
) {
    tokio::spawn(async move {
        let mut pending: Vec<ApiEvent> = Vec::new();
        let mut pending_keys = HashSet::new();
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

                    let Some(event) = convert(event) else {
                        continue;
                    };

                    if is_refresh(&event) {
                        if pending_keys.insert(refresh_key(&event)) {
                            pending.push(event);
                        }
                        flush_at.get_or_insert_with(|| Instant::now() + REFRESH_DEBOUNCE);
                    } else {
                        // Nobody listening isn't an error, since the page can be closed.
                        let _ = sender.send(event);
                    }
                }
                () = flush => {
                    flush_at = None;
                    pending_keys.clear();

                    for event in pending.drain(..) {
                        let _ = sender.send(event);
                    }
                }
            }
        }
    });
}
