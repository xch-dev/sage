use chia_wallet_sdk::prelude::*;
use sage_database::{SerializePrimitive, SerializedDid};

use crate::{
    wallet::memos::{calculate_memos, Hint},
    WalletError,
};

use super::Wallet;

impl Wallet {
    pub async fn create_did(
        &self,
        fee: u64,
    ) -> Result<(Vec<CoinSpend>, SerializedDid), WalletError> {
        let mut ctx = SpendContext::new();

        let outputs = self
            .spend(
                &mut ctx,
                vec![],
                &[Action::fee(fee), Action::create_empty_did()],
            )
            .await?;

        let did = outputs.dids[&Id::New(1)].serialize(&ctx)?;

        Ok((ctx.take(), did))
    }

    pub async fn transfer_dids(
        &self,
        did_ids: Vec<Bytes32>,
        puzzle_hash: Bytes32,
        fee: u64,
        clawback: Option<u64>,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let sender_puzzle_hash = self.change_p2_puzzle_hash().await?;

        let mut ctx = SpendContext::new();
        let mut actions = vec![Action::fee(fee)];

        for did_id in did_ids {
            let amount = self
                .db
                .did(did_id)
                .await?
                .ok_or(WalletError::MissingDid(did_id))?
                .coin
                .amount;

            let clawback = clawback.map(|seconds| {
                ClawbackV2::new(sender_puzzle_hash, puzzle_hash, seconds, amount, true)
            });

            let memos = calculate_memos(
                &mut ctx,
                if let Some(clawback) = clawback {
                    Hint::Clawback(clawback)
                } else {
                    Hint::P2PuzzleHash(puzzle_hash)
                },
                vec![],
            )?;

            let p2_puzzle_hash = if let Some(clawback) = clawback {
                clawback.tree_hash().into()
            } else {
                puzzle_hash
            };

            actions.push(Action::send(
                Id::Existing(did_id),
                p2_puzzle_hash,
                amount,
                memos,
            ));
        }

        self.spend(&mut ctx, vec![], &actions).await?;

        Ok(ctx.take())
    }

    pub async fn normalize_dids(
        &self,
        did_ids: Vec<Bytes32>,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();
        let mut actions = vec![Action::fee(fee)];

        for did_id in did_ids {
            actions.push(Action::update_did(
                Id::Existing(did_id),
                Some(Some(Bytes32::from(tree_hash_atom(&[])))),
                Some(1),
                None,
            ));
        }

        self.spend(&mut ctx, vec![], &actions).await?;

        Ok(ctx.take())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::TestWallet;

    use test_log::test;
    use tokio::time::sleep;

    #[test(tokio::test)]
    async fn test_create_did() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1).await?;

        let (coin_spends, did) = test.wallet.create_did(0).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        for _ in 0..2 {
            let coin_spends = test
                .wallet
                .transfer_dids(vec![did.info.launcher_id], test.puzzle_hash, 0, None)
                .await?;
            test.transact(coin_spends).await?;

            test.wait_for_coins().await;

            let coin_spends = test
                .wallet
                .normalize_dids(vec![did.info.launcher_id], 0)
                .await?;
            test.transact(coin_spends).await?;

            test.wait_for_coins().await;
        }

        assert_ne!(test.wallet.db.did(did.info.launcher_id).await?, None);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_transfer_did_external() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, did) = alice.wallet.create_did(0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let did_id = did.info.launcher_id;

        let coin_spends = alice
            .wallet
            .transfer_dids(vec![did_id], bob.puzzle_hash, 0, None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        alice.transact(coin_spends).await?;

        alice.wait_for_coins().await;

        assert!(alice.wallet.db.did(did_id).await?.is_none());
        assert!(alice.wallet.db.spendable_did(did_id).await?.is_none());

        bob.wait_for_puzzles().await;

        assert!(bob.wallet.db.did(did_id).await?.is_some());
        assert!(bob.wallet.db.spendable_did(did_id).await?.is_some());

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_transfer_did_with_clawback_self() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;

        let (coin_spends, did) = test.wallet.create_did(0).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let did_id = did.info.launcher_id;

        let timestamp = test.new_block_with_current_time().await?;

        let coin_spends = test
            .wallet
            .transfer_dids(vec![did_id], test.puzzle_hash, 0, Some(timestamp + 5))
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert!(test.wallet.db.spendable_did(did_id).await?.is_some());

        sleep(Duration::from_secs(6)).await;
        test.new_block_with_current_time().await?;

        assert!(test.wallet.db.spendable_did(did_id).await?.is_some());

        let coin_spends = test
            .wallet
            .transfer_dids(vec![did_id], test.puzzle_hash, 0, None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert!(test.wallet.db.spendable_did(did_id).await?.is_some());

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_transfer_did_with_clawback_external() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, did) = alice.wallet.create_did(0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let did_id = did.info.launcher_id;

        let timestamp = alice.new_block_with_current_time().await?;

        let coin_spends = alice
            .wallet
            .transfer_dids(vec![did_id], bob.puzzle_hash, 0, Some(timestamp + 5))
            .await?;

        assert_eq!(coin_spends.len(), 1);

        alice.transact(coin_spends).await?;

        alice.wait_for_coins().await;

        assert!(alice.wallet.db.spendable_did(did_id).await?.is_some());
        assert!(bob.wallet.db.spendable_did(did_id).await?.is_none());

        bob.wait_for_puzzles().await;

        assert!(alice.wallet.db.spendable_did(did_id).await?.is_some());
        assert!(bob.wallet.db.spendable_did(did_id).await?.is_none());

        sleep(Duration::from_secs(6)).await;
        bob.new_block_with_current_time().await?;

        assert!(alice.wallet.db.spendable_did(did_id).await?.is_none());
        assert!(bob.wallet.db.spendable_did(did_id).await?.is_some());

        let coin_spends = bob
            .wallet
            .transfer_dids(vec![did_id], alice.puzzle_hash, 0, None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;
        alice.wait_for_puzzles().await;

        assert!(alice.wallet.db.spendable_did(did_id).await?.is_some());
        assert!(bob.wallet.db.spendable_did(did_id).await?.is_none());

        Ok(())
    }
}
