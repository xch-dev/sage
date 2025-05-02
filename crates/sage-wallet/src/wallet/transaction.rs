mod action;
mod id;
mod selection;

pub use action::*;
pub use id::*;
pub use selection::*;

use std::collections::HashSet;

use chia::{
    clvm_utils::ToTreeHash,
    protocol::{Bytes32, Coin, CoinSpend},
};
use chia_wallet_sdk::{
    driver::{
        Cat, CatSpend, HashedPtr, Launcher, SpendContext, SpendWithConditions, StandardLayer,
    },
    types::Conditions,
};
use indexmap::{IndexMap, IndexSet};
use sage_database::CoinKind;

use crate::WalletError;

use super::{memos::calculate_memos, Wallet};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct TransactionConfig {
    pub actions: Vec<Action>,
    pub preselected_coin_ids: Vec<Bytes32>,
    pub fee: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransactionResult {
    pub coin_spends: Vec<CoinSpend>,
}

impl Wallet {
    pub async fn build_transaction(
        &self,
        tx: &TransactionConfig,
        change_puzzle_hash: Bytes32,
    ) -> Result<TransactionResult, WalletError> {
        let mut ctx = SpendContext::new();

        let mut selection = self.select_transaction(&mut ctx, tx).await?;

        let mut send_xch = Vec::new();
        let mut send_cats = IndexMap::new();
        let mut single_issue_cats = Vec::new();
        let mut create_did = Vec::new();

        for (i, &action) in tx.actions.iter().enumerate() {
            match action {
                Action::Send(action) => {
                    if let Some(id) = action.asset_id {
                        send_cats.entry(id).or_insert_with(Vec::new).push(action);
                    } else {
                        send_xch.push(action);
                    }
                }
                Action::IssueCat(action) => {
                    single_issue_cats.push((i, action));
                }
                Action::CreateDid(action) => {
                    create_did.push((i, action));
                }
            }
        }

        let selected_xch: i64 = selection.xch.amount.try_into()?;
        let change_xch = selected_xch - selection.required_xch;

        if change_xch > 0 {
            send_xch.push(SendAction {
                asset_id: None,
                puzzle_hash: change_puzzle_hash,
                amount: change_xch as u64,
            });
        }

        for (i, &coin) in selection.xch.coins.iter().enumerate() {
            let synthetic_key = self.db.synthetic_key(coin.puzzle_hash).await?;
            let p2 = StandardLayer::new(synthetic_key);

            if i != 0 {
                p2.spend(
                    &mut ctx,
                    coin,
                    Conditions::new().assert_concurrent_spend(selection.xch.coins[0].coin_id()),
                )?;

                continue;
            }

            let mut extra_conditions = Conditions::new();
            let mut existing_children = HashSet::new();

            if tx.fee > 0 {
                extra_conditions = extra_conditions.reserve_fee(tx.fee);
            }

            let mut launcher_index = 0;

            for (i, _action) in &create_did {
                let launcher = Launcher::new(coin.coin_id(), launcher_index * 2);
                launcher_index += 1;

                let launcher_coin = launcher.coin();
                let (create_did, _did) = launcher
                    .with_singleton_amount(1)
                    .create_simple_did(&mut ctx, &p2)?;

                extra_conditions = extra_conditions.extend(create_did);
                existing_children.insert((launcher_coin.puzzle_hash, launcher_coin.amount));
            }

            make_payments(
                &mut ctx,
                PaymentCoin::Xch(coin),
                p2,
                send_xch.clone(),
                extra_conditions,
                &mut Vec::new(),
                existing_children,
            )?;
        }

        todo!()
    }
}

#[must_use]
#[derive(Debug, Clone, Copy)]
pub enum PaymentCoin {
    Xch(Coin),
    Cat(Cat),
}

impl PaymentCoin {
    pub fn child(&self, p2_puzzle_hash: Bytes32, amount: u64) -> Self {
        match self {
            Self::Xch(coin) => Self::Xch(Coin::new(coin.coin_id(), p2_puzzle_hash, amount)),
            Self::Cat(cat) => Self::Cat(cat.wrapped_child(p2_puzzle_hash, amount)),
        }
    }
}

pub fn make_payments(
    ctx: &mut SpendContext,
    coin: PaymentCoin,
    p2: StandardLayer,
    payments: Vec<SendAction>,
    extra_conditions: Conditions,
    cat_spends: &mut Vec<CatSpend>,
    mut existing_children: HashSet<(Bytes32, u64)>,
) -> Result<(), WalletError> {
    let total_amount = payments.iter().map(|p| p.amount).sum();

    let mut conditions = extra_conditions;
    let mut remainder = Vec::new();

    for payment in payments {
        if !existing_children.insert((payment.puzzle_hash, payment.amount)) {
            remainder.push(payment);
            continue;
        }

        let memos = calculate_memos(
            ctx,
            payment.puzzle_hash,
            matches!(coin, PaymentCoin::Cat(..)),
            None,
        )?;

        conditions = conditions.create_coin(payment.puzzle_hash, payment.amount, memos);
    }

    if !remainder.is_empty() {
        let p2_puzzle_hash = p2.tree_hash().into();

        let memos = calculate_memos(
            ctx,
            p2_puzzle_hash,
            matches!(coin, PaymentCoin::Cat(..)),
            None,
        )?;

        conditions = conditions.create_coin(p2_puzzle_hash, total_amount, memos);

        make_payments(
            ctx,
            coin.child(p2_puzzle_hash, total_amount),
            p2,
            remainder,
            Conditions::new(),
            cat_spends,
            HashSet::new(),
        )?;
    }

    match coin {
        PaymentCoin::Xch(coin) => {
            p2.spend(ctx, coin, conditions)?;
        }
        PaymentCoin::Cat(cat) => {
            cat_spends.push(CatSpend::new(
                cat,
                p2.spend_with_conditions(ctx, conditions)?,
            ));
        }
    }

    Ok(())
}
