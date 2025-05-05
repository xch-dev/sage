use chia::protocol::Bytes32;
use chia_wallet_sdk::driver::SpendContext;

use crate::{Action, AssetCoin, AssetSpend, Id, Spends, Summary, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransferNftAction {
    pub nft_id: Id,
    pub puzzle_hash: Bytes32,
}

impl TransferNftAction {
    pub fn new(nft_id: Id, puzzle_hash: Bytes32) -> Self {
        Self {
            nft_id,
            puzzle_hash,
        }
    }
}

impl Action for TransferNftAction {
    fn summarize(&self, summary: &mut Summary, _index: usize) {
        summary.spent_nfts.insert(self.nft_id);
    }

    fn spend(
        &self,
        ctx: &mut SpendContext,
        spends: &mut Spends,
        _index: usize,
    ) -> Result<(), WalletError> {
        let item = spends
            .nfts
            .get_mut(&self.nft_id)
            .ok_or(WalletError::MissingAsset)?;

        let (spend, nft) = item.nft()?;

        let new_nft = nft.transfer(ctx, &spend.p2, self.puzzle_hash, spend.conditions.clone())?;

        *spend = AssetSpend::new(AssetCoin::Nft(new_nft), spend.p2);

        Ok(())
    }
}
