use std::{collections::HashSet, future, sync::Arc, time::Duration};

use sage::{Result, Sage};
use sage_api::SyncEvent as ApiEvent;
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

/// How long a burst of refresh events is collected before being emitted.
const REFRESH_DEBOUNCE: Duration = Duration::from_millis(500);

/// Refresh events, which tell the frontend "this kind of data changed, refetch".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum RefreshEvent {
    Derivation,
    CoinState,
    PuzzleBatchSynced,
    CatInfo,
    DidInfo,
    NftData,
}

impl RefreshEvent {
    /// Returns `None` for events that must be delivered as they happen, either
    /// because they carry a payload or because they are one-off state changes.
    fn from_api_event(event: &ApiEvent) -> Option<Self> {
        match event {
            ApiEvent::Derivation => Some(Self::Derivation),
            ApiEvent::CoinState => Some(Self::CoinState),
            ApiEvent::PuzzleBatchSynced => Some(Self::PuzzleBatchSynced),
            ApiEvent::CatInfo => Some(Self::CatInfo),
            ApiEvent::DidInfo => Some(Self::DidInfo),
            ApiEvent::NftData => Some(Self::NftData),
            ApiEvent::Start { .. }
            | ApiEvent::Stop
            | ApiEvent::Subscribed
            | ApiEvent::TransactionFailed { .. } => None,
        }
    }

    fn into_api_event(self) -> ApiEvent {
        match self {
            Self::Derivation => ApiEvent::Derivation,
            Self::CoinState => ApiEvent::CoinState,
            Self::PuzzleBatchSynced => ApiEvent::PuzzleBatchSynced,
            Self::CatInfo => ApiEvent::CatInfo,
            Self::DidInfo => ApiEvent::DidInfo,
            Self::NftData => ApiEvent::NftData,
        }
    }
}

pub struct Initialized(pub Mutex<bool>);

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
                    let event = match event {
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
                        // TODO: New event?
                        SyncEvent::CoinsUpdated
                        | SyncEvent::TransactionUpdated { .. }
                        | SyncEvent::OfferUpdated { .. } => ApiEvent::CoinState,
                        SyncEvent::PuzzleBatchSynced => ApiEvent::PuzzleBatchSynced,
                        SyncEvent::CatInfo => ApiEvent::CatInfo,
                        SyncEvent::DidInfo => ApiEvent::DidInfo,
                        SyncEvent::NftData => ApiEvent::NftData,
                        SyncEvent::WalletChanged { .. } | SyncEvent::NetworkChanged { .. } => {
                            continue;
                        }
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
