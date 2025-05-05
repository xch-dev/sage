use chia::protocol::Bytes32;
use chia_wallet_sdk::driver::SpendContext;

use crate::{Action, Id, Spends, Summary, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransferOptionAction {
    pub option_id: Id,
    pub puzzle_hash: Bytes32,
}

impl TransferOptionAction {
    pub fn new(option_id: Id, puzzle_hash: Bytes32) -> Self {
        Self {
            option_id,
            puzzle_hash,
        }
    }
}

impl Action for TransferOptionAction {
    fn summarize(&self, summary: &mut Summary, _index: usize) {
        summary.spent_options.insert(self.option_id);
    }

    fn spend(
        &self,
        _ctx: &mut SpendContext,
        spends: &mut Spends,
        _index: usize,
    ) -> Result<(), WalletError> {
        let item = spends
            .options
            .get_mut(&self.option_id)
            .ok_or(WalletError::MissingAsset)?;

        item.set_p2_puzzle_hash(self.puzzle_hash);

        Ok(())
    }
}
