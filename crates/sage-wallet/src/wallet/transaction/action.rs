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

use super::Preselection;

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
}
