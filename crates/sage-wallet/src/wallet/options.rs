use std::{
    mem,
    time::{SystemTime, UNIX_EPOCH},
};

use chia_wallet_sdk::{
    chia::puzzle_types::offer::{NotarizedPayment, Payment},
    driver::{Delta, FungibleSpend},
    prelude::*,
    puzzles::SETTLEMENT_PAYMENT_HASH,
    utils::CoinSelectionError,
};
use indexmap::IndexSet;
use sage_database::CoinKind;

use crate::{
    WalletError,
    wallet::memos::{Hint, calculate_memos},
};

use super::Wallet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalletOptionMint {
    pub expiration_seconds: u64,
    pub underlying_type: OptionType,
    pub strike_type: OptionType,
}

/// The spends and actions shared by exercising and cancelling a set of options.
#[derive(Debug, Default)]
struct OptionExercise {
    options: Vec<OptionContract>,
    underlyings: Vec<OptionUnderlying>,
    settlement_coins: Vec<Coin>,
    settlement_cats: Vec<Cat>,
    melt_actions: Vec<Action>,
    settle_actions: Vec<Action>,
    strike_ids: IndexSet<Id>,
}

impl OptionExercise {
    fn actions(&self, fee: u64) -> Vec<Action> {
        let mut actions = vec![Action::fee(fee)];
        actions.extend_from_slice(&self.melt_actions);
        actions.extend_from_slice(&self.settle_actions);
        actions
    }

    fn add_settlements(&self, spends: &mut Spends) {
        for &coin in &self.settlement_coins {
            spends.add(coin);
        }

        for &cat in &self.settlement_cats {
            spends.add(cat);
        }
    }
}

impl Wallet {
    pub async fn mint_option(
        &self,
        mint: WalletOptionMint,
        fee: u64,
    ) -> Result<(Vec<CoinSpend>, OptionContract), WalletError> {
        let creator_puzzle_hash = self.change_p2_puzzle_hash().await?;

        let mut ctx = SpendContext::new();

        let underlying_id = match mint.underlying_type {
            OptionType::Xch { .. } => Id::Xch,
            OptionType::Cat { asset_id, .. } | OptionType::RevocableCat { asset_id, .. } => {
                Id::Existing(asset_id)
            }
            OptionType::Nft { .. } => return Err(WalletError::NftOptionNotSupported),
        };

        if matches!(mint.strike_type, OptionType::Nft { .. }) {
            return Err(WalletError::NftOptionNotSupported);
        }

        let outputs = self
            .spend(
                &mut ctx,
                vec![],
                &[
                    Action::fee(fee),
                    Action::mint_option(
                        creator_puzzle_hash,
                        mint.expiration_seconds,
                        underlying_id,
                        mint.underlying_type.amount(),
                        mint.strike_type,
                        1,
                    ),
                ],
            )
            .await?;

        let option = outputs.options[&Id::New(1)];

        Ok((ctx.take(), option))
    }

    pub async fn transfer_options(
        &self,
        option_ids: Vec<Bytes32>,
        puzzle_hash: Bytes32,
        fee: u64,
        clawback: Option<u64>,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let sender_puzzle_hash = self.change_p2_puzzle_hash().await?;

        let mut ctx = SpendContext::new();
        let mut actions = vec![Action::fee(fee)];

        for option_id in option_ids {
            let amount = self
                .db
                .option(option_id)
                .await?
                .ok_or(WalletError::MissingOption(option_id))?
                .coin
                .amount;

            let clawback = clawback.map(|seconds| {
                ClawbackV2::new(sender_puzzle_hash, puzzle_hash, seconds, amount, true)
            });

            let memos = calculate_memos(
                &mut ctx,
                if let Some(clawback) = clawback {
                    Hint::Clawback(clawback)
                } else {
                    Hint::P2PuzzleHash(puzzle_hash)
                },
                vec![],
            )?;

            let p2_puzzle_hash = if let Some(clawback) = clawback {
                clawback.tree_hash().into()
            } else {
                puzzle_hash
            };

            actions.push(Action::send(
                Id::Existing(option_id),
                p2_puzzle_hash,
                amount,
                memos,
            ));
        }

        self.spend(&mut ctx, vec![], &actions).await?;

        Ok(ctx.take())
    }

