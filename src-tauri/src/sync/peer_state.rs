use std::{net::IpAddr, sync::Arc};

use chia::{
    protocol::{Bytes32, Message, NewPeakWallet, ProtocolMessageTypes},
    traits::Streamable,
};
use chia_wallet_sdk::Peer;
use tokio::{
    sync::{mpsc, Mutex},
    task::JoinHandle,
};
use tracing::{debug, info, instrument};

use crate::error::Error;

use super::sync_state::SyncState;

pub struct PeerState {
    pub peer: Peer,
    pub claimed_peak: u32,
    pub header_hash: Bytes32,
    pub task: JoinHandle<()>,
}

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
        sender
            .send(PeerEvent {
                ip,
                message: Some(message),
            })
            .await
            .unwrap();
    }

    sender.send(PeerEvent { ip, message: None }).await.unwrap();
}

pub async fn handle_peer_events(
    mut receiver: mpsc::Receiver<PeerEvent>,
    state: Arc<Mutex<SyncState>>,
) {
    while let Some(event) = receiver.recv().await {
        debug!("Received peer event {event:?}");

        match event.message {
            Some(message) => {
                if let Err(error) = handle_peer_event(event.ip, message, &state).await {
                    info!("Error handling peer event: {error}");
                }
            }
            None => {
                state.lock().await.remove_peer(event.ip);
            }
        }
    }
}

async fn handle_peer_event(
    ip: IpAddr,
    message: Message,
    state: &Arc<Mutex<SyncState>>,
) -> Result<(), Error> {
    if message.msg_type == ProtocolMessageTypes::NewPeakWallet {
        let message = NewPeakWallet::from_bytes(&message.data)?;
        state
            .lock()
            .await
            .update_peak(ip, message.height, message.header_hash);
    }

    Ok(())
}
