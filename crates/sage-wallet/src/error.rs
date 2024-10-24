use std::time::SystemTimeError;

use chia::{bls::PublicKey, protocol::Bytes32};
use chia_wallet_sdk::{ClientError, CoinSelectionError, DriverError, SignerError};
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

    #[error("Driver error: {0}")]
    Driver(#[from] DriverError),

    #[error("Signer error: {0}")]
    Signer(#[from] SignerError),

    #[error("Join error: {0}")]
    Join(#[from] JoinError),

    #[error("Coin selection error: {0}")]
    CoinSelection(#[from] CoinSelectionError),

    #[error("Output amount exceeds input coin total")]
    InsufficientFunds,

    #[error("Not enough keys have been derived")]
    InsufficientDerivations,

    #[error("Spendable DID not found: {0}")]
    MissingDid(Bytes32),

    #[error("Unknown public key in transaction: {0:?}")]
    UnknownPublicKey(PublicKey),

    #[error("Time error: {0}")]
    Time(#[from] SystemTimeError),
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
}
