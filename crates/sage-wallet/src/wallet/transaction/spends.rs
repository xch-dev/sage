mod fungible_asset;
mod p2;
mod singleton;

pub use fungible_asset::*;
pub use p2::*;
pub use singleton::*;

use std::collections::HashMap;

use chia::protocol::{Bytes32, Coin};
use chia_wallet_sdk::{
    driver::{Cat, CatSpend, Did, HashedPtr, Nft, OptionContract, SpendContext, StandardLayer},
    types::Conditions,
};
use indexmap::IndexMap;

use crate::{Wallet, WalletError};

use super::{Action, Id, Selection, Summary, TransactionConfig};

#[derive(Debug, Clone)]
pub struct Spends {
    pub xch: FungibleAsset<Coin>,
    pub cats: IndexMap<Id, FungibleAsset<Cat>>,
    pub dids: IndexMap<Id, SingletonLineage<Did<HashedPtr>>>,
    pub nfts: IndexMap<Id, SingletonLineage<Nft<HashedPtr>>>,
    pub options: IndexMap<Id, SingletonLineage<OptionContract>>,
}

impl Wallet {
    pub async fn spend(
        &self,
        ctx: &mut SpendContext,
        summary: &Summary,
        selection: &Selection,
        tx: &TransactionConfig,
    ) -> Result<Spends, WalletError> {
        let p2 = self.fetch_p2_map(selection).await?;

        let mut spends = Self::initial_spends(selection, &p2);

        for (index, action) in tx.actions.iter().enumerate() {
            action.spend(ctx, &mut spends, index)?;
        }

        if tx.fee > 0 {
            if let Some(p2) = spends
                .xch
                .items
                .iter_mut()
                .find_map(|item| item.p2.as_standard_mut())
            {
                p2.add_conditions(Conditions::new().reserve_fee(tx.fee));
            }
        }

        self.send_change(ctx, summary, selection, &mut spends)
            .await?;

        Self::finalize_singletons(ctx, &mut spends)?;

        let mut coin_ids = Vec::new();
        let mut skip_counts = Vec::new();

        // This is a complicated way of forming a ring of assert concurrent spend conditions.
        for collect in [true, false] {
            if !collect && !coin_ids.is_empty() {
                // We need to shift to the right by one to form a ring.
                let last = coin_ids.remove(coin_ids.len() - 1);
                coin_ids.insert(0, last);

                // If there's only one coin, there's no point in forming a ring.
                if coin_ids.len() == 1 {
                    coin_ids = Vec::new();
                }

                // Reverse the coin ids since we're going to be popping them from the end.
                coin_ids.reverse();
                skip_counts.reverse();
            }

            for item in &mut spends.xch.items {
                match &mut item.p2 {
                    P2::Standard(p2) => {
                        if collect {
                            coin_ids.push(item.coin.coin_id());
                            continue;
                        }

                        if let Some(coin_id) = coin_ids.pop() {
                            p2.add_conditions(Conditions::new().assert_concurrent_spend(coin_id));
                        }
                    }
                    P2::Offer(_) => {
                        if collect {
                            continue;
                        }
                    }
                }

                item.p2.spend(ctx, item.coin)?;
            }

            'cats: for cat in spends.cats.values_mut() {
                let mut cat_spends = Vec::new();

                for item in &mut cat.items {
                    let inner_spend = match &mut item.p2 {
                        P2::Standard(p2) => {
                            if collect {
                                coin_ids.push(item.coin.coin.coin_id());
                                continue 'cats;
                            }

                            if let Some(coin_id) = coin_ids.pop() {
                                p2.add_conditions(
                                    Conditions::new().assert_concurrent_spend(coin_id),
                                );
                            }

                            p2.inner_spend(ctx)?
                        }
                        P2::Offer(p2) => {
                            if collect {
                                continue;
                            }

                            p2.inner_spend(ctx)?
                        }
                    };

                    cat_spends.push(CatSpend::new(item.coin, inner_spend));
                }

                Cat::spend_all(ctx, &cat_spends)?;
            }

            macro_rules! singleton {
                ( $name:ident ) => {
                    for lineage in spends.$name.values_mut() {
                        let mut last_coin_id = None;
                        let mut skip_count = if collect {
                            skip_counts.pop().unwrap_or_default()
                        } else {
                            0
                        };

                        for item in lineage.iter_mut() {
                            if !item.p2().is_empty() {
                                let inner_spend = match item.p2_mut() {
                                    P2::Standard(p2) => {
                                        if collect {
                                            last_coin_id = Some(item.coin_id());
                                            skip_count += 1;
                                            continue;
                                        }

                                        if skip_count == 0 {
                                            if let Some(coin_id) = coin_ids.pop() {
                                                p2.add_conditions(
                                                    Conditions::new()
                                                        .assert_concurrent_spend(coin_id),
                                                );
                                            }
                                        } else {
                                            skip_count -= 1;
                                        }

                                        p2.inner_spend(ctx)?
                                    }
                                    P2::Offer(p2) => {
                                        if collect {
                                            continue;
                                        }

                                        p2.inner_spend(ctx)?
                                    }
                                };

                                item.coin().spend(ctx, inner_spend)?;
                            }
                        }

                        if let Some(coin_id) = last_coin_id {
                            if collect {
                                coin_ids.push(coin_id);
                                skip_counts.push(skip_count - 1);
                            }
                        }
                    }
                };
            }

            singleton!(dids);
            singleton!(nfts);
            singleton!(options);
        }

