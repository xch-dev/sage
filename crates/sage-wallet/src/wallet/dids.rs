use chia::{
    clvm_utils::tree_hash_atom,
    protocol::{Bytes32, CoinSpend, Program},
};
use chia_wallet_sdk::driver::{Action, Did, Id, SpendContext};

use crate::WalletError;

use super::Wallet;

impl Wallet {
    pub async fn create_did(
        &self,
        fee: u64,
    ) -> Result<(Vec<CoinSpend>, Did<Program>), WalletError> {
        let mut ctx = SpendContext::new();

        let outputs = self
            .spend(
                &mut ctx,
                vec![],
                &[Action::fee(fee), Action::create_empty_did()],
            )
            .await?;

        let did = outputs.dids[&Id::New(1)];
        let metadata = ctx.serialize(&did.info.metadata)?;

        Ok((ctx.take(), did.with_metadata(metadata)))
    }

    pub async fn transfer_dids(
        &self,
        did_ids: Vec<Bytes32>,
        puzzle_hash: Bytes32,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();
        let mut actions = vec![Action::fee(fee)];

        for did_id in did_ids {
            let hint = ctx.hint(puzzle_hash)?;
            actions.push(Action::send(Id::Existing(did_id), puzzle_hash, 1, hint));
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
    use crate::TestWallet;

    use test_log::test;

    #[test(tokio::test)]
    async fn test_create_did() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1).await?;

        let (coin_spends, did) = test.wallet.create_did(0).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        for _ in 0..2 {
            let coin_spends = test
                .wallet
                .transfer_dids(vec![did.info.launcher_id], test.puzzle_hash, 0)
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

        assert_ne!(
            test.wallet.db.spendable_did(did.info.launcher_id).await?,
            None
        );

        Ok(())
    }
}
