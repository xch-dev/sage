mod fungible_asset;
mod singleton;

pub use fungible_asset::*;
pub use singleton::*;

use std::collections::HashMap;

use chia::protocol::{Bytes32, Coin};
use chia_wallet_sdk::driver::{
    Cat, CatSpend, Did, HashedPtr, Nft, OptionContract, SpendContext, SpendWithConditions,
    StandardLayer,
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

        self.send_change(ctx, summary, selection, &mut spends)
            .await?;

        Self::finalize_singletons(ctx, &mut spends)?;

        for item in &spends.xch.items {
            item.p2.spend(ctx, item.coin, item.conditions.clone())?;
        }

        for cat in spends.cats.values() {
            let mut cat_spends = Vec::new();

            for item in &cat.items {
                cat_spends.push(CatSpend::new(
                    item.coin,
                    item.p2
                        .spend_with_conditions(ctx, item.conditions.clone())?,
                ));
            }

            Cat::spend_all(ctx, &cat_spends)?;
        }

        for lineage in spends.dids.values_mut() {
            for item in lineage.iter() {
                if item.has_conditions() {
                    item.spend(ctx)?;
                }
            }
        }

        for lineage in spends.nfts.values_mut() {
            for item in lineage.iter() {
                if item.has_conditions() {
                    item.spend(ctx)?;
                }
            }
        }

        for lineage in spends.options.values_mut() {
            for item in lineage.iter() {
                if item.has_conditions() {
                    item.spend(ctx)?;
                }
            }
        }

        Ok(spends)
    }

    // TODO: Improve how the p2 is fetched and make it work on the fly
    async fn fetch_p2_map(
        &self,
        selection: &Selection,
    ) -> Result<HashMap<Bytes32, StandardLayer>, WalletError> {
        let mut p2 = HashMap::new();

        for p2_puzzle_hash in selection
            .xch
            .coins
            .iter()
            .map(|coin| coin.puzzle_hash)
            .chain(
                selection
                    .cats
                    .values()
                    .flat_map(|selected| selected.coins.iter().map(|cat| cat.p2_puzzle_hash)),
            )
            .chain(selection.nfts.values().map(|nft| nft.info.p2_puzzle_hash))
            .chain(selection.dids.values().map(|did| did.info.p2_puzzle_hash))
            .chain(
                selection
                    .options
                    .values()
                    .map(|option| option.info.p2_puzzle_hash),
            )
        {
            let synthetic_key = self.db.synthetic_key(p2_puzzle_hash).await?;
            p2.insert(p2_puzzle_hash, StandardLayer::new(synthetic_key));
        }

        Ok(p2)
    }

    fn initial_spends(selection: &Selection, p2: &HashMap<Bytes32, StandardLayer>) -> Spends {
        let xch = FungibleAsset::new(
            selection
                .xch
                .coins
                .iter()
                .map(|&coin| AssetCoin::new(coin, p2[&coin.puzzle_hash]))
                .collect(),
            false,
        );

        let cats = selection
            .cats
            .iter()
            .map(|(&id, selected)| {
                let spends = FungibleAsset::new(
                    selected
                        .coins
                        .iter()
                        .map(|&cat| AssetCoin::new(cat, p2[&cat.p2_puzzle_hash]))
                        .collect(),
                    false,
                );
                (id, spends)
            })
            .collect();

        let dids = selection
            .dids
            .iter()
            .map(|(&id, &did)| {
                let singleton = SingletonLineage::new(did, p2[&did.info.p2_puzzle_hash], false);
                (id, singleton)
            })
            .collect();

        let nfts = selection
            .nfts
            .iter()
            .map(|(&id, &nft)| {
                let singleton = SingletonLineage::new(nft, p2[&nft.info.p2_puzzle_hash], false);
                (id, singleton)
            })
            .collect();

        let options = selection
            .options
            .iter()
            .map(|(&id, &option)| {
                let singleton =
                    SingletonLineage::new(option, p2[&option.info.p2_puzzle_hash], false);
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
            spends
                .xch
                .create_coin(ctx, change_puzzle_hash, change_amount, false, None)?;
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
                cat.create_coin(ctx, change_puzzle_hash, change_amount, true, None)?;
            }
        }

        Ok(())
    }

    fn finalize_singletons(ctx: &mut SpendContext, spends: &mut Spends) -> Result<(), WalletError> {
        for lineage in spends.dids.values_mut() {
            lineage.recreate(ctx)?;
        }

        for lineage in spends.nfts.values_mut() {
            lineage.recreate(ctx)?;
        }

        for lineage in spends.options.values_mut() {
            lineage.recreate(ctx)?;
        }

        Ok(())
    }
}
