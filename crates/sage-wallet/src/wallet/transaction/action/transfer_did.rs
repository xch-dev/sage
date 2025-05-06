use chia::protocol::Bytes32;
use chia_wallet_sdk::driver::SpendContext;

use crate::{Action, Id, Spends, Summary, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransferDidAction {
    pub did_id: Id,
    pub puzzle_hash: Bytes32,
}

impl TransferDidAction {
    pub fn new(did_id: Id, puzzle_hash: Bytes32) -> Self {
        Self {
            did_id,
            puzzle_hash,
        }
    }
}

impl Action for TransferDidAction {
    fn summarize(&self, summary: &mut Summary, _index: usize) {
        summary.spent_dids.insert(self.did_id);
    }

    fn spend(
        &self,
        _ctx: &mut SpendContext,
        spends: &mut Spends,
        _index: usize,
    ) -> Result<(), WalletError> {
        let item = spends
            .dids
            .get_mut(&self.did_id)
            .ok_or(WalletError::MissingAsset)?;

        item.set_p2_puzzle_hash(self.puzzle_hash);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{SpendAction, TestWallet};

    use test_log::test;

    #[test(tokio::test)]
    async fn test_action_transfer_did() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, did) = alice.wallet.create_did(0).await?;
        let did_id = did.info.launcher_id;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let coin_spends = alice
            .wallet
            .transact(vec![SpendAction::transfer_did(did_id, bob.puzzle_hash)], 0)
            .await?
            .coin_spends;

        assert_eq!(coin_spends.len(), 1);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        assert!(alice.wallet.db.spendable_did(did_id).await?.is_none());
        assert!(bob.wallet.db.spendable_did(did_id).await?.is_some());

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_action_transfer_multiple_dids() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(2).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, did) = alice.wallet.create_did(0).await?;
        let did_one = did.info.launcher_id;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, did) = alice.wallet.create_did(0).await?;
        let did_two = did.info.launcher_id;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let coin_spends = alice
            .wallet
            .transact(
                vec![
                    SpendAction::transfer_did(did_one, bob.puzzle_hash),
                    SpendAction::transfer_did(did_two, bob.puzzle_hash),
                ],
                0,
            )
            .await?
            .coin_spends;

        assert_eq!(coin_spends.len(), 2);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        assert!(alice.wallet.db.spendable_did(did_one).await?.is_none());
        assert!(alice.wallet.db.spendable_did(did_two).await?.is_none());
        assert!(bob.wallet.db.spendable_did(did_one).await?.is_some());
        assert!(bob.wallet.db.spendable_did(did_two).await?.is_some());

        Ok(())
    }
}