    pub async fn exercise_options(
        &self,
        option_ids: Vec<Bytes32>,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();

        let exercise = self.prepare_exercise(&mut ctx, option_ids).await?;

        let actions = exercise.actions(fee);

        let mut spends = self.prepare_spends(&mut ctx, vec![], &actions).await?;

        exercise.add_settlements(&mut spends);

        let deltas = spends.apply(&mut ctx, &actions)?;
        self.complete_spends(&mut ctx, &deltas, spends).await?;

        Ok(ctx.take())
    }

    /// Exercises options that the wallet both minted and owns, returning the underlying assets.
    ///
    /// Since the strike payment goes to the creator, it doesn't need to be funded. Its coin is spent
    /// in the same transaction, and its value pays for the settlement coin that created it. Only
    /// balances within the transaction are enforced, so this is effectively a flash loan, and a coin
    /// of the strike asset of any amount (including 0) is enough to start from. That coin isn't
    /// needed if the strike asset is also the underlying asset. An XCH coin of any amount is needed
    /// for the melted option's value as well, unless XCH is the underlying or strike asset.
    pub async fn cancel_options(
        &self,
        option_ids: Vec<Bytes32>,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        let mut ctx = SpendContext::new();

        let exercise = self.prepare_exercise(&mut ctx, option_ids).await?;

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        for underlying in &exercise.underlyings {
            if !self
                .db
                .is_custody_p2_puzzle_hash(underlying.creator_puzzle_hash)
                .await?
            {
                return Err(WalletError::NotOptionCreator(underlying.launcher_id));
            }

            if timestamp >= underlying.seconds {
                return Err(WalletError::OptionExpired(underlying.launcher_id));
            }
        }

        let mut spends = self.prepare_spends_for_selection(&mut ctx, &[]).await?;

        for &option in &exercise.options {
            spends.add(option);
        }

        exercise.add_settlements(&mut spends);

        for &id in &exercise.strike_ids {
            self.select_any_coin(&mut ctx, &mut spends, id, WalletError::MissingStrikeCoin)
                .await?;
        }

        // The melted option's value is returned as change, which needs an XCH spend as well
        if spends.xch.items.is_empty() {
            self.select_any_coin(&mut ctx, &mut spends, Id::Xch, WalletError::MissingMeltCoin)
                .await?;
        }

        self.select_spends(&mut ctx, &mut spends, &[Action::fee(fee)])
            .await?;

        let mut deltas = spends.apply(&mut ctx, &exercise.actions(fee))?;

        // The strike payments are spent as part of the transaction, rather than being received
        for coin in mem::take(&mut spends.outputs.xch) {
            *deltas.update(Id::Xch) += Delta::new(coin.amount, 0);
            spends.xch.items.push(FungibleSpend::new(coin, true));
        }

        for (id, cats) in mem::take(&mut spends.outputs.cats) {
            let cat_spends = spends.cats.entry(id).or_default();

            for cat in cats {
                *deltas.update(id) += Delta::new(cat.coin.amount, 0);
                cat_spends.items.push(FungibleSpend::new(cat, true));
            }
        }

        self.complete_spends(&mut ctx, &deltas, spends).await?;

        Ok(ctx.take())
    }

    /// Selects a coin of the asset of any amount, unless the spends already have some of it.
    async fn select_any_coin(
        &self,
        ctx: &mut SpendContext,
        spends: &mut Spends,
        id: Id,
        missing: WalletError,
    ) -> Result<(), WalletError> {
        let mut deltas = Deltas::new();
        deltas.update(id);
        deltas.set_needed(id);

        match self
            .select_spends_for_deltas(ctx, spends, deltas, &[])
            .await
        {
            Err(WalletError::CoinSelection(CoinSelectionError::NoSpendableCoins)) => Err(missing),
            result => result,
        }
    }

