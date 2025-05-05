use chia_wallet_sdk::driver::{DidOwner, SpendContext};

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
        ctx: &mut SpendContext,
        spends: &mut Spends,
        _index: usize,
    ) -> Result<(), WalletError> {
        let nft_lineage = spends
            .nfts
            .get_mut(&self.nft_id)
            .ok_or(WalletError::MissingAsset)?;

        let nft = nft_lineage.coin();

        let owner = if let Some(did_id) = self.did_id {
            let did_lineage = spends
                .dids
                .get_mut(&did_id)
                .ok_or(WalletError::MissingAsset)?;

            let did = did_lineage.coin();

            did_lineage.authorize_nft_ownership(nft.coin.puzzle_hash, nft.info.launcher_id);

            Some(DidOwner::new(
                did.info.launcher_id,
                did.info.inner_puzzle_hash().into(),
            ))
        } else {
            None
        };

        nft_lineage.set_did_owner(ctx, owner)?;

        Ok(())
    }
}
