mod add_nft_uri;
mod assign_nft;
mod create_did;
mod issue_cat;
mod mint_nft;
mod mint_option;
mod normalize_did;
mod send;
mod transfer_did;
mod transfer_nft;
mod transfer_option;

pub use add_nft_uri::*;
pub use assign_nft::*;
pub use create_did::*;
pub use issue_cat::*;
pub use mint_nft::*;
pub use mint_option::*;
pub use normalize_did::*;
pub use send::*;
pub use transfer_did::*;
pub use transfer_nft::*;
pub use transfer_option::*;

use crate::WalletError;

use super::{Id, Spends, Summary};

use chia::protocol::{Bytes, Bytes32};
use chia_wallet_sdk::driver::SpendContext;

#[derive(Debug, Clone)]
pub enum SpendAction {
    Send(SendAction),
    IssueCat(IssueCatAction),
    MintNft(MintNftAction),
    TransferNft(TransferNftAction),
    AssignNft(AssignNftAction),
    AddNftUri(AddNftUriAction),
    CreateDid(CreateDidAction),
    TransferDid(TransferDidAction),
    NormalizeDid(NormalizeDidAction),
    MintOption(MintOptionAction),
    TransferOption(TransferOptionAction),
}

impl SpendAction {
    pub fn send_xch(puzzle_hash: Bytes32, amount: u64, memos: Option<Vec<Bytes>>) -> Self {
        Self::Send(SendAction::new(
            None,
            puzzle_hash,
            amount,
            Hint::Default,
            memos,
        ))
    }

    pub fn send_cat(
        asset_id: Bytes32,
        puzzle_hash: Bytes32,
        amount: u64,
        memos: Option<Vec<Bytes>>,
    ) -> Self {
        Self::Send(SendAction::new(
            Some(Id::Existing(asset_id)),
            puzzle_hash,
            amount,
            Hint::Default,
            memos,
        ))
    }

    pub fn send_new_cat(
        index: usize,
        puzzle_hash: Bytes32,
        amount: u64,
        memos: Option<Vec<Bytes>>,
    ) -> Self {
        Self::Send(SendAction::new(
            Some(Id::New(index)),
            puzzle_hash,
            amount,
            Hint::Default,
            memos,
        ))
    }

    pub fn issue_cat(amount: u64) -> Self {
        Self::IssueCat(IssueCatAction::new(amount))
    }

    pub fn create_did() -> Self {
        Self::CreateDid(CreateDidAction)
    }

    pub fn transfer_did(did_id: Bytes32, puzzle_hash: Bytes32) -> Self {
        Self::TransferDid(TransferDidAction::new(Id::Existing(did_id), puzzle_hash))
    }

    pub fn transfer_new_did(index: usize, puzzle_hash: Bytes32) -> Self {
        Self::TransferDid(TransferDidAction::new(Id::New(index), puzzle_hash))
    }
}

pub trait Action {
    fn summarize(&self, summary: &mut Summary, index: usize);

    fn spend(
        &self,
        ctx: &mut SpendContext,
        distribution: &mut Spends,
        index: usize,
    ) -> Result<(), WalletError> {
        let _ = ctx;
        let _ = distribution;
        let _ = index;
        Ok(())
    }
}

impl Action for SpendAction {
    fn summarize(&self, summary: &mut Summary, index: usize) {
        match self {
            SpendAction::Send(action) => action.summarize(summary, index),
            SpendAction::IssueCat(action) => action.summarize(summary, index),
            SpendAction::MintNft(action) => action.summarize(summary, index),
            SpendAction::TransferNft(action) => action.summarize(summary, index),
            SpendAction::AssignNft(action) => action.summarize(summary, index),
            SpendAction::AddNftUri(action) => action.summarize(summary, index),
            SpendAction::CreateDid(action) => action.summarize(summary, index),
            SpendAction::TransferDid(action) => action.summarize(summary, index),
            SpendAction::NormalizeDid(action) => action.summarize(summary, index),
            SpendAction::MintOption(action) => action.summarize(summary, index),
            SpendAction::TransferOption(action) => action.summarize(summary, index),
        }
    }

    fn spend(
        &self,
        ctx: &mut SpendContext,
        distribution: &mut Spends,
        index: usize,
    ) -> Result<(), WalletError> {
        match self {
            SpendAction::Send(action) => action.spend(ctx, distribution, index),
            SpendAction::IssueCat(action) => action.spend(ctx, distribution, index),
            SpendAction::MintNft(action) => action.spend(ctx, distribution, index),
            SpendAction::TransferNft(action) => action.spend(ctx, distribution, index),
            SpendAction::AssignNft(action) => action.spend(ctx, distribution, index),
            SpendAction::AddNftUri(action) => action.spend(ctx, distribution, index),
            SpendAction::CreateDid(action) => action.spend(ctx, distribution, index),
            SpendAction::TransferDid(action) => action.spend(ctx, distribution, index),
            SpendAction::NormalizeDid(action) => action.spend(ctx, distribution, index),
            SpendAction::MintOption(action) => action.spend(ctx, distribution, index),
            SpendAction::TransferOption(action) => action.spend(ctx, distribution, index),
        }
    }
}
