use std::mem;

use chia_wallet_sdk::{
    driver::{did_puzzle_assertion, SpendContext},
    prelude::TransferNft,
};

use crate::{Action, Id, Spends, Summary, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssignNftAction {
    pub nft_id: Id,
    pub did_id: Option<Id>,
}

impl AssignNftAction {
    pub fn new(nft_id: Id, did_id: Option<Id>) -> Self {
        Self { nft_id, did_id }
    }
}

impl Action for AssignNftAction {
    fn summarize(&self, summary: &mut Summary, _index: usize) {
        summary.spent_nfts.insert(self.nft_id);

        if let Some(did_id) = self.did_id {
            summary.spent_dids.insert(did_id);
        }
    }

    fn spend(
        &self,
        _ctx: &mut SpendContext,
        spends: &mut Spends,
        _index: usize,
    ) -> Result<(), WalletError> {
        let nft = spends
            .nfts
            .get_mut(&self.nft_id)
            .ok_or(WalletError::MissingAsset)?;

        let transfer_condition = if let Some(did_id) = self.did_id {
            let did = spends
                .dids
                .get_mut(&did_id)
                .ok_or(WalletError::MissingAsset)?;

            let transfer_condition = TransferNft::new(
                Some(did.coin.info.launcher_id),
                Vec::new(),
                Some(did.coin.info.inner_puzzle_hash().into()),
            );

            did.conditions = mem::take(&mut did.conditions)
                .assert_puzzle_announcement(did_puzzle_assertion(
                    nft.coin.coin.puzzle_hash,
                    &transfer_condition,
                ))
                .create_puzzle_announcement(nft.coin.info.launcher_id.into());

            transfer_condition
        } else {
            TransferNft::new(None, Vec::new(), None)
        };

        nft.child_info = nft.child_info.with_owner(transfer_condition.did_id);
        nft.conditions = mem::take(&mut nft.conditions).with(transfer_condition);

        Ok(())
    }
}
