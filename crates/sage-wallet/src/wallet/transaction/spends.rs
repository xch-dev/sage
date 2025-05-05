mod fungible_asset;
mod singleton;

pub use fungible_asset::*;
pub use singleton::*;

use std::collections::HashMap;

use chia::protocol::Coin;
use chia_wallet_sdk::driver::{
    Cat, Did, HashedPtr, Nft, OptionContract, SpendContext, StandardLayer,
};
use indexmap::IndexMap;
use itertools::Itertools;

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
        let change_puzzle_hash = self.p2_puzzle_hash(false, true).await?;

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

        let mut spends = Spends {
            xch,
            cats,
            dids,
            nfts,
            options,
        };

        for (index, action) in tx.actions.iter().enumerate() {
            action.spend(ctx, &mut spends, index)?;
        }

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

        let xch_spends = spends
            .xch
            .items
            .iter()
            .map(|spend| (spend.coin.coin(), spend.conditions.clone()))
            .collect_vec();

        self.spend_p2_coins_separately(ctx, xch_spends.into_iter())
            .await?;

        for cat in spends.cats.values() {
            let cat_spends = cat
                .items
                .iter()
                .map(|spend| (spend.coin, spend.conditions.clone()))
                .collect_vec();

            self.spend_cat_coins(ctx, cat_spends.into_iter()).await?;
        }

        for lineage in spends.dids.values_mut() {
            lineage.recreate(ctx)?;

            for item in lineage.iter() {
                if item.has_conditions() {
                    item.spend(ctx)?;
                }
            }
        }

        for lineage in spends.nfts.values_mut() {
            lineage.recreate(ctx)?;

            for item in lineage.iter() {
                if item.has_conditions() {
                    item.spend(ctx)?;
                }
            }
        }

        for lineage in spends.options.values_mut() {
            lineage.recreate(ctx)?;

            for item in lineage.iter() {
                if item.has_conditions() {
                    item.spend(ctx)?;
                }
            }
        }

        Ok(spends)
    }
}
