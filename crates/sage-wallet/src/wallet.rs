use std::{collections::HashSet, mem, ops::Range};

use chia::{
    bls::{DerivableKey, PublicKey},
    protocol::{Bytes, Bytes32, Coin, CoinSpend},
    puzzles::{standard::StandardArgs, DeriveSynthetic},
};
use chia_wallet_sdk::{select_coins, Cat, Conditions, SpendContext, StandardLayer};
use sage_database::{Database, DatabaseTx};

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

    /// Inserts a range of unhardened derivations to the database.
    pub async fn insert_unhardened_derivations(
        &self,
        tx: &mut DatabaseTx<'_>,
        range: Range<u32>,
    ) -> Result<Vec<Bytes32>, WalletError> {
        let mut puzzle_hashes = Vec::new();

        for index in range {
            let synthetic_key = self
                .intermediate_pk
                .derive_unhardened(index)
                .derive_synthetic();

            let p2_puzzle_hash = StandardArgs::curry_tree_hash(synthetic_key).into();

            tx.insert_derivation(p2_puzzle_hash, index, false, synthetic_key)
                .await?;

            puzzle_hashes.push(p2_puzzle_hash);
        }

        Ok(puzzle_hashes)
    }

    pub async fn p2_puzzle_hashes(
        &self,
        count: u32,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<Bytes32>, WalletError> {
        let mut tx = self.db.tx().await?;

        let max_used = tx.max_used_derivation_index(hardened).await?;
        let next_index = tx.derivation_index(hardened).await?;

        let (mut start, mut end) = if reuse {
            let start = max_used.unwrap_or(0);
            let end = next_index.min(start + count);
            (start, end)
        } else {
            let start = max_used.map_or(0, |i| i + 1);
            let end = next_index.min(start + count);
            (start, end)
        };

        if end - start < count && reuse {
            start = start.saturating_sub(count - (end - start));
        }

        if end - start < count {
            end = next_index.min(end + count - (end - start));
        }

        if end - start < count {
            return Err(WalletError::InsufficientDerivations);
        }

        let mut p2_puzzle_hashes = Vec::new();

        for index in start..end {
            let p2_puzzle_hash = tx.p2_puzzle_hash(index, hardened).await?;
            p2_puzzle_hashes.push(p2_puzzle_hash);
        }

        tx.commit().await?;

        Ok(p2_puzzle_hashes)
    }

    pub async fn p2_puzzle_hash(
        &self,
        hardened: bool,
        reuse: bool,
    ) -> Result<Bytes32, WalletError> {
        Ok(self.p2_puzzle_hashes(1, hardened, reuse).await?[0])
    }

    /// Selects one or more unspent p2 coins from the database.
    async fn select_p2_coins(&self, amount: u128) -> Result<Vec<Coin>, WalletError> {
        let spendable_coins = self.db.unspent_p2_coins().await?;
        Ok(select_coins(spendable_coins, amount)?)
    }

    /// Spends the given coins individually with the given conditions. No outputs are created automatically.
    async fn spend_p2_coins_separately(
        &self,
        ctx: &mut SpendContext,
        coins: impl Iterator<Item = (Coin, Conditions)>,
    ) -> Result<(), WalletError> {
        let mut tx = self.db.tx().await?;

        for (coin, conditions) in coins {
            // We need to figure out what the synthetic public key is for this p2 coin.
            let synthetic_key = tx.synthetic_key(coin.puzzle_hash).await?;

            // Create the standard p2 layer for the key.
            let p2 = StandardLayer::new(synthetic_key);

            // Spend the coin with the given conditions.
            p2.spend(ctx, coin, conditions)?;
        }

        Ok(())
    }

    /// Spends the coins with the first coin producing all of the output conditions.
    /// The other coins assert that the first coin is spent within the transaction.
    /// This prevents the first coin from being removed from the transaction to steal the funds.
    async fn spend_p2_coins(
        &self,
        ctx: &mut SpendContext,
        coins: Vec<Coin>,
        mut conditions: Conditions,
    ) -> Result<(), WalletError> {
        let first_coin_id = coins[0].coin_id();

        self.spend_p2_coins_separately(
            ctx,
            coins.into_iter().enumerate().map(|(i, coin)| {
                let conditions = if i == 0 {
                    mem::take(&mut conditions)
                } else {
                    Conditions::new().assert_concurrent_spend(first_coin_id)
                };
                (coin, conditions)
            }),
        )
        .await
    }

    /// Sends the given amount of XCH to the given puzzle hash, minus the fee.
    pub async fn send_xch(
        &self,
        puzzle_hash: Bytes32,
        amount: u64,
        fee: u64,
        memos: Vec<Bytes>,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let total = amount as u128 + fee as u128;
        let coins = self.select_p2_coins(total).await?;
        let selected: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        let change_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

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

        let mut ctx = SpendContext::new();
        self.spend_p2_coins(&mut ctx, coins, conditions).await?;
        Ok(ctx.take())
    }

    /// Combines multiple p2 coins into a single coin, with the given fee subtracted from the output.
    pub async fn combine_xch(
        &self,
        coins: Vec<Coin>,
        fee: u64,
        memos: Vec<Bytes>,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let total: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        if fee as u128 > total {
            return Err(WalletError::InsufficientFunds);
        }

        let change_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let change: u64 = (total - fee as u128)
            .try_into()
            .expect("change amount overflow");

        let mut conditions = Conditions::new();

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(change_puzzle_hash, change, memos.clone());
        }

        let mut ctx = SpendContext::new();
        self.spend_p2_coins(&mut ctx, coins, conditions).await?;
        Ok(ctx.take())
    }

    /// Splits the given coins into multiple new coins, with the given fee subtracted from the output.
    pub async fn split_xch(
        &self,
        coins: &[Coin],
        output_count: usize,
        fee: u64,
        memos: Vec<Bytes>,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let total: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        if fee as u128 > total {
            return Err(WalletError::InsufficientFunds);
        }

        let mut remaining_count = output_count;
        let mut remaining_amount = total - fee as u128;

        let max_individual_amount: u64 = remaining_amount
            .div_ceil(output_count as u128)
            .try_into()
            .expect("output amount overflow");

        let derivations_needed: u32 = output_count
            .div_ceil(coins.len())
            .try_into()
            .expect("derivation count overflow");

        let derivations = self
            .p2_puzzle_hashes(derivations_needed, hardened, reuse)
            .await?;

        let mut ctx = SpendContext::new();

        self.spend_p2_coins_separately(
            &mut ctx,
            coins.iter().enumerate().map(|(i, coin)| {
                let mut conditions = Conditions::new();

                if i == 0 && fee > 0 {
                    conditions = conditions.reserve_fee(fee);
                }

                if coins.len() > 1 {
                    if i == coins.len() - 1 {
                        conditions = conditions.assert_concurrent_spend(coins[0].coin_id());
                    } else {
                        conditions = conditions.assert_concurrent_spend(coins[i + 1].coin_id());
                    }
                }

                for &derivation in &derivations {
                    if remaining_count == 0 {
                        break;
                    }

                    let amount: u64 = (max_individual_amount as u128)
                        .min(remaining_amount)
                        .try_into()
                        .expect("output amount overflow");

                    remaining_amount -= amount as u128;

                    conditions = conditions.create_coin(derivation, amount, memos.clone());

                    remaining_count -= 1;
                }

                (*coin, conditions)
            }),
        )
        .await?;

        Ok(ctx.take())
    }

    /// Creates a transaction that transfers the given coins to the given puzzle hash, minus the fee as needed.
    /// Since the parent coins are all unique, there are no coin id conflicts in the output.
    pub async fn transfer_xch(
        &self,
        coins: Vec<Coin>,
        puzzle_hash: Bytes32,
        mut fee: u64,
        memos: Vec<Bytes>,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        // Select the most optimal coins to use for the fee, to keep cost to a minimum.
        let fee_coins: HashSet<Coin> = select_coins(coins.clone(), fee as u128)?
            .into_iter()
            .collect();

        let mut ctx = SpendContext::new();

        self.spend_p2_coins_separately(
            &mut ctx,
            coins.iter().enumerate().map(|(i, coin)| {
                let conditions = if fee > 0 && fee_coins.contains(coin) {
                    // Consume as much as possible from the fee.
                    let consumed = fee.min(coin.amount);
                    fee -= consumed;

                    // If there is excess amount in this coin after the fee is paid, create a new output.
                    if consumed < coin.amount {
                        Conditions::new().create_coin(
                            puzzle_hash,
                            coin.amount - consumed,
                            memos.clone(),
                        )
                    } else {
                        Conditions::new()
                    }
                } else {
                    // Otherwise, just create a new output coin at the given puzzle hash.
                    Conditions::new().create_coin(puzzle_hash, coin.amount, memos.clone())
                };

                // Ensure that there is a ring of assertions for all of the coins.
                // This prevents any of them from being removed from the transaction later.
                let conditions = if coins.len() > 1 {
                    if i == coins.len() - 1 {
                        conditions.assert_concurrent_spend(coins[0].coin_id())
                    } else {
                        conditions.assert_concurrent_spend(coins[i + 1].coin_id())
                    }
                } else {
                    conditions
                };

                // The fee is reserved by one coin, even though it can come from multiple coins.
                let conditions = if i == 0 {
                    conditions.reserve_fee(fee)
                } else {
                    conditions
                };

                (*coin, conditions)
            }),
        )
        .await?;

        Ok(ctx.take())
    }

    pub async fn issue_cat(
        &self,
        amount: u64,
        fee: u64,
        multi_issuance_key: Option<PublicKey>,
        hardened: bool,
        reuse: bool,
    ) -> Result<(Vec<CoinSpend>, Bytes32), WalletError> {
        let total_amount = amount as u128 + fee as u128;
        let coins = self.select_p2_coins(total_amount).await?;
        let selected: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        let change: u64 = (selected - total_amount)
            .try_into()
            .expect("change amount overflow");

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut ctx = SpendContext::new();

        let eve_conditions = Conditions::new().create_coin(
            p2_puzzle_hash,
            amount,
            vec![p2_puzzle_hash.to_vec().into()],
        );

        let (mut conditions, eve) = match multi_issuance_key {
            Some(pk) => {
                Cat::multi_issuance_eve(&mut ctx, coins[0].coin_id(), pk, amount, eve_conditions)?
            }
            None => Cat::single_issuance_eve(&mut ctx, coins[0].coin_id(), amount, eve_conditions)?,
        };

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(p2_puzzle_hash, change, Vec::new());
        }

        self.spend_p2_coins(&mut ctx, coins, conditions).await?;

        Ok((ctx.take(), eve.asset_id))
    }
}
