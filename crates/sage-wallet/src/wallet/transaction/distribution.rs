use std::{collections::HashMap, mem};

use chia::protocol::{Bytes, Bytes32, Coin};
use chia_puzzles::SINGLETON_LAUNCHER_HASH;
use chia_wallet_sdk::{
    driver::{Cat, Did, HashedPtr, Launcher, Nft, OptionContract, SpendContext, StandardLayer},
    types::Conditions,
};
use indexmap::IndexSet;
use itertools::Itertools;

use crate::{wallet::memos::calculate_memos, Wallet, WalletError};

use super::{Action, Id, Selected, Selection, Summary, TransactionConfig};

#[derive(Debug)]
pub struct Distribution<'a> {
    pub ctx: &'a mut SpendContext,
    pub asset_id: Option<Id>,
    pub items: Vec<DistributionItem>,
    pub launcher_index: usize,
    pub parent_index: usize,
    pub new_assets: NewAssets,
}

#[derive(Debug, Default, Clone)]
pub struct NewAssets {
    pub cats: HashMap<Id, NewCat>,
    pub nfts: HashMap<Id, Nft<HashedPtr>>,
    pub dids: HashMap<Id, Did<HashedPtr>>,
    pub options: HashMap<Id, OptionContract>,
}

#[derive(Debug, Clone)]
pub struct NewCat {
    pub asset_id: Bytes32,
    pub items: Vec<DistributionItem>,
}

#[derive(Debug, Clone)]
pub struct DistributionItem {
    pub coin: DistributionCoin,
    pub p2: StandardLayer,
    pub payments: IndexSet<Payment>,
    pub conditions: Conditions,
}

impl DistributionItem {
    pub fn new(coin: DistributionCoin, p2: StandardLayer) -> Self {
        Self {
            coin,
            p2,
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
            parent_index: 0,
            new_assets: NewAssets::default(),
        }
    }

    pub fn asset_id(&self) -> Option<Id> {
        self.asset_id
    }

