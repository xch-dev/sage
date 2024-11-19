use std::{
    array::TryFromSliceError,
    io,
    num::{ParseIntError, TryFromIntError},
};

use chia::{clvm_traits::ToClvmError, protocol::Bytes32};
use chia_wallet_sdk::{AddressError, ClientError};
use hex::FromHexError;
use sage_api::ErrorKind;
use sage_database::DatabaseError;
use sage_keychain::KeychainError;
use sage_wallet::{SyncCommand, UriError, WalletError};
use sqlx::migrate::MigrateError;
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tracing::metadata::ParseLevelError;
use tracing_appender::rolling::InitError;
use tracing_subscriber::util::TryInitError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Keychain error: {0}")]
    Keychain(#[from] KeychainError),

    #[error("Address error: {0}")]
    Address(#[from] AddressError),

    #[error("Wallet error: {0}")]
    Wallet(#[from] WalletError),

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Client error: {0}")]
    Client(#[from] ClientError),

    #[error("URI error: {0}")]
    Uri(#[from] UriError),

    #[error("To CLVM error: {0}")]
    ToClvm(#[from] ToClvmError),

    #[error("BLS error: {0}")]
    Bls(#[from] chia::bls::Error),

    #[error("BIP39 error: {0}")]
    Bip39(#[from] bip39::Error),

    #[error("Bech32m error: {0}")]
    Bech32m(#[from] bech32::Error),

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

    #[error("Try from int error: {0}")]
    TryFromInt(#[from] TryFromIntError),

    #[error("Parse int error: {0}")]
    ParseInt(#[from] ParseIntError),

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

    #[error("Invalid key")]
    InvalidKey,

    #[error("Wrong address prefix: {0}")]
    AddressPrefix(String),

    #[error("Invalid CAT amount: {0}")]
    InvalidCatAmount(String),

    #[error("Invalid coin amount: {0}")]
    InvalidCoinAmount(String),

    #[error("Invalid genesis id: {0}")]
    InvalidGenesisChallenge(String),

    #[error("Invalid puzzle hash: {0}")]
    InvalidPuzzleHash(String),

    #[error("Invalid coin id: {0}")]
    InvalidCoinId(String),

    #[error("Invalid DID id: {0}")]
    InvalidDidId(String),

    #[error("Invalid NFT id: {0}")]
    InvalidNftId(String),

    #[error("Invalid collection id: {0}")]
    InvalidCollectionId(String),

    #[error("Invalid asset id: {0}")]
    InvalidAssetId(String),

    #[error("Invalid percentage: {0}")]
    InvalidPercentage(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Wallet is cold and cannot be used for signing")]
    NoSigningKey,

    #[error("Missing coin: {0}")]
    MissingCoin(Bytes32),

    #[error("Missing CAT coin: {0}")]
    MissingCatCoin(Bytes32),

    #[error("Coin already spent: {0}")]
    CoinSpent(Bytes32),
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Wallet(..) => ErrorKind::Wallet,
            Self::NotLoggedIn | Self::NoSigningKey => ErrorKind::Unauthorized,
            Self::Keychain(error) => match error {
                KeychainError::Decrypt => ErrorKind::Unauthorized,
                KeychainError::KeyExists
                | KeychainError::Bincode(..)
                | KeychainError::Encrypt
                | KeychainError::Bls(..)
                | KeychainError::Bip39(..)
                | KeychainError::Argon2(..) => ErrorKind::Internal,
            },
            Self::Send(..)
            | Self::Io(..)
            | Self::Client(..)
            | Self::Sqlx(..)
            | Self::SqlxMigration(..)
            | Self::Bip39(..)
            | Self::TomlDe(..)
            | Self::TomlSer(..)
            | Self::LogAppender(..)
            | Self::LogSubscriber(..)
            | Self::ParseLogLevel(..)
            | Self::Database(..)
            | Self::Bech32m(..)
            | Self::ToClvm(..) => ErrorKind::Internal,
            Self::UnknownFingerprint
            | Self::UnknownNetwork
            | Self::MissingCoin(..)
            | Self::MissingCatCoin(..) => ErrorKind::NotFound,
            Self::Bls(..)
            | Self::Hex(..)
            | Self::InvalidKey
            | Self::TryFromSlice(..)
            | Self::TryFromInt(..)
            | Self::ParseInt(..)
            | Self::AddressPrefix(..)
            | Self::InvalidCatAmount(..)
            | Self::InvalidCoinAmount(..)
            | Self::Address(..)
            | Self::InvalidDidId(..)
            | Self::InvalidNftId(..)
            | Self::InvalidCollectionId(..)
            | Self::InvalidGenesisChallenge(..)
            | Self::InvalidCoinId(..)
            | Self::InvalidPuzzleHash(..)
            | Self::InvalidAssetId(..)
            | Self::InvalidPercentage(..)
            | Self::InvalidSignature(..)
            | Self::CoinSpent(..)
            | Self::Uri(..) => ErrorKind::Api,
        }
    }
}
