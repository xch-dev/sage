use std::{net::IpAddr, sync::Arc};

use chia::{
    protocol::{Message, NewPeakWallet, ProtocolMessageTypes},
    traits::Streamable,
};
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, instrument};

use crate::WalletError;

use super::PeerState;

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
    mut receiver: mpsc::Receiver<PeerEvent>,
    state: Arc<Mutex<PeerState>>,
) {
    while let Some(event) = receiver.recv().await {
        debug!("Received peer event {event:?}");

        match event.message {
            Some(message) => {
                if let Err(error) = handle_peer_event(event.ip, message, &state).await {
                    debug!("Error handling peer event: {error}");
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
    state: &Arc<Mutex<PeerState>>,
) -> Result<(), WalletError> {
    if message.msg_type == ProtocolMessageTypes::NewPeakWallet {
        let message = NewPeakWallet::from_bytes(&message.data).unwrap();
        state
            .lock()
            .await
            .update_peak(ip, message.height, message.header_hash);
    }

    Ok(())
}
