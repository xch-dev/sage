use std::mem;

use chia::protocol::{Bytes, Bytes32, Coin};
use chia_puzzles::SINGLETON_LAUNCHER_HASH;
use chia_wallet_sdk::{
    driver::{Cat, Launcher, SpendContext},
    types::Conditions,
};
use indexmap::IndexSet;
use itertools::Itertools;

use crate::{wallet::memos::calculate_memos, Wallet, WalletError};

use super::{Action, Id, Preselection, Selection, TransactionConfig};

#[derive(Debug)]
pub struct Distribution<'a> {
    ctx: &'a mut SpendContext,
    asset_id: Option<Id>,
    items: Vec<DistributionItem>,
    launcher_index: usize,
}

#[derive(Debug, Clone)]
pub struct DistributionItem {
    coin: DistributionCoin,
    payments: IndexSet<Payment>,
    conditions: Conditions,
}

impl DistributionItem {
    pub fn new(coin: DistributionCoin) -> Self {
        Self {
            coin,
            payments: IndexSet::new(),
            conditions: Conditions::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistributionCoin {
    Xch(Coin),
    Cat(Cat),
}

impl DistributionCoin {
    #[must_use]
    pub fn child(&self, p2_puzzle_hash: Bytes32, amount: u64) -> Self {
        match self {
            Self::Xch(coin) => Self::Xch(Coin::new(coin.coin_id(), p2_puzzle_hash, amount)),
            Self::Cat(cat) => Self::Cat(cat.wrapped_child(p2_puzzle_hash, amount)),
        }
    }

    pub fn coin(&self) -> Coin {
        match self {
            Self::Xch(coin) => *coin,
            Self::Cat(cat) => cat.coin,
        }
    }

    pub fn p2_puzzle_hash(&self) -> Bytes32 {
        match self {
            Self::Xch(coin) => coin.puzzle_hash,
            Self::Cat(cat) => cat.p2_puzzle_hash,
        }
    }
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

impl<'a> Distribution<'a> {
    pub fn new(
        ctx: &'a mut SpendContext,
        asset_id: Option<Id>,
        items: Vec<DistributionItem>,
    ) -> Self {
        Self {
            ctx,
            asset_id,
            items,
            launcher_index: 0,
        }
    }

    pub fn asset_id(&self) -> Option<Id> {
        self.asset_id
    }

    pub fn add(
        &mut self,
        payment: Payment,
        f: impl FnOnce(
            &mut SpendContext,
            &DistributionCoin,
            Conditions,
        ) -> Result<Conditions, WalletError>,
    ) -> Result<(), WalletError> {
        let item = if let Some(item) = self
            .items
            .iter_mut()
            .find(|item| !item.payments.contains(&payment))
        {
            item
        } else {
            let Some(parent) = self.items.iter_mut().find(|item| {
                !item
                    .payments
                    .contains(&Payment::new(item.coin.p2_puzzle_hash(), 0))
            }) else {
                return Ok(());
            };

            parent
                .payments
                .insert(Payment::new(parent.coin.p2_puzzle_hash(), 0));

            parent.conditions = mem::take(&mut parent.conditions).create_coin(
                parent.coin.p2_puzzle_hash(),
                0,
                calculate_memos(
                    self.ctx,
                    parent.coin.p2_puzzle_hash(),
                    matches!(parent.coin, DistributionCoin::Cat(..)),
                    None,
                )?,
            );

            let child = parent.coin.child(parent.coin.p2_puzzle_hash(), 0);
            self.items.push(DistributionItem::new(child));
            self.items.last_mut().expect("item should exist")
        };

        item.payments.insert(payment);

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

    pub fn create_launcher(
        &mut self,
        f: impl FnOnce(&mut SpendContext, Launcher, Conditions) -> Result<Conditions, WalletError>,
    ) -> Result<(), WalletError> {
        let launcher_amount = self.launcher_index as u64 * 2;
        self.launcher_index += 1;

        let p2_puzzle_hash = SINGLETON_LAUNCHER_HASH.into();

        self.add(
            Payment::new(p2_puzzle_hash, launcher_amount),
            |ctx, coin, conditions| {
                let launcher =
                    Launcher::new(coin.coin().coin_id(), launcher_amount).with_singleton_amount(1);

                f(ctx, launcher, conditions)
            },
        )?;

        Ok(())
    }
}

impl Wallet {
    pub async fn distribute(
        &self,
        ctx: &mut SpendContext,
        preselection: &Preselection,
        selection: &Selection,
        tx: &TransactionConfig,
    ) -> Result<(), WalletError> {
        let mut distribution = Distribution::new(
            ctx,
            None,
            selection
                .xch
                .coins
                .iter()
                .enumerate()
                .map(|(i, &coin)| {
                    let mut item = DistributionItem::new(DistributionCoin::Xch(coin));

                    if i == 0 && tx.fee > 0 {
                        item.conditions = item.conditions.reserve_fee(tx.fee);
                    }

                    item
                })
                .collect(),
        );

        let change_amount = (selection.xch.existing_amount + preselection.created_xch)
            .saturating_sub(preselection.spent_xch);

        if change_amount > 0 {
            distribution.create_coin(tx.change_address, change_amount, false, None)?;
        }

        for action in &tx.actions {
            action.distribute(&mut distribution)?;
        }

        let items = distribution
            .items
            .into_iter()
            .map(|item| (item.coin.coin(), item.conditions))
            .collect_vec();

        self.spend_p2_coins_separately(ctx, items.into_iter())
            .await?;

        Ok(())
    }
}
