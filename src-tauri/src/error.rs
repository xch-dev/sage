use std::num::ParseIntError;

use chia::{clvm_traits::ToClvmError, protocol::Bytes32};
use chia_wallet_sdk::{ParseError, SslError};
use sage::KeychainError;
use serde::{Serialize, Serializer};
use thiserror::Error;
use tokio::time::error::Elapsed;
use tracing::metadata::ParseLevelError;
use tracing_appender::rolling::InitError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Could not parse log level: {0}")]
    ParseLogLevel(#[from] ParseLevelError),

    #[error("Log init error: {0}")]
    LogInit(#[from] InitError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Bip39 error: {0}")]
    Bip39(#[from] bip39::Error),

    #[error("BLS error: {0}")]
    Bls(#[from] chia::bls::Error),

    #[error("Bincode error: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("TOML deserialization error: {0}")]
    DeserializeToml(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    SerializeToml(#[from] toml::ser::Error),

    #[error("Invalid key size")]
    InvalidKeySize,

    #[error("ParseInt error: {0}")]
    ParseInt(#[from] ParseIntError),

    #[error("Keychain error: {0}")]
    Keychain(#[from] KeychainError),

    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("SQLx migration error: {0}")]
    SqlxMigration(#[from] sqlx::migrate::MigrateError),

    #[error("Wallet error: {0}")]
    Wallet(#[from] sage::Error),

    #[error("SSL error: {0}")]
    Ssl(#[from] SslError),

    #[error("TLS error: {0}")]
    Tls(#[from] native_tls::Error),

    #[error("No active wallet")]
    NoActiveWallet,

    #[error("Unknown wallet fingerprint: {0}")]
    Fingerprint(u32),

    #[error("Unknown network: {0}")]
    UnknownNetwork(String),

    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("Client error: {0}")]
    Client(#[from] sage_client::Error),

    #[error("Timeout error")]
    Timeout(#[from] Elapsed),

    #[error("Subscription limit reached")]
    SubscriptionLimitReached,

    #[error("To CLVM error: {0}")]
    ToClvm(#[from] ToClvmError),

    #[error("Rejected puzzle and solution for coin id {0}")]
    RejectPuzzleSolution(Bytes32),

    #[error("Rejected coin state for coin id {0}")]
    RejectCoinState(Bytes32),

    #[error("Missing coin state for coin id {0}")]
    MissingCoinState(Bytes32),

    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("Unknown coin {0} with type {1}")]
    UnknownCoinType(Bytes32, Bytes32),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
