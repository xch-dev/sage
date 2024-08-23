use chia::protocol::{NodeType, ProtocolMessageTypes};
use thiserror::Error;
use tokio::sync::oneshot::error::RecvError;

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("SSL error: {0}")]
    Ssl(#[from] chia::ssl::Error),

    #[error("Peer is missing certificate")]
    MissingCertificate,

    #[error("Streamable error: {0}")]
    Streamable(#[from] chia::traits::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tungstenite::Error),

    #[error("TLS error: {0}")]
    Tls(#[from] native_tls::Error),

    #[error("Unexpected message received with type {0:?}")]
    UnexpectedMessage(ProtocolMessageTypes),

    #[error("Expected response with type {0:?}, found {1:?}")]
    InvalidResponse(Vec<ProtocolMessageTypes>, ProtocolMessageTypes),

    #[error("Failed to send event")]
    EventNotSent,

    #[error("Failed to receive message")]
    Recv(#[from] RecvError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Missing response during handshake")]
    MissingHandshake,

    #[error("Expected node type {0:?}, but found {1:?}")]
    WrongNodeType(NodeType, NodeType),

    #[error("Expected network {0}, but found {1}")]
    WrongNetwork(String, String),

    #[error("The peer is banned")]
    BannedPeer,
}
