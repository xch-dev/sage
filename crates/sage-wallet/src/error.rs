use sage_database::DatabaseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ParseError {
    #[error("Could not allocate puzzle reveal")]
    AllocatePuzzle,

    #[error("Could not allocate solution")]
    AllocateSolution,

    #[error("Could not serialize CLVM")]
    Serialize,
}
