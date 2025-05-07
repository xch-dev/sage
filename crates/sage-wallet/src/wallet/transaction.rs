mod action;
mod id;
mod selection;
mod spends;
mod summary;

use std::collections::HashMap;

pub use action::*;
use clvmr::Allocator;
pub use id::*;
use itertools::Itertools;
pub use selection::*;
pub use spends::*;
pub use summary::*;

use chia::{
    clvm_traits::FromClvm,
    protocol::{Bytes32, Coin, CoinSpend, Program},
};
use chia_wallet_sdk::driver::{Cat, Did, HashedPtr, Nft, OptionContract, SpendContext};
use indexmap::IndexMap;

use crate::WalletError;

use super::Wallet;

#[derive(Debug, Default, Clone)]
pub struct TransactionConfig {
    pub actions: Vec<SpendAction>,
    pub preselection: Selection,
    pub fee: u64,
}

impl TransactionConfig {
    pub fn new(actions: Vec<SpendAction>, fee: u64) -> Self {
        Self {
            actions,
            preselection: Selection::default(),
            fee,
        }
    }

    pub fn new_preselected(actions: Vec<SpendAction>, preselection: Selection, fee: u64) -> Self {
        Self {
            actions,
            preselection,
            fee,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TransactionResult {
    pub coin_spends: Vec<CoinSpend>,
    pub unspent_assets: UnspentAssets,
    pub spends: Spends,
    pub ids: HashMap<Id, Bytes32>,
}

#[derive(Debug, Clone)]
pub struct OwnedTransactionResult {
    pub coin_spends: Vec<CoinSpend>,
    pub unspent_assets: OwnedUnspentAssets,
    pub ids: HashMap<Id, Bytes32>,
}

#[derive(Debug, Default, Clone)]
pub struct UnspentAssets {
    pub xch: Vec<Coin>,
    pub cats: IndexMap<Id, Vec<Cat>>,
    pub nfts: IndexMap<Id, Nft<HashedPtr>>,
    pub dids: IndexMap<Id, Did<HashedPtr>>,
    pub options: IndexMap<Id, OptionContract>,
}

impl UnspentAssets {
    pub fn from_spends(spends: &Spends) -> Self {
        Self {
            xch: spends
                .xch
                .items
                .iter()
                .flat_map(|item| {
                    item.payments.iter().map(|payment| {
                        Coin::new(item.coin.coin_id(), payment.puzzle_hash, payment.amount)
                    })
                })
                .filter(|coin| {
                    !spends
                        .xch
                        .items
                        .iter()
                        .any(|item| item.coin.coin_id() == coin.coin_id())
                })
                .collect_vec(),
            cats: spends
                .cats
                .iter()
                .map(|(&id, spend)| {
                    (
                        id,
                        spend
                            .items
                            .iter()
                            .flat_map(|item| {
                                item.payments.iter().map(|payment| {
                                    item.coin.wrapped_child(payment.puzzle_hash, payment.amount)
                                })
                            })
                            .filter(|cat| {
                                !spend
                                    .items
                                    .iter()
                                    .any(|item| item.coin.coin.coin_id() == cat.coin.coin_id())
                            })
                            .collect_vec(),
                    )
                })
                .filter(|(_, cats)| !cats.is_empty())
                .collect(),
            nfts: spends
                .nfts
                .iter()
                .filter(|(_, item)| item.current().p2().is_empty())
                .map(|(&id, item)| (id, item.last_coin()))
                .collect(),
            dids: spends
                .dids
                .iter()
                .filter(|(_, item)| item.current().p2().is_empty())
                .map(|(&id, item)| (id, item.last_coin()))
                .collect(),
            options: spends
                .options
                .iter()
                .filter(|(_, item)| item.current().p2().is_empty())
                .map(|(&id, item)| (id, item.last_coin()))
                .collect(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct OwnedUnspentAssets {
    pub xch: Vec<Coin>,
    pub cats: IndexMap<Id, Vec<Cat>>,
    pub nfts: IndexMap<Id, Nft<Program>>,
    pub dids: IndexMap<Id, Did<Program>>,
    pub options: IndexMap<Id, OptionContract>,
}

impl OwnedUnspentAssets {
    pub fn from_unspent(
        allocator: &Allocator,
        unspent: UnspentAssets,
    ) -> Result<Self, WalletError> {
        let mut nfts = IndexMap::new();

        for (id, nft) in unspent.nfts {
            nfts.insert(
                id,
                nft.with_metadata(Program::from_clvm(allocator, nft.info.metadata.ptr())?),
            );
        }

        let mut dids = IndexMap::new();

        for (id, did) in unspent.dids {
            dids.insert(
                id,
                did.with_metadata(Program::from_clvm(allocator, did.info.metadata.ptr())?),
            );
        }

        Ok(Self {
            xch: unspent.xch,
            cats: unspent.cats,
            nfts,
            dids,
            options: unspent.options,
        })
    }
}

fn collect_ids(spends: &Spends) -> HashMap<Id, Bytes32> {
    let mut ids = HashMap::new();

    for (&id, spends) in &spends.cats {
        if let Some(item) = spends.items.first() {
            ids.insert(id, item.coin.asset_id);
        }
    }

    for (&id, lineage) in &spends.nfts {
        ids.insert(id, lineage.last_coin().info.launcher_id);
    }

    for (&id, lineage) in &spends.dids {
        ids.insert(id, lineage.last_coin().info.launcher_id);
    }

    for (&id, lineage) in &spends.options {
        ids.insert(id, lineage.last_coin().info.launcher_id);
    }

    ids
}

impl Wallet {
    pub async fn transact_preselected_alloc(
        &self,
        ctx: &mut SpendContext,
        tx: &mut TransactionConfig,
    ) -> Result<TransactionResult, WalletError> {
        let summary = self.summarize(tx)?;
        self.select(ctx, &mut tx.preselection, &summary).await?;
        let spends = self.spend(ctx, &summary, &tx.preselection, tx).await?;
        let ids = collect_ids(&spends);
        Ok(TransactionResult {
            coin_spends: ctx.take(),
            unspent_assets: UnspentAssets::from_spends(&spends),
            spends,
            ids,
        })
    }

    pub async fn transact_with_coin_ids_alloc(
        &self,
        ctx: &mut SpendContext,
        coin_ids: Vec<Bytes32>,
        actions: Vec<SpendAction>,
        fee: u64,
    ) -> Result<TransactionResult, WalletError> {
        let preselection = self.preselect(ctx, coin_ids).await?;
        let mut tx = TransactionConfig::new_preselected(actions, preselection, fee);

        let summary = self.summarize(&tx)?;

        self.select(ctx, &mut tx.preselection, &summary).await?;

        let spends = self.spend(ctx, &summary, &tx.preselection, &tx).await?;

        let ids = collect_ids(&spends);

        Ok(TransactionResult {
            coin_spends: ctx.take(),
            unspent_assets: UnspentAssets::from_spends(&spends),
            spends,
            ids,
        })
    }

    pub async fn transact_alloc(
        &self,
        ctx: &mut SpendContext,
        actions: Vec<SpendAction>,
        fee: u64,
    ) -> Result<TransactionResult, WalletError> {
        let mut tx = TransactionConfig::new(actions, fee);

        let summary = self.summarize(&tx)?;

        self.select(ctx, &mut tx.preselection, &summary).await?;

        let spends = self.spend(ctx, &summary, &tx.preselection, &tx).await?;

        let ids = collect_ids(&spends);

        Ok(TransactionResult {
            coin_spends: ctx.take(),
            unspent_assets: UnspentAssets::from_spends(&spends),
            spends,
            ids,
        })
    }

    pub async fn transact_preselected(
        &self,
        tx: &mut TransactionConfig,
    ) -> Result<OwnedTransactionResult, WalletError> {
        let mut ctx = SpendContext::new();
        let result = self.transact_preselected_alloc(&mut ctx, tx).await?;
        let unspent = OwnedUnspentAssets::from_unspent(&ctx, result.unspent_assets)?;
        Ok(OwnedTransactionResult {
            coin_spends: result.coin_spends,
            unspent_assets: unspent,
            ids: result.ids,
        })
    }

    pub async fn transact_with_coin_ids(
        &self,
        coin_ids: Vec<Bytes32>,
        actions: Vec<SpendAction>,
        fee: u64,
    ) -> Result<OwnedTransactionResult, WalletError> {
        let mut ctx = SpendContext::new();
        let result = self
            .transact_with_coin_ids_alloc(&mut ctx, coin_ids, actions, fee)
            .await?;
        let unspent = OwnedUnspentAssets::from_unspent(&ctx, result.unspent_assets)?;
        Ok(OwnedTransactionResult {
            coin_spends: result.coin_spends,
            unspent_assets: unspent,
            ids: result.ids,
        })
    }

    pub async fn transact(
        &self,
        actions: Vec<SpendAction>,
        fee: u64,
    ) -> Result<OwnedTransactionResult, WalletError> {
        let mut ctx = SpendContext::new();
        let result = self.transact_alloc(&mut ctx, actions, fee).await?;
        let unspent = OwnedUnspentAssets::from_unspent(&ctx, result.unspent_assets)?;
        Ok(OwnedTransactionResult {
            coin_spends: result.coin_spends,
            unspent_assets: unspent,
            ids: result.ids,
        })
    }
}
