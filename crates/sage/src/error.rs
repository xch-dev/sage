use std::{array::TryFromSliceError, io};

use chia_wallet_sdk::ClientError;
use hex::FromHexError;
use sage_keychain::KeychainError;
use sage_wallet::SyncCommand;
use sqlx::migrate::MigrateError;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tracing::metadata::ParseLevelError;
use tracing_appender::rolling::InitError;
use tracing_subscriber::util::TryInitError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Keychain error: {0}")]
    Keychain(#[from] KeychainError),

    #[error("Client error: {0}")]
    Client(#[from] ClientError),

    #[error("Send error: {0}")]
    Send(#[from] SendError<SyncCommand>),

    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    #[error("Logging initialization error: {0}")]
    LogSubscriber(#[from] TryInitError),

    #[error("Logging initialization error: {0}")]
    LogAppender(#[from] InitError),

    #[error("Parse log level error: {0}")]
    ParseLogLevel(#[from] ParseLevelError),

    #[error("Hex decoding error: {0}")]
    Hex(#[from] FromHexError),

    #[error("Try from slice error: {0}")]
    TryFromSlice(#[from] TryFromSliceError),

    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("SQLx Migration error: {0}")]
    SqlxMigration(#[from] MigrateError),

    #[error("Unknown network")]
    UnknownNetwork,

    #[error("Unknown fingerprint")]
    UnknownFingerprint,

    #[error("Not logged in")]
    NotLoggedIn,
}

pub type Result<T> = std::result::Result<T, Error>;
