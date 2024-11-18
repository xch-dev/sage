use std::{
    array::TryFromSliceError,
    fmt,
    num::{ParseIntError, TryFromIntError},
};

use chia::clvm_traits::{FromClvmError, ToClvmError};
use chia_wallet_sdk::{AddressError, OfferError};
use hex::FromHexError;
use sage_database::DatabaseError;
use sage_keychain::KeychainError;
use sage_wallet::{SyncCommand, UriError, WalletError};
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::mpsc::error::SendError;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct Error {
    pub kind: ErrorKind,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    Wallet,
    Api,
    NotFound,
    Internal,
}

impl From<sage::Error> for Error {
    fn from(error: sage::Error) -> Self {
        Self {
            kind: ErrorKind::Wallet,
            reason: error.to_string(),
        }
    }
}

impl From<DatabaseError> for Error {
    fn from(error: DatabaseError) -> Self {
        Self {
            kind: ErrorKind::Wallet,
            reason: error.to_string(),
        }
    }
}

impl From<AddressError> for Error {
    fn from(error: AddressError) -> Self {
        Self {
            kind: ErrorKind::Api,
            reason: error.to_string(),
        }
    }
}

impl From<FromHexError> for Error {
    fn from(error: FromHexError) -> Self {
        Self {
            kind: ErrorKind::Api,
            reason: error.to_string(),
        }
    }
}

impl From<ParseIntError> for Error {
    fn from(error: ParseIntError) -> Self {
        Self {
            kind: ErrorKind::Internal,
            reason: error.to_string(),
        }
    }
}

impl From<KeychainError> for Error {
    fn from(error: KeychainError) -> Self {
        Self {
            kind: ErrorKind::Internal,
            reason: error.to_string(),
        }
    }
}

impl From<bech32::Error> for Error {
    fn from(error: bech32::Error) -> Self {
        Self {
            kind: ErrorKind::Internal,
            reason: error.to_string(),
        }
    }
}

impl From<WalletError> for Error {
    fn from(error: WalletError) -> Self {
        Self {
            kind: ErrorKind::Wallet,
            reason: error.to_string(),
        }
    }
}

impl From<ToClvmError> for Error {
    fn from(error: ToClvmError) -> Self {
        Self {
            kind: ErrorKind::Internal,
            reason: error.to_string(),
        }
    }
}

impl From<FromClvmError> for Error {
    fn from(error: FromClvmError) -> Self {
        Self {
            kind: ErrorKind::Internal,
            reason: error.to_string(),
        }
    }
}

impl From<TryFromIntError> for Error {
    fn from(error: TryFromIntError) -> Self {
        Self {
            kind: ErrorKind::Internal,
            reason: error.to_string(),
        }
    }
}

impl From<TryFromSliceError> for Error {
    fn from(error: TryFromSliceError) -> Self {
        Self {
            kind: ErrorKind::Internal,
            reason: error.to_string(),
        }
    }
}

impl From<bip39::Error> for Error {
    fn from(error: bip39::Error) -> Self {
        Self {
            kind: ErrorKind::Internal,
            reason: error.to_string(),
        }
    }
}

impl From<chia::bls::Error> for Error {
    fn from(error: chia::bls::Error) -> Self {
        Self {
            kind: ErrorKind::Internal,
            reason: error.to_string(),
        }
    }
}

impl From<SendError<SyncCommand>> for Error {
    fn from(error: SendError<SyncCommand>) -> Self {
        Self {
            kind: ErrorKind::Internal,
            reason: error.to_string(),
        }
    }
}

impl From<UriError> for Error {
    fn from(error: UriError) -> Self {
        Self {
            kind: ErrorKind::Api,
            reason: error.to_string(),
        }
    }
}

impl From<OfferError> for Error {
    fn from(error: OfferError) -> Self {
        Self {
            kind: ErrorKind::Wallet,
            reason: error.to_string(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
