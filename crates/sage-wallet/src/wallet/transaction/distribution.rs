use std::mem;

use chia::protocol::{Bytes, Bytes32, Coin};
use chia_puzzles::SINGLETON_LAUNCHER_HASH;
use chia_wallet_sdk::{
    driver::{Cat, Did, HashedPtr, Launcher, Nft, OptionContract, SpendContext, StandardLayer},
    types::Conditions,
};
use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;

use crate::{wallet::memos::calculate_memos, Wallet, WalletError};

use super::{Action, Id, Selected, Selection, Summary, TransactionConfig};

#[derive(Debug)]
pub struct Distribution<'a> {
    ctx: &'a mut SpendContext,
    asset_id: Option<Id>,
    asset_type: AssetType,
    items: Vec<DistributionItem>,
    launcher_index: usize,
    parent_index: usize,
    new_assets: &'a mut NewAssets,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetType {
    Fungible,
    Did,
    Nft,
    Option,
}

#[derive(Debug, Default, Clone)]
pub struct NewAssets {
    pub cats: IndexMap<Id, NewCat>,
    pub nfts: IndexMap<Id, Nft<HashedPtr>>,
    pub dids: IndexMap<Id, Did<HashedPtr>>,
    pub options: IndexMap<Id, OptionContract>,
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

#[derive(Debug, Clone, Copy)]
pub enum DistributionCoin {
    Xch(Coin),
    Cat(Cat),
    Did(Did<HashedPtr>),
    Nft(Nft<HashedPtr>),
    Option(OptionContract),
}

impl DistributionCoin {
    #[must_use]
    pub fn child(&self, p2_puzzle_hash: Bytes32, amount: u64) -> Self {
        match self {
            Self::Xch(coin) => Self::Xch(Coin::new(coin.coin_id(), p2_puzzle_hash, amount)),
            Self::Cat(cat) => Self::Cat(cat.wrapped_child(p2_puzzle_hash, amount)),
            Self::Did(did) => Self::Xch(Coin::new(did.coin.coin_id(), p2_puzzle_hash, amount)),
            Self::Nft(nft) => Self::Xch(Coin::new(nft.coin.coin_id(), p2_puzzle_hash, amount)),
            Self::Option(option) => {
                Self::Xch(Coin::new(option.coin.coin_id(), p2_puzzle_hash, amount))
            }
        }
    }

    pub fn coin(&self) -> Coin {
        match self {
            Self::Xch(coin) => *coin,
            Self::Cat(cat) => cat.coin,
            Self::Did(did) => did.coin,
            Self::Nft(nft) => nft.coin,
            Self::Option(option) => option.coin,
        }
    }

