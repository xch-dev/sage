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

use chia::{
    protocol::{Bytes, Bytes32},
    puzzles::offer::Payment,
};
use chia_puzzles::SETTLEMENT_PAYMENT_HASH;
use chia_wallet_sdk::{driver::SpendContext, prelude::TradePrice};

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
            None,
        ))
    }

    pub fn fulfill_xch_payment(nonce: Bytes32, payment: Payment) -> Self {
        Self::Send(SendAction::new(
            None,
            payment.puzzle_hash,
            payment.amount,
            Hint::No,
            payment.memos.map(|memos| memos.0),
            Some(nonce),
        ))
    }

    pub fn offer_xch(amount: u64) -> Self {
        Self::send_xch(SETTLEMENT_PAYMENT_HASH.into(), amount, None)
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
            None,
        ))
    }

    pub fn fulfill_cat_payment(nonce: Bytes32, asset_id: Bytes32, payment: Payment) -> Self {
        Self::Send(SendAction::new(
            Some(Id::Existing(asset_id)),
            payment.puzzle_hash,
            payment.amount,
            Hint::No,
            payment.memos.map(|memos| memos.0),
            Some(nonce),
        ))
    }

    pub fn offer_cat(asset_id: Bytes32, amount: u64) -> Self {
        Self::send_cat(asset_id, SETTLEMENT_PAYMENT_HASH.into(), amount, None)
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
            None,
        ))
    }

    pub fn offer_new_cat(index: usize, amount: u64) -> Self {
        Self::send_new_cat(index, SETTLEMENT_PAYMENT_HASH.into(), amount, None)
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

    pub fn transfer_nft(nft_id: Bytes32, puzzle_hash: Bytes32) -> Self {
        Self::TransferNft(TransferNftAction::new(
            Id::Existing(nft_id),
            puzzle_hash,
            Hint::Default,
            None,
            None,
        ))
    }

    pub fn offer_nft(nft_id: Bytes32, trade_prices: Vec<TradePrice>) -> [Self; 2] {
        [
            Self::AssignNft(AssignNftAction::new(
                Id::Existing(nft_id),
                None,
                trade_prices,
            )),
            Self::TransferNft(TransferNftAction::new(
                Id::Existing(nft_id),
                SETTLEMENT_PAYMENT_HASH.into(),
                Hint::Default,
                None,
                None,
            )),
        ]
    }

    pub fn fulfill_nft_payment(nft_id: Bytes32, nonce: Bytes32, payment: Payment) -> Self {
        Self::TransferNft(TransferNftAction::new(
            Id::Existing(nft_id),
            payment.puzzle_hash,
            Hint::No,
            payment.memos.map(|memos| memos.0),
            Some(nonce),
        ))
    }

    pub fn transfer_new_nft(index: usize, puzzle_hash: Bytes32) -> Self {
        Self::TransferNft(TransferNftAction::new(
            Id::New(index),
            puzzle_hash,
            Hint::Default,
            None,
            None,
        ))
    }

    pub fn offer_new_nft(index: usize, trade_prices: Vec<TradePrice>) -> [Self; 2] {
        [
            Self::AssignNft(AssignNftAction::new(Id::New(index), None, trade_prices)),
            Self::TransferNft(TransferNftAction::new(
                Id::New(index),
                SETTLEMENT_PAYMENT_HASH.into(),
                Hint::Default,
                None,
                None,
            )),
        ]
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
