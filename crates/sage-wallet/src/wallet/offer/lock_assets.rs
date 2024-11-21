use std::{collections::HashMap, mem};

use chia::{
    protocol::{Bytes32, Coin},
    puzzles::offer::{
        Memos, NotarizedPayment, Payment, SettlementPaymentsSolution,
        SETTLEMENT_PAYMENTS_PUZZLE_HASH,
    },
};
use chia_wallet_sdk::{
    Cat, CatSpend, Conditions, HashedPtr, Layer, SettlementLayer, SpendContext, StandardLayer,
    TradePrice,
};

use crate::{Wallet, WalletError};

use super::{LockedCoins, OfferAmounts, OfferCoins, Royalties};

#[derive(Debug, Clone)]
pub struct OfferSpend {
    pub amounts: OfferAmounts,
    pub coins: OfferCoins,
    pub royalties: Royalties,
    pub trade_prices: Vec<TradePrice>,
    pub fee: u64,
    pub change_puzzle_hash: Bytes32,
    pub extra_conditions: Conditions,
}

impl Wallet {
    pub async fn lock_assets(
        &self,
        ctx: &mut SpendContext,
        OfferSpend {
            amounts,
            coins,
            royalties,
            trade_prices,
            fee,
            change_puzzle_hash,
            mut extra_conditions,
        }: OfferSpend,
    ) -> Result<LockedCoins, WalletError> {
        let primary_coins = coins.primary_coin_ids();

        // Calculate conditions for each primary coin.
        let mut primary_conditions = HashMap::new();

        if primary_coins.len() == 1 {
            primary_conditions.insert(primary_coins[0], extra_conditions);
        } else {
            for (i, &coin_id) in primary_coins.iter().enumerate() {
                let relation = if i == 0 {
                    *primary_coins.last().expect("empty primary coins")
                } else {
                    primary_coins[i - 1]
                };

                primary_conditions.insert(
                    coin_id,
                    mem::take(&mut extra_conditions).assert_concurrent_spend(relation),
                );
            }
        }

        // Keep track of the coins that are locked.
        let mut locked = LockedCoins {
            fee,
            ..Default::default()
        };

        // Spend the XCH.
        if let Some(primary_xch_coin) = coins.xch.first().copied() {
            let mut conditions = primary_conditions
                .remove(&primary_xch_coin.coin_id())
                .unwrap_or_default();

            if amounts.xch > 0 {
                conditions = conditions.create_coin(
                    SETTLEMENT_PAYMENTS_PUZZLE_HASH.into(),
                    amounts.xch,
                    Vec::new(),
                );

                locked.xch.push(Coin::new(
                    primary_xch_coin.coin_id(),
                    SETTLEMENT_PAYMENTS_PUZZLE_HASH.into(),
                    amounts.xch,
                ));
            }

            // Handle royalties.
            for royalty in &royalties.xch {
                conditions = conditions.create_coin(
                    SETTLEMENT_PAYMENTS_PUZZLE_HASH.into(),
                    royalty.amount,
                    Vec::new(),
                );

                let royalty_coin = Coin::new(
                    primary_xch_coin.coin_id(),
                    SETTLEMENT_PAYMENTS_PUZZLE_HASH.into(),
                    royalty.amount,
                );

                let coin_spend = SettlementLayer.construct_coin_spend(
                    ctx,
                    royalty_coin,
                    SettlementPaymentsSolution {
                        notarized_payments: vec![NotarizedPayment {
                            nonce: royalty.nft_id,
                            payments: vec![Payment::with_memos(
                                royalty.p2_puzzle_hash,
                                royalty.amount,
                                Memos(vec![royalty.p2_puzzle_hash.into()]),
                            )],
                        }],
                    },
                )?;
                ctx.insert(coin_spend);
            }

            let total_amount = coins.xch.iter().map(|coin| coin.amount).sum::<u64>();
            let change = total_amount - amounts.xch - fee - royalties.xch_amount();

            if change > 0 {
                conditions = conditions.create_coin(change_puzzle_hash, change, Vec::new());
            }

            if fee > 0 {
                conditions = conditions.reserve_fee(fee);
            }

            self.spend_p2_coins(ctx, coins.xch, conditions).await?;
        }

        // Spend the CATs.
        for (asset_id, cat_coins) in coins.cats {
            let Some(primary_cat) = cat_coins.first().copied() else {
                continue;
            };

            let amount = amounts.cats.get(&asset_id).copied().unwrap_or(0);
            let total_amount = cat_coins.iter().map(|cat| cat.coin.amount).sum::<u64>();
            let change = total_amount - amount;

            let mut conditions = primary_conditions
                .remove(&primary_cat.coin.coin_id())
                .unwrap_or_default()
                .create_coin(
                    SETTLEMENT_PAYMENTS_PUZZLE_HASH.into(),
                    amount,
                    vec![Bytes32::from(SETTLEMENT_PAYMENTS_PUZZLE_HASH).into()],
                );

            locked
                .cats
                .entry(asset_id)
                .or_default()
                .push(primary_cat.wrapped_child(SETTLEMENT_PAYMENTS_PUZZLE_HASH.into(), amount));

            if change > 0 {
                conditions = conditions.create_coin(
                    change_puzzle_hash,
                    change,
                    vec![change_puzzle_hash.into()],
                );
            }

            // Handle royalties.
            for royalty in royalties.cats.get(&asset_id).cloned().unwrap_or_default() {
                conditions = conditions.create_coin(
                    SETTLEMENT_PAYMENTS_PUZZLE_HASH.into(),
                    royalty.amount,
                    vec![Bytes32::from(SETTLEMENT_PAYMENTS_PUZZLE_HASH).into()],
                );

                let royalty_cat = primary_cat
                    .wrapped_child(SETTLEMENT_PAYMENTS_PUZZLE_HASH.into(), royalty.amount);

                let inner_spend = SettlementLayer.construct_spend(
                    ctx,
                    SettlementPaymentsSolution {
                        notarized_payments: vec![NotarizedPayment {
                            nonce: royalty.nft_id,
                            payments: vec![Payment::with_memos(
                                royalty.p2_puzzle_hash,
                                royalty.amount,
                                Memos(vec![royalty.p2_puzzle_hash.into()]),
                            )],
                        }],
                    },
                )?;

                Cat::spend_all(ctx, &[CatSpend::new(royalty_cat, inner_spend)])?;
            }

            self.spend_cat_coins(
                ctx,
                cat_coins
                    .into_iter()
                    .map(|cat| (cat, mem::take(&mut conditions))),
            )
            .await?;
        }

        // Spend the NFTs.
        for nft in coins.nfts.into_values() {
            let metadata_ptr = ctx.alloc(&nft.info.metadata)?;
            let nft = nft.with_metadata(HashedPtr::from_ptr(&ctx.allocator, metadata_ptr));

            let synthetic_key = self.db.synthetic_key(nft.info.p2_puzzle_hash).await?;
            let p2 = StandardLayer::new(synthetic_key);

            let conditions = primary_conditions
                .remove(&nft.coin.coin_id())
                .unwrap_or_default();

            let nft = nft.lock_settlement(ctx, &p2, trade_prices.clone(), conditions)?;

            locked.nfts.insert(nft.info.launcher_id, nft);
        }

        Ok(locked)
    }
}
