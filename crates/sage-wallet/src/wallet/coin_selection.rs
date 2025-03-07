use std::collections::HashMap;

use chia::protocol::{Bytes32, Coin};
use chia_wallet_sdk::{driver::Cat, utils::select_coins};

use crate::WalletError;

use super::Wallet;

impl Wallet {
    /// Selects one or more unspent p2 coins from the database.
    pub(crate) async fn select_p2_coins(&self, amount: u128) -> Result<Vec<Coin>, WalletError> {
        let spendable_coins = self.db.spendable_coins().await?;
        Ok(select_coins(spendable_coins, amount)?)
    }

    /// Selects one or more unspent CAT coins from the database.
    pub(crate) async fn select_cat_coins(
        &self,
        asset_id: Bytes32,
        amount: u128,
    ) -> Result<Vec<Cat>, WalletError> {
        let cat_coins = self.db.spendable_cat_coins(asset_id).await?;

        let mut cats = HashMap::with_capacity(cat_coins.len());
        let mut spendable_coins = Vec::with_capacity(cat_coins.len());

        for cat in &cat_coins {
            cats.insert(
                cat.coin,
                Cat {
                    coin: cat.coin,
                    lineage_proof: Some(cat.lineage_proof),
                    asset_id,
                    p2_puzzle_hash: cat.p2_puzzle_hash,
                },
            );
            spendable_coins.push(cat.coin);
        }

        Ok(select_coins(spendable_coins, amount)?
            .into_iter()
            .map(|coin| cats[&coin])
            .collect())
    }
}
