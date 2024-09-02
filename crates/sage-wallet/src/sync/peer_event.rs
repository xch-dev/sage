use std::{net::IpAddr, sync::Arc};

use chia::{
    protocol::{CoinStateUpdate, Message, NewPeakWallet, ProtocolMessageTypes},
    traits::Streamable,
};
use chia_wallet_sdk::ClientError;
use sage_database::Database;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info, instrument};

use crate::WalletError;

use super::{wallet_sync::update_coins, PeerState, SyncEvent};

#[derive(Debug)]
pub struct PeerEvent {
    pub ip: IpAddr,
    pub message: Option<Message>,
}

/// Pipes all of the [`Message`] received from a [`Peer`] into [`PeerEvent`] on a central sender.
/// Also sends a [`PeerEvent`] with a [`None`] message when the receiver is closed.
#[instrument(skip(receiver, sender))]
pub async fn handle_peer(
    ip: IpAddr,
    mut receiver: mpsc::Receiver<Message>,
    sender: mpsc::Sender<PeerEvent>,
) {
    while let Some(message) = receiver.recv().await {
        if sender
            .send(PeerEvent {
                ip,
                message: Some(message),
            })
            .await
            .is_err()
        {
            return;
        }
    }

    sender.send(PeerEvent { ip, message: None }).await.ok();
}

pub async fn handle_peer_events(
    db: Option<Database>,
    mut receiver: mpsc::Receiver<PeerEvent>,
    state: Arc<Mutex<PeerState>>,
    sync_sender: mpsc::Sender<SyncEvent>,
) {
    while let Some(event) = receiver.recv().await {
        let Some(message) = event.message else {
            state.lock().await.remove_peer(event.ip);
            debug!("Peer {} disconnected gracefully", event.ip);
            continue;
        };

        if let Err(error) =
            handle_peer_event(db.as_ref(), event.ip, message, &state, &sync_sender).await
        {
            debug!("Error handling peer event: {error}");
        }
    }
}

async fn handle_peer_event(
    db: Option<&Database>,
    ip: IpAddr,
    message: Message,
    state: &Arc<Mutex<PeerState>>,
    sync_sender: &mpsc::Sender<SyncEvent>,
) -> Result<(), WalletError> {
    match message.msg_type {
        ProtocolMessageTypes::NewPeakWallet => {
            let message = NewPeakWallet::from_bytes(&message.data).map_err(ClientError::from)?;
            state
                .lock()
                .await
                .update_peak(ip, message.height, message.header_hash);
        }
        ProtocolMessageTypes::CoinStateUpdate => {
            let message = CoinStateUpdate::from_bytes(&message.data).map_err(ClientError::from)?;
            if let Some(db) = db {
                update_coins(db, message.items).await?;
                db.insert_peak(message.height, message.peak_hash).await?;
                info!(
                    "Received updates and synced to peak {} with header hash {}",
                    message.height, message.peak_hash
                );
                sync_sender.send(SyncEvent::CoinUpdate).await.ok();
            }
        }
        _ => {
            debug!("Received unexpected message type: {:?}", message.msg_type);
        }
    }

    Ok(())
}
