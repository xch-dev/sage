use chia_wallet_sdk::driver::{MetadataUpdate, SpendContext};

use crate::{Action, Id, Spends, Summary, WalletError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddNftUriAction {
    pub nft_id: Id,
    pub add_uri: MetadataUpdate,
}

impl AddNftUriAction {
    pub fn new(nft_id: Id, add_uri: MetadataUpdate) -> Self {
        Self { nft_id, add_uri }
    }
}

impl Action for AddNftUriAction {
    fn summarize(&self, summary: &mut Summary, _index: usize) {
        summary.spent_nfts.insert(self.nft_id);
    }

    fn spend(
        &self,
        ctx: &mut SpendContext,
        distribution: &mut Spends,
        _index: usize,
    ) -> Result<(), WalletError> {
        Ok(())
    }
}
