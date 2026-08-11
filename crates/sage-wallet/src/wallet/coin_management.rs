use chia_wallet_sdk::prelude::*;

use crate::WalletError;

use super::Wallet;

impl Wallet {
    pub async fn combine(
        &self,
        selected_coin_ids: Vec<Bytes32>,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();

        self.spend(&mut ctx, selected_coin_ids, &[Action::fee(fee)])
            .await?;

        Ok(ctx.take())
    }

    pub async fn split(
        &self,
        selected_coin_ids: Vec<Bytes32>,
        output_count: usize,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();

        let mut spends = self
            .prepare_spends_for_selection(&mut ctx, &selected_coin_ids)
            .await?;

        let asset_id = spends
            .cats
            .values()
            .next()
            .and_then(|cat| cat.items.first().map(|item| item.asset.info.asset_id));

        let mut actions = vec![Action::fee(fee)];

        let total = if let Some(asset_id) = asset_id {
            self.select_spends(&mut ctx, &mut spends, &actions).await?;
            spends.cats[&Id::Existing(asset_id)].selected_amount()
        } else {
            let total = spends.xch.selected_amount();
            if fee > total {
                return Err(WalletError::InsufficientFunds);
            }
            total
        };

        let mut remaining_count = output_count;
        let mut remaining_amount = total - if asset_id.is_none() { fee } else { 0 };

        let max_individual_amount = remaining_amount.div_ceil(output_count as u64);
        let derivations_needed = output_count.div_ceil(selected_coin_ids.len()) as u32;

        let puzzle_hashes = if let Some(change_p2_puzzle_hash) = self.change_p2_puzzle_hash {
            [change_p2_puzzle_hash].repeat(derivations_needed as usize)
        } else {
            self.p2_puzzle_hashes(derivations_needed, false, true)
                .await?
        };

        for &puzzle_hash in &puzzle_hashes {
            for _ in 0..selected_coin_ids.len() {
                if remaining_count == 0 {
                    break;
                }

                let amount = max_individual_amount.min(remaining_amount);
                remaining_amount -= amount;
                remaining_count -= 1;

                actions.push(Action::send(
                    asset_id.map_or(Id::Xch, Id::Existing),
                    puzzle_hash,
                    amount,
                    if asset_id.is_some() {
                        ctx.hint(puzzle_hash)?
                    } else {
                        Memos::None
                    },
                ));
            }
        }

        let deltas = spends.apply(&mut ctx, &actions)?;
        self.complete_spends(&mut ctx, &deltas, spends).await?;

        Ok(ctx.take())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use test_log::test;

    use crate::TestWallet;

    #[test(tokio::test)]
    async fn test_xch_coin_management() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1000).await?;

        let coins = test.wallet.db.selectable_xch_coins().await?;
        let coin_spends = test
            .wallet
            .split(coins.iter().map(Coin::coin_id).collect(), 3, 0)
            .await?;
        assert_eq!(coin_spends.len(), 1);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.xch_balance().await?, 1000);
        assert_eq!(test.wallet.db.selectable_xch_coins().await?.len(), 3);

        let coins = test.wallet.db.selectable_xch_coins().await?;
        let coin_spends = test
            .wallet
            .combine(coins.iter().map(Coin::coin_id).collect(), 0)
            .await?;
        assert_eq!(coin_spends.len(), 3);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.xch_balance().await?, 1000);
        assert_eq!(test.wallet.db.selectable_xch_coins().await?.len(), 1);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_cat_coin_management() -> anyhow::Result<()> {
        let mut test = TestWallet::new(100).await?;

        let (coin_spends, asset_id) = test.wallet.issue_cat(100, 0, None).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let mut cats = test.wallet.db.selectable_cat_coins(asset_id).await?;
        assert_eq!(test.wallet.db.cat_balance(asset_id).await?, 100);
        assert_eq!(cats.len(), 1);

        let cat = cats.remove(0);
        let coin_spends = test.wallet.split(vec![cat.coin.coin_id()], 2, 0).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let cats = test.wallet.db.selectable_cat_coins(asset_id).await?;
        assert_eq!(test.wallet.db.cat_balance(asset_id).await?, 100);
        assert_eq!(cats.len(), 2);

        let coin_spends = test
            .wallet
            .combine(cats.iter().map(|cat| cat.coin.coin_id()).collect(), 0)
            .await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        assert_eq!(test.wallet.db.cat_balance(asset_id).await?, 100);
        assert_eq!(
            test.wallet.db.selectable_cat_coins(asset_id).await?.len(),
            1
        );

        Ok(())
    }
}
