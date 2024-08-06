use std::sync::Arc;

use chia::{
    protocol::{
        ChiaProtocolMessage, CoinStateUpdate, Handshake, Message, NewPeakWallet,
        ProtocolMessageTypes, SendTransaction, TransactionAck,
    },
    traits::Streamable,
};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use sha2::{digest::FixedOutput, Digest, Sha256};
use tokio::{
    net::TcpStream,
    sync::{
        mpsc::{self, Receiver},
        oneshot, Mutex,
    },
    task::JoinHandle,
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

use crate::{Error, Event, Response, Result};

use super::requests::Requests;

pub(super) type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type Sink = SplitSink<WebSocket, tungstenite::Message>;
type Stream = SplitStream<WebSocket>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PeerId([u8; 32]);

#[derive(Debug, Clone)]
pub struct Peer(Arc<PeerInner>);

#[derive(Debug)]
struct PeerInner {
    sink: Mutex<Sink>,
    inbound_handle: JoinHandle<Result<()>>,
    requests: Arc<Requests>,
    peer_id: PeerId,
}

impl Peer {
    pub fn new(ws: WebSocket) -> Result<(Self, Receiver<Event>)> {
        let cert = match ws.get_ref() {
            MaybeTlsStream::NativeTls(tls) => tls.get_ref().peer_certificate()?,
            _ => None,
        };

        let Some(cert) = cert else {
            return Err(Error::MissingCertificate);
        };

        let mut hasher = Sha256::new();
        hasher.update(cert.to_der()?);

        let peer_id = PeerId(hasher.finalize_fixed().into());
        let (sink, stream) = ws.split();
        let (sender, receiver) = mpsc::channel(32);
        let requests = Arc::new(Requests::new());

        let inbound_handle =
            tokio::spawn(handle_inbound_messages(stream, sender, requests.clone()));

        let peer = Self(Arc::new(PeerInner {
            sink: Mutex::new(sink),
            inbound_handle,
            requests,
            peer_id,
        }));

        Ok((peer, receiver))
    }

    pub fn peer_id(&self) -> PeerId {
        self.0.peer_id
    }

    pub async fn send_transaction(&self, body: SendTransaction) -> Result<TransactionAck> {
        self.request_infallible(body).await
    }

    pub async fn send<T>(&self, body: T) -> Result<()>
    where
        T: Streamable + ChiaProtocolMessage,
    {
        let message = Message {
            msg_type: T::msg_type(),
            id: None,
            data: body.to_bytes()?.into(),
        }
        .to_bytes()?
        .into();

        self.0.sink.lock().await.send(message).await?;

        Ok(())
    }

    pub async fn request_fallible<T, E, B>(&self, body: B) -> Result<Response<T, E>>
    where
        T: Streamable + ChiaProtocolMessage,
        E: Streamable + ChiaProtocolMessage,
        B: Streamable + ChiaProtocolMessage,
    {
        let message = self.request_raw(body).await?;
        if message.msg_type != T::msg_type() && message.msg_type != E::msg_type() {
            return Err(Error::InvalidResponse(
                vec![T::msg_type(), E::msg_type()],
                message.msg_type,
            ));
        }
        if message.msg_type == T::msg_type() {
            Ok(Response::Success(T::from_bytes(&message.data)?))
        } else {
            Ok(Response::Rejection(E::from_bytes(&message.data)?))
        }
    }

    pub async fn request_infallible<T, B>(&self, body: B) -> Result<T>
    where
        T: Streamable + ChiaProtocolMessage,
        B: Streamable + ChiaProtocolMessage,
    {
        let message = self.request_raw(body).await?;
        if message.msg_type != T::msg_type() {
            return Err(Error::InvalidResponse(
                vec![T::msg_type()],
                message.msg_type,
            ));
        }
        Ok(T::from_bytes(&message.data)?)
    }

    pub async fn request_raw<T>(&self, body: T) -> Result<Message>
    where
        T: Streamable + ChiaProtocolMessage,
    {
        let (sender, receiver) = oneshot::channel();

        let message = Message {
            msg_type: T::msg_type(),
            id: Some(self.0.requests.insert(sender).await),
            data: body.to_bytes()?.into(),
        }
        .to_bytes()?
        .into();

        self.0.sink.lock().await.send(message).await?;
        Ok(receiver.await?)
    }
}

impl Drop for PeerInner {
    fn drop(&mut self) {
        self.inbound_handle.abort();
    }
}

async fn handle_inbound_messages(
    mut stream: Stream,
    sender: mpsc::Sender<Event>,
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
