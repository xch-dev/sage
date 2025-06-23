use std::{num::TryFromIntError, time::SystemTimeError};

use chia::{
    clvm_traits::{FromClvmError, ToClvmError},
    protocol::Bytes32,
};
use chia_wallet_sdk::{
    client::ClientError, driver::DriverError, signer::SignerError, utils::CoinSelectionError,
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

    #[error("Coin selection error: {0}")]
    CoinSelection(#[from] CoinSelectionError),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Timeout exceeded")]
    Elapsed(#[from] Elapsed),

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

    #[error("Missing XCH coin with id {0}")]
    MissingXchCoin(Bytes32),

    #[error("Missing CAT coin with id {0}")]
    MissingCatCoin(Bytes32),

    #[error("Missing DID coin with id {0}")]
    MissingDidCoin(Bytes32),

    #[error("Missing NFT coin with id {0}")]
    MissingNftCoin(Bytes32),

    #[error("Missing coin with id {0}")]
    MissingCoin(Bytes32),

    #[error("Missing option contract coin with id {0}")]
    MissingOptionCoin(Bytes32),

    #[error("Missing DID with id {0}. It may have been spent recently. Please try again later.")]
    MissingDid(Bytes32),

    #[error("Missing NFT with id {0}. It may have been spent recently. Please try again later.")]
    MissingNft(Bytes32),

    #[error("Missing option contract with id {0}. It may have been spent recently. Please try again later.")]
    MissingOptionContract(Bytes32),

    #[error("Missing asset with id {0}")]
    MissingAsset(Bytes32),

    #[error("Uncancellable offer")]
    UncancellableOffer,

    #[error("Try from int error: {0}")]
    TryFromInt(#[from] TryFromIntError),
}
