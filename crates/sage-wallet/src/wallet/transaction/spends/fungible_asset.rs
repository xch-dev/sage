use std::mem;

use chia::{
    protocol::{Bytes, Bytes32, Coin},
    puzzles::offer::{self, NotarizedPayment},
};
use chia_puzzles::{SETTLEMENT_PAYMENT_HASH, SINGLETON_LAUNCHER_HASH};
use chia_wallet_sdk::driver::{Cat, Launcher, OptionLauncher, OptionLauncherInfo, SpendContext};
use indexmap::IndexSet;

use crate::{
    wallet::memos::{calculate_memos, calculate_memos_list},
    WalletError,
};

use super::{P2Selection, SettlementP2, P2};

#[derive(Debug, Clone)]
pub struct FungibleAsset<T> {
    pub items: Vec<AssetCoin<T>>,
    pub launcher_index: u64,
    pub parent_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AssetCoinRef(usize);

#[derive(Debug, Clone)]
pub struct AssetCoin<T> {
    pub coin: T,
    pub p2: P2,
    pub payments: IndexSet<Payment>,
}

impl<T> AssetCoin<T> {
    pub fn new(coin: T, p2: P2) -> Self {
        Self {
            coin,
            p2,
            payments: IndexSet::new(),
        }
    }
}

pub trait AssetCoinExt {
    fn p2_puzzle_hash(&self) -> Bytes32;
    fn include_hint(&self) -> bool;
    #[must_use]
    fn child(&self, p2_puzzle_hash: Bytes32, amount: u64) -> Self;
    fn coin(&self) -> Coin;
}

impl AssetCoinExt for Coin {
    fn p2_puzzle_hash(&self) -> Bytes32 {
        self.puzzle_hash
    }

    fn include_hint(&self) -> bool {
        false
    }

