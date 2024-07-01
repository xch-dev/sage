use chia::protocol::ProtocolMessageTypes;
use thiserror::Error;
use tokio::sync::oneshot::error::RecvError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Invalid length {0}, expected {1}")]
    InvalidLength(usize, usize),

    #[error("Precision lost in cast")]
    PrecisionLoss,

    #[error("Streamable error: {0}")]
    Streamable(#[from] chia::traits::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tungstenite::Error),

    #[error("Unexpected peer message: {0:?}")]
    UnexpectedMessage(ProtocolMessageTypes),

    #[error("Expected response with type {0:?}, found {1:?}")]
    InvalidResponse(Vec<ProtocolMessageTypes>, ProtocolMessageTypes),

    #[error("Could not send event")]
    SendError,

    #[error("Failed to receive response: {0}")]
    Response(#[from] RecvError),

    #[error("TLS error: {0}")]
    Tls(#[from] native_tls::Error),

    #[error("Missing certificate")]
    MissingCertificate,
}

pub type Result<T> = std::result::Result<T, Error>;
