use std::collections::{HashMap, HashSet};

use chia::protocol::Coin;
use chia_wallet_sdk::driver::{Cat, Did, HashedPtr, Nft, OptionContract, SpendContext};
use indexmap::IndexSet;
use sage_database::CoinKind;

use crate::{Wallet, WalletError};

use super::{Id, TransactionConfig};

#[derive(Debug, Default, Clone)]
pub struct Selection {
    pub xch: Selected<Coin>,
    pub cats: HashMap<Id, Selected<Cat>>,
    pub nfts: HashMap<Id, Nft<HashedPtr>>,
    pub dids: HashMap<Id, Did<HashedPtr>>,
    pub options: HashMap<Id, OptionContract>,
    pub spent_xch: i64,
    pub spent_cats: HashMap<Id, i64>,
    pub spent_nfts: HashSet<Id>,
    pub spent_dids: HashSet<Id>,
    pub needs_xch_parent: bool,
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

pub trait Select {
    fn select(&self, selection: &mut Selection, index: usize) -> Result<(), WalletError>;
}

impl Wallet {
    pub async fn select_transaction(
        &self,
        ctx: &mut SpendContext,
        tx: &TransactionConfig,
    ) -> Result<Selection, WalletError> {
        let mut selection = Selection {
            spent_xch: tx.fee.try_into()?,
            ..Default::default()
        };

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

        for (index, action) in tx.actions.iter().enumerate() {
            action.select(&mut selection, index)?;
        }

        if selection.spent_xch >= 0
            && (selection.spent_xch as u64 > selection.xch.amount || selection.needs_xch_parent)
        {
            let missing = selection.spent_xch as u64 - selection.xch.amount;

            let coins = self.select_p2_coins(missing as u128).await?;

            for coin in coins {
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
