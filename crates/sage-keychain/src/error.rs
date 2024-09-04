use thiserror::Error;

#[derive(Debug, Error)]
pub enum KeychainError {
    #[error("Encoding error: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("Could not encrypt key data.")]
    Encrypt,

    #[error("Could not decrypt key data.")]
    Decrypt,

    #[error("Argon2 error: {0}")]
    Argon2(argon2::Error),
}