    async fn prepare_exercise(
        &self,
        ctx: &mut SpendContext,
        option_ids: Vec<Bytes32>,
    ) -> Result<OptionExercise, WalletError> {
        let mut exercise = OptionExercise::default();

        // An option can only be spent once, so repeated ids are ignored
        let option_ids: IndexSet<Bytes32> = option_ids.into_iter().collect();

        for option_id in option_ids {
            let option = self
                .db
                .option(option_id)
                .await?
                .ok_or(WalletError::MissingOption(option_id))?;

            let underlying = self
                .db
                .option_underlying(option_id)
                .await?
                .ok_or(WalletError::MissingOption(option_id))?;

            let underlying_spend = underlying.exercise_spend(
                ctx,
                option.info.inner_puzzle_hash().into(),
                option.coin.amount,
            )?;

            match self
                .db
                .underlying_coin_kind(option_id)
                .await?
                .ok_or(WalletError::MissingOption(option_id))?
            {
                CoinKind::Xch => {
                    let Some(coin) = self.db.xch_coin(option.info.underlying_coin_id).await? else {
                        return Err(WalletError::MissingCoin(option.info.underlying_coin_id));
                    };

                    ctx.spend(coin, underlying_spend)?;

                    exercise.settlement_coins.push(Coin::new(
                        coin.coin_id(),
                        SETTLEMENT_PAYMENT_HASH.into(),
                        coin.amount,
                    ));
                }
                CoinKind::Cat => {
                    let Some(cat) = self.db.cat_coin(option.info.underlying_coin_id).await? else {
                        return Err(WalletError::MissingCatCoin(option.info.underlying_coin_id));
                    };

                    let children = Cat::spend_all(ctx, &[CatSpend::new(cat, underlying_spend)])?;

                    exercise.settlement_cats.push(children[0]);
                }
                kind => {
                    return Err(WalletError::UnsupportedUnderlyingCoinKind(kind));
                }
            }

            exercise.melt_actions.push(Action::melt_singleton(
                Id::Existing(option.info.launcher_id),
                1,
            ));

            match underlying.strike_type {
                OptionType::Xch { amount } => {
                    exercise.strike_ids.insert(Id::Xch);
                    exercise.settle_actions.push(Action::settle(
                        Id::Xch,
                        NotarizedPayment::new(
                            option.info.launcher_id,
                            vec![Payment::new(
                                underlying.creator_puzzle_hash,
                                amount,
                                Memos::None,
                            )],
                        ),
                    ));
                }
                OptionType::Cat { asset_id, amount }
                | OptionType::RevocableCat {
                    asset_id, amount, ..
                } => {
                    let hint = ctx.hint(underlying.creator_puzzle_hash)?;

                    exercise.strike_ids.insert(Id::Existing(asset_id));
                    exercise.settle_actions.push(Action::settle(
                        Id::Existing(asset_id),
                        NotarizedPayment::new(
                            option.info.launcher_id,
                            vec![Payment::new(underlying.creator_puzzle_hash, amount, hint)],
                        ),
                    ));
                }
                OptionType::Nft { .. } => {
                    return Err(WalletError::NftOptionNotSupported);
                }
            }

            exercise.options.push(option);
            exercise.underlyings.push(underlying);
        }

        Ok(exercise)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{TestWallet, WalletError, WalletOptionMint};

    use chia_wallet_sdk::{driver::OptionType, prelude::Bytes32};
    use sage_database::{OptionFilter, OptionSortMode};
    use test_log::test;
    use tokio::time::sleep;

    #[test(tokio::test)]
    async fn test_mint_option() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1001).await?;

        let (coin_spends, asset_id) = test.wallet.issue_cat(1000, 0, None).await?;
        assert_eq!(coin_spends.len(), 2);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let timestamp = test.new_block_with_current_time().await?;

        let (coin_spends, option) = test
            .wallet
            .mint_option(
                WalletOptionMint {
                    expiration_seconds: timestamp + 1000,
                    underlying_type: OptionType::Cat {
                        asset_id,
                        amount: 1000,
                    },
                    strike_type: OptionType::Xch { amount: 2000 },
                },
                0,
            )
            .await?;

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        for _ in 0..2 {
            let coin_spends = test
                .wallet
                .transfer_options(vec![option.info.launcher_id], test.puzzle_hash, 0, None)
                .await?;
            test.transact(coin_spends).await?;

            test.wait_for_coins().await;
        }

