use std::mem;

use chia::clvm_traits::clvm_list;
use chia_wallet_sdk::{
    driver::{HashedPtr, MetadataUpdate, SpendContext},
    prelude::NewMetadataOutput,
};
use clvmr::NodePtr;

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
        spends: &mut Spends,
        _index: usize,
    ) -> Result<(), WalletError> {
        let nft = spends
            .nfts
            .get_mut(&self.nft_id)
            .ok_or(WalletError::MissingAsset)?;

        let metadata_update = self.add_uri.spend(ctx)?;

        let metadata_updater_solution = ctx.alloc(&clvm_list!(
            nft.coin.info.metadata,
            nft.coin.info.metadata_updater_puzzle_hash,
            metadata_update.solution
        ))?;
        let ptr = ctx.run(metadata_update.puzzle, metadata_updater_solution)?;
        let output = ctx.extract::<NewMetadataOutput<HashedPtr, NodePtr>>(ptr)?;

        nft.child_info = nft
            .child_info
            .with_metadata(output.metadata_info.new_metadata);
        nft.child_info.metadata_updater_puzzle_hash = output.metadata_info.new_updater_puzzle_hash;
        nft.conditions = mem::take(&mut nft.conditions)
            .update_nft_metadata(metadata_update.puzzle, metadata_update.solution);

        Ok(())
    }
}
