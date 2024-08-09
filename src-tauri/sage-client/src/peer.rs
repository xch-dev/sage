use std::{fmt, net::SocketAddr, sync::Arc};

use chia::protocol::{
    Bytes32, ChiaProtocolMessage, CoinStateFilters, Message, PuzzleSolutionResponse,
    RegisterForCoinUpdates, RegisterForPhUpdates, RejectCoinState, RejectPuzzleSolution,
    RejectPuzzleState, RequestChildren, RequestCoinState, RequestPeers, RequestPuzzleSolution,
    RequestPuzzleState, RequestTransaction, RespondChildren, RespondCoinState, RespondPeers,
    RespondPuzzleSolution, RespondPuzzleState, RespondToCoinUpdates, RespondToPhUpdates,
    RespondTransaction, SendTransaction, SpendBundle, TransactionAck,
};
use chia::traits::Streamable;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use native_tls::TlsConnector;
use sha2::{digest::FixedOutput, Digest, Sha256};
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot, Mutex},
    task::JoinHandle,
};
use tokio_tungstenite::{Connector, MaybeTlsStream, WebSocketStream};

use crate::request_map::RequestMap;
use crate::{Error, Result};

type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;
type Sink = SplitSink<WebSocket, tungstenite::Message>;
type Stream = SplitStream<WebSocket>;
type Response<T, E> = std::result::Result<T, E>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PeerId([u8; 32]);

impl fmt::Display for PeerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

#[derive(Debug, Clone)]
pub struct Peer(Arc<PeerInner>);

#[derive(Debug)]
struct PeerInner {
    sink: Mutex<Sink>,
    inbound_handle: JoinHandle<()>,
    requests: Arc<RequestMap>,
    peer_id: PeerId,
    socket_addr: SocketAddr,
}

impl Peer {
    /// Connects to a peer using its IP address and port.
    pub async fn connect(
        socket_addr: SocketAddr,
        tls_connector: TlsConnector,
    ) -> Result<(Self, mpsc::Receiver<Message>)> {
        Self::connect_full_uri(&format!("wss://{socket_addr}/ws"), tls_connector).await
    }

    /// Connects to a peer using its full websocket URI.
    /// For example, `wss://127.0.0.1:8444/ws`.
    pub async fn connect_full_uri(
        uri: &str,
        tls_connector: TlsConnector,
    ) -> Result<(Self, mpsc::Receiver<Message>)> {
        let (ws, _) = tokio_tungstenite::connect_async_tls_with_config(
            uri,
            None,
            false,
            Some(Connector::NativeTls(tls_connector)),
        )
        .await?;
        Self::from_websocket(ws)
    }

    /// Creates a peer from an existing websocket connection.
    /// The connection must be secured with TLS, so that the certificate can be hashed in a peer id.
    pub fn from_websocket(ws: WebSocket) -> Result<(Self, mpsc::Receiver<Message>)> {
        let (socket_addr, cert) = match ws.get_ref() {
            MaybeTlsStream::NativeTls(tls) => {
                let tls_stream = tls.get_ref();
                let tcp_stream = tls_stream.get_ref().get_ref();
                (tcp_stream.peer_addr()?, tls_stream.peer_certificate()?)
            }
            _ => return Err(Error::MissingCertificate),
        };

        let Some(cert) = cert else {
            return Err(Error::MissingCertificate);
        };

        let mut hasher = Sha256::new();
        hasher.update(cert.to_der()?);

        let peer_id = PeerId(hasher.finalize_fixed().into());
        let (sink, stream) = ws.split();
        let (sender, receiver) = mpsc::channel(32);

        let requests = Arc::new(RequestMap::new());
        let requests_clone = requests.clone();

        let inbound_handle = tokio::spawn(async move {
            if let Err(error) = handle_inbound_messages(stream, sender, requests_clone).await {
                log::warn!("Error handling message: {error}");
            }
        });

        let peer = Self(Arc::new(PeerInner {
            sink: Mutex::new(sink),
            inbound_handle,
            requests,
            peer_id,
            socket_addr,
        }));

        Ok((peer, receiver))
    }

    /// The hash of the TLS certificate used by the peer.
    pub fn peer_id(&self) -> PeerId {
        self.0.peer_id
    }

    /// The IP address and port of the peer connection.
    pub fn socket_addr(&self) -> SocketAddr {
        self.0.socket_addr
    }

    pub async fn send_transaction(&self, spend_bundle: SpendBundle) -> Result<TransactionAck> {
        self.request_infallible(SendTransaction::new(spend_bundle))
            .await
    }