        assert_ne!(test.wallet.db.option(option.info.launcher_id).await?, None);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_transfer_option_external() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1001).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;
        assert_eq!(coin_spends.len(), 2);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let timestamp = alice.new_block_with_current_time().await?;

        let (coin_spends, option) = alice
            .wallet
            .mint_option(
                WalletOptionMint {
                    expiration_seconds: timestamp + 1000,
                    underlying_type: OptionType::Cat {
                        asset_id,
                        amount: 1000,
                    },
                    strike_type: OptionType::Xch { amount: 2000 },
                },
                0,
            )
            .await?;

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let option_id = option.info.launcher_id;

        let coin_spends = alice
            .wallet
            .transfer_options(vec![option_id], bob.puzzle_hash, 0, None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        alice.transact(coin_spends).await?;

        alice.wait_for_coins().await;

        assert!(alice.wallet.db.option(option_id).await?.is_none());
        assert!(alice.wallet.db.spendable_option(option_id).await?.is_none());

        bob.wait_for_puzzles().await;

        assert!(bob.wallet.db.option(option_id).await?.is_some());
        assert!(bob.wallet.db.spendable_option(option_id).await?.is_some());

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_option_underlying_history_only_shows_own_creator() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1001).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let timestamp = alice.new_block_with_current_time().await?;

        let (coin_spends, option) = alice
            .wallet
            .mint_option(
                WalletOptionMint {
                    expiration_seconds: timestamp + 1000,
                    underlying_type: OptionType::Cat {
                        asset_id,
                        amount: 1000,
                    },
                    strike_type: OptionType::Xch { amount: 2000 },
                },
                0,
            )
            .await?;

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let underlying_coin_id = option.info.underlying_coin_id;

        // The creator sees their own custody address, rather than the underlying's wrapper
        let creator_puzzle_hash = alice
            .history_custody(underlying_coin_id)
            .await?
            .expect("creator should see their own address");

        assert!(
            alice
                .wallet
                .db
                .is_custody_p2_puzzle_hash(creator_puzzle_hash)
                .await?
        );

        let coin_spends = alice
            .wallet
            .transfer_options(vec![option.info.launcher_id], bob.puzzle_hash, 0, None)
            .await?;

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        // The holder sees nothing, since the underlying coin belongs to someone else
        assert_eq!(bob.history_custody(underlying_coin_id).await?, None);

        Ok(())
    }

    async fn list_options(
        test: &TestWallet,
        filter: OptionFilter,
    ) -> anyhow::Result<Vec<(Bytes32, Option<u32>)>> {
        let (rows, total) = test
            .wallet
            .db
            .owned_options(10, 0, OptionSortMode::Name, true, None, true, filter)
            .await?;

        assert_eq!(rows.len(), total as usize);

        Ok(rows
            .into_iter()
            .map(|row| (row.asset.hash, row.coin_row.spent_height))
            .collect())
    }

    #[test(tokio::test)]
    async fn test_created_option_history() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1001).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let timestamp = alice.new_block_with_current_time().await?;

        let (coin_spends, option) = alice
            .wallet
            .mint_option(
                WalletOptionMint {
                    expiration_seconds: timestamp + 1000,
                    underlying_type: OptionType::Cat {
                        asset_id,
                        amount: 1000,
                    },
                    strike_type: OptionType::Xch { amount: 2000 },
                },
                0,
            )
            .await?;

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let option_id = option.info.launcher_id;

        // While held, the minted option is both owned and created
        for filter in [
            OptionFilter::Owned,
            OptionFilter::Created,
            OptionFilter::All,
        ] {
            let options = list_options(&alice, filter).await?;
            assert_eq!(options.len(), 1);
            assert_eq!(options[0].0, option_id);
            assert_eq!(options[0].1, None);
        }

        let coin_spends = alice
            .wallet
            .transfer_options(vec![option_id], bob.puzzle_hash, 0, None)
            .await?;

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        // Once transferred, it is no longer owned but remains in the creator's history
        assert!(list_options(&alice, OptionFilter::Owned).await?.is_empty());

        for filter in [OptionFilter::Created, OptionFilter::All] {
            let options = list_options(&alice, filter).await?;
            assert_eq!(options.len(), 1);
            assert_eq!(options[0].0, option_id);
            assert!(options[0].1.is_some());
        }

        bob.wait_for_puzzles().await;

        // The recipient owns the option, but did not create it
        assert_eq!(list_options(&bob, OptionFilter::Owned).await?.len(), 1);
        assert!(list_options(&bob, OptionFilter::Created).await?.is_empty());
        assert_eq!(list_options(&bob, OptionFilter::All).await?.len(), 1);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_exercise_option() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1001).await?;
        let mut bob = alice.next(1999).await?; // 1 mojo is returned from melting the option

        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;
        assert_eq!(coin_spends.len(), 2);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let timestamp = alice.new_block_with_current_time().await?;

        let (coin_spends, option) = alice
            .wallet
            .mint_option(
                WalletOptionMint {
                    expiration_seconds: timestamp + 1000,
                    underlying_type: OptionType::Cat {
                        asset_id,
                        amount: 1000,
                    },
                    strike_type: OptionType::Xch { amount: 2000 },
                },
                0,
            )
            .await?;

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let option_id = option.info.launcher_id;

        let coin_spends = alice
            .wallet
            .transfer_options(vec![option_id], bob.puzzle_hash, 0, None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        alice.transact(coin_spends).await?;

        alice.wait_for_coins().await;

        assert!(alice.wallet.db.option(option_id).await?.is_none());
        assert!(alice.wallet.db.spendable_option(option_id).await?.is_none());

        bob.wait_for_puzzles().await;

        assert!(bob.wallet.db.option(option_id).await?.is_some());
        assert!(bob.wallet.db.spendable_option(option_id).await?.is_some());

        let coin_spends = bob.wallet.exercise_options(vec![option_id], 0).await?;

        assert_eq!(coin_spends.len(), 5);

        bob.transact(coin_spends).await?;

        bob.wait_for_coins().await;

        assert!(bob.wallet.db.option(option_id).await?.is_none());
        assert!(bob.wallet.db.spendable_option(option_id).await?.is_none());
        assert_eq!(bob.wallet.db.selectable_xch_balance().await?, 0);
        assert_eq!(bob.wallet.db.selectable_cat_balance(asset_id).await?, 1000);

        Ok(())
    }

    async fn mint_option(
        test: &mut TestWallet,
        underlying_type: OptionType,
        strike_type: OptionType,
    ) -> anyhow::Result<Bytes32> {
        let timestamp = test.new_block_with_current_time().await?;

        let (coin_spends, option) = test
            .wallet
            .mint_option(
                WalletOptionMint {
                    expiration_seconds: timestamp + 1000,
                    underlying_type,
                    strike_type,
                },
                0,
            )
            .await?;

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        Ok(option.info.launcher_id)
    }

    async fn mint_cat_option(
        test: &mut TestWallet,
        asset_id: Bytes32,
        strike_type: OptionType,
    ) -> anyhow::Result<Bytes32> {
        let underlying_type = OptionType::Cat {
            asset_id,
            amount: 1000,
        };

        mint_option(test, underlying_type, strike_type).await
    }

    #[test(tokio::test)]
    async fn test_cancel_option_xch_strike() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1002).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        // The strike amount is far more than the wallet holds
        let option_id =
            mint_cat_option(&mut alice, asset_id, OptionType::Xch { amount: 1_000_000 }).await?;

        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 1);
        assert_eq!(alice.wallet.db.selectable_cat_balance(asset_id).await?, 0);

        let coin_spends = alice.wallet.cancel_options(vec![option_id], 1).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        assert!(alice.wallet.db.option(option_id).await?.is_none());
        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 1);
        assert_eq!(
            alice.wallet.db.selectable_cat_balance(asset_id).await?,
            1000
        );

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_cancel_option_zero_amount_coin() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1001).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let coin_spends = alice
            .wallet
            .send_xch(vec![(alice.puzzle_hash, 0)], 0, vec![], None)
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let option_id =
            mint_cat_option(&mut alice, asset_id, OptionType::Xch { amount: 2000 }).await?;

        // Only the zero amount coin remains
        assert_eq!(alice.wallet.db.selectable_xch_coins().await?.len(), 1);
        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 0);

        let coin_spends = alice.wallet.cancel_options(vec![option_id], 0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        assert!(alice.wallet.db.option(option_id).await?.is_none());
        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 1);
        assert_eq!(
            alice.wallet.db.selectable_cat_balance(asset_id).await?,
            1000
        );

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_cancel_option_cat_strike() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1102).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, strike_asset_id) = alice.wallet.issue_cat(100, 0, None).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let strike_type = OptionType::Cat {
            asset_id: strike_asset_id,
            amount: 1_000_000,
        };

        let option_id = mint_cat_option(&mut alice, asset_id, strike_type).await?;

        let coin_spends = alice.wallet.cancel_options(vec![option_id], 0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        assert!(alice.wallet.db.option(option_id).await?.is_none());
        assert_eq!(
            alice.wallet.db.selectable_cat_balance(asset_id).await?,
            1000
        );
        assert_eq!(
            alice
                .wallet
                .db
                .selectable_cat_balance(strike_asset_id)
                .await?,
            100
        );

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_cancel_multiple_options_with_same_strike() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(2003).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(2000, 0, None).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let strike_type = OptionType::Xch { amount: 5000 };

        let first = mint_cat_option(&mut alice, asset_id, strike_type).await?;
        let second = mint_cat_option(&mut alice, asset_id, strike_type).await?;

        let coin_spends = alice.wallet.cancel_options(vec![first, second], 0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        assert!(alice.wallet.db.option(first).await?.is_none());
        assert!(alice.wallet.db.option(second).await?.is_none());
        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 3);
        assert_eq!(
            alice.wallet.db.selectable_cat_balance(asset_id).await?,
            2000
        );

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_cancel_option_same_asset() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1002).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let option_id = mint_cat_option(
            &mut alice,
            asset_id,
            OptionType::Cat {
                asset_id,
                amount: 5000,
            },
        )
        .await?;

        assert_eq!(alice.wallet.db.selectable_cat_balance(asset_id).await?, 0);

        let coin_spends = alice.wallet.cancel_options(vec![option_id], 0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        assert!(alice.wallet.db.option(option_id).await?.is_none());
        assert_eq!(
            alice.wallet.db.selectable_cat_balance(asset_id).await?,
            1000
        );

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_cancel_option_revocable_cat_strike() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1102).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let hidden_puzzle_hash = Bytes32::new([1; 32]);

        let (coin_spends, strike_asset_id) = alice
            .wallet
            .issue_cat_with_hidden_puzzle_hash(100, 0, None, Some(hidden_puzzle_hash))
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let option_id = mint_cat_option(
            &mut alice,
            asset_id,
            OptionType::RevocableCat {
                asset_id: strike_asset_id,
                hidden_puzzle_hash,
                amount: 5000,
            },
        )
        .await?;

        let coin_spends = alice.wallet.cancel_options(vec![option_id], 0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        assert!(alice.wallet.db.option(option_id).await?.is_none());
        assert_eq!(
            alice.wallet.db.selectable_cat_balance(asset_id).await?,
            1000
        );
        assert_eq!(
            alice
                .wallet
                .db
                .selectable_cat_balance(strike_asset_id)
                .await?,
            100
        );

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_cancel_option_xch_underlying_xch_strike() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1001).await?;

        let option_id = mint_option(
            &mut alice,
            OptionType::Xch { amount: 1000 },
            OptionType::Xch { amount: 5000 },
        )
        .await?;

        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 0);

        let coin_spends = alice.wallet.cancel_options(vec![option_id], 0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        assert!(alice.wallet.db.option(option_id).await?.is_none());
        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 1001);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_cancel_option_xch_underlying_cat_strike() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1101).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(100, 0, None).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let option_id = mint_option(
            &mut alice,
            OptionType::Xch { amount: 1000 },
            OptionType::Cat {
                asset_id,
                amount: 5000,
            },
        )
        .await?;

        let coin_spends = alice.wallet.cancel_options(vec![option_id], 0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        assert!(alice.wallet.db.option(option_id).await?.is_none());
        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 1001);
        assert_eq!(alice.wallet.db.selectable_cat_balance(asset_id).await?, 100);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_cancel_option_requires_xch_coin() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1101).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, strike_asset_id) = alice.wallet.issue_cat(100, 0, None).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let option_id = mint_cat_option(
            &mut alice,
            asset_id,
            OptionType::Cat {
                asset_id: strike_asset_id,
                amount: 5000,
            },
        )
        .await?;

        assert!(alice.wallet.db.selectable_xch_coins().await?.is_empty());

        assert!(matches!(
            alice.wallet.cancel_options(vec![option_id], 0).await,
            Err(WalletError::MissingMeltCoin)
        ));

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_cancel_option_requires_unexpired() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1002).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let timestamp = alice.new_block_with_current_time().await?;

        let (coin_spends, option) = alice
            .wallet
            .mint_option(
                WalletOptionMint {
                    expiration_seconds: timestamp - 1,
                    underlying_type: OptionType::Cat {
                        asset_id,
                        amount: 1000,
                    },
                    strike_type: OptionType::Xch { amount: 2000 },
                },
                0,
            )
            .await?;

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let option_id = option.info.launcher_id;

        assert!(matches!(
            alice.wallet.cancel_options(vec![option_id], 0).await,
            Err(WalletError::OptionExpired(id)) if id == option_id
        ));

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_cancel_option_repeated_id() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1002).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let option_id =
            mint_cat_option(&mut alice, asset_id, OptionType::Xch { amount: 2000 }).await?;

        let coin_spends = alice
            .wallet
            .cancel_options(vec![option_id, option_id], 0)
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        assert!(alice.wallet.db.option(option_id).await?.is_none());
        assert_eq!(
            alice.wallet.db.selectable_cat_balance(asset_id).await?,
            1000
        );

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_cancel_option_xch_underlying_with_fee() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1001).await?;

        let option_id = mint_option(
            &mut alice,
            OptionType::Xch { amount: 1000 },
            OptionType::Xch { amount: 5000 },
        )
        .await?;

        let coin_spends = alice.wallet.cancel_options(vec![option_id], 1).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        assert!(alice.wallet.db.option(option_id).await?.is_none());
        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 1000);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_cancel_multiple_xch_underlying_options_with_same_strike() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(2002).await?;

        let strike_type = OptionType::Xch { amount: 5000 };

        let first = mint_option(&mut alice, OptionType::Xch { amount: 1000 }, strike_type).await?;
        let second = mint_option(&mut alice, OptionType::Xch { amount: 1000 }, strike_type).await?;

        let coin_spends = alice.wallet.cancel_options(vec![first, second], 0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        assert!(alice.wallet.db.option(first).await?.is_none());
        assert!(alice.wallet.db.option(second).await?.is_none());
        assert_eq!(alice.wallet.db.selectable_xch_balance().await?, 2002);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_is_creator_requires_custody() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1001).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let option_id =
            mint_cat_option(&mut alice, asset_id, OptionType::Xch { amount: 2000 }).await?;

        let coin_spends = alice
            .wallet
            .transfer_options(vec![option_id], bob.puzzle_hash, 0, None)
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        let creator_puzzle_hash = bob
            .wallet
            .db
            .option_underlying(option_id)
            .await?
            .expect("option should exist")
            .creator_puzzle_hash;

        // A clawback puzzle is known to the wallet, but doesn't make it the creator
        sqlx::query("INSERT INTO p2_puzzles (hash, kind) VALUES (?, 1)")
            .bind(creator_puzzle_hash.as_ref())
            .execute(&bob.pool)
            .await?;

        let row = bob.wallet.db.wallet_option(option_id).await?.unwrap();
        assert!(!row.is_creator);

        let (rows, _) = bob
            .wallet
            .db
            .owned_options(
                10,
                0,
                OptionSortMode::Name,
                true,
                None,
                true,
                OptionFilter::Owned,
            )
            .await?;
        assert!(!rows[0].is_creator);

        assert!(list_options(&bob, OptionFilter::Created).await?.is_empty());

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_cancel_option_requires_strike_coin() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1001).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let option_id =
            mint_cat_option(&mut alice, asset_id, OptionType::Xch { amount: 2000 }).await?;

        assert!(alice.wallet.db.selectable_xch_coins().await?.is_empty());

        assert!(matches!(
            alice.wallet.cancel_options(vec![option_id], 0).await,
            Err(WalletError::MissingStrikeCoin)
        ));

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_cancel_option_requires_creator() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1001).await?;
        let mut bob = alice.next(1).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let option_id =
            mint_cat_option(&mut alice, asset_id, OptionType::Xch { amount: 2000 }).await?;

        let coin_spends = alice
            .wallet
            .transfer_options(vec![option_id], bob.puzzle_hash, 0, None)
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;
        bob.wait_for_puzzles().await;

        assert!(matches!(
            bob.wallet.cancel_options(vec![option_id], 0).await,
            Err(WalletError::NotOptionCreator(id)) if id == option_id
        ));

        let row = bob.wallet.db.wallet_option(option_id).await?.unwrap();
        assert!(!row.is_creator);

        let (rows, _) = bob
            .wallet
            .db
            .owned_options(
                10,
                0,
                OptionSortMode::Name,
                true,
                None,
                true,
                OptionFilter::Owned,
            )
            .await?;
        assert!(!rows[0].is_creator);

        let (rows, _) = alice
            .wallet
            .db
            .owned_options(
                10,
                0,
                OptionSortMode::Name,
                true,
                None,
                true,
                OptionFilter::All,
            )
            .await?;
        assert!(rows[0].is_creator);

        let row = alice.wallet.db.wallet_option(option_id).await?.unwrap();
        assert!(row.is_creator);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_expired_option() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1001).await?;
        let mut bob = alice.next(1999).await?; // 1 mojo is returned from melting the option

        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;
        assert_eq!(coin_spends.len(), 2);

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let timestamp = alice.new_block_with_current_time().await?;

        let (coin_spends, option) = alice
            .wallet
            .mint_option(
                WalletOptionMint {
                    expiration_seconds: timestamp + 5,
                    underlying_type: OptionType::Cat {
                        asset_id,
                        amount: 1000,
                    },
                    strike_type: OptionType::Xch { amount: 2000 },
                },
                0,
            )
            .await?;

        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let option_id = option.info.launcher_id;

        let coin_spends = alice
            .wallet
            .transfer_options(vec![option_id], bob.puzzle_hash, 0, None)
            .await?;

        assert_eq!(coin_spends.len(), 1);

        alice.transact(coin_spends).await?;

        alice.wait_for_coins().await;

        assert!(alice.wallet.db.option(option_id).await?.is_none());
        assert!(alice.wallet.db.spendable_option(option_id).await?.is_none());

        bob.wait_for_puzzles().await;

        assert!(bob.wallet.db.option(option_id).await?.is_some());
        assert!(bob.wallet.db.spendable_option(option_id).await?.is_some());

        assert_eq!(alice.wallet.db.selectable_cat_balance(asset_id).await?, 0);

        sleep(Duration::from_secs(6)).await;
        alice.new_block_with_current_time().await?;

        assert_eq!(
            alice.wallet.db.selectable_cat_balance(asset_id).await?,
            1000
        );

        let coin_spends = alice
            .wallet
            .send_cat(
                asset_id,
                vec![(alice.puzzle_hash, 1000)],
                0,
                true,
                vec![],
                None,
            )
            .await?;

        assert_eq!(coin_spends.len(), 1);

        alice.transact(coin_spends).await?;

        alice.wait_for_coins().await;

        Ok(())
    }
}
