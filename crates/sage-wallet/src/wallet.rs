use std::collections::{HashMap, HashSet};

use chia::{
    bls::PublicKey,
    protocol::{Bytes32, Coin},
};
use chia_wallet_sdk::{
    driver::{Action, Cat, CatInfo, Deltas, Id, Outputs, SpendContext, Spends},
    utils::select_coins,
};
use indexmap::{indexmap, IndexMap};
use itertools::Itertools;
use sage_database::{CoinKind, Database};

mod cat_spends;
mod cats;
mod coin_management;
mod derivations;
mod dids;
mod memos;
mod multi_send;
mod nfts;
mod offer;
mod p2_send;
mod p2_spends;
mod signing;

pub use multi_send::*;
pub use nfts::*;
pub use offer::*;

use crate::WalletError;

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

    pub(crate) async fn select_p2_coins(&self, amount: u64) -> Result<Vec<Coin>, WalletError> {
        self.select_p2_coins_without(amount, &HashSet::new()).await
    }

    async fn select_p2_coins_without(
        &self,
        amount: u64,
        selected_coin_ids: &HashSet<Bytes32>,
    ) -> Result<Vec<Coin>, WalletError> {
        let mut spendable_coins = self.db.spendable_coins().await?;
        spendable_coins.retain(|coin| !selected_coin_ids.contains(&coin.coin_id()));

        Ok(select_coins(spendable_coins, amount)?)
    }

    pub(crate) async fn select_cat_coins(
        &self,
        asset_id: Bytes32,
        amount: u64,
    ) -> Result<Vec<Cat>, WalletError> {
        self.select_cat_coins_without(asset_id, amount, &HashSet::new())
            .await
    }

    async fn select_cat_coins_without(
        &self,
        asset_id: Bytes32,
        amount: u64,
        selected_coin_ids: &HashSet<Bytes32>,
    ) -> Result<Vec<Cat>, WalletError> {
        let mut cat_coins = self.db.spendable_cat_coins(asset_id).await?;
        cat_coins.retain(|cat| !selected_coin_ids.contains(&cat.coin.coin_id()));

        let mut cats = HashMap::with_capacity(cat_coins.len());
        let mut spendable_coins = Vec::with_capacity(cat_coins.len());

        for cat in &cat_coins {
            cats.insert(
                cat.coin,
                Cat::new(
                    cat.coin,
                    Some(cat.lineage_proof),
                    CatInfo::new(asset_id, None, cat.p2_puzzle_hash),
                ),
            );
            spendable_coins.push(cat.coin);
        }

        Ok(select_coins(spendable_coins, amount)?
            .into_iter()
            .map(|coin| cats[&coin])
            .collect())
    }

    pub async fn spend(
        &self,
        ctx: &mut SpendContext,
        selected_coin_ids: Vec<Bytes32>,
        actions: &[Action],
    ) -> Result<Outputs, WalletError> {
        let mut spends = self.prepare_spends(ctx, selected_coin_ids, actions).await?;
        let deltas = spends.apply(ctx, actions)?;
        self.complete_spends(ctx, &deltas, spends).await
    }

    pub async fn prepare_spends_for_selection(
        &self,
        ctx: &mut SpendContext,
        selected_coin_ids: &[Bytes32],
    ) -> Result<Spends, WalletError> {
        let self_puzzle_hash = self.p2_puzzle_hash(false, true).await?;

        let mut spends = Spends::new(self_puzzle_hash);

        for &coin_id in selected_coin_ids {
            let Some(row) = self.db.full_coin_state(coin_id).await? else {
                return Err(WalletError::MissingCoin(coin_id));
            };

            let coin = row.coin_state.coin;

            match row.kind {
                CoinKind::Xch => {
                    spends.add(coin);
                }
                CoinKind::Cat => {
                    let Some(cat) = self.db.cat_coin(coin_id).await? else {
                        return Err(WalletError::MissingCoin(coin_id));
                    };
                    spends.add(cat);
                }
                CoinKind::Did => {
                    let Some(did) = self.db.did_by_coin_id(coin_id).await? else {
                        return Err(WalletError::MissingCoin(coin_id));
                    };
                    let metadata_ptr = ctx.alloc_hashed(&did.info.metadata)?;
                    spends.add(did.with_metadata(metadata_ptr));
                }
                CoinKind::Nft => {
                    let Some(nft) = self.db.nft_by_coin_id(coin_id).await? else {
                        return Err(WalletError::MissingCoin(coin_id));
                    };
                    let metadata_ptr = ctx.alloc_hashed(&nft.info.metadata)?;
                    spends.add(nft.with_metadata(metadata_ptr));
                }
                CoinKind::Unknown => {
                    return Err(WalletError::MissingCoin(coin_id));
                }
            }
        }

        Ok(spends)
    }

    pub async fn prepare_spends(
        &self,
        ctx: &mut SpendContext,
        selected_coin_ids: Vec<Bytes32>,
        actions: &[Action],
    ) -> Result<Spends, WalletError> {
        let mut spends = self
            .prepare_spends_for_selection(ctx, &selected_coin_ids)
            .await?;

        self.select_spends(ctx, &mut spends, selected_coin_ids, actions)
            .await?;

        Ok(spends)
    }

    pub async fn select_spends(
        &self,
        ctx: &mut SpendContext,
        spends: &mut Spends,
        selected_coin_ids: Vec<Bytes32>,
        actions: &[Action],
    ) -> Result<(), WalletError> {
        let deltas = Deltas::from_actions(actions);

        for &id in deltas.ids() {
            let Id::Existing(launcher_id) = id else {
                continue;
            };

            if let Some(did) = self.db.spendable_did(launcher_id).await? {
                let metadata_ptr = ctx.alloc_hashed(&did.info.metadata)?;
                spends.add(did.with_metadata(metadata_ptr));
            } else if let Some(nft) = self.db.spendable_nft(launcher_id).await? {
                let metadata_ptr = ctx.alloc_hashed(&nft.info.metadata)?;
                spends.add(nft.with_metadata(metadata_ptr));
            }
        }

        let mut selected = indexmap! { None => spends.xch.selected_amount() };

        for cat in spends.cats.values() {
            if cat.items.is_empty() {
                continue;
            }

            let asset_id = cat.items[0].asset.info.asset_id;

            *selected.entry(Some(asset_id)).or_insert(0) += cat.selected_amount();
        }

        let requested = deltas
            .ids()
            .filter_map(|&id| {
                let Id::Existing(asset_id) = id else {
                    return None;
                };

                if selected.contains_key(&Some(asset_id)) {
                    return None;
                }

                Some((Some(asset_id), 0))
            })
            .collect_vec();

        let selected_coin_ids: HashSet<Bytes32> = selected_coin_ids.into_iter().collect();

        for (asset_id, amount) in selected.into_iter().chain(requested) {
            let id = asset_id.map(Id::Existing);

            if let Some(id) = &id {
                if spends.dids.contains_key(id)
                    || spends.nfts.contains_key(id)
                    || spends.options.contains_key(id)
                {
                    continue;
                }
            }

            let id = id.unwrap_or(Id::Xch);

            let delta = deltas.get(&id).copied().unwrap_or_default();
            let required_amount = delta.output.saturating_sub(amount + delta.input);

            if required_amount > 0 || deltas.is_needed(&id) {
                if let Some(asset_id) = asset_id {
                    for cat in self
                        .select_cat_coins_without(asset_id, required_amount, &selected_coin_ids)
                        .await?
                    {
                        spends.add(cat);
                    }
                } else {
                    for coin in self
                        .select_p2_coins_without(required_amount, &selected_coin_ids)
                        .await?
                    {
                        spends.add(coin);
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn complete_spends(
        &self,
        ctx: &mut SpendContext,
        deltas: &Deltas,
        spends: Spends,
    ) -> Result<Outputs, WalletError> {
        let mut keys = IndexMap::new();

        for p2_puzzle_hash in spends.p2_puzzle_hashes() {
            let key = self.db.synthetic_key(p2_puzzle_hash).await?;
            keys.insert(p2_puzzle_hash, key);
        }

        Ok(spends.finish_with_keys(ctx, deltas, &keys)?)
    }
}
