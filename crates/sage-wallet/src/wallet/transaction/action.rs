mod add_nft_uri;
mod assign_nft;
mod create_did;
mod issue_cat;
mod mint_nft;
mod normalize_did;
mod send;
mod transfer_did;
mod transfer_nft;

pub use add_nft_uri::*;
pub use assign_nft::*;
pub use create_did::*;
pub use issue_cat::*;
pub use mint_nft::*;
pub use normalize_did::*;
pub use send::*;
pub use transfer_did::*;
pub use transfer_nft::*;

use chia::protocol::Coin;
use chia_wallet_sdk::driver::Cat;

use crate::WalletError;

use super::{Distribution, Preselection};

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
}

pub trait Action {
    fn preselect(&self, preselection: &mut Preselection, index: usize);

    fn distribute_xch(&self, distribution: &mut Distribution<'_, Coin>) -> Result<(), WalletError> {
        let _ = distribution;
        Ok(())
    }

    fn distribute_cat(&self, distribution: &mut Distribution<'_, Cat>) -> Result<(), WalletError> {
        let _ = distribution;
        Ok(())
    }
}

impl Action for SpendAction {
    fn preselect(&self, preselection: &mut Preselection, index: usize) {
        match self {
            SpendAction::Send(action) => action.preselect(preselection, index),
            SpendAction::IssueCat(action) => action.preselect(preselection, index),
            SpendAction::MintNft(action) => action.preselect(preselection, index),
            SpendAction::TransferNft(action) => action.preselect(preselection, index),
            SpendAction::AssignNft(action) => action.preselect(preselection, index),
            SpendAction::AddNftUri(action) => action.preselect(preselection, index),
            SpendAction::CreateDid(action) => action.preselect(preselection, index),
            SpendAction::TransferDid(action) => action.preselect(preselection, index),
            SpendAction::NormalizeDid(action) => action.preselect(preselection, index),
        }
    }

    fn distribute_xch(&self, distribution: &mut Distribution<'_, Coin>) -> Result<(), WalletError> {
        match self {
            SpendAction::Send(action) => action.distribute_xch(distribution),
            SpendAction::IssueCat(action) => action.distribute_xch(distribution),
            SpendAction::MintNft(action) => action.distribute_xch(distribution),
            SpendAction::TransferNft(action) => action.distribute_xch(distribution),
            SpendAction::AssignNft(action) => action.distribute_xch(distribution),
            SpendAction::AddNftUri(action) => action.distribute_xch(distribution),
            SpendAction::CreateDid(action) => action.distribute_xch(distribution),
            SpendAction::TransferDid(action) => action.distribute_xch(distribution),
            SpendAction::NormalizeDid(action) => action.distribute_xch(distribution),
        }
    }
}
