use chia::protocol::Bytes32;
use chia_wallet_sdk::{ClientError, CoinSelectionError, DriverError};
use sage_database::DatabaseError;
use thiserror::Error;
use tokio::task::JoinError;

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Client error: {0}")]
    Client(#[from] ClientError),

    #[error("Sync error: {0}")]
    Sync(#[from] SyncError),

    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("Coin selection error: {0}")]
    CoinSelection(#[from] CoinSelectionError),

    #[error("Output amount exceeds input coin total")]
    InsufficientFunds,

    #[error("Driver error: {0}")]
    Driver(#[from] DriverError),

    #[error("Not enough keys have been derived")]
    InsufficientDerivations,

    #[error("Join error: {0}")]
    Join(#[from] JoinError),
}

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("Timeout exceeded")]
    Timeout,

    #[error("Unexpected rejection")]
    Rejection,

    #[error("Subscription limit exceeded")]
    SubscriptionLimit,

    #[error("Missing coin state {0}")]
    MissingCoinState(Bytes32),

    #[error("Unconfirmed coin {0}")]
    UnconfirmedCoin(Bytes32),

    #[error("Missing puzzle and solution for {0}")]
    MissingPuzzleAndSolution(Bytes32),

    #[error("Error fetching CAT {0}: {1}")]
    FetchCat(Bytes32, reqwest::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ParseError {
    #[error("Could not allocate puzzle reveal")]
    AllocatePuzzle,

    #[error("Could not allocate solution")]
    AllocateSolution,

    #[error("Could not allocate metadata")]
    AllocateMetadata,

    #[error("Could not serialize CLVM")]
    Serialize,

    #[error("Could not evaluate puzzle and solution")]
    Eval,

    #[error("Invalid condition list")]
    InvalidConditions,

    #[error("Unknown coin is missing hint")]
    MissingHint,
}
