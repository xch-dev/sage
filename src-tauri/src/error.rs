use std::net::AddrParseError;
use std::num::ParseIntError;

use chia::clvm_traits::FromClvmError;
use chia::{clvm_traits::ToClvmError, protocol::Bytes32};
use chia_wallet_sdk::DriverError;
use chia_wallet_sdk::{AddressError, ClientError};
use sage::KeychainError;
use sage_database::DatabaseError;
use serde::{Serialize, Serializer};
use thiserror::Error;
use tokio::task::JoinError;
use tokio::time::error::Elapsed;
use tracing::metadata::ParseLevelError;
use tracing_appender::rolling::InitError;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Could not parse log level: {0}")]
    ParseLogLevel(#[from] ParseLevelError),

    #[error("IP address parse error: {0}")]
    ParseIp(#[from] AddrParseError),

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

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Driver error: {0}")]
    Driver(#[from] DriverError),

    #[error("Client error: {0}")]
    Client(#[from] ClientError),

    #[error("No active wallet")]
    NoActiveWallet,

    #[error("Unknown wallet fingerprint: {0}")]
    Fingerprint(u32),

    #[error("Unknown network: {0}")]
    UnknownNetwork(String),

    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("Timeout error")]
    Timeout(#[from] Elapsed),

    #[error("Subscription limit reached")]
    SubscriptionLimitReached,

    #[error("To CLVM error: {0}")]
    ToClvm(#[from] ToClvmError),

    #[error("From CLVM error: {0}")]
    FromClvm(#[from] FromClvmError),

    #[error("Coin {0} has unknown puzzle mod hash {1}")]
    UnknownPuzzle(Bytes32, Bytes32),

    #[error("Unexpected rejection")]
    Rejection,

    #[error("Coin state not found")]
    CoinStateNotFound,

    #[error("Missing created height")]
    MissingCreatedHeight,

    #[error("Streamable error: {0}")]
    Streamable(#[from] chia::traits::Error),

    #[error("Join error: {0}")]
    Join(#[from] JoinError),

    #[error("Address error: {0}")]
    Address(#[from] AddressError),

    #[error("Bech32 error: {0}")]
    Bech32(#[from] bech32::Error),
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
