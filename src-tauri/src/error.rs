use std::{
    array::TryFromSliceError,
    fmt, io,
    num::{ParseIntError, TryFromIntError},
    time::SystemTimeError,
};

use chia::{
    clvm_traits::{FromClvmError, ToClvmError},
    protocol::Bytes32,
};
use chia_wallet_sdk::{AddressError, ClientError, DriverError};
use hex::FromHexError;
use sage_api::Amount;
use sage_database::DatabaseError;
use sage_keychain::KeychainError;
use sage_wallet::{SyncCommand, WalletError};
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::migrate::MigrateError;
use tokio::sync::{mpsc, oneshot::error::RecvError};
use tracing::metadata::ParseLevelError;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Error {
    kind: ErrorKind,
    reason: String,
}

impl Error {
    pub fn unknown_network(network: &str) -> Self {
        Self {
            kind: ErrorKind::UnknownNetwork,
            reason: format!("Unknown network {network}"),
        }
    }

    pub fn unknown_fingerprint(fingerprint: u32) -> Self {
        Self {
            kind: ErrorKind::UnknownFingerprint,
            reason: format!("Unknown fingerprint {fingerprint}"),
        }
    }

    pub fn invalid_amount(amount: &Amount) -> Self {
        Self {
            kind: ErrorKind::InvalidAmount,
            reason: format!("Invalid amount {amount}"),
        }
    }

    pub fn invalid_royalty(amount: &Amount) -> Self {
        Self {
            kind: ErrorKind::InvalidRoyalty,
            reason: format!("Invalid royalty {amount}"),
        }
    }

    pub fn invalid_prefix(prefix: &str) -> Self {
        Self {
            kind: ErrorKind::InvalidAddress,
            reason: format!("Invalid address prefix {prefix}"),
        }
    }

    pub fn invalid_asset_id() -> Self {
        Self {
            kind: ErrorKind::InvalidAssetId,
            reason: "Invalid asset id".to_string(),
        }
    }

    pub fn invalid_launcher_id() -> Self {
        Self {
            kind: ErrorKind::InvalidLauncherId,
            reason: "Invalid launcher id".to_string(),
        }
    }

    pub fn insufficient_funds() -> Self {
        Self {
            kind: ErrorKind::InsufficientFunds,
            reason: "Insufficient funds".to_string(),
        }
    }

    pub fn no_secret_key() -> Self {
        Self {
            kind: ErrorKind::TransactionFailed,
            reason: "No secret key available".to_string(),
        }
    }

    pub fn unknown_coin_id() -> Self {
        Self {
            kind: ErrorKind::TransactionFailed,
            reason: "Unknown coin id".to_string(),
        }
    }

    pub fn already_spent(coin_id: Bytes32) -> Self {
        Self {
            kind: ErrorKind::TransactionFailed,
            reason: format!("Coin {coin_id} has already been spent"),
        }
    }

    pub fn insufficient_coin_total() -> Self {
        Self {
            kind: ErrorKind::TransactionFailed,
            reason: "Insufficient coin total".to_string(),
        }
    }

    pub fn no_peers() -> Self {
        Self {
            kind: ErrorKind::TransactionFailed,
            reason: "No peers available to broadcast transaction to".to_string(),
        }
    }

    pub fn not_logged_in() -> Self {
        Self {
            kind: ErrorKind::NotLoggedIn,
            reason: "Not currently logged into a wallet".to_string(),
        }
    }

    pub fn invalid_key(reason: &str) -> Self {
        Self {
            kind: ErrorKind::InvalidKey,
            reason: format!("Invalid key: {reason}"),
        }
    }

