use std::sync::Arc;

use chia::{
    protocol::{CoinStateUpdate, Handshake, Message, NewPeakWallet, ProtocolMessageTypes},
    traits::Streamable,
};
use futures_util::{stream::SplitStream, StreamExt};
use tokio::sync::mpsc::Sender;

use crate::{Error, Event, Result};

use super::{requests::Requests, WebSocket};

type Stream = SplitStream<WebSocket>;

pub(super) async fn handle_inbound_messages(
    mut stream: Stream,
    sender: Sender<Event>,
    requests: Arc<Requests>,
) -> Result<()> {
    while let Some(message) = stream.next().await {
        let message = Message::from_bytes(&message?.into_data())?;

        match message.msg_type {
            ProtocolMessageTypes::CoinStateUpdate => {
                let event = Event::CoinStateUpdate(CoinStateUpdate::from_bytes(&message.data)?);
                sender.send(event).await.map_err(|error| {
                    log::error!("Failed to send `CoinStateUpdate` event: {error}");
                    Error::SendError
                })?;
            }
            ProtocolMessageTypes::NewPeakWallet => {
                let event = Event::NewPeakWallet(NewPeakWallet::from_bytes(&message.data)?);
                sender.send(event).await.map_err(|error| {
                    log::error!("Failed to send `NewPeakWallet` event: {error}");
                    Error::SendError
                })?;
            }
            ProtocolMessageTypes::Handshake => {
                let event = Event::Handshake(Handshake::from_bytes(&message.data)?);
                sender.send(event).await.map_err(|error| {
                    log::error!("Failed to send `Handshake` event: {error}");
                    Error::SendError
                })?;
            }
            kind => {
                let Some(id) = message.id else {
                    log::error!("Received unknown message without an id.");
                    return Err(Error::UnexpectedMessage(kind));
                };
                let Some(request) = requests.remove(id).await else {
                    log::error!("Received message with untracked id {id}.");
                    return Err(Error::UnexpectedMessage(kind));
                };
                request.send(message);
            }
        }
    }
    Ok(())
}
