use std::sync::Arc;

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
