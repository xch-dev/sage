mod add_nft_uri;
mod assign_nft;
mod create_did;
mod issue_cat;
mod normalize_did;
mod send;
mod take_offer;
mod transfer_did;
mod transfer_nft;

pub use add_nft_uri::*;
pub use assign_nft::*;
pub use create_did::*;
pub use issue_cat::*;
pub use normalize_did::*;
pub use send::*;
pub use take_offer::*;
pub use transfer_did::*;
pub use transfer_nft::*;

use crate::WalletError;

use super::{Select, Selection};

#[derive(Debug, Clone)]
pub enum Action {
    Send(SendAction),
    IssueCat(IssueCatAction),
    CreateDid(CreateDidAction),
    TransferNft(TransferNftAction),
    AssignNft(AssignNftAction),
    AddNftUri(AddNftUriAction),
    TransferDid(TransferDidAction),
    NormalizeDid(NormalizeDidAction),
    TakeOffer(TakeOfferAction),
}

impl Select for Action {
    fn select(&self, selection: &mut Selection, index: usize) -> Result<(), WalletError> {
        match self {
            Action::Send(action) => action.select(selection, index),
            Action::IssueCat(action) => action.select(selection, index),
            Action::CreateDid(action) => action.select(selection, index),
            Action::TransferNft(action) => action.select(selection, index),
            Action::AssignNft(action) => action.select(selection, index),
            Action::AddNftUri(action) => action.select(selection, index),
            Action::TransferDid(action) => action.select(selection, index),
            Action::NormalizeDid(action) => action.select(selection, index),
            Action::TakeOffer(action) => action.select(selection, index),
        }
    }
}