    pub fn make_payment(
        &mut self,
        payment: Payment,
        f: impl FnOnce(
            &mut SpendContext,
            &mut NewAssets,
            &DistributionItem,
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
            let p2 = parent.p2;
            self.items.push(DistributionItem::new(child, p2));
            self.items.last_mut().expect("item should exist")
        };

        item.payments.insert(payment);

        let conditions = mem::take(&mut item.conditions);
        let new_conditions = f(self.ctx, &mut self.new_assets, item, conditions)?;
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
        self.make_payment(
            Payment::new(p2_puzzle_hash, amount),
            |ctx, _new_assets, _coin, conditions| {
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
        f: impl FnOnce(
            &mut SpendContext,
            &mut NewAssets,
            &DistributionItem,
            Launcher,
            Conditions,
        ) -> Result<Conditions, WalletError>,
    ) -> Result<(), WalletError> {
        let launcher_amount = self.launcher_index as u64 * 2;
        self.launcher_index += 1;

        let p2_puzzle_hash = SINGLETON_LAUNCHER_HASH.into();

        self.make_payment(
            Payment::new(p2_puzzle_hash, launcher_amount),
            |ctx, new_assets, item, conditions| {
                let launcher = Launcher::new(item.coin.coin().coin_id(), launcher_amount)
                    .with_singleton_amount(1);

                f(ctx, new_assets, item, launcher, conditions)
            },
        )?;

        Ok(())
    }

    pub fn create_from_unique_parent(
        &mut self,
        f: impl FnOnce(
            &mut SpendContext,
            &mut NewAssets,
            &DistributionItem,
            Conditions,
        ) -> Result<Conditions, WalletError>,
    ) -> Result<(), WalletError> {
        let item = if let Some(item) = self.items.get_mut(self.parent_index) {
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
            let p2 = parent.p2;
            self.items.push(DistributionItem::new(child, p2));
            self.items.last_mut().expect("item should exist")
        };

        self.parent_index += 1;

        let conditions = mem::take(&mut item.conditions);
        let new_conditions = f(self.ctx, &mut self.new_assets, item, conditions)?;
        item.conditions = new_conditions;

        Ok(())
    }
}

impl Wallet {
    pub async fn distribute(
        &self,
        ctx: &mut SpendContext,
        summary: &Summary,
        selection: &Selection,
        tx: &TransactionConfig,
    ) -> Result<NewAssets, WalletError> {
        let change_puzzle_hash = self.p2_puzzle_hash(false, true).await?;

        let new_assets = self
            .distribute_xch(ctx, summary, selection, tx, change_puzzle_hash)
            .await?;

        let mut new_selection = selection.clone();

        for &id in new_assets.cats.keys() {
            new_selection.cats.entry(id).or_default();
        }

        for (&id, selected) in &new_selection.cats {
            self.distribute_cat(
                ctx,
                summary,
                id,
                &new_assets,
                selected,
                tx,
                change_puzzle_hash,
            )
            .await?;
        }

        Ok(new_assets)
    }

    async fn distribute_xch(
        &self,
        ctx: &mut SpendContext,
        summary: &Summary,
        selection: &Selection,
        tx: &TransactionConfig,
        change_puzzle_hash: Bytes32,
    ) -> Result<NewAssets, WalletError> {
        let mut items = Vec::new();

        for (i, &coin) in selection.xch.coins.iter().enumerate() {
            let synthetic_key = self.db.synthetic_key(coin.puzzle_hash).await?;

            let mut item = DistributionItem::new(
                DistributionCoin::Xch(coin),
                StandardLayer::new(synthetic_key),
            );

            if i == 0 && tx.fee > 0 {
                item.conditions = item.conditions.reserve_fee(tx.fee);
            }

            items.push(item);
        }

        let mut distribution = Distribution::new(ctx, None, items);

        let change_amount =
            (selection.xch.existing_amount + summary.created_xch).saturating_sub(summary.spent_xch);

        if change_amount > 0 {
            distribution.create_coin(change_puzzle_hash, change_amount, false, None)?;
        }

        for (index, action) in tx.actions.iter().enumerate() {
            action.distribute(&mut distribution, index)?;
        }

        let items = distribution
            .items
            .into_iter()
            .map(|item| (item.coin.coin(), item.conditions))
            .collect_vec();

        let new_assets = distribution.new_assets;

        self.spend_p2_coins_separately(ctx, items.into_iter())
            .await?;

        Ok(new_assets)
    }

    #[allow(clippy::too_many_arguments)]
    async fn distribute_cat(
        &self,
        ctx: &mut SpendContext,
        summary: &Summary,
        id: Id,
        new_assets: &NewAssets,
        selected: &Selected<Cat>,
        tx: &TransactionConfig,
        change_puzzle_hash: Bytes32,
    ) -> Result<(), WalletError> {
        let mut items = Vec::new();

        for &cat in &selected.coins {
            let synthetic_key = self.db.synthetic_key(cat.p2_puzzle_hash).await?;

            items.push(DistributionItem::new(
                DistributionCoin::Cat(cat),
                StandardLayer::new(synthetic_key),
            ));
        }

        let mut distribution = Distribution::new(
            ctx,
            Some(id),
            items
                .into_iter()
                .chain(
                    new_assets
                        .cats
                        .get(&id)
                        .map(|asset| asset.items.clone())
                        .unwrap_or_default(),
                )
                .collect(),
        );

        let created_amount = summary.created_cats.get(&id).copied().unwrap_or_default();

        let spent_amount = summary.spent_cats.get(&id).copied().unwrap_or_default();

        let change_amount =
            (selected.existing_amount + created_amount).saturating_sub(spent_amount);

        if change_amount > 0 {
            distribution.create_coin(change_puzzle_hash, change_amount, false, None)?;
        }

        for (index, action) in tx.actions.iter().enumerate() {
            action.distribute(&mut distribution, index)?;
        }

        let items = distribution
            .items
            .into_iter()
            .map(|item| {
                (
                    match item.coin {
                        DistributionCoin::Xch(..) => unreachable!(),
                        DistributionCoin::Cat(cat) => cat,
                    },
                    item.conditions,
                )
            })
            .collect_vec();

        self.spend_cat_coins(ctx, items.into_iter()).await?;

        Ok(())
    }
}
