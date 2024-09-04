use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeychainError {
    #[error("Encoding error: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("Argon2 error: {0}")]
    Argon2(argon2::Error),

    #[error("BLS error: {0}")]
    Bls(#[from] chia::bls::Error),

    #[error("BIP39 error: {0}")]
    Bip39(#[from] bip39::Error),

    #[error("Could not encrypt key data")]
    Encrypt,

    #[error("Could not decrypt key data")]
    Decrypt,

    #[error("Key already exists")]
    KeyExists,
}
