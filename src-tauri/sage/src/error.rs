use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Precision lost during cast")]
    PrecisionLost,

    #[error("Invalid length {0}, expected {1}")]
    InvalidLength(usize, usize),
}

pub type Result<T> = std::result::Result<T, Error>;
