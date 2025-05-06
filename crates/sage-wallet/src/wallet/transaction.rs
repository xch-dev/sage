mod action;
mod id;
mod selection;
mod spends;
mod summary;

use std::collections::HashMap;

pub use action::*;
pub use id::*;
use itertools::Itertools;
pub use selection::*;
pub use spends::*;
pub use summary::*;

use chia::{
    clvm_traits::FromClvm,
    protocol::{Bytes32, Coin, CoinSpend, Program},
};
use chia_wallet_sdk::driver::{Cat, Did, Nft, OptionContract, SpendContext};
use clvmr::Allocator;
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

#[derive(Debug, Default, Clone)]
pub struct UnspentAssets {
    pub xch: Vec<Coin>,
    pub cats: IndexMap<Id, Vec<Cat>>,
    pub nfts: IndexMap<Id, Nft<Program>>,
    pub dids: IndexMap<Id, Did<Program>>,
    pub options: IndexMap<Id, OptionContract>,
}

impl UnspentAssets {
    pub fn from_spends(allocator: &Allocator, spends: &Spends) -> Result<Self, WalletError> {
        Ok(Self {
            xch: spends
                .xch
                .items
                .iter()
                .flat_map(|item| {
                    item.payments.iter().map(|payment| {
                        Coin::new(item.coin.coin_id(), payment.puzzle_hash, payment.amount)
                    })
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
                            .collect_vec(),
                    )
                })
                .filter(|(_, cats)| !cats.is_empty())
                .collect(),
            nfts: spends
                .nfts
                .iter()
                .filter(|(_, item)| item.was_created())
                .map(|(&id, item)| {
                    let nft = item.coin();
                    Ok((
                        id,
                        nft.with_metadata(Program::from_clvm(allocator, nft.info.metadata.ptr())?),
                    ))
                })
                .collect::<Result<_, WalletError>>()?,
            dids: spends
                .dids
                .iter()
                .filter(|(_, item)| item.was_created())
                .map(|(&id, item)| {
                    let did = item.coin();
                    Ok((
                        id,
                        did.with_metadata(Program::from_clvm(allocator, did.info.metadata.ptr())?),
                    ))
                })
                .collect::<Result<_, WalletError>>()?,
            options: spends
                .options
                .iter()
                .filter(|(_, item)| item.was_created())
                .map(|(&id, item)| (id, item.coin()))
                .collect(),
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
        ids.insert(id, lineage.coin().info.launcher_id);
    }

    for (&id, lineage) in &spends.dids {
        ids.insert(id, lineage.coin().info.launcher_id);
    }

    for (&id, lineage) in &spends.options {
        ids.insert(id, lineage.coin().info.launcher_id);
    }

    ids
}

impl Wallet {
    pub async fn transact_preselected(
        &self,
        ctx: &mut SpendContext,
        tx: &mut TransactionConfig,
    ) -> Result<Spends, WalletError> {
        let summary = self.summarize(tx)?;
        self.select(ctx, &mut tx.preselection, &summary).await?;
        self.spend(ctx, &summary, &tx.preselection, tx).await
    }

    pub async fn transact_with_coin_ids(
        &self,
        coin_ids: Vec<Bytes32>,
        actions: Vec<SpendAction>,
        fee: u64,
    ) -> Result<TransactionResult, WalletError> {
        let mut ctx = SpendContext::new();

        let preselection = self.preselect(&mut ctx, coin_ids).await?;
        let mut tx = TransactionConfig::new_preselected(actions, preselection, fee);

        let summary = self.summarize(&tx)?;

        self.select(&mut ctx, &mut tx.preselection, &summary)
            .await?;

        let spends = self
            .spend(&mut ctx, &summary, &tx.preselection, &tx)
            .await?;

        let ids = collect_ids(&spends);

        Ok(TransactionResult {
            coin_spends: ctx.take(),
            unspent_assets: UnspentAssets::from_spends(&ctx, &spends)?,
            spends,
            ids,
        })
    }

    pub async fn transact(
        &self,
        actions: Vec<SpendAction>,
        fee: u64,
    ) -> Result<TransactionResult, WalletError> {
        let mut ctx = SpendContext::new();
        let mut tx = TransactionConfig::new(actions, fee);

        let summary = self.summarize(&tx)?;

        self.select(&mut ctx, &mut tx.preselection, &summary)
            .await?;

        let spends = self
            .spend(&mut ctx, &summary, &tx.preselection, &tx)
            .await?;

        let ids = collect_ids(&spends);

        Ok(TransactionResult {
            coin_spends: ctx.take(),
            unspent_assets: UnspentAssets::from_spends(&ctx, &spends)?,
            spends,
            ids,
        })
    }
}
