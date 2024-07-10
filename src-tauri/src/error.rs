use std::num::ParseIntError;

use sage::KeychainError;
use serde::{Serialize, Serializer};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
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
