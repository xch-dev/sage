use chia_wallet_sdk::driver::{HashedPtr, SpendContext};

use crate::{Action, Id, SingletonLineage, Spends, Summary, WalletError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreateDidAction;

impl Action for CreateDidAction {
    fn summarize(&self, summary: &mut Summary, index: usize) {
        summary.created_dids.insert(Id::New(index));
        summary.spent_xch += 1;
    }

    fn spend(
        &self,
        ctx: &mut SpendContext,
        spends: &mut Spends,
        index: usize,
    ) -> Result<(), WalletError> {
        let (item_ref, launcher) = spends.xch.create_launcher(ctx)?;
        let item = spends.xch.get_mut(item_ref)?;

        let (create_did, did) = launcher.create_eve_did(ctx, item.coin.puzzle_hash, None, 1, ())?;
        let did = did.with_metadata(HashedPtr::NIL);

        let p2 = item
            .p2
            .as_standard_mut()
            .ok_or(WalletError::P2Unsupported)?;

        p2.add_conditions(create_did);

        spends.dids.insert(
            Id::New(index),
            SingletonLineage::new(did, item.p2.cleared(), true, true),
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{SpendAction, TestWallet};

    use itertools::Itertools;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_action_create_multiple_dids() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(2).await?;
        let mut bob = alice.next(0).await?;

        let result = alice
            .wallet
            .transact(
                vec![SpendAction::create_did(), SpendAction::create_did()],
                0,
            )
            .await?;

        assert_eq!(result.coin_spends.len(), 5);

        let did_ids = result.ids.into_values().collect_vec();

        alice.transact(result.coin_spends).await?;
        alice.wait_for_coins().await;

        let coin_spends = alice
            .wallet
            .transact(
                vec![
                    SpendAction::transfer_did(did_ids[0], bob.puzzle_hash),
                    SpendAction::transfer_did(did_ids[1], bob.puzzle_hash),
                ],
                0,
            )
            .await?
            .coin_spends;

        assert_eq!(coin_spends.len(), 2);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        assert!(alice.wallet.db.spendable_did(did_ids[0]).await?.is_none());
        assert!(alice.wallet.db.spendable_did(did_ids[1]).await?.is_none());
        assert!(bob.wallet.db.spendable_did(did_ids[0]).await?.is_some());
        assert!(bob.wallet.db.spendable_did(did_ids[1]).await?.is_some());

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_action_create_did_and_transfer() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1).await?;
        let mut bob = alice.next(0).await?;

        let result = alice
            .wallet
            .transact(
                vec![
                    SpendAction::create_did(),
                    SpendAction::transfer_new_did(0, bob.puzzle_hash),
                ],
                0,
            )
            .await?;

        assert_eq!(result.coin_spends.len(), 3);

        let did_id = result.ids.into_values().next().expect("no did id");

        alice.transact(result.coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        assert!(alice.wallet.db.spendable_did(did_id).await?.is_none());
        assert!(bob.wallet.db.spendable_did(did_id).await?.is_some());

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_action_create_did_without_transfer() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1).await?;

        let result = alice
            .wallet
            .transact(vec![SpendAction::create_did()], 0)
            .await?;

        assert_eq!(result.coin_spends.len(), 3);

        let did_id = result.ids.into_values().next().expect("no did id");

        alice.transact(result.coin_spends).await?;
        alice.wait_for_coins().await;

        assert!(alice.wallet.db.spendable_did(did_id).await?.is_some());

        Ok(())
    }
}