    pub fn p2_puzzle_hash(&self) -> Bytes32 {
        match self {
            Self::Xch(coin) => coin.puzzle_hash,
            Self::Cat(cat) => cat.p2_puzzle_hash,
            Self::Did(did) => did.info.p2_puzzle_hash,
            Self::Nft(nft) => nft.info.p2_puzzle_hash,
            Self::Option(option) => option.info.p2_puzzle_hash,
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
        asset_type: AssetType,
        items: Vec<DistributionItem>,
        new_assets: &'a mut NewAssets,
    ) -> Self {
        Self {
            ctx,
            asset_id,
            asset_type,
            items,
            launcher_index: 0,
            parent_index: 0,
            new_assets,
        }
    }

    pub fn asset_id(&self) -> Option<Id> {
        self.asset_id
    }

    pub fn asset_type(&self) -> AssetType {
        self.asset_type
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
        let new_conditions = f(self.ctx, self.new_assets, item, conditions)?;
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
        let launcher_amount = self.launcher_index as u64
            * match self.asset_type {
                AssetType::Fungible => 1,
                AssetType::Did | AssetType::Nft | AssetType::Option => 2,
            };
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
        let new_conditions = f(self.ctx, self.new_assets, item, conditions)?;
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

        let mut new_assets = NewAssets::default();

        self.distribute_xch(
            ctx,
            summary,
            selection,
            tx,
            change_puzzle_hash,
            &mut new_assets,
        )
        .await?;

        let mut new_selection = selection.clone();

        for &id in new_assets.cats.keys() {
            new_selection.cats.entry(id).or_default();
        }

        new_selection.dids.extend(new_assets.dids.clone());
        new_selection.nfts.extend(new_assets.nfts.clone());
        new_selection.options.extend(new_assets.options.clone());

        for (&id, selected) in &new_selection.cats {
            self.distribute_cat(
                ctx,
                summary,
                id,
                selected,
                tx,
                change_puzzle_hash,
                &mut new_assets,
            )
            .await?;
        }

        for (&id, &did) in &new_selection.dids {
            self.distribute_did(ctx, id, did, tx, &mut new_assets)
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
        new_assets: &mut NewAssets,
    ) -> Result<(), WalletError> {
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

        let mut distribution = Distribution::new(ctx, None, AssetType::Fungible, items, new_assets);

        let change_amount =
            (selection.xch.existing_amount + summary.created_xch).saturating_sub(summary.spent_xch);

        if change_amount > 0 {
            distribution.create_coin(change_puzzle_hash, change_amount, false, None)?;
        }

        for (index, action) in tx.actions.iter().enumerate() {
            action.distribute(&mut distribution, index)?;
        }

        let coin_ids = distribution
            .items
            .iter()
            .map(|item| item.coin.coin().coin_id())
            .collect_vec();

        let items = distribution
            .items
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                let mut conditions = item.conditions;

                if coin_ids.len() > 1 {
                    conditions = conditions.assert_concurrent_spend(if i == 0 {
                        coin_ids[coin_ids.len() - 1]
                    } else {
                        coin_ids[i - 1]
                    });
                }

                (item.coin.coin(), conditions)
            })
            .collect_vec();

        self.spend_p2_coins_separately(ctx, items.into_iter())
            .await?;

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    async fn distribute_cat(
        &self,
        ctx: &mut SpendContext,
        summary: &Summary,
        id: Id,
        selected: &Selected<Cat>,
        tx: &TransactionConfig,
        change_puzzle_hash: Bytes32,
        new_assets: &mut NewAssets,
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
            AssetType::Fungible,
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
            new_assets,
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
                        DistributionCoin::Cat(cat) => cat,
                        _ => unreachable!(),
                    },
                    item.conditions,
                )
            })
            .collect_vec();

        self.spend_cat_coins(ctx, items.into_iter()).await?;

        Ok(())
    }

    #[allow(clippy::large_types_passed_by_value)]
    async fn distribute_did(
        &self,
        ctx: &mut SpendContext,
        id: Id,
        did: Did<HashedPtr>,
        tx: &TransactionConfig,
        new_assets: &mut NewAssets,
    ) -> Result<(), WalletError> {
        let synthetic_key = self.db.synthetic_key(did.info.p2_puzzle_hash).await?;
        let p2 = StandardLayer::new(synthetic_key);

        let mut distribution = Distribution::new(
            ctx,
            Some(id),
            AssetType::Did,
            vec![DistributionItem::new(DistributionCoin::Did(did), p2)],
            new_assets,
        );

        for (index, action) in tx.actions.iter().enumerate() {
            action.distribute(&mut distribution, index)?;
        }

        let mut xch = Vec::new();
        let mut did_conditions = Conditions::new();

        for item in distribution.items {
            match item.coin {
                DistributionCoin::Xch(coin) => xch.push((coin, item.conditions)),
                DistributionCoin::Did(..) => {
                    did_conditions = did_conditions.extend(item.conditions);
                }
                _ => unreachable!(),
            }
        }

        self.spend_p2_coins_separately(ctx, xch.into_iter()).await?;

        if !did_conditions.is_empty() {
            let did = did.update(ctx, &p2, did_conditions)?;
            new_assets.dids.insert(id, did);
        }

        Ok(())
    }
}
