use std::{collections::HashMap, mem};

use chia::protocol::{Bytes32, Coin};
use chia_puzzles::SETTLEMENT_PAYMENT_HASH;
use chia_wallet_sdk::{
    driver::{HashedPtr, SpendContext, StandardLayer},
    prelude::TradePrice,
    types::Conditions,
};

use crate::{Wallet, WalletError};

use super::{
    make_royalty_payments, LockedCoins, OfferAmounts, OfferCoins, PaymentOrigin, Royalties,
};

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
                conditions =
                    conditions.create_coin(SETTLEMENT_PAYMENT_HASH.into(), amounts.xch, None);

                locked.xch.push(Coin::new(
                    primary_xch_coin.coin_id(),
                    SETTLEMENT_PAYMENT_HASH.into(),
                    amounts.xch,
                ));
            }

            // Handle royalties.
            let royalty_amount = royalties.xch_amount();

            if royalty_amount > 0 {
                conditions = conditions.with(make_royalty_payments(
                    ctx,
                    royalty_amount,
                    royalties.xch.clone(),
                    PaymentOrigin::Xch(primary_xch_coin),
                )?);
            }

            let total_amount = coins.xch.iter().map(|coin| coin.amount).sum::<u64>();
            let change = total_amount - amounts.xch - fee - royalties.xch_amount();

            if change > 0 {
                conditions = conditions.create_coin(change_puzzle_hash, change, None);
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
            let change = total_amount - amount - royalties.cat_amount(asset_id);

            let settlement_hint = ctx.hint(SETTLEMENT_PAYMENT_HASH.into())?;

            let mut conditions = primary_conditions
                .remove(&primary_cat.coin.coin_id())
                .unwrap_or_default()
                .create_coin(
                    SETTLEMENT_PAYMENT_HASH.into(),
                    amount,
                    Some(settlement_hint),
                );

            locked
                .cats
                .entry(asset_id)
                .or_default()
                .push(primary_cat.wrapped_child(SETTLEMENT_PAYMENT_HASH.into(), amount));

            if change > 0 {
                let change_hint = ctx.hint(change_puzzle_hash)?;
                conditions = conditions.create_coin(change_puzzle_hash, change, Some(change_hint));
            }

            // Handle royalties.
            let royalty_amount = royalties.cat_amount(asset_id);

            if royalty_amount > 0 {
                conditions = conditions.with(make_royalty_payments(
                    ctx,
                    royalty_amount,
                    royalties.cats[&asset_id].clone(),
                    PaymentOrigin::Cat(primary_cat),
                )?);
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
            let nft = nft.with_metadata(HashedPtr::from_ptr(ctx, metadata_ptr));

            let synthetic_key = self.db.synthetic_key(nft.info.p2_puzzle_hash).await?;
            let p2 = StandardLayer::new(synthetic_key);

            let conditions = primary_conditions
                .remove(&nft.coin.coin_id())
                .unwrap_or_default();

            let nft = nft.lock_settlement(
                ctx,
                &p2,
                if nft.info.royalty_ten_thousandths > 0 {
                    trade_prices.clone()
                } else {
                    Vec::new()
                },
                conditions,
            )?;

            locked.nfts.insert(nft.info.launcher_id, nft);
        }

        Ok(locked)
    }
}
