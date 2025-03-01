use std::sync::Arc;

use sage::{Result, Sage};
use sage_api::SyncEvent as ApiEvent;
use sage_wallet::SyncEvent;
use tauri::{AppHandle, Emitter};
use tokio::{sync::Mutex, task::JoinHandle};

pub struct Initialized(pub Mutex<bool>);

pub struct RpcTask(pub Mutex<Option<JoinHandle<anyhow::Result<()>>>>);

pub type AppState = Arc<Mutex<Sage>>;

pub async fn initialize(app_handle: AppHandle, sage: &mut Sage) -> Result<()> {
    let mut receiver = sage.initialize().await?;

    tokio::spawn(async move {
        while let Some(event) = receiver.recv().await {
            let event = match event {
                SyncEvent::Start(ip) => ApiEvent::Start { ip: ip.to_string() },
                SyncEvent::Stop => ApiEvent::Stop,
                SyncEvent::Subscribed => ApiEvent::Subscribed,
                SyncEvent::DerivationIndex { .. } => ApiEvent::Derivation,
                // TODO: New event?
                SyncEvent::CoinsUpdated { .. }
                | SyncEvent::TransactionUpdated { .. }
                | SyncEvent::TransactionEnded { .. }
                | SyncEvent::OfferUpdated { .. } => ApiEvent::CoinState,
                SyncEvent::PuzzleBatchSynced => ApiEvent::PuzzleBatchSynced,
                SyncEvent::CatInfo => ApiEvent::CatInfo,
                SyncEvent::DidInfo => ApiEvent::DidInfo,
                SyncEvent::NftData => ApiEvent::NftData,
            };
            if app_handle.emit("sync-event", event).is_err() {
                break;
            }
        }

        Result::Ok(())
    });

    Ok(())
}
