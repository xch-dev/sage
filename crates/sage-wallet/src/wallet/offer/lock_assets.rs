use chia::{protocol::Bytes32, puzzles::offer::Payment};
use chia_puzzles::SETTLEMENT_PAYMENT_HASH;
use chia_wallet_sdk::{
    driver::{HashedPtr, SpendContext},
    prelude::TradePrice,
    types::Conditions,
};
use itertools::Itertools;

use crate::{Id, Selection, SpendAction, TransactionConfig, Wallet, WalletError};

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
        let mut selection = Selection::default();

        for coin in coins.xch {
            selection.xch.coins.push(coin);
            selection.xch.existing_amount += coin.amount;
        }

        for (asset_id, cats) in coins.cats.clone() {
            for cat in cats {
                let selected = selection.cats.entry(Id::Existing(asset_id)).or_default();
                selected.coins.push(cat);
                selected.existing_amount += cat.coin.amount;
            }
        }

        for (launcher_id, nft) in coins.nfts.clone() {
            let metadata = ctx.alloc(&nft.info.metadata)?;
            let nft = nft.with_metadata(HashedPtr::from_ptr(ctx, metadata));
            selection.nfts.insert(Id::Existing(launcher_id), nft);
        }

        let mut actions = Vec::new();

        if amounts.xch > 0 {
            actions.push(SpendAction::offer_xch(amounts.xch));
        }

        let royalty_amount = royalties.xch_amount();

        if royalty_amount > 0 {
            for royalty in &royalties.xch {
                actions.push(SpendAction::fulfill_xch_payment(
                    royalty.nft_id,
                    Payment::with_memos(
                        royalty.p2_puzzle_hash,
                        royalty.amount,
                        vec![royalty.p2_puzzle_hash.into()],
                    ),
                ));
            }
        }

        for &asset_id in coins.cats.keys() {
            let amount = amounts.cats.get(&asset_id).copied().unwrap_or(0);

            if amount > 0 {
                actions.push(SpendAction::offer_cat(asset_id, amount));
            }

            let royalty_amount = royalties.cat_amount(asset_id);

            if royalty_amount > 0 {
                for royalty in &royalties.cats[&asset_id] {
                    actions.push(SpendAction::fulfill_cat_payment(
                        royalty.nft_id,
                        asset_id,
                        Payment::with_memos(
                            royalty.p2_puzzle_hash,
                            royalty.amount,
                            vec![royalty.p2_puzzle_hash.into()],
                        ),
                    ));
                }
            }
        }

        for &nft in coins.nfts.keys() {
            actions.extend(SpendAction::offer_nft(nft, trade_prices.clone()));
        }

        let result = self
            .transact_preselected_alloc(
                ctx,
                &mut TransactionConfig::new_preselected(actions, selection, fee),
            )
            .await?;

        for coin_spend in result.coin_spends {
            ctx.insert(coin_spend);
        }

        Ok(LockedCoins {
            xch: result
                .unspent_assets
                .xch
                .into_iter()
                .filter(|coin| coin.puzzle_hash == SETTLEMENT_PAYMENT_HASH.into())
                .collect(),
            cats: result
                .unspent_assets
                .cats
                .into_iter()
                .map(|(id, cats)| {
                    (
                        result.ids[&id],
                        cats.into_iter()
                            .filter(|cat| cat.p2_puzzle_hash == SETTLEMENT_PAYMENT_HASH.into())
                            .collect_vec(),
                    )
                })
                .filter(|(_, cats)| !cats.is_empty())
                .collect(),
            nfts: result
                .unspent_assets
                .nfts
                .into_iter()
                .map(|(id, nft)| (result.ids[&id], nft))
                .filter(|(_, nft)| nft.info.p2_puzzle_hash == SETTLEMENT_PAYMENT_HASH.into())
                .collect(),
            fee,
        })
    }
}
