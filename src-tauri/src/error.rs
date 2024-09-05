use std::{array::TryFromSliceError, fmt, io, num::ParseIntError};

use chia_wallet_sdk::{AddressError, ClientError, DriverError, SignerError};
use hex::FromHexError;
use sage_api::Amount;
use sage_database::DatabaseError;
use sage_keychain::KeychainError;
use serde::{Deserialize, Serialize};
use specta::Type;
use sqlx::migrate::MigrateError;
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

    pub fn invalid_prefix(prefix: &str) -> Self {
        Self {
            kind: ErrorKind::InvalidAddress,
            reason: format!("Invalid address prefix {prefix}"),
        }
    }

    pub fn insufficient_funds() -> Self {
        Self {
            kind: ErrorKind::InsufficientFunds,
            reason: "Insufficient funds".to_string(),
        }
    }

    pub fn no_change_address() -> Self {
        Self {
            kind: ErrorKind::TransactionFailed,
            reason: "No change address available".to_string(),
        }
    }

    pub fn no_secret_key() -> Self {
        Self {
            kind: ErrorKind::TransactionFailed,
            reason: "No secret key available".to_string(),
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
    InsufficientFunds,
    TransactionFailed,
    UnknownNetwork,
    UnknownFingerprint,
    NotLoggedIn,
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

impl From<SignerError> for Error {
    fn from(value: SignerError) -> Self {
        Self {
            kind: ErrorKind::TransactionFailed,
            reason: value.to_string(),
        }
    }
}
