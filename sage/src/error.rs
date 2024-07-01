use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("invalid length {0}, expected {1}")]
    InvalidLength(usize, usize),

    #[error("precision lost in cast")]
    PrecisionLoss,
}

pub type Result<T> = std::result::Result<T, Error>;
