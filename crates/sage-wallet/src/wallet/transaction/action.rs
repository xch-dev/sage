mod add_nft_uri;
mod create_did;
mod issue_cat;
mod mint_nft;
mod normalize_did;
mod send;
mod transfer_did;
mod transfer_nft;

pub use add_nft_uri::*;
pub use create_did::*;
pub use issue_cat::*;
pub use mint_nft::*;
pub use normalize_did::*;
pub use send::*;
pub use transfer_did::*;
pub use transfer_nft::*;

use crate::WalletError;

use super::{lineation::Lineation, Distribution, Summary};

#[derive(Debug, Clone)]
pub enum SpendAction {
    Send(SendAction),
    IssueCat(IssueCatAction),
    MintNft(MintNftAction),
    TransferNft(TransferNftAction),
    AddNftUri(AddNftUriAction),
    CreateDid(CreateDidAction),
    TransferDid(TransferDidAction),
    NormalizeDid(NormalizeDidAction),
}

pub trait Action {
    fn summarize(&self, summary: &mut Summary, index: usize);

    fn distribute(
        &self,
        distribution: &mut Distribution<'_>,
        index: usize,
    ) -> Result<(), WalletError> {
        let _ = distribution;
        let _ = index;
        Ok(())
    }

    fn lineate(&self, lineation: &mut Lineation<'_>, index: usize) -> Result<(), WalletError> {
        let _ = lineation;
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
            SpendAction::AddNftUri(action) => action.summarize(summary, index),
            SpendAction::CreateDid(action) => action.summarize(summary, index),
            SpendAction::TransferDid(action) => action.summarize(summary, index),
            SpendAction::NormalizeDid(action) => action.summarize(summary, index),
        }
    }

    fn distribute(
        &self,
        distribution: &mut Distribution<'_>,
        index: usize,
    ) -> Result<(), WalletError> {
        match self {
            SpendAction::Send(action) => action.distribute(distribution, index),
            SpendAction::IssueCat(action) => action.distribute(distribution, index),
            SpendAction::MintNft(action) => action.distribute(distribution, index),
            SpendAction::TransferNft(action) => action.distribute(distribution, index),
            SpendAction::AddNftUri(action) => action.distribute(distribution, index),
            SpendAction::CreateDid(action) => action.distribute(distribution, index),
            SpendAction::TransferDid(action) => action.distribute(distribution, index),
            SpendAction::NormalizeDid(action) => action.distribute(distribution, index),
        }
    }

    fn lineate(&self, lineation: &mut Lineation<'_>, index: usize) -> Result<(), WalletError> {
        match self {
            SpendAction::Send(action) => action.lineate(lineation, index),
            SpendAction::IssueCat(action) => action.lineate(lineation, index),
            SpendAction::MintNft(action) => action.lineate(lineation, index),
            SpendAction::TransferNft(action) => action.lineate(lineation, index),
            SpendAction::AddNftUri(action) => action.lineate(lineation, index),
            SpendAction::CreateDid(action) => action.lineate(lineation, index),
            SpendAction::TransferDid(action) => action.lineate(lineation, index),
            SpendAction::NormalizeDid(action) => action.lineate(lineation, index),
        }
    }
}
