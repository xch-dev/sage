mod action;
mod distribution;
mod id;
mod selection;
mod summary;

pub use action::*;
use chia::protocol::{Bytes32, CoinSpend};
pub use distribution::*;
pub use id::*;
pub use selection::*;
pub use summary::*;

use chia_wallet_sdk::driver::SpendContext;

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

impl Wallet {
    pub async fn transact_preselected(
        &self,
        ctx: &mut SpendContext,
        tx: &mut TransactionConfig,
    ) -> Result<(), WalletError> {
        let summary = self.summarize(tx)?;
        self.select(ctx, &mut tx.preselection, &summary).await?;
        self.distribute(ctx, &summary, &tx.preselection, tx).await?;
        Ok(())
    }

    pub async fn transact_with_coin_ids(
        &self,
        coin_ids: Vec<Bytes32>,
        actions: Vec<SpendAction>,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();

        let preselection = self.preselect(&mut ctx, coin_ids).await?;
        let mut tx = TransactionConfig::new_preselected(actions, preselection, fee);

        let summary = self.summarize(&tx)?;

        self.select(&mut ctx, &mut tx.preselection, &summary)
            .await?;

        self.distribute(&mut ctx, &summary, &tx.preselection, &tx)
            .await?;

        Ok(ctx.take())
    }

    pub async fn transact(
        &self,

        actions: Vec<SpendAction>,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();
        let mut tx = TransactionConfig::new(actions, fee);

        let summary = self.summarize(&tx)?;

        self.select(&mut ctx, &mut tx.preselection, &summary)
            .await?;

        self.distribute(&mut ctx, &summary, &tx.preselection, &tx)
            .await?;

        Ok(ctx.take())
    }
}
