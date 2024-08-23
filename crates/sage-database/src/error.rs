use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Precision lost during cast")]
    PrecisionLost,

    #[error("Invalid length {0}, expected {1}")]
    InvalidLength(usize, usize),
}

pub(crate) type Result<T> = std::result::Result<T, DatabaseError>;
