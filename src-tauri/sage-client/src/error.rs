use chia::protocol::ProtocolMessageTypes;
use thiserror::Error;
use tokio::sync::oneshot::error::RecvError;

#[derive(Debug, Error)]
pub enum Error {
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
}

pub type Result<T> = std::result::Result<T, Error>;
