mod action;
mod id;
mod selection;
mod spends;
mod summary;

pub use action::*;
pub use id::*;
pub use selection::*;
pub use spends::*;
pub use summary::*;

use chia::{
    clvm_traits::FromClvm,
    protocol::{Bytes32, CoinSpend, Program},
};
use chia_wallet_sdk::driver::{Did, Nft, OptionContract, SpendContext};
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
    pub new_assets: NewAssets,
}

#[derive(Debug, Default, Clone)]
pub struct NewAssets {
    pub cats: IndexMap<Id, Bytes32>,
    pub nfts: IndexMap<Id, Nft<Program>>,
    pub dids: IndexMap<Id, Did<Program>>,
    pub options: IndexMap<Id, OptionContract>,
}

impl NewAssets {
    pub fn from_spends(allocator: &Allocator, spends: Spends) -> Result<Self, WalletError> {
        Ok(Self {
            cats: spends
                .cats
                .into_iter()
                .filter_map(|(id, spend)| {
                    if spend.was_created {
                        spend.items.first().and_then(|item| {
                            if let AssetCoin::Cat(cat) = item.coin {
                                Some((id, cat.asset_id))
                            } else {
                                None
                            }
                        })
                    } else {
                        None
                    }
                })
                .collect(),
            nfts: spends
                .nfts
                .into_iter()
                .map(|(id, mut item)| {
                    let nft = item.nft()?.1;
                    Ok((
                        id,
                        nft.with_metadata(Program::from_clvm(allocator, nft.info.metadata.ptr())?),
                    ))
                })
                .collect::<Result<_, WalletError>>()?,
            dids: spends
                .dids
                .into_iter()
                .map(|(id, mut item)| {
                    let did = item.did()?.1;
                    Ok((
                        id,
                        did.with_metadata(Program::from_clvm(allocator, did.info.metadata.ptr())?),
                    ))
                })
                .collect::<Result<_, WalletError>>()?,
            options: spends
                .options
                .into_iter()
                .map(|(id, mut item)| {
                    let option = item.option()?.1;
                    Ok((id, option))
                })
                .collect::<Result<_, WalletError>>()?,
        })
    }
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

        Ok(TransactionResult {
            coin_spends: ctx.take(),
            new_assets: NewAssets::from_spends(&ctx, spends)?,
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

        Ok(TransactionResult {
            coin_spends: ctx.take(),
            new_assets: NewAssets::from_spends(&ctx, spends)?,
        })
    }
}
