use std::{num::TryFromIntError, time::SystemTimeError};

use chia::{
    clvm_traits::{FromClvmError, ToClvmError},
    protocol::Bytes32,
};
use chia_wallet_sdk::{
    client::ClientError,
    driver::{DriverError, OfferError},
    signer::SignerError,
    utils::CoinSelectionError,
};
use clvmr::reduction::EvalErr;
use sage_database::DatabaseError;
use thiserror::Error;
use tokio::{task::JoinError, time::error::Elapsed};

#[derive(Debug, Error)]
pub enum WalletError {
    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("Client error: {0}")]
    Client(#[from] ClientError),

    #[error("Driver error: {0}")]
    Driver(#[from] DriverError),

    #[error("Signer error: {0}")]
    Signer(#[from] SignerError),

    #[error("Offer error: {0}")]
    Offer(#[from] OfferError),

    #[error("Coin selection error: {0}")]
    CoinSelection(#[from] CoinSelectionError),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Timeout exceeded")]
    Elapsed(#[from] Elapsed),

    #[error("Missing coin with id {0}")]
    MissingCoin(Bytes32),

    #[error("Missing spend with id {0}")]
    MissingSpend(Bytes32),

    #[error("Missing child of id {0}")]
    MissingChild(Bytes32),

    #[error("Peer misbehaved")]
    PeerMisbehaved,

    #[error("Subscription limit reached")]
    SubscriptionLimitReached,

    #[error("System time error: {0}")]
    SystemTime(#[from] SystemTimeError),

    #[error("Join error: {0}")]
    Join(#[from] JoinError),

    #[error("CLVM encoding error: {0}")]
    ToClvm(#[from] ToClvmError),

    #[error("CLVM decoding error: {0}")]
    FromClvm(#[from] FromClvmError),

    #[error("CLVM error: {0}")]
    Clvm(#[from] EvalErr),

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Insufficient derivations")]
    InsufficientDerivations,

    #[error("Missing secret key")]
    UnknownPublicKey,

    #[error("SECP is not supported")]
    SecpNotSupported,

    #[error("Missing DID with id {0}")]
    MissingDid(Bytes32),

    #[error("Missing NFT with id {0}")]
    MissingNft(Bytes32),

    #[error("Uncancellable offer")]
    UncancellableOffer,

    #[error("Invalid trade price")]
    InvalidTradePrice,

    #[error("Invalid royalty amount")]
    InvalidRoyaltyAmount,

    #[error("Invalid requested payment")]
    InvalidRequestedPayment,

    #[error("Unknown requested payment mod hash {0}")]
    UnknownRequestedPayment(Bytes32),

    #[error("Duplicate NFT requested payment with id {0}")]
    DuplicateNftRequestedPayment(Bytes32),

    #[error("Empty bulk transfer")]
    EmptyBulkTransfer,

    #[error("Try from int error: {0}")]
    TryFromInt(#[from] TryFromIntError),
}