    pub async fn request_puzzle_state(
        &self,
        puzzle_hashes: Vec<Bytes32>,
        previous_height: Option<u32>,
        header_hash: Bytes32,
        filters: CoinStateFilters,
        subscribe_when_finished: bool,
    ) -> Result<Response<RespondPuzzleState, RejectPuzzleState>> {
        self.request_fallible(RequestPuzzleState::new(
            puzzle_hashes,
            previous_height,
            header_hash,
            filters,
            subscribe_when_finished,
        ))
        .await
    }

    pub async fn request_coin_state(
        &self,
        coin_ids: Vec<Bytes32>,
        previous_height: Option<u32>,
        header_hash: Bytes32,
        subscribe: bool,
    ) -> Result<Response<RespondCoinState, RejectCoinState>> {
        self.request_fallible(RequestCoinState::new(
            coin_ids,
            previous_height,
            header_hash,
            subscribe,
        ))
        .await
    }

    pub async fn register_for_ph_updates(
        &self,
        puzzle_hashes: Vec<Bytes32>,
        min_height: u32,
    ) -> Result<RespondToPhUpdates> {
        self.request_infallible(RegisterForPhUpdates::new(puzzle_hashes, min_height))
            .await
    }

    pub async fn register_for_coin_updates(
        &self,
        coin_ids: Vec<Bytes32>,
        min_height: u32,
    ) -> Result<RespondToCoinUpdates> {
        self.request_infallible(RegisterForCoinUpdates::new(coin_ids, min_height))
            .await
    }

    pub async fn request_transaction(&self, transaction_id: Bytes32) -> Result<RespondTransaction> {
        self.request_infallible(RequestTransaction::new(transaction_id))
            .await
    }

    pub async fn request_puzzle_and_solution(
        &self,
        coin_id: Bytes32,
        height: u32,
    ) -> Result<Response<PuzzleSolutionResponse, RejectPuzzleSolution>> {
        match self
            .request_fallible::<RespondPuzzleSolution, _, _>(RequestPuzzleSolution::new(
                coin_id, height,
            ))
            .await?
        {
            Ok(response) => Ok(Ok(response.response)),
            Err(rejection) => Ok(Err(rejection)),
        }
    }

    pub async fn request_children(&self, coin_id: Bytes32) -> Result<RespondChildren> {
        self.request_infallible(RequestChildren::new(coin_id)).await
    }

    pub async fn request_peers(&self) -> Result<RespondPeers> {
        self.request_infallible(RequestPeers::new()).await
    }

    /// Sends a message to the peer, but does not expect any response.
    pub async fn send<T>(&self, body: T) -> Result<()>
    where
        T: Streamable + ChiaProtocolMessage,
    {
        let message = Message::new(T::msg_type(), None, body.to_bytes()?.into())
            .to_bytes()?
            .into();

        self.0.sink.lock().await.send(message).await?;

        Ok(())
    }

    /// Sends a message to the peer and expects a message that's either a response or a rejection.
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
            Ok(Ok(T::from_bytes(&message.data)?))
        } else {
            Ok(Err(E::from_bytes(&message.data)?))
        }
    }

    /// Sends a message to the peer and expects a specific response message.
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

    /// Sends a message to the peer and expects any arbitrary protocol message without parsing it.
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
    sender: mpsc::Sender<Message>,
    requests: Arc<RequestMap>,
) -> Result<()> {
    use tungstenite::Message::{Binary, Close, Frame, Ping, Pong, Text};

    while let Some(message) = stream.next().await {
        let message = message?;

        match message {
            Text(text) => {
                log::warn!("Received unexpected text message: {text}");
            }
            Close(close) => {
                log::warn!("Received close: {close:?}");
                break;
            }
            Ping(_ping) => {}
            Pong(_pong) => {}
            Binary(binary) => {
                let message = Message::from_bytes(&binary)?;

                let Some(id) = message.id else {
                    sender.send(message).await.map_err(|error| {
                        log::warn!("Failed to send peer message event: {error}");
                        Error::EventNotSent
                    })?;
                    continue;
                };

                let Some(request) = requests.remove(id).await else {
                    log::warn!(
                        "Received {:?} message with untracked id {id}",
                        message.msg_type
                    );
                    return Err(Error::UnexpectedMessage(message.msg_type));
                };

                request.send(message);
            }
            Frame(frame) => {
                log::warn!("Received frame: {frame}");
            }
        }
    }
    Ok(())
}
