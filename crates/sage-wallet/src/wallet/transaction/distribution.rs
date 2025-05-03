use std::mem;

use chia::protocol::{Bytes, Bytes32};
use chia_wallet_sdk::{driver::SpendContext, types::Conditions};
use itertools::Itertools;

use crate::{wallet::memos::calculate_memos, WalletError};

use super::Id;

#[derive(Debug)]
pub struct Distribution<'a, T> {
    ctx: &'a mut SpendContext,
    asset_id: Option<Id>,
    coins: Vec<DistributionCoin<T>>,
}

#[derive(Debug, Clone)]
pub struct DistributionCoin<T> {
    coin: T,
    payments: Vec<Payment>,
    conditions: Conditions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Payment {
    pub puzzle_hash: Bytes32,
    pub amount: u64,
}

impl Payment {
    pub fn new(puzzle_hash: Bytes32, amount: u64) -> Self {
        Self {
            puzzle_hash,
            amount,
        }
    }
}

impl<'a, T> Distribution<'a, T> {
    pub fn new(ctx: &'a mut SpendContext, asset_id: Option<Id>) -> Self {
        Self {
            ctx,
            asset_id,
            coins: Vec::new(),
        }
    }

    pub fn asset_id(&self) -> Option<Id> {
        self.asset_id
    }

    pub fn add(
        &mut self,
        payment: Payment,
        f: impl FnOnce(&mut SpendContext, &T, Conditions) -> Result<Conditions, WalletError>,
    ) -> Result<(), WalletError> {
        let Some(item) = self
            .coins
            .iter_mut()
            .sorted_by_key(|c| c.payments.iter().filter(|&p| p == &payment).count())
            .next()
        else {
            return Ok(());
        };

        item.payments.push(payment);

        let conditions = mem::take(&mut item.conditions);
        let new_conditions = f(self.ctx, &item.coin, conditions)?;
        item.conditions = new_conditions;

        Ok(())
    }

    pub fn create_coin(
        &mut self,
        p2_puzzle_hash: Bytes32,
        amount: u64,
        include_hint: bool,
        memos: Option<Vec<Bytes>>,
    ) -> Result<(), WalletError> {
        self.add(
            Payment::new(p2_puzzle_hash, amount),
            |ctx, _coin, conditions| {
                Ok(conditions.create_coin(
                    p2_puzzle_hash,
                    amount,
                    calculate_memos(ctx, p2_puzzle_hash, include_hint, memos)?,
                ))
            },
        )?;

        Ok(())
    }
}
