use chia::{protocol::Bytes32, puzzles::nft::NftMetadata};

use crate::{Action, Id, Summary};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MintNftAction {
    pub metadata: NftMetadata,
    pub royalty_puzzle_hash: Bytes32,
    pub royalty_ten_thousandths: u16,
    pub minter_did: Id,
}

impl Action for MintNftAction {
    fn summarize(&self, summary: &mut Summary, index: usize) {
        summary.created_nfts.insert(Id::New(index));
        summary.spent_dids.insert(self.minter_did);
        summary.spent_xch += 1;
    }
}
