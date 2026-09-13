use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use sage::{Result, Sage};
use sage_api::SyncEvent as ApiEvent;
#[cfg(not(mobile))]
use sage_apps::{AppsHostState, emit_selected_wallet_changed, process_sage_network_change};
use sage_wallet::SyncEvent;
#[cfg(not(mobile))]
use tauri::Manager;
use tauri::{AppHandle, Emitter};
use tokio::{sync::Mutex, task::JoinHandle};

pub struct Initialized(pub Mutex<bool>);

pub struct RpcTask(pub Mutex<Option<JoinHandle<anyhow::Result<()>>>>);

#[derive(Clone)]
pub struct CancellationFlag(Arc<AtomicBool>);

impl CancellationFlag {
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

#[derive(Default)]
pub struct OfferCreationCancellation(Mutex<Vec<Arc<AtomicBool>>>);

impl OfferCreationCancellation {
    pub async fn begin(&self) -> CancellationFlag {
        let flag = Arc::new(AtomicBool::new(false));
        self.0.lock().await.push(flag.clone());
        CancellationFlag(flag)
    }

    pub async fn end(&self, flag: &CancellationFlag) {
        self.0.lock().await.retain(|f| !Arc::ptr_eq(f, &flag.0));
    }

    pub async fn cancel(&self) {
        for flag in self.0.lock().await.iter() {
            flag.store(true, Ordering::Relaxed);
        }
    }
}

pub type AppState = Arc<Mutex<Sage>>;

pub async fn initialize(app_handle: AppHandle, sage: &mut Sage) -> Result<()> {
    let mut receiver = sage.initialize().await?;

    tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
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
                SyncEvent::WalletChanged { .. } | SyncEvent::NetworkChanged { .. } => continue,
            };
            if app_handle.emit("sync-event", event).is_err() {
                break;
            }
        }

        Result::Ok(())
    });

    Ok(())
}
