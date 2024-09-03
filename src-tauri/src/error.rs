use std::array::TryFromSliceError;
use std::net::AddrParseError;
use std::num::ParseIntError;

use chia::clvm_traits::{FromClvmError, ToClvmError};
use chia_wallet_sdk::{AddressError, ClientError, DriverError};
use hex::FromHexError;
use sage_database::DatabaseError;
use sage_keychain::KeychainError;
use sage_wallet::ParseError;
use serde::Serialize;
use specta::Type;
use thiserror::Error;
use tokio::task::JoinError;
use tokio::time::error::Elapsed;
use tracing::metadata::ParseLevelError;
use tracing_appender::rolling::InitError;

#[derive(Debug, Error, Serialize, Type)]
pub enum Error {
    #[error("Could not parse log level: {0}")]
    ParseLogLevel(
        #[serde(skip)]
        #[from]
        ParseLevelError,
    ),

    #[error("IP address parse error: {0}")]
    ParseIp(
        #[serde(skip)]
        #[from]
        AddrParseError,
    ),

    #[error("Log init error: {0}")]
    LogInit(
        #[serde(skip)]
        #[from]
        InitError,
    ),

    #[error("IO error: {0}")]
    Io(
        #[serde(skip)]
        #[from]
        std::io::Error,
    ),

    #[error("Bip39 error: {0}")]
    Bip39(
        #[serde(skip)]
        #[from]
        bip39::Error,
    ),

    #[error("BLS error: {0}")]
    Bls(
        #[serde(skip)]
        #[from]
        chia::bls::Error,
    ),

    #[error("Bincode error: {0}")]
    Bincode(
        #[serde(skip)]
        #[from]
        bincode::Error,
    ),

    #[error("TOML deserialization error: {0}")]
    DeserializeToml(
        #[serde(skip)]
        #[from]
        toml::de::Error,
    ),

    #[error("TOML serialization error: {0}")]
    SerializeToml(
        #[serde(skip)]
        #[from]
        toml::ser::Error,
    ),

    #[error("Invalid key size")]
    InvalidKeySize,

    #[error("ParseInt error: {0}")]
    ParseInt(
        #[serde(skip)]
        #[from]
        ParseIntError,
    ),

    #[error("Keychain error: {0}")]
    Keychain(
        #[serde(skip)]
        #[from]
        KeychainError,
    ),

    #[error("SQLx error: {0}")]
    Sqlx(
        #[serde(skip)]
        #[from]
        sqlx::Error,
    ),

    #[error("SQLx migration error: {0}")]
    SqlxMigration(
        #[serde(skip)]
        #[from]
        sqlx::migrate::MigrateError,
    ),

    #[error("Database error: {0}")]
    Database(
        #[serde(skip)]
        #[from]
        DatabaseError,
    ),

    #[error("Driver error: {0}")]
    Driver(
        #[serde(skip)]
        #[from]
        DriverError,
    ),

    #[error("Client error: {0}")]
    Client(
        #[serde(skip)]
        #[from]
        ClientError,
    ),

    #[error("No active wallet")]
    NoActiveWallet,

    #[error("Unknown wallet fingerprint: {0}")]
    Fingerprint(#[serde(skip)] u32),

    #[error("Unknown network: {0}")]
    UnknownNetwork(#[serde(skip)] String),

    #[error("Tauri error: {0}")]
    Tauri(
        #[serde(skip)]
        #[from]
        tauri::Error,
    ),

    #[error("Timeout error")]
    Timeout(
        #[serde(skip)]
        #[from]
        Elapsed,
    ),

    #[error("To CLVM error: {0}")]
    ToClvm(
        #[serde(skip)]
        #[from]
        ToClvmError,
    ),

    #[error("From CLVM error: {0}")]
    FromClvm(
        #[serde(skip)]
        #[from]
        FromClvmError,
    ),

    #[error("Coin state not found")]
    CoinStateNotFound,

    #[error("Streamable error: {0}")]
    Streamable(
        #[serde(skip)]
        #[from]
        chia::traits::Error,
    ),

    #[error("Join error: {0}")]
    Join(
        #[serde(skip)]
        #[from]
        JoinError,
    ),

    #[error("Address error: {0}")]
    Address(
        #[serde(skip)]
        #[from]
        AddressError,
    ),

    #[error("Bech32 error: {0}")]
    Bech32(
        #[serde(skip)]
        #[from]
        bech32::Error,
    ),

    #[error("Parse error: {0}")]
    Parse(
        #[serde(skip)]
        #[from]
        ParseError,
    ),

    #[error("From hex error: {0}")]
    FromHex(
        #[serde(skip)]
        #[from]
        FromHexError,
    ),

    #[error("Try from slice error: {0}")]
    TryFromSlice(
        #[serde(skip)]
        #[from]
        TryFromSliceError,
    ),
}

pub type Result<T> = std::result::Result<T, Error>;
