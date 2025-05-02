use chia::{protocol::Bytes32, puzzles::nft::NftMetadata};

use crate::{Action, Id, Preselection};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MintNftAction {
    pub metadata: NftMetadata,
    pub royalty_puzzle_hash: Bytes32,
    pub royalty_ten_thousandths: u16,
    pub minter_did: Id,
}

impl Action for MintNftAction {
    fn preselect(&self, preselection: &mut Preselection, index: usize) {
        preselection.created_nfts.insert(Id::New(index));
        preselection.spent_dids.insert(self.minter_did);
        preselection.spent_xch += 1;
    }
}