        Ok(spends)
    }

    // TODO: Improve how the p2 is fetched and make it work on the fly
    async fn fetch_p2_map(
        &self,
        selection: &Selection,
    ) -> Result<HashMap<Bytes32, P2>, WalletError> {
        let mut p2 = HashMap::new();

        for coin in &selection.xch.coins {
            let synthetic_key = self.db.synthetic_key(coin.puzzle_hash).await?;
            p2.insert(
                coin.puzzle_hash,
                P2::Standard(StandardP2::new(StandardLayer::new(synthetic_key))),
            );
        }

        for cat in selection.cats.values() {
            for cat in &cat.coins {
                let synthetic_key = self.db.synthetic_key(cat.p2_puzzle_hash).await?;
                p2.insert(
                    cat.p2_puzzle_hash,
                    P2::Standard(StandardP2::new(StandardLayer::new(synthetic_key))),
                );
            }
        }

        for nft in selection.nfts.values() {
            let synthetic_key = self.db.synthetic_key(nft.info.p2_puzzle_hash).await?;
            p2.insert(
                nft.info.p2_puzzle_hash,
                P2::Standard(StandardP2::new(StandardLayer::new(synthetic_key))),
            );
        }

        for did in selection.dids.values() {
            let synthetic_key = self.db.synthetic_key(did.info.p2_puzzle_hash).await?;
            p2.insert(
                did.info.p2_puzzle_hash,
                P2::Standard(StandardP2::new(StandardLayer::new(synthetic_key))),
            );
        }

        for option in selection.options.values() {
            let synthetic_key = self.db.synthetic_key(option.info.p2_puzzle_hash).await?;
            p2.insert(
                option.info.p2_puzzle_hash,
                P2::Standard(StandardP2::new(StandardLayer::new(synthetic_key))),
            );
        }

        Ok(p2)
    }

    fn initial_spends(selection: &Selection, p2: &HashMap<Bytes32, P2>) -> Spends {
        let xch = FungibleAsset::new(
            selection
                .xch
                .coins
                .iter()
                .map(|&coin| AssetCoin::new(coin, p2[&coin.puzzle_hash].clone()))
                .collect(),
        );

        let cats = selection
            .cats
            .iter()
            .map(|(&id, selected)| {
                let spends = FungibleAsset::new(
                    selected
                        .coins
                        .iter()
                        .map(|&cat| AssetCoin::new(cat, p2[&cat.p2_puzzle_hash].clone()))
                        .collect(),
                );
                (id, spends)
            })
            .collect();

        let dids = selection
            .dids
            .iter()
            .map(|(&id, &did)| {
                let singleton =
                    SingletonLineage::new(did, p2[&did.info.p2_puzzle_hash].clone(), false, true);
                (id, singleton)
            })
            .collect();

        let nfts = selection
            .nfts
            .iter()
            .map(|(&id, &nft)| {
                let singleton =
                    SingletonLineage::new(nft, p2[&nft.info.p2_puzzle_hash].clone(), false, true);
                (id, singleton)
            })
            .collect();

        let options = selection
            .options
            .iter()
            .map(|(&id, &option)| {
                let singleton = SingletonLineage::new(
                    option,
                    p2[&option.info.p2_puzzle_hash].clone(),
                    false,
                    true,
                );
                (id, singleton)
            })
            .collect();

        Spends {
            xch,
            cats,
            dids,
            nfts,
            options,
        }
    }

    async fn send_change(
        &self,
        ctx: &mut SpendContext,
        summary: &Summary,
        selection: &Selection,
        spends: &mut Spends,
    ) -> Result<(), WalletError> {
        let change_puzzle_hash = self.p2_puzzle_hash(false, true).await?;

        let change_amount =
            (selection.xch.existing_amount + summary.created_xch).saturating_sub(summary.spent_xch);

        if change_amount > 0 {
            spends.xch.create_coin(
                ctx,
                change_puzzle_hash,
                change_amount,
                false,
                None,
                P2Selection::Payment,
            )?;
        }

        for (id, cat) in &mut spends.cats {
            let existing_amount = selection
                .cats
                .get(id)
                .map_or(0, |selected| selected.existing_amount);

            let created_amount = summary.created_cats.get(id).copied().unwrap_or_default();
            let spent_amount = summary.spent_cats.get(id).copied().unwrap_or_default();

            let change_amount = (existing_amount + created_amount).saturating_sub(spent_amount);

            if change_amount > 0 {
                cat.create_coin(
                    ctx,
                    change_puzzle_hash,
                    change_amount,
                    true,
                    None,
                    P2Selection::Payment,
                )?;
            }
        }

        Ok(())
    }

    fn finalize_singletons(ctx: &mut SpendContext, spends: &mut Spends) -> Result<(), WalletError> {
        for lineage in spends.dids.values_mut() {
            if lineage.current().needs_spend() {
                lineage.recreate(ctx)?;
            }
        }

        for lineage in spends.nfts.values_mut() {
            if lineage.current().needs_spend() {
                lineage.recreate(ctx)?;
            }
        }

        for lineage in spends.options.values_mut() {
            if lineage.current().needs_spend() {
                lineage.recreate(ctx)?;
            }
        }

        Ok(())
    }
}