    pub fn no_peak() -> Self {
        Self {
            kind: ErrorKind::TransactionFailed,
            reason: "There is no blockchain peak yet, you haven't started syncing".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
pub enum ErrorKind {
    Io,
    Database,
    Client,
    Keychain,
    Logging,
    Serialization,
    InvalidAddress,
    InvalidMnemonic,
    InvalidKey,
    InvalidAmount,
    InvalidRoyalty,
    InvalidAssetId,
    InvalidLauncherId,
    InsufficientFunds,
    TransactionFailed,
    UnknownNetwork,
    UnknownFingerprint,
    NotLoggedIn,
    Sync,
    Wallet,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self {
            kind: ErrorKind::Io,
            reason: error.to_string(),
        }
    }
}

impl From<bincode::Error> for Error {
    fn from(value: bincode::Error) -> Self {
        Self {
            kind: ErrorKind::Serialization,
            reason: value.to_string(),
        }
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Self {
            kind: ErrorKind::Serialization,
            reason: value.to_string(),
        }
    }
}

impl From<toml::ser::Error> for Error {
    fn from(value: toml::ser::Error) -> Self {
        Self {
            kind: ErrorKind::Serialization,
            reason: value.to_string(),
        }
    }
}

impl From<DatabaseError> for Error {
    fn from(value: DatabaseError) -> Self {
        Self {
            kind: ErrorKind::Database,
            reason: value.to_string(),
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(value: sqlx::Error) -> Self {
        Self {
            kind: ErrorKind::Database,
            reason: value.to_string(),
        }
    }
}

impl From<MigrateError> for Error {
    fn from(value: MigrateError) -> Self {
        Self {
            kind: ErrorKind::Database,
            reason: value.to_string(),
        }
    }
}

impl From<bech32::Error> for Error {
    fn from(value: bech32::Error) -> Self {
        Self {
            kind: ErrorKind::InvalidAddress,
            reason: value.to_string(),
        }
    }
}

impl From<chia::bls::Error> for Error {
    fn from(value: chia::bls::Error) -> Self {
        Self {
            kind: ErrorKind::InvalidKey,
            reason: value.to_string(),
        }
    }
}

impl From<chia::ssl::Error> for Error {
    fn from(value: chia::ssl::Error) -> Self {
        Self {
            kind: ErrorKind::Client,
            reason: value.to_string(),
        }
    }
}

impl From<ClientError> for Error {
    fn from(value: ClientError) -> Self {
        Self {
            kind: ErrorKind::Client,
            reason: value.to_string(),
        }
    }
}

impl From<ParseIntError> for Error {
    fn from(value: ParseIntError) -> Self {
        Self {
            kind: ErrorKind::Serialization,
            reason: value.to_string(),
        }
    }
}

impl From<KeychainError> for Error {
    fn from(value: KeychainError) -> Self {
        Self {
            kind: ErrorKind::Keychain,
            reason: value.to_string(),
        }
    }
}

impl From<bip39::Error> for Error {
    fn from(value: bip39::Error) -> Self {
        Self {
            kind: ErrorKind::InvalidMnemonic,
            reason: value.to_string(),
        }
    }
}

impl From<FromHexError> for Error {
    fn from(value: FromHexError) -> Self {
        Self {
            kind: ErrorKind::Serialization,
            reason: value.to_string(),
        }
    }
}

impl From<TryFromSliceError> for Error {
    fn from(value: TryFromSliceError) -> Self {
        Self {
            kind: ErrorKind::Serialization,
            reason: value.to_string(),
        }
    }
}

impl From<ParseLevelError> for Error {
    fn from(value: ParseLevelError) -> Self {
        Self {
            kind: ErrorKind::Logging,
            reason: value.to_string(),
        }
    }
}

impl From<tracing_appender::rolling::InitError> for Error {
    fn from(value: tracing_appender::rolling::InitError) -> Self {
        Self {
            kind: ErrorKind::Logging,
            reason: value.to_string(),
        }
    }
}

impl From<AddressError> for Error {
    fn from(value: AddressError) -> Self {
        Self {
            kind: ErrorKind::InvalidAddress,
            reason: value.to_string(),
        }
    }
}

impl From<DriverError> for Error {
    fn from(value: DriverError) -> Self {
        Self {
            kind: ErrorKind::TransactionFailed,
            reason: value.to_string(),
        }
    }
}

impl From<mpsc::error::SendError<SyncCommand>> for Error {
    fn from(value: mpsc::error::SendError<SyncCommand>) -> Self {
        Self {
            kind: ErrorKind::Sync,
            reason: value.to_string(),
        }
    }
}

impl From<TryFromIntError> for Error {
    fn from(value: TryFromIntError) -> Self {
        Self {
            kind: ErrorKind::InvalidAmount,
            reason: value.to_string(),
        }
    }
}

impl From<FromClvmError> for Error {
    fn from(value: FromClvmError) -> Self {
        Self {
            kind: ErrorKind::Serialization,
            reason: value.to_string(),
        }
    }
}

impl From<ToClvmError> for Error {
    fn from(value: ToClvmError) -> Self {
        Self {
            kind: ErrorKind::Serialization,
            reason: value.to_string(),
        }
    }
}

impl From<WalletError> for Error {
    fn from(value: WalletError) -> Self {
        Self {
            kind: ErrorKind::Wallet,
            reason: value.to_string(),
        }
    }
}

impl From<RecvError> for Error {
    fn from(value: RecvError) -> Self {
        Self {
            kind: ErrorKind::Wallet,
            reason: value.to_string(),
        }
    }
}

impl From<SystemTimeError> for Error {
    fn from(value: SystemTimeError) -> Self {
        Self {
            kind: ErrorKind::Io,
            reason: value.to_string(),
        }
    }
}
