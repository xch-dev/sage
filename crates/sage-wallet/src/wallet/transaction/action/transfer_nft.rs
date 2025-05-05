use chia::protocol::Bytes32;
use chia_wallet_sdk::driver::SpendContext;

use crate::{Action, Id, SingletonCoinExt, Spends, Summary, WalletError};

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

        if item.coin.info.p2_puzzle_hash != item.child_info.p2_puzzle_hash {
            return Err(WalletError::P2PuzzleHashChange);
        }

        let _ = item
            .coin
            .transfer(ctx, &item.p2, self.puzzle_hash, item.conditions.clone())?;

        *item = item.child_with(
            item.coin
                .child_with_info(item.child_info.with_p2_puzzle_hash(self.puzzle_hash)),
        );

        Ok(())
    }
}