    fn child(&self, p2_puzzle_hash: Bytes32, amount: u64) -> Self {
        Coin::new(self.coin_id(), p2_puzzle_hash, amount)
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

    fn child(&self, p2_puzzle_hash: Bytes32, amount: u64) -> Self {
        self.wrapped_child(p2_puzzle_hash, amount)
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
    pub fn new(items: Vec<AssetCoin<T>>) -> Self {
        Self {
            items,
            launcher_index: 0,
            parent_index: 0,
        }
    }

    pub fn get_mut(&mut self, item_ref: AssetCoinRef) -> Result<&mut AssetCoin<T>, WalletError> {
        self.items
            .get_mut(item_ref.0)
            .ok_or(WalletError::MissingAsset)
    }

    pub fn make_payment(
        &mut self,
        ctx: &mut SpendContext,
        payment: Payment,
        p2_selection: P2Selection,
    ) -> Result<AssetCoinRef, WalletError> {
        // This weird duplicated logic is due to a flaw in the borrow checker.
        if self
            .items
            .iter()
            .any(|item| !item.payments.contains(&payment) && p2_selection.matches(&item.p2))
        {
            let item_ref = AssetCoinRef(
                self.items
                    .iter_mut()
                    .position(|item| {
                        !item.payments.contains(&payment) && p2_selection.matches(&item.p2)
                    })
                    .expect("missing item"),
            );
            self.get_mut(item_ref)?.payments.insert(payment);
            return Ok(item_ref);
        }

        let Some(parent) = self.items.iter_mut().find(|item| {
            !item.payments.contains(&Payment::new(
                if matches!(p2_selection, P2Selection::Offer(_)) {
                    SETTLEMENT_PAYMENT_HASH.into()
                } else {
                    item.coin.p2_puzzle_hash()
                },
                1,
            ))
        }) else {
            return Err(WalletError::NoIntermediateParent);
        };

        let intermediate_puzzle_hash = if matches!(p2_selection, P2Selection::Offer(_)) {
            SETTLEMENT_PAYMENT_HASH.into()
        } else {
            parent.coin.p2_puzzle_hash()
        };

        parent
            .payments
            .insert(Payment::new(intermediate_puzzle_hash, 1));

        match &mut parent.p2 {
            P2::Standard(p2) => {
                p2.conditions = mem::take(&mut p2.conditions).create_coin(
                    intermediate_puzzle_hash,
                    1,
                    calculate_memos(
                        ctx,
                        intermediate_puzzle_hash,
                        parent.coin.include_hint(),
                        None,
                    )?,
                );
            }
            P2::Offer(p2) => {
                let nonce = p2_selection.nonce();

                let offer_payment = offer::Payment {
                    puzzle_hash: intermediate_puzzle_hash,
                    amount: 1,
                    memos: calculate_memos_list(
                        intermediate_puzzle_hash,
                        parent.coin.include_hint(),
                        None,
                    )
                    .map(offer::Memos),
                };

                if let Some(np) = p2
                    .notarized_payments
                    .iter_mut()
                    .find(|np| np.nonce == nonce)
                {
                    np.payments.push(offer_payment);
                } else {
                    p2.notarized_payments.push(NotarizedPayment {
                        nonce,
                        payments: vec![offer_payment],
                    });
                }
            }
        }

        let child = parent.coin.child(intermediate_puzzle_hash, 1);
        let p2 = if matches!(p2_selection, P2Selection::Offer(_)) {
            P2::Offer(SettlementP2::new())
        } else {
            parent.p2.cleared()
        };

        self.items.push(AssetCoin::new(child, p2));
        let item_ref = AssetCoinRef(self.items.len() - 1);

        self.get_mut(item_ref)?.payments.insert(payment);

        Ok(item_ref)
    }

    pub fn create_coin(
        &mut self,
        ctx: &mut SpendContext,
        p2_puzzle_hash: Bytes32,
        amount: u64,
        include_hint: bool,
        memos: Option<Vec<Bytes>>,
        p2_selection: P2Selection,
    ) -> Result<T, WalletError> {
        let item_ref =
            self.make_payment(ctx, Payment::new(p2_puzzle_hash, amount), p2_selection)?;
        let item = self.get_mut(item_ref)?;

        match &mut item.p2 {
            P2::Standard(p2) => {
                p2.conditions = mem::take(&mut p2.conditions).create_coin(
                    p2_puzzle_hash,
                    amount,
                    calculate_memos(ctx, p2_puzzle_hash, include_hint, memos)?,
                );
            }
            P2::Offer(p2) => {
                let nonce = p2_selection.nonce();

                let offer_payment = offer::Payment {
                    puzzle_hash: p2_puzzle_hash,
                    amount,
                    memos: calculate_memos_list(p2_puzzle_hash, include_hint, memos)
                        .map(offer::Memos),
                };

                if let Some(np) = p2
                    .notarized_payments
                    .iter_mut()
                    .find(|np| np.nonce == nonce)
                {
                    np.payments.push(offer_payment);
                } else {
                    p2.notarized_payments.push(NotarizedPayment {
                        nonce,
                        payments: vec![offer_payment],
                    });
                }
            }
        }

        Ok(item.coin.child(p2_puzzle_hash, amount))
    }

    pub fn create_launcher(
        &mut self,
        ctx: &mut SpendContext,
    ) -> Result<(AssetCoinRef, Launcher), WalletError> {
        let launcher_amount = self.launcher_index;
        self.launcher_index += 1;

        let p2_puzzle_hash = SINGLETON_LAUNCHER_HASH.into();

        let item_ref = self.make_payment(
            ctx,
            Payment::new(p2_puzzle_hash, launcher_amount),
            P2Selection::Standard,
        )?;
        let item = self.get_mut(item_ref)?;

        let launcher =
            Launcher::new(item.coin.coin().coin_id(), launcher_amount).with_singleton_amount(1);

        Ok((item_ref, launcher))
    }

    pub fn create_option_launcher(
        &mut self,
        ctx: &mut SpendContext,
        info: OptionLauncherInfo,
    ) -> Result<(AssetCoinRef, OptionLauncher), WalletError> {
        let launcher_amount = self.launcher_index;
        self.launcher_index += 1;

        let p2_puzzle_hash = SINGLETON_LAUNCHER_HASH.into();

        let item_ref = self.make_payment(
            ctx,
            Payment::new(p2_puzzle_hash, launcher_amount),
            P2Selection::Standard,
        )?;
        let item = self.get_mut(item_ref)?;

        let launcher =
            OptionLauncher::with_amount(ctx, item.coin.coin().coin_id(), launcher_amount, info)?;

        Ok((item_ref, launcher))
    }

    pub fn create_from_unique_parent(
        &mut self,
        ctx: &mut SpendContext,
    ) -> Result<AssetCoinRef, WalletError> {
        // This weird duplicated logic is due to a flaw in the borrow checker.
        if let Some(index) = self
            .items
            .iter()
            .skip(self.parent_index)
            .position(|item| P2Selection::Standard.matches(&item.p2))
        {
            self.parent_index = index + 1;
            return Ok(AssetCoinRef(index));
        }

        let Some(parent) = self.items.iter_mut().find(|item| {
            !item
                .payments
                .contains(&Payment::new(item.coin.p2_puzzle_hash(), 1))
                && P2Selection::Standard.matches(&item.p2)
        }) else {
            return Err(WalletError::NoIntermediateParent);
        };

        parent
            .payments
            .insert(Payment::new(parent.coin.p2_puzzle_hash(), 1));

        let p2 = parent
            .p2
            .as_standard_mut()
            .ok_or(WalletError::P2Unsupported)?;

        p2.conditions = mem::take(&mut p2.conditions).create_coin(
            parent.coin.p2_puzzle_hash(),
            1,
            calculate_memos(
                ctx,
                parent.coin.p2_puzzle_hash(),
                parent.coin.include_hint(),
                None,
            )?,
        );

        let child = parent.coin.child(parent.coin.p2_puzzle_hash(), 1);
        let p2 = parent.p2.cleared();

        self.items.push(AssetCoin::new(child, p2));
        self.parent_index += 1;

        Ok(AssetCoinRef(self.items.len() - 1))
    }
}
