use chia_wallet_sdk::prelude::*;
use sage_database::DatabaseError;
use thiserror::Error;

use crate::{HandleError, WalletError, XchandlesApiError};

#[derive(Debug, Error)]
pub enum XchandlesError {
    #[error("Invalid handle: {0}")]
    InvalidHandle(#[from] HandleError),

    #[error("{0}")]
    Api(#[from] XchandlesApiError),

    #[error("Wallet error: {0}")]
    Wallet(#[from] WalletError),

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("XCHandles is not supported on this network")]
    UnsupportedNetwork,

    #[error("No peers are available to verify the handle")]
    NoPeers,

    #[error("Timed out while verifying the handle")]
    Timeout,

    #[error("The API response is for a different registry or handle")]
    ResponseMismatch,

    #[error("The handle slot does not descend from the trusted registry")]
    InvalidLineage,

    #[error("The handle slot {0} is not confirmed on chain")]
    SlotNotFound(Bytes32),

    #[error("The handle slot {0} has already been spent")]
    SlotSpent(Bytes32),

    #[error("Handle expired at {expiration}, before the timestamp {timestamp}")]
    Expired { expiration: u64, timestamp: u64 },

    #[error("The API did not return the singleton the handle resolves to")]
    ResolutionUnavailable,

    #[error("The resolved singleton {0} has been melted")]
    ResolvedSingletonMelted(Bytes32),

    #[error("The resolved singleton coin {0} is not confirmed on chain")]
    ResolvedSingletonNotFound(Bytes32),

    #[error("The resolved singleton coin {0} is not an NFT or DID for the handle")]
    InvalidResolvedSingleton(Bytes32),

    #[error("The resolved singleton {0} is in a clawback and has no settled owner")]
    ResolvedSingletonInClawback(Bytes32),

    #[error(
        "The resolved singleton {0} hasn't been transferred since it was launched, so its \
        owner can't be verified"
    )]
    ResolvedSingletonUnspentEve(Bytes32),
}

impl XchandlesError {
    /// Whether the handle may have been valid, but couldn't be verified because the API,
    /// the peers, or the API's view of the chain was unavailable or out of date. These are
    /// the only failures that may fall back to a previously verified resolution, and they
    /// leave the stored mapping untouched.
    pub fn is_unavailable(&self) -> bool {
        match self {
            Self::Api(error) => error.is_unavailable(),
            Self::Wallet(..)
            | Self::NoPeers
            | Self::Timeout
            | Self::SlotNotFound(..)
            | Self::SlotSpent(..)
            | Self::ResolutionUnavailable
            | Self::ResolvedSingletonNotFound(..) => true,
            Self::InvalidHandle(..)
            | Self::Database(..)
            | Self::UnsupportedNetwork
            | Self::ResponseMismatch
            | Self::InvalidLineage
            | Self::Expired { .. }
            | Self::ResolvedSingletonMelted(..)
            | Self::InvalidResolvedSingleton(..)
            | Self::ResolvedSingletonInClawback(..)
            | Self::ResolvedSingletonUnspentEve(..) => false,
        }
    }

    /// Whether the lookup produced an answer that was rejected, or showed that the handle
    /// is no longer registered. The stored mapping is marked unverified when this happens.
    pub fn is_rejection(&self) -> bool {
        match self {
            Self::Api(error) => error.is_not_registered(),
            Self::ResponseMismatch
            | Self::InvalidLineage
            | Self::Expired { .. }
            | Self::ResolvedSingletonMelted(..)
            | Self::InvalidResolvedSingleton(..)
            | Self::ResolvedSingletonInClawback(..) => true,
            Self::InvalidHandle(..)
            | Self::Wallet(..)
            | Self::Database(..)
            | Self::UnsupportedNetwork
            | Self::NoPeers
            | Self::Timeout
            | Self::SlotNotFound(..)
            | Self::SlotSpent(..)
            | Self::ResolutionUnavailable
            | Self::ResolvedSingletonNotFound(..)
            | Self::ResolvedSingletonUnspentEve(..) => false,
        }
    }

    /// Whether the handle has no live registration, as opposed to a failed verification.
    pub fn is_not_registered(&self) -> bool {
        match self {
            Self::Api(error) => error.is_not_registered(),
            Self::Expired { .. } => true,
            _ => false,
        }
    }
}
