use std::num::TryFromIntError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Precision lost during cast")]
    PrecisionLost(#[from] TryFromIntError),

    #[error("Invalid length {0}, expected {1}")]
    InvalidLength(usize, usize),

    #[error("BLS error: {0}")]
    Bls(#[from] chia::bls::Error),

    #[error("Invalid enum variant")]
    InvalidEnumVariant,
}

pub(crate) type Result<T> = std::result::Result<T, DatabaseError>;
