use std::mem;

use chia::{
    bls::PublicKey,
    protocol::{Bytes, Bytes32, Coin, CoinSpend},
};
use chia_wallet_sdk::{select_coins, Conditions, SpendContext, StandardLayer};
use sage_database::Database;

use crate::WalletError;

#[derive(Debug)]
pub struct Wallet {
    pub db: Database,
    pub fingerprint: u32,
    pub intermediate_pk: PublicKey,
    pub genesis_challenge: Bytes32,
}

impl Wallet {
    pub fn new(
        db: Database,
        fingerprint: u32,
        intermediate_pk: PublicKey,
        genesis_challenge: Bytes32,
    ) -> Self {
        Self {
            db,
            fingerprint,
            intermediate_pk,
            genesis_challenge,
        }
    }

    pub async fn unused_puzzle_hash(&self) -> Result<Bytes32, WalletError> {
        let mut tx = self.db.tx().await?;

        let next_index = tx.derivation_index(false).await?;

        if next_index == 0 {
            return Err(WalletError::NoDerivations);
        }

        let max = next_index - 1;
        let max_used = tx.max_used_derivation_index(false).await?;
        let mut index = max_used.map_or(0, |i| i + 1);
        if index > max {
            index = max;
        }
        let puzzle_hash = tx.p2_puzzle_hash(index, false).await?;

        Ok(puzzle_hash)
    }

    pub async fn select_p2_coins(&self, amount: u128) -> Result<Vec<Coin>, WalletError> {
        let spendable_coins = self.db.unspent_p2_coins().await?;
        Ok(select_coins(spendable_coins, amount)?)
    }

    pub async fn spend_p2_coins(
        &self,
        coins: Vec<Coin>,
        mut conditions: Conditions,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();
        let mut tx = self.db.tx().await?;

        let first_coin_id = coins[0].coin_id();

        for (i, coin) in coins.into_iter().enumerate() {
            let synthetic_key = tx.synthetic_key(coin.puzzle_hash).await?;
            let p2 = StandardLayer::new(synthetic_key);

            let conditions = if i == 0 {
                mem::take(&mut conditions)
            } else {
                Conditions::new().assert_concurrent_spend(first_coin_id)
            };

            p2.spend(&mut ctx, coin, conditions)?;
        }

        Ok(ctx.take())
    }

    pub async fn send_xch(
        &self,
        puzzle_hash: Bytes32,
        amount: u64,
        fee: u64,
        memos: Vec<Bytes>,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let total = amount as u128 + fee as u128;
        let coins = self.select_p2_coins(total).await?;
        let selected: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        let change_puzzle_hash = self.unused_puzzle_hash().await?;

        let change: u64 = (selected - total)
            .try_into()
            .expect("change amount overflow");

        let mut conditions = Conditions::new().create_coin(puzzle_hash, amount, memos);

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(change_puzzle_hash, change, Vec::new());
        }

        self.spend_p2_coins(coins, conditions).await
    }

    pub async fn combine_xch(
        &self,
        coins: Vec<Coin>,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let total: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        if fee as u128 > total {
            return Err(WalletError::InsufficientFunds);
        }

        let change_puzzle_hash = self.unused_puzzle_hash().await?;

        let change: u64 = (total - fee as u128)
            .try_into()
            .expect("change amount overflow");

        let mut conditions = Conditions::new();

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(change_puzzle_hash, change, Vec::new());
        }

        self.spend_p2_coins(coins, conditions).await
    }
}
