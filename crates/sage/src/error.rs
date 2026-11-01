use std::{
    array::TryFromSliceError,
    io,
    net::AddrParseError,
    num::{ParseIntError, TryFromIntError},
};

use chia_wallet_sdk::{
    client::ClientError,
    clvm_traits::{FromClvmError, ToClvmError},
    clvmr::error::EvalErr,
    prelude::*,
    utils::Bech32Error,
};
use hex::FromHexError;
use sage_api::{ErrorKind, HandleErrorCode, RecipientError};
use sage_assets::UriError;
use sage_database::DatabaseError;
use sage_keychain::KeychainError;
use sage_wallet::{SyncCommand, WalletError, XchandlesApiError, XchandlesError};
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
    Address(#[from] Bech32Error),

    #[error("Wallet error: {0}")]
    Wallet(#[from] WalletError),

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Client error: {0}")]
    Client(#[from] ClientError),

    #[error("Driver error: {0}")]
    Driver(#[from] DriverError),

    #[error("XCHandles error: {0}")]
    Xchandles(#[from] XchandlesError),

    #[error("URI error: {0}")]
    Uri(#[from] UriError),

    #[error("To CLVM error: {0}")]
    ToClvm(#[from] ToClvmError),

    #[error("From CLVM error: {0}")]
    FromClvm(#[from] FromClvmError),

    #[error("BLS error: {0}")]
    Bls(#[from] chia_wallet_sdk::chia::bls::Error),

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

    #[error("Wallet {0} already exists")]
    WalletExists(u32),

    #[error("Key {0} is used by a wallet, so it can only be removed by deleting the wallet")]
    KeyInUse(u32),

    #[error("A different key with fingerprint {0} is already in the keychain")]
    KeyConflict(u32),

    #[error("Key {0} is already in the keychain")]
    KeyExists(u32),

    #[error("Fingerprint {0} is already used by a different kind of wallet")]
    FingerprintCollision(u32),

    #[error("No free fingerprint could be found for this wallet")]
    NoFreeFingerprint,

    #[error("A watch wallet needs at least one address")]
    NoWatchAddresses,

    #[error("Wallet {0} isn't a watch wallet")]
    NotWatchWallet(u32),

    #[error("Not logged in")]
    NotLoggedIn,

    #[error("Invalid key")]
    InvalidKey,

    #[error("{0}")]
    InvalidMnemonic(String),

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

    #[error("Invalid option contract id: {0}")]
    InvalidOptionId(String),

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

    #[error("Missing CAT: {0}")]
    MissingCat(Bytes32),

    #[error("Missing DID: {0}")]
    MissingDid(Bytes32),

    #[error("Missing NFT: {0}")]
    MissingNft(Bytes32),

    #[error("Missing option: {0}")]
    MissingOption(Bytes32),

    #[error("Missing CAT coin: {0}")]
    MissingCatCoin(Bytes32),

    #[error("Missing DID coin: {0}")]
    MissingDidCoin(Bytes32),

    #[error("Missing NFT coin: {0}")]
    MissingNftCoin(Bytes32),

    #[error("Missing offer: {0}")]
    MissingOffer(Bytes32),

    #[error("Coin already spent: {0}")]
    CoinSpent(Bytes32),

    #[error(
        "Coin {0} cannot be selected: it must be this wallet's, confirmed, unspent, of an asset this transaction spends, and not pending or locked in an offer"
    )]
    CoinNotSelectable(Bytes32),

    #[error("IP addr parse error: {0}")]
    IpAddrParse(#[from] AddrParseError),

    #[error("No peers are currently available")]
    NoPeers,

    #[error("Could not fetch NFT with id: {0}")]
    CouldNotFetchNft(Bytes32),

    #[error("Could not fetch option with id: {0}")]
    CouldNotFetchOption(Bytes32),

    #[error("Invalid theme JSON format")]
    InvalidThemeJson,

    #[error("Missing theme data in NFT metadata")]
    MissingThemeData,

    #[error("CLVM eval error: {0}")]
    Eval(#[from] EvalErr),

    #[error("Missing asset id")]
    MissingAssetId,

    #[error("Database version too old")]
    DatabaseVersionTooOld,

    #[error("Sage did not initialize successfully, so changes cannot be saved")]
    NotInitialized,

    #[error("Another instance of Sage is already running with this data directory")]
    AlreadyRunning,

    #[error(
        "The config file at {path} is invalid: {error}\nFix or delete this file, then restart Sage. Deleting it resets the settings it contains, but doesn't affect your keys."
    )]
    InvalidConfig {
        path: String,
        error: toml::de::Error,
    },

    #[error("Timeout")]
    Timeout(#[from] Elapsed),

    #[error("At most {0} addresses can be looked up at once")]
    TooManyAddresses(usize),

    #[error("Contact {0} not found")]
    MissingContact(u32),

    #[error("Invalid contact: {0}")]
    InvalidContact(String),

    #[error("{recipient} is already saved as the contact {name}")]
    DuplicateContactRecipient {
        recipient: String,
        name: String,
        id: u32,
    },

    #[error("Another contact is already named {name}")]
    DuplicateContactName { name: String, id: u32 },

    #[error("These contacts are for the network {0}, not the current network")]
    ContactNetworkMismatch(String),

    #[error("{}", recipient_errors_message(.0))]
    Recipients(Vec<RecipientError>),

    #[error("The offer can't be taken: {0}")]
    OfferHandleNft(String),
}

fn recipient_errors_message(errors: &[RecipientError]) -> String {
    errors
        .iter()
        .map(|error| error.reason.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

impl Error {
    /// The recipients which couldn't be used, if that's why the request failed.
    pub fn recipient_errors(&self) -> Option<&[RecipientError]> {
        match self {
            Self::Recipients(errors) => Some(errors),
            _ => None,
        }
    }

    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Recipients(..) => ErrorKind::Recipient,
            Self::Wallet(..) | Self::NoSigningKey => ErrorKind::Wallet,
            Self::Xchandles(error) => match error {
                XchandlesError::Database(..) => ErrorKind::Internal,
                error if error.is_not_registered() => ErrorKind::NotFound,
                // Failures that may succeed if retried, rather than a problem with the request.
                error if error.is_unavailable() => ErrorKind::Internal,
                _ => ErrorKind::Api,
            },
            Self::NotLoggedIn => ErrorKind::Unauthorized,
            Self::Keychain(error) => match error {
                KeychainError::Decrypt => ErrorKind::Unauthorized,
                KeychainError::KeyExists
                | KeychainError::Bincode(..)
                | KeychainError::Encrypt
                | KeychainError::Bls(..)
                | KeychainError::Bip39(..)
                | KeychainError::Argon2(..) => ErrorKind::Internal,
            },
            Self::SqlxMigration(..) | Self::DatabaseVersionTooOld => ErrorKind::DatabaseMigration,
            Self::Send(..)
            | Self::Io(..)
            | Self::Client(..)
            | Self::Sqlx(..)
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
            | Self::Timeout(..)
            | Self::NotInitialized
            | Self::InvalidConfig { .. }
            | Self::FingerprintCollision(..)
            | Self::NoFreeFingerprint => ErrorKind::Internal,
            Self::AlreadyRunning => ErrorKind::AlreadyRunning,
            Self::UnknownFingerprint
            | Self::UnknownNetwork
            | Self::MissingCoin(..)
            | Self::MissingCatCoin(..)
            | Self::MissingDidCoin(..)
            | Self::MissingNftCoin(..)
            | Self::MissingCat(..)
            | Self::MissingDid(..)
            | Self::MissingNft(..)
            | Self::MissingOption(..)
            | Self::MissingOffer(..)
            | Self::MissingContact(..) => ErrorKind::NotFound,
            Self::Bls(..)
            | Self::Hex(..)
            | Self::InvalidKey
            | Self::WalletExists(..)
            | Self::KeyInUse(..)
            | Self::KeyConflict(..)
            | Self::KeyExists(..)
            | Self::NoWatchAddresses
            | Self::NotWatchWallet(..)
            | Self::InvalidMnemonic(..)
            | Self::TryFromSlice(..)
            | Self::TryFromInt(..)
            | Self::ParseInt(..)
            | Self::AddressPrefix(..)
            | Self::InvalidAmount(..)
            | Self::InvalidCoinAmount(..)
            | Self::Address(..)
            | Self::InvalidDidId(..)
            | Self::InvalidNftId(..)
            | Self::InvalidOptionId(..)
            | Self::InvalidCollectionId(..)
            | Self::InvalidCoinId(..)
            | Self::InvalidHash(..)
            | Self::InvalidAssetId(..)
            | Self::InvalidOfferId(..)
            | Self::InvalidPercentage(..)
            | Self::InvalidSignature(..)
            | Self::InvalidPublicKey(..)
            | Self::CoinSpent(..)
            | Self::CoinNotSelectable(..)
            | Self::Uri(..)
            | Self::IpAddrParse(..)
            | Self::NoPeers
            | Self::CouldNotFetchNft(..)
            | Self::CouldNotFetchOption(..)
            | Self::MissingAssetId
            | Self::InvalidGroup
            | Self::InvalidThemeJson
            | Self::MissingThemeData
            | Self::TooManyAddresses(..)
            | Self::InvalidContact(..)
            | Self::DuplicateContactRecipient { .. }
            | Self::DuplicateContactName { .. }
            | Self::ContactNetworkMismatch(..)
            | Self::OfferHandleNft(..) => ErrorKind::Api,
        }
    }

    /// A machine-readable reason for an `XCHandles` failure, so that callers can tell the
    /// cases apart without parsing the message.
    pub fn handle_error_code(&self) -> Option<HandleErrorCode> {
        let Self::Xchandles(error) = self else {
            return None;
        };

        Some(match error {
            XchandlesError::InvalidHandle(..) => HandleErrorCode::InvalidHandle,
            XchandlesError::UnsupportedNetwork => HandleErrorCode::UnsupportedNetwork,
            XchandlesError::Expired { .. }
            | XchandlesError::Api(XchandlesApiError::HandleExpired) => HandleErrorCode::Expired,
            XchandlesError::Api(XchandlesApiError::HandleNotFound) => HandleErrorCode::NotFound,
            XchandlesError::Database(..) => return None,
            error if error.is_unavailable() => HandleErrorCode::Unavailable,
            _ => HandleErrorCode::VerificationFailed,
        })
    }
}

#[cfg(test)]
mod tests {
    use sage_wallet::HandleError;

    use super::*;

    #[test]
    fn test_xchandles_error_kinds() {
        let kind = |error: XchandlesError| Error::from(error).kind();

        assert!(matches!(kind(XchandlesError::NoPeers), ErrorKind::Internal));
        assert!(matches!(kind(XchandlesError::Timeout), ErrorKind::Internal));
        assert!(matches!(
            kind(XchandlesError::InvalidLineage),
            ErrorKind::Api
        ));
        assert!(matches!(
            kind(XchandlesError::UnsupportedNetwork),
            ErrorKind::Api
        ));
        assert!(matches!(
            kind(XchandlesError::Expired {
                expiration: 0,
                timestamp: 0
            }),
            ErrorKind::NotFound
        ));
    }

    #[test]
    fn test_xchandles_error_codes() {
        let code = |error: XchandlesError| Error::from(error).handle_error_code();

        assert_eq!(
            code(XchandlesError::InvalidHandle(HandleError::InvalidLength)),
            Some(HandleErrorCode::InvalidHandle)
        );
        assert_eq!(
            code(XchandlesError::UnsupportedNetwork),
            Some(HandleErrorCode::UnsupportedNetwork)
        );
        assert_eq!(
            code(XchandlesApiError::HandleNotFound.into()),
            Some(HandleErrorCode::NotFound)
        );
        assert_eq!(
            code(XchandlesApiError::HandleExpired.into()),
            Some(HandleErrorCode::Expired)
        );
        assert_eq!(
            code(XchandlesError::Expired {
                expiration: 0,
                timestamp: 0
            }),
            Some(HandleErrorCode::Expired)
        );
        for error in [
            XchandlesError::NoPeers,
            XchandlesError::Timeout,
            XchandlesApiError::IndexStale.into(),
            XchandlesError::SlotSpent(Bytes32::default()),
        ] {
            assert_eq!(code(error), Some(HandleErrorCode::Unavailable));
        }
        for error in [
            XchandlesError::InvalidLineage,
            XchandlesError::ResponseMismatch,
            XchandlesError::ResolvedSingletonMelted(Bytes32::default()),
        ] {
            assert_eq!(code(error), Some(HandleErrorCode::VerificationFailed));
        }
        assert_eq!(Error::NotLoggedIn.handle_error_code(), None);
    }
}
