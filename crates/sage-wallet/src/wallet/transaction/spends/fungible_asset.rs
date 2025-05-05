use std::mem;

use chia::protocol::{Bytes, Bytes32, Coin};
use chia_puzzles::SINGLETON_LAUNCHER_HASH;
use chia_wallet_sdk::{
    driver::{Cat, Launcher, SpendContext, StandardLayer},
    types::Conditions,
};
use indexmap::IndexSet;

use crate::{wallet::memos::calculate_memos, WalletError};

#[derive(Debug, Clone)]
pub struct FungibleAsset<T> {
    pub items: Vec<AssetCoin<T>>,
    pub launcher_index: u64,
    pub parent_index: usize,
    pub was_created: bool,
}

#[derive(Debug, Clone)]
pub struct AssetCoin<T> {
    pub coin: T,
    pub p2: StandardLayer,
    pub payments: IndexSet<Payment>,
    pub conditions: Conditions,
}

impl<T> AssetCoin<T> {
    pub fn new(coin: T, p2: StandardLayer) -> Self {
        Self {
            coin,
            p2,
            payments: IndexSet::new(),
            conditions: Conditions::new(),
        }
    }
}

pub trait AssetCoinExt {
    fn p2_puzzle_hash(&self) -> Bytes32;
    fn include_hint(&self) -> bool;
    #[must_use]
    fn intermediate_child(&self) -> Self;
    fn coin(&self) -> Coin;
}

impl AssetCoinExt for Coin {
    fn p2_puzzle_hash(&self) -> Bytes32 {
        self.puzzle_hash
    }

    fn include_hint(&self) -> bool {
        false
    }

    fn intermediate_child(&self) -> Self {
        Coin::new(self.coin_id(), self.puzzle_hash, 0)
    }

    fn coin(&self) -> Coin {
        *self
    }
}

impl AssetCoinExt for Cat {
    fn p2_puzzle_hash(&self) -> Bytes32 {
        self.p2_puzzle_hash
    }

    fn include_hint(&self) -> bool {
        true
    }

    fn intermediate_child(&self) -> Self {
        self.wrapped_child(self.p2_puzzle_hash, 0)
    }

    fn coin(&self) -> Coin {
        self.coin
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

impl<T> FungibleAsset<T>
where
    T: AssetCoinExt,
{
    pub fn new(items: Vec<AssetCoin<T>>, was_created: bool) -> Self {
        Self {
            items,
            launcher_index: 0,
            parent_index: 0,
            was_created,
        }
    }

    pub fn make_payment(
        &mut self,
        ctx: &mut SpendContext,
        payment: Payment,
    ) -> Result<&mut AssetCoin<T>, WalletError> {
        // This weird duplicated logic is due to a flaw in the borrow checker.
        if self
            .items
            .iter()
            .any(|item| !item.payments.contains(&payment))
        {
            let item = self
                .items
                .iter_mut()
                .find(|item| !item.payments.contains(&payment))
                .expect("missing item");
            item.payments.insert(payment);
            return Ok(item);
        }

        let Some(parent) = self.items.iter_mut().find(|item| {
            !item
                .payments
                .contains(&Payment::new(item.coin.p2_puzzle_hash(), 0))
        }) else {
            return Err(WalletError::NoIntermediateParent);
        };

        parent
            .payments
            .insert(Payment::new(parent.coin.p2_puzzle_hash(), 0));

        parent.conditions = mem::take(&mut parent.conditions).create_coin(
            parent.coin.p2_puzzle_hash(),
            0,
            calculate_memos(
                ctx,
                parent.coin.p2_puzzle_hash(),
                parent.coin.include_hint(),
                None,
            )?,
        );

        let child = parent.coin.intermediate_child();
        let p2 = parent.p2;

        self.items.push(AssetCoin::new(child, p2));
        let item = self.items.last_mut().expect("item should exist");

        item.payments.insert(payment);

        Ok(item)
    }

    pub fn create_coin(
        &mut self,
        ctx: &mut SpendContext,
        p2_puzzle_hash: Bytes32,
        amount: u64,
        include_hint: bool,
        memos: Option<Vec<Bytes>>,
    ) -> Result<(), WalletError> {
        let item = self.make_payment(ctx, Payment::new(p2_puzzle_hash, amount))?;

        item.conditions = mem::take(&mut item.conditions).create_coin(
            p2_puzzle_hash,
            amount,
            calculate_memos(ctx, p2_puzzle_hash, include_hint, memos)?,
        );

        Ok(())
    }

    pub fn create_launcher(
        &mut self,
        ctx: &mut SpendContext,
    ) -> Result<(&mut AssetCoin<T>, Launcher), WalletError> {
        let launcher_amount = self.launcher_index;
        self.launcher_index += 1;

        let p2_puzzle_hash = SINGLETON_LAUNCHER_HASH.into();

        let item = self.make_payment(ctx, Payment::new(p2_puzzle_hash, launcher_amount))?;

        let launcher =
            Launcher::new(item.coin.coin().coin_id(), launcher_amount).with_singleton_amount(1);

        Ok((item, launcher))
    }

    pub fn create_from_unique_parent(
        &mut self,
        ctx: &mut SpendContext,
    ) -> Result<&mut AssetCoin<T>, WalletError> {
        // This weird duplicated logic is due to a flaw in the borrow checker.
        if self.parent_index < self.items.len() {
            return Ok(self.items.get_mut(self.parent_index).expect("missing item"));
        }

        let Some(parent) = self.items.iter_mut().find(|item| {
            !item
                .payments
                .contains(&Payment::new(item.coin.p2_puzzle_hash(), 0))
        }) else {
            return Err(WalletError::NoIntermediateParent);
        };

        parent
            .payments
            .insert(Payment::new(parent.coin.p2_puzzle_hash(), 0));

        parent.conditions = mem::take(&mut parent.conditions).create_coin(
            parent.coin.p2_puzzle_hash(),
            0,
            calculate_memos(
                ctx,
                parent.coin.p2_puzzle_hash(),
                parent.coin.include_hint(),
                None,
            )?,
        );

        let child = parent.coin.intermediate_child();
        let p2 = parent.p2;

        self.items.push(AssetCoin::new(child, p2));
        let item = self.items.last_mut().expect("item should exist");

        self.parent_index += 1;

        Ok(item)
    }
}
