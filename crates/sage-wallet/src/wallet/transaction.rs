mod action;
mod distribution;
mod id;
mod preselection;
mod selection;

pub use action::*;
pub use distribution::*;
pub use id::*;
pub use preselection::*;
pub use selection::*;

use chia::protocol::{Bytes32, CoinSpend};
use chia_wallet_sdk::driver::SpendContext;

use crate::WalletError;

use super::Wallet;

#[derive(Debug, Default, Clone)]
pub struct TransactionConfig {
    pub actions: Vec<SpendAction>,
    pub preselected_coin_ids: Vec<Bytes32>,
    pub fee: u64,
    pub change_address: Bytes32,
}

impl TransactionConfig {
    pub fn new(actions: Vec<SpendAction>, fee: u64, change_address: Bytes32) -> Self {
        Self {
            actions,
            preselected_coin_ids: Vec::new(),
            fee,
            change_address,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionResult {
    pub coin_spends: Vec<CoinSpend>,
}

impl Wallet {
    pub async fn transact(&self, tx: &TransactionConfig) -> Result<TransactionResult, WalletError> {
        let mut ctx = SpendContext::new();

        let preselection = self.preselect(tx)?;
        let selection = self.select(&mut ctx, &preselection, tx).await?;
        self.distribute(&mut ctx, &preselection, &selection, tx)
            .await?;

        let coin_spends = ctx.take();

        Ok(TransactionResult { coin_spends })
    }
}
