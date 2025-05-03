use std::collections::HashMap;

use chia::protocol::Coin;
use chia_wallet_sdk::{
    driver::{Cat, Did, HashedPtr, Nft, OptionContract, SpendContext},
    utils::select_coins,
};
use indexmap::IndexSet;
use sage_database::CoinKind;

use crate::{Wallet, WalletError};

use super::{Id, Preselection, TransactionConfig};

#[derive(Debug, Default, Clone)]
pub struct Selection {
    pub xch: Selected<Coin>,
    pub cats: HashMap<Id, Selected<Cat>>,
    pub nfts: HashMap<Id, Nft<HashedPtr>>,
    pub dids: HashMap<Id, Did<HashedPtr>>,
    pub options: HashMap<Id, OptionContract>,
}

#[derive(Debug, Clone)]
pub struct Selected<T> {
    pub coins: Vec<T>,
    pub existing_amount: u64,
}

impl<T> Default for Selected<T> {
    fn default() -> Self {
        Self {
            coins: Vec::new(),
            existing_amount: 0,
        }
    }
}

impl Wallet {
    pub async fn select(
        &self,
        ctx: &mut SpendContext,
        preselection: &Preselection,
        tx: &TransactionConfig,
    ) -> Result<Selection, WalletError> {
        let mut selection = Selection::default();

        for &coin_id in tx.preselected_coin_ids.iter().collect::<IndexSet<_>>() {
            let Some(row) = self.db.full_coin_state(coin_id).await? else {
                return Err(WalletError::MissingCoin(coin_id));
            };

            match row.kind {
                CoinKind::Unknown => {
                    return Err(WalletError::MissingCoin(coin_id));
                }
                CoinKind::Xch => {
                    selection.xch.coins.push(row.coin_state.coin);
                    selection.xch.existing_amount += row.coin_state.coin.amount;
                }
                CoinKind::Cat => {
                    let Some(cat) = self.db.cat_coin(coin_id).await? else {
                        return Err(WalletError::MissingCoin(coin_id));
                    };

                    let existing = selection
                        .cats
                        .entry(Id::Existing(cat.asset_id))
                        .or_default();

                    existing.coins.push(cat);
                    existing.existing_amount += cat.coin.amount;
                }
                CoinKind::Nft => {
                    let Some(nft) = self.db.nft_by_coin_id(coin_id).await? else {
                        return Err(WalletError::MissingCoin(coin_id));
                    };

                    let metadata_ptr = ctx.alloc(&nft.info.metadata)?;
                    let metadata = HashedPtr::from_ptr(ctx, metadata_ptr);
                    let nft = nft.with_metadata(metadata);

                    selection
                        .nfts
                        .insert(Id::Existing(nft.info.launcher_id), nft);
                }
                CoinKind::Did => {
                    let Some(did) = self.db.did_by_coin_id(coin_id).await? else {
                        return Err(WalletError::MissingCoin(coin_id));
                    };

                    let metadata_ptr = ctx.alloc(&did.info.metadata)?;
                    let metadata = HashedPtr::from_ptr(ctx, metadata_ptr);
                    let did = did.with_metadata(metadata);

                    selection
                        .dids
                        .insert(Id::Existing(did.info.launcher_id), did);
                }
                CoinKind::Option => {
                    let Some(option) = self.db.option_by_coin_id(coin_id).await? else {
                        return Err(WalletError::MissingCoin(coin_id));
                    };

                    selection
                        .options
                        .insert(Id::Existing(option.info.launcher_id), option);
                }
            }
        }

        let xch_deficit = preselection
            .spent_xch
            .saturating_sub(preselection.created_xch)
            .saturating_sub(selection.xch.existing_amount);

        if xch_deficit > 0 || (selection.xch.coins.is_empty() && preselection.spent_xch > 0) {
            let mut spendable_coins = self.db.spendable_coins().await?;
            spendable_coins.retain(|coin| !selection.xch.coins.contains(coin));

            for coin in select_coins(spendable_coins, xch_deficit as u128)? {
                selection.xch.coins.push(coin);
                selection.xch.existing_amount += coin.amount;
            }
        }

        for (id, spent) in preselection.spent_cats.clone() {
            let selected = selection.cats.entry(id).or_default();

            let created = preselection
                .created_cats
                .get(&id)
                .copied()
                .unwrap_or_default();

            let deficit = spent
                .saturating_sub(created)
                .saturating_sub(selected.existing_amount);

            if deficit > 0 {
                if let Id::Existing(asset_id) = id {
                    let mut rows = self.db.spendable_cat_coins(asset_id).await?;
                    rows.retain(|cat| !selected.coins.iter().any(|c| c.coin == cat.coin));

                    let selected_coins =
                        select_coins(rows.iter().map(|r| r.coin).collect(), deficit as u128)?;

                    for coin in selected_coins {
                        let row = rows
                            .iter()
                            .find(|r| r.coin == coin)
                            .copied()
                            .expect("missing row");

                        let cat = Cat::new(
                            row.coin,
                            Some(row.lineage_proof),
                            asset_id,
                            row.p2_puzzle_hash,
                        );

                        selected.coins.push(cat);
                        selected.existing_amount += cat.coin.amount;
                    }
                }
            }
        }

        for &id in preselection
            .spent_nfts
            .difference(&preselection.created_nfts)
        {
            if selection.nfts.contains_key(&id) {
                continue;
            }

            match id {
                Id::Existing(launcher_id) => {
                    let Some(nft) = self.db.spendable_nft(launcher_id).await? else {
                        return Err(WalletError::MissingNft(launcher_id));
                    };

                    let metadata_ptr = ctx.alloc(&nft.info.metadata)?;
                    let metadata = HashedPtr::from_ptr(ctx, metadata_ptr);
                    let nft = nft.with_metadata(metadata);

                    selection.nfts.insert(id, nft);
                }
                Id::New(id) => {
                    return Err(WalletError::InvalidNewId(id));
                }
            }
        }

        for &id in preselection
            .spent_dids
            .difference(&preselection.created_dids)
        {
            if selection.dids.contains_key(&id) {
                continue;
            }

            match id {
                Id::Existing(launcher_id) => {
                    let Some(did) = self.db.spendable_did(launcher_id).await? else {
                        return Err(WalletError::MissingDid(launcher_id));
                    };

                    let metadata_ptr = ctx.alloc(&did.info.metadata)?;
                    let metadata = HashedPtr::from_ptr(ctx, metadata_ptr);
                    let did = did.with_metadata(metadata);

                    selection.dids.insert(id, did);
                }
                Id::New(id) => {
                    return Err(WalletError::InvalidNewId(id));
                }
            }
        }

        for &id in preselection
            .spent_options
            .difference(&preselection.created_options)
        {
            if selection.options.contains_key(&id) {
                continue;
            }

            match id {
                Id::Existing(launcher_id) => {
                    let Some(option) = self.db.spendable_option(launcher_id).await? else {
                        return Err(WalletError::MissingOption(launcher_id));
                    };

                    selection.options.insert(id, option);
                }
                Id::New(id) => {
                    return Err(WalletError::InvalidNewId(id));
                }
            }
        }

        Ok(selection)
    }
}
