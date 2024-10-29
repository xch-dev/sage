use std::{
    collections::{HashMap, HashSet},
    mem,
    ops::Range,
};

use chia::{
    bls::{
        master_to_wallet_unhardened_intermediate, sign, DerivableKey, PublicKey, SecretKey,
        Signature,
    },
    protocol::{Bytes, Bytes32, Coin, CoinSpend, CoinState, Program, SpendBundle},
    puzzles::{
        nft::{NftMetadata, NFT_METADATA_UPDATER_PUZZLE_HASH},
        standard::StandardArgs,
        DeriveSynthetic,
    },
};
use chia_wallet_sdk::{
    select_coins, AggSigConstants, Cat, CatSpend, Conditions, Did, DidOwner, HashedPtr, Launcher,
    Nft, NftMint, RequiredSignature, SpendContext, SpendWithConditions, StandardLayer,
};
use clvmr::Allocator;
use sage_database::{Database, DatabaseTx};

use crate::{ChildKind, Transaction, WalletError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletNftMint {
    pub metadata: NftMetadata,
    pub royalty_puzzle_hash: Option<Bytes32>,
    pub royalty_ten_thousandths: u16,
}

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
        let spendable_coins = self.db.spendable_coins().await?;
        Ok(select_coins(spendable_coins, amount)?)
    }

    /// Selects one or more unspent CAT coins from the database.
    async fn select_cat_coins(
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

    /// Spends the CATs with the given conditions. No outputs are created automatically.
    async fn spend_cat_coins(
        &self,
        ctx: &mut SpendContext,
        cats: impl Iterator<Item = (Cat, Conditions)>,
    ) -> Result<(), WalletError> {
        let mut tx = self.db.tx().await?;
        let mut cat_spends = Vec::new();

        for (cat, conditions) in cats {
            // We need to figure out what the synthetic public key is for this CAT coin.
            let synthetic_key = tx.synthetic_key(cat.p2_puzzle_hash).await?;

            // Create the standard p2 layer for the key.
            let p2 = StandardLayer::new(synthetic_key);

            // Spend the CAT with the given conditions.
            cat_spends.push(CatSpend::new(
                cat,
                p2.spend_with_conditions(ctx, conditions)?,
            ));
        }

        Cat::spend_all(ctx, &cat_spends)?;

        Ok(())
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
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let total: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        if fee as u128 > total {
            return Err(WalletError::InsufficientFunds);
        }

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let change: u64 = (total - fee as u128)
            .try_into()
            .expect("change amount overflow");

        let mut conditions = Conditions::new();

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(p2_puzzle_hash, change, Vec::new());
        }

        let mut ctx = SpendContext::new();
        self.spend_p2_coins(&mut ctx, coins, conditions).await?;
        Ok(ctx.take())
    }

    /// Splits the given XCH coins into multiple new coins, with the given fee subtracted from the output.
    pub async fn split_xch(
        &self,
        coins: &[Coin],
        output_count: usize,
        fee: u64,
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

                    conditions = conditions.create_coin(derivation, amount, Vec::new());

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
                            Vec::new(),
                        )
                    } else {
                        Conditions::new()
                    }
                } else {
                    // Otherwise, just create a new output coin at the given puzzle hash.
                    Conditions::new().create_coin(puzzle_hash, coin.amount, Vec::new())
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

    /// Combines multiple CAT coins into a single coin, with the given fee subtracted from the output.
    pub async fn combine_cat(
        &self,
        cats: Vec<Cat>,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let fee_coins = self.select_p2_coins(fee as u128).await?;
        let fee_total: u128 = fee_coins.iter().map(|coin| coin.amount as u128).sum();
        let cat_total: u128 = cats.iter().map(|cat| cat.coin.amount as u128).sum();

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let fee_change: u64 = (fee_total - fee as u128)
            .try_into()
            .expect("change amount overflow");

        let mut fee_conditions = Conditions::new().assert_concurrent_spend(cats[0].coin.coin_id());

        if fee > 0 {
            fee_conditions = fee_conditions.reserve_fee(fee);
        }

        if fee_change > 0 {
            fee_conditions = fee_conditions.create_coin(p2_puzzle_hash, fee_change, Vec::new());
        }

        let mut ctx = SpendContext::new();

        self.spend_p2_coins(&mut ctx, fee_coins, fee_conditions)
            .await?;

        self.spend_cat_coins(
            &mut ctx,
            cats.into_iter().enumerate().map(|(i, cat)| {
                if i == 0 {
                    (
                        cat,
                        Conditions::new().create_coin(
                            p2_puzzle_hash,
                            cat_total.try_into().expect("output amount overflow"),
                            vec![p2_puzzle_hash.into()],
                        ),
                    )
                } else {
                    (cat, Conditions::new())
                }
            }),
        )
        .await?;

        Ok(ctx.take())
    }

    /// Splits the given CAT coins into multiple new coins, with the given fee subtracted from the output.
    pub async fn split_cat(
        &self,
        cats: Vec<Cat>,
        output_count: usize,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let fee_coins = self.select_p2_coins(fee as u128).await?;
        let fee_total: u128 = fee_coins.iter().map(|coin| coin.amount as u128).sum();
        let cat_total: u128 = cats.iter().map(|cat| cat.coin.amount as u128).sum();

        let mut remaining_count = output_count;
        let mut remaining_amount = cat_total;

        let max_individual_amount: u64 = remaining_amount
            .div_ceil(output_count as u128)
            .try_into()
            .expect("output amount overflow");

        let derivations_needed: u32 = output_count
            .div_ceil(cats.len())
            .try_into()
            .expect("derivation count overflow");

        let derivations = self
            .p2_puzzle_hashes(derivations_needed, hardened, reuse)
            .await?;

        let mut ctx = SpendContext::new();

        let fee_change: u64 = (fee_total - fee as u128)
            .try_into()
            .expect("change amount overflow");

        let mut fee_conditions = Conditions::new().assert_concurrent_spend(cats[0].coin.coin_id());

        if fee > 0 {
            fee_conditions = fee_conditions.reserve_fee(fee);
        }

        if fee_change > 0 {
            fee_conditions = fee_conditions.create_coin(derivations[0], fee_change, Vec::new());
        }

        self.spend_p2_coins(&mut ctx, fee_coins, fee_conditions)
            .await?;

        self.spend_cat_coins(
            &mut ctx,
            cats.into_iter().map(|cat| {
                let mut conditions = Conditions::new();

                for &derivation in &derivations {
                    if remaining_count == 0 {
                        break;
                    }

                    let amount: u64 = (max_individual_amount as u128)
                        .min(remaining_amount)
                        .try_into()
                        .expect("output amount overflow");

                    remaining_amount -= amount as u128;

                    conditions =
                        conditions.create_coin(derivation, amount, vec![derivation.into()]);

                    remaining_count -= 1;
                }

                (cat, conditions)
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

    /// Sends the given amount of XCH to the given puzzle hash, minus the fee.
    pub async fn send_cat(
        &self,
        asset_id: Bytes32,
        puzzle_hash: Bytes32,
        amount: u64,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let fee_coins = self.select_p2_coins(fee as u128).await?;
        let fee_selected: u128 = fee_coins.iter().map(|coin| coin.amount as u128).sum();
        let fee_change: u64 = (fee_selected - fee as u128)
            .try_into()
            .expect("fee change overflow");

        let cats = self.select_cat_coins(asset_id, amount as u128).await?;
        let cat_selected: u128 = cats.iter().map(|cat| cat.coin.amount as u128).sum();
        let cat_change: u64 = (cat_selected - amount as u128)
            .try_into()
            .expect("change amount overflow");

        let change_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut conditions = Conditions::new().assert_concurrent_spend(cats[0].coin.coin_id());

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if fee_change > 0 {
            conditions = conditions.create_coin(change_puzzle_hash, fee_change, Vec::new());
        }

        let mut ctx = SpendContext::new();

        self.spend_p2_coins(&mut ctx, fee_coins, conditions).await?;

        self.spend_cat_coins(
            &mut ctx,
            cats.into_iter().enumerate().map(|(i, cat)| {
                if i != 0 {
                    return (cat, Conditions::new());
                }

                let mut conditions =
                    Conditions::new().create_coin(puzzle_hash, amount, vec![puzzle_hash.into()]);

                if cat_change > 0 {
                    conditions = conditions.create_coin(
                        change_puzzle_hash,
                        cat_change,
                        vec![change_puzzle_hash.into()],
                    );
                }

                (cat, conditions)
            }),
        )
        .await?;

        Ok(ctx.take())
    }

    pub async fn create_did(
        &self,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<(Vec<CoinSpend>, Did<()>), WalletError> {
        let total_amount = fee as u128 + 1;
        let coins = self.select_p2_coins(total_amount).await?;
        let selected: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        let change: u64 = (selected - total_amount)
            .try_into()
            .expect("change amount overflow");

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut ctx = SpendContext::new();

        let synthetic_key = self.db.synthetic_key(coins[0].puzzle_hash).await?;
        let p2 = StandardLayer::new(synthetic_key);
        let (mut conditions, did) =
            Launcher::new(coins[0].coin_id(), 1).create_simple_did(&mut ctx, &p2)?;

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(p2_puzzle_hash, change, Vec::new());
        }

        self.spend_p2_coins(&mut ctx, coins, conditions).await?;

        Ok((ctx.take(), did))
    }

    pub async fn bulk_mint_nfts(
        &self,
        fee: u64,
        did_id: Bytes32,
        mints: Vec<WalletNftMint>,
        hardened: bool,
        reuse: bool,
    ) -> Result<(Vec<CoinSpend>, Vec<Nft<NftMetadata>>, Did<Program>), WalletError> {
        let Some(did) = self.db.did(did_id).await? else {
            return Err(WalletError::MissingDid(did_id));
        };

        let total_amount = fee as u128 + 1;
        let coins = self.select_p2_coins(total_amount).await?;
        let selected: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        let change: u64 = (selected - total_amount)
            .try_into()
            .expect("change amount overflow");

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut ctx = SpendContext::new();

        let did_metadata_ptr = ctx.alloc(&did.info.metadata)?;
        let did = did.with_metadata(HashedPtr::from_ptr(&ctx.allocator, did_metadata_ptr));

        let synthetic_key = self.db.synthetic_key(did.info.p2_puzzle_hash).await?;
        let p2 = StandardLayer::new(synthetic_key);

        let mut did_conditions = Conditions::new();
        let mut nfts = Vec::with_capacity(mints.len());

        for (i, mint) in mints.into_iter().enumerate() {
            let mint = NftMint {
                metadata: mint.metadata,
                metadata_updater_puzzle_hash: NFT_METADATA_UPDATER_PUZZLE_HASH.into(),
                royalty_puzzle_hash: mint.royalty_puzzle_hash.unwrap_or(p2_puzzle_hash),
                royalty_ten_thousandths: mint.royalty_ten_thousandths,
                p2_puzzle_hash,
                owner: Some(DidOwner::from_did_info(&did.info)),
            };

            let (mint_nft, nft) = Launcher::new(did.coin.coin_id(), i as u64 * 2)
                .with_singleton_amount(1)
                .mint_nft(&mut ctx, mint)?;

            did_conditions = did_conditions.extend(mint_nft);
            nfts.push(nft);
        }

        let new_did = did.update(&mut ctx, &p2, did_conditions)?;

        let mut conditions = Conditions::new().assert_concurrent_spend(did.coin.coin_id());

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(p2_puzzle_hash, change, Vec::new());
        }

        self.spend_p2_coins(&mut ctx, coins, conditions).await?;

        let new_did = new_did.with_metadata(ctx.serialize(&new_did.info.metadata)?);

        Ok((ctx.take(), nfts, new_did))
    }

    pub async fn transfer_nft(
        &self,
        nft_id: Bytes32,
        puzzle_hash: Bytes32,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<(Vec<CoinSpend>, Nft<Program>), WalletError> {
        let Some(nft) = self.db.nft(nft_id).await? else {
            return Err(WalletError::MissingNft(nft_id));
        };

        let total_amount = fee as u128 + 1;
        let coins = self.select_p2_coins(total_amount).await?;
        let selected: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        let change: u64 = (selected - total_amount)
            .try_into()
            .expect("change amount overflow");

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut ctx = SpendContext::new();

        let nft_metadata_ptr = ctx.alloc(&nft.info.metadata)?;
        let nft = nft.with_metadata(HashedPtr::from_ptr(&ctx.allocator, nft_metadata_ptr));

        let synthetic_key = self.db.synthetic_key(nft.info.p2_puzzle_hash).await?;
        let p2 = StandardLayer::new(synthetic_key);

        let new_nft = nft.transfer(&mut ctx, &p2, puzzle_hash, Conditions::new())?;

        let mut conditions = Conditions::new().assert_concurrent_spend(nft.coin.coin_id());

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(p2_puzzle_hash, change, Vec::new());
        }

        self.spend_p2_coins(&mut ctx, coins, conditions).await?;

        let new_nft = new_nft.with_metadata(ctx.serialize(&new_nft.info.metadata)?);

        Ok((ctx.take(), new_nft))
    }

    pub async fn transfer_did(
        &self,
        did_id: Bytes32,
        puzzle_hash: Bytes32,
        fee: u64,
        hardened: bool,
        reuse: bool,
    ) -> Result<(Vec<CoinSpend>, Did<Program>), WalletError> {
        let Some(did) = self.db.did(did_id).await? else {
            return Err(WalletError::MissingDid(did_id));
        };

        let total_amount = fee as u128;
        let coins = self.select_p2_coins(total_amount).await?;
        let selected: u128 = coins.iter().map(|coin| coin.amount as u128).sum();

        let change: u64 = (selected - total_amount)
            .try_into()
            .expect("change amount overflow");

        let p2_puzzle_hash = self.p2_puzzle_hash(hardened, reuse).await?;

        let mut ctx = SpendContext::new();

        let did_metadata_ptr = ctx.alloc(&did.info.metadata)?;
        let did = did.with_metadata(HashedPtr::from_ptr(&ctx.allocator, did_metadata_ptr));

        let synthetic_key = self.db.synthetic_key(did.info.p2_puzzle_hash).await?;
        let p2 = StandardLayer::new(synthetic_key);

        let new_did = did.transfer(&mut ctx, &p2, puzzle_hash, Conditions::new())?;

        let mut conditions = Conditions::new().assert_concurrent_spend(did.coin.coin_id());

        if fee > 0 {
            conditions = conditions.reserve_fee(fee);
        }

        if change > 0 {
            conditions = conditions.create_coin(p2_puzzle_hash, change, Vec::new());
        }

        self.spend_p2_coins(&mut ctx, coins, conditions).await?;

        let new_did = new_did.with_metadata(ctx.serialize(&new_did.info.metadata)?);

        Ok((ctx.take(), new_did))
    }

    pub async fn sign_transaction(
        &self,
        mut coin_spends: Vec<CoinSpend>,
        agg_sig_constants: &AggSigConstants,
        master_sk: SecretKey,
    ) -> Result<SpendBundle, WalletError> {
        let required_signatures = RequiredSignature::from_coin_spends(
            &mut Allocator::new(),
            &coin_spends,
            agg_sig_constants,
        )?;

        let mut indices = HashMap::new();

        for required in &required_signatures {
            let pk = required.public_key();
            let Some(index) = self.db.synthetic_key_index(pk).await? else {
                return Err(WalletError::UnknownPublicKey(pk));
            };
            indices.insert(pk, index);
        }

        let intermediate_sk = master_to_wallet_unhardened_intermediate(&master_sk);

        let secret_keys: HashMap<PublicKey, SecretKey> = indices
            .iter()
            .map(|(pk, index)| {
                let sk = intermediate_sk.derive_unhardened(*index).derive_synthetic();
                (*pk, sk)
            })
            .collect();

        let mut aggregated_signature = Signature::default();

        for required in required_signatures {
            let sk = secret_keys[&required.public_key()].clone();
            aggregated_signature += &sign(&sk, required.final_message());
        }

        coin_spends.sort_by_key(|cs| cs.coin.coin_id());

        Ok(SpendBundle::new(coin_spends, aggregated_signature))
    }

    pub async fn insert_transaction(&self, spend_bundle: SpendBundle) -> Result<(), WalletError> {
        let transaction_id = spend_bundle.name();
        let transaction = Transaction::from_coin_spends(spend_bundle.coin_spends)?;

        let mut tx = self.db.tx().await?;

        tx.insert_transaction(
            transaction_id,
            spend_bundle.aggregated_signature.clone(),
            transaction.fee,
        )
        .await?;

        for input in transaction.inputs {
            tx.insert_transaction_spend(
                input.coin_spend.coin,
                transaction_id,
                input.coin_spend.puzzle_reveal,
                input.coin_spend.solution,
            )
            .await?;

            for output in input.outputs {
                let coin_state = CoinState::new(output.coin, None, None);
                let coin_id = output.coin.coin_id();

                macro_rules! insert_coin {
                    () => {
                        tx.insert_coin_state(coin_state, true, Some(transaction_id))
                            .await?;
                    };
                }

                if tx.is_p2_puzzle_hash(output.coin.puzzle_hash).await? {
                    insert_coin!();
                    tx.insert_p2_coin(coin_id).await?;
                    continue;
                }

                match output.kind {
                    ChildKind::Launcher => {}
                    ChildKind::Cat {
                        asset_id,
                        lineage_proof,
                        p2_puzzle_hash,
                    } => {
                        if tx.is_p2_puzzle_hash(p2_puzzle_hash).await? {
                            insert_coin!();
                            tx.sync_coin(coin_id, Some(p2_puzzle_hash)).await?;
                            tx.insert_cat_coin(coin_id, lineage_proof, p2_puzzle_hash, asset_id)
                                .await?;
                        }
                    }
                    ChildKind::Did {
                        lineage_proof,
                        info,
                    } => {
                        if tx.is_p2_puzzle_hash(info.p2_puzzle_hash).await? {
                            insert_coin!();
                            tx.sync_coin(coin_id, Some(info.p2_puzzle_hash)).await?;
                            tx.insert_new_did(info.launcher_id, None, true).await?;
                            tx.insert_did_coin(coin_id, lineage_proof, info).await?;
                        }
                    }
                    ChildKind::Nft {
                        lineage_proof,
                        info,
                        metadata,
                    } => {
                        let data_hash = metadata.as_ref().and_then(|m| m.data_hash);
                        let metadata_hash = metadata.as_ref().and_then(|m| m.metadata_hash);
                        let license_hash = metadata.as_ref().and_then(|m| m.license_hash);

                        if tx.is_p2_puzzle_hash(info.p2_puzzle_hash).await? {
                            insert_coin!();

                            tx.sync_coin(coin_id, Some(info.p2_puzzle_hash)).await?;
                            tx.insert_new_nft(info.launcher_id, true).await?;
                            tx.insert_nft_coin(
                                coin_id,
                                lineage_proof,
                                info,
                                data_hash,
                                metadata_hash,
                                license_hash,
                            )
                            .await?;

                            if let Some(metadata) = metadata {
                                if let Some(hash) = data_hash {
                                    for uri in metadata.data_uris {
                                        tx.insert_nft_uri(uri, hash).await?;
                                    }
                                }

                                if let Some(hash) = metadata_hash {
                                    for uri in metadata.metadata_uris {
                                        tx.insert_nft_uri(uri, hash).await?;
                                    }
                                }

                                if let Some(hash) = license_hash {
                                    for uri in metadata.license_uris {
                                        tx.insert_nft_uri(uri, hash).await?;
                                    }
                                }
                            }
                        }
                    }
                    ChildKind::Unknown { hint } => {
                        let Some(p2_puzzle_hash) = hint else {
                            continue;
                        };

                        if tx.is_p2_puzzle_hash(p2_puzzle_hash).await? {
                            insert_coin!();
                            tx.sync_coin(coin_id, hint).await?;
                            tx.insert_unknown_coin(coin_id).await?;
                        }
                    }
                }
            }
        }

        tx.commit().await?;

        Ok(())
    }
}
