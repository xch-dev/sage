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
    pub amount: u64,
}

impl<T> Default for Selected<T> {
    fn default() -> Self {
        Self {
            coins: Vec::new(),
            amount: 0,
        }
    }
}

impl Wallet {
    pub async fn select_transaction(
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
                    selection.xch.amount += row.coin_state.coin.amount;
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
                    existing.amount += cat.coin.amount;
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
            .saturating_sub(selection.xch.amount);

        if xch_deficit > 0 || (selection.xch.coins.is_empty() && preselection.needs_xch_parent) {
            let mut spendable_coins = self.db.spendable_coins().await?;
            spendable_coins.retain(|coin| !selection.xch.coins.contains(coin));

            for coin in select_coins(spendable_coins, xch_deficit as u128)? {
                selection.xch.coins.push(coin);
                selection.xch.amount += coin.amount;
            }
        }

        for (&id, selected) in &mut selection.cats {
            let required = selection.spent_cats.get(&id).copied().unwrap_or_default();

            if required >= 0 && required as u64 > selected.amount {
                if let Id::Existing(asset_id) = id {
                    let missing = required as u64 - selected.amount;

                    let cats = self.select_cat_coins(asset_id, missing as u128).await?;

                    for cat in cats {
                        selected.coins.push(cat);
                        selected.amount += cat.coin.amount;
                    }
                }
            }
        }

        Ok(selection)
    }
}
