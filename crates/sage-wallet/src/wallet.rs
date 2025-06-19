use std::collections::HashMap;

use chia::{
    bls::PublicKey,
    protocol::{Bytes32, Coin},
};
use chia_wallet_sdk::{
    driver::{Action, Cat, CatInfo, Deltas, Id, Outputs, SpendContext, Spends},
    utils::select_coins,
};
use indexmap::IndexMap;
use sage_database::{CoinKind, Database};

mod cat_coin_management;
mod cat_spends;
mod cats;
mod derivations;
mod did_assign;
mod dids;
mod memos;
mod multi_send;
mod nfts;
mod offer;
mod p2_coin_management;
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
        let spendable_coins = self.db.spendable_coins().await?;
        Ok(select_coins(spendable_coins, amount)?)
    }

    pub(crate) async fn select_cat_coins(
        &self,
        asset_id: Bytes32,
        amount: u64,
    ) -> Result<Vec<Cat>, WalletError> {
        let cat_coins = self.db.spendable_cat_coins(asset_id).await?;

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

    pub async fn prepare_spends(
        &self,
        ctx: &mut SpendContext,
        selected_coin_ids: Vec<Bytes32>,
        actions: &[Action],
    ) -> Result<Spends, WalletError> {
        let self_puzzle_hash = self.p2_puzzle_hash(false, true).await?;
        let deltas = Deltas::from_actions(actions);

        let mut spends = Spends::new(self_puzzle_hash);
        let mut selected = IndexMap::new();

        for coin_id in selected_coin_ids {
            let Some(row) = self.db.full_coin_state(coin_id).await? else {
                return Err(WalletError::MissingCoin(coin_id));
            };

            let coin = row.coin_state.coin;

            match row.kind {
                CoinKind::Xch => {
                    spends.add(coin);
                    *selected.entry(None).or_insert(0) += coin.amount;
                }
                CoinKind::Cat => {
                    let Some(cat) = self.db.cat_coin(coin_id).await? else {
                        return Err(WalletError::MissingCoin(coin_id));
                    };
                    spends.add(cat);
                    *selected.entry(Some(cat.info.asset_id)).or_insert(0) += coin.amount;
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

        for (asset_id, amount) in selected {
            let id = asset_id.map(Id::Existing);
            let delta = deltas.get(id).copied().unwrap_or_default();
            let required_amount = delta.output.saturating_sub(amount + delta.input);

            if required_amount > 0 || deltas.is_needed(id) {
                if let Some(asset_id) = asset_id {
                    for cat in self.select_cat_coins(asset_id, required_amount).await? {
                        spends.add(cat);
                    }
                } else {
                    for coin in self.select_p2_coins(required_amount).await? {
                        spends.add(coin);
                    }
                }
            }
        }

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

        Ok(spends)
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
