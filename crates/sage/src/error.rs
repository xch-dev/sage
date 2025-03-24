use std::{
    array::TryFromSliceError,
    io,
    net::AddrParseError,
    num::{ParseIntError, TryFromIntError},
};

use chia::{
    clvm_traits::{FromClvmError, ToClvmError},
    protocol::Bytes32,
};
use chia_wallet_sdk::{
    client::ClientError,
    driver::{DriverError, OfferError},
    utils::AddressError,
};
use clvmr::reduction::EvalErr;
use hex::FromHexError;
use sage_api::ErrorKind;
use sage_assets::UriError;
use sage_database::DatabaseError;
use sage_keychain::KeychainError;
use sage_wallet::{SyncCommand, WalletError};
use sqlx::migrate::MigrateError;
use thiserror::Error;
use tokio::{sync::mpsc::error::SendError, time::error::Elapsed};
use tracing::{metadata::ParseLevelError, subscriber::SetGlobalDefaultError};
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

    #[error("Driver error: {0}")]
    Driver(#[from] DriverError),

    #[error("Offer error: {0}")]
    Offer(#[from] OfferError),

    #[error("URI error: {0}")]
    Uri(#[from] UriError),

    #[error("To CLVM error: {0}")]
    ToClvm(#[from] ToClvmError),

    #[error("From CLVM error: {0}")]
    FromClvm(#[from] FromClvmError),

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

    #[error("Bincode error: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("Logging initialization error: {0}")]
    LogSubscriber(#[from] TryInitError),

    #[error("Logging initialization error: {0}")]
    LogAppender(#[from] InitError),

    #[error("Set global default logger error: {0}")]
    SetGlobalDefault(#[from] SetGlobalDefaultError),

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

    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    #[error("Invalid coin amount: {0}")]
    InvalidCoinAmount(String),

    #[error("Invalid hash: {0}")]
    InvalidHash(String),

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

    #[error("Invalid offer id: {0}")]
    InvalidOfferId(String),

    #[error("Invalid percentage: {0}")]
    InvalidPercentage(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("Cannot specify both collection and DID")]
    InvalidGroup,

    #[error("Wallet is cold and cannot be used for signing")]
    NoSigningKey,

    #[error("Missing coin: {0}")]
    MissingCoin(Bytes32),

    #[error("Missing DID: {0}")]
    MissingDid(Bytes32),

    #[error("Missing NFT: {0}")]
    MissingNft(Bytes32),

    #[error("Missing CAT coin: {0}")]
    MissingCatCoin(Bytes32),

    #[error("Missing offer: {0}")]
    MissingOffer(Bytes32),

    #[error("Coin already spent: {0}")]
    CoinSpent(Bytes32),

    #[error("IP addr parse error: {0}")]
    IpAddrParse(#[from] AddrParseError),

    #[error("No peers are currently available")]
    NoPeers,

    #[error("Could not fetch NFT with id: {0}")]
    CouldNotFetchNft(Bytes32),

    #[error("CLVM eval error: {0}")]
    Eval(#[from] EvalErr),

    #[error("Missing asset id")]
    MissingAssetId,

    #[error("Timeout")]
    Timeout(#[from] Elapsed),
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
            | Self::SetGlobalDefault(..)
            | Self::ParseLogLevel(..)
            | Self::Database(..)
            | Self::Bech32m(..)
            | Self::ToClvm(..)
            | Self::FromClvm(..)
            | Self::Bincode(..)
            | Self::Eval(..)
            | Self::Driver(..)
            | Self::Timeout(..) => ErrorKind::Internal,
            Self::UnknownFingerprint
            | Self::UnknownNetwork
            | Self::MissingCoin(..)
            | Self::MissingCatCoin(..)
            | Self::MissingDid(..)
            | Self::MissingNft(..)
            | Self::MissingOffer(..) => ErrorKind::NotFound,
            Self::Bls(..)
            | Self::Hex(..)
            | Self::InvalidKey
            | Self::TryFromSlice(..)
            | Self::TryFromInt(..)
            | Self::ParseInt(..)
            | Self::AddressPrefix(..)
            | Self::InvalidAmount(..)
            | Self::InvalidCoinAmount(..)
            | Self::Address(..)
            | Self::InvalidDidId(..)
            | Self::InvalidNftId(..)
            | Self::InvalidCollectionId(..)
            | Self::InvalidCoinId(..)
            | Self::InvalidHash(..)
            | Self::InvalidAssetId(..)
            | Self::InvalidOfferId(..)
            | Self::InvalidPercentage(..)
            | Self::InvalidSignature(..)
            | Self::InvalidPublicKey(..)
            | Self::CoinSpent(..)
            | Self::Uri(..)
            | Self::IpAddrParse(..)
            | Self::Offer(..)
            | Self::NoPeers
            | Self::CouldNotFetchNft(..)
            | Self::MissingAssetId
            | Self::InvalidGroup => ErrorKind::Api,
        }
    }
}
