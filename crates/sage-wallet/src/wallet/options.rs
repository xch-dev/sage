use chia_wallet_sdk::{
    chia::puzzle_types::offer::{NotarizedPayment, Payment},
    prelude::*,
    puzzles::SETTLEMENT_PAYMENT_HASH,
};
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
        let mut actions = vec![Action::fee(fee)];

        let mut settlement_coins = Vec::new();
        let mut settlement_cats = Vec::new();

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
                &mut ctx,
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

                    settlement_coins.push(Coin::new(
                        coin.coin_id(),
                        SETTLEMENT_PAYMENT_HASH.into(),
                        coin.amount,
                    ));
                }
                CoinKind::Cat => {
                    let Some(cat) = self.db.cat_coin(option.info.underlying_coin_id).await? else {
                        return Err(WalletError::MissingCatCoin(option.info.underlying_coin_id));
                    };

                    let children =
                        Cat::spend_all(&mut ctx, &[CatSpend::new(cat, underlying_spend)])?;

                    settlement_cats.push(children[0]);
                }
                kind => {
                    return Err(WalletError::UnsupportedUnderlyingCoinKind(kind));
                }
            }

            actions.push(Action::melt_singleton(
                Id::Existing(option.info.launcher_id),
                1,
            ));

            match underlying.strike_type {
                OptionType::Xch { amount } => {
                    actions.push(Action::settle(
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

                    actions.push(Action::settle(
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
        }

        let mut spends = self.prepare_spends(&mut ctx, vec![], &actions).await?;

        for coin in settlement_coins {
            spends.add(coin);
        }

        for cat in settlement_cats {
            spends.add(cat);
        }

        let deltas = spends.apply(&mut ctx, &actions)?;
        self.complete_spends(&mut ctx, &deltas, spends).await?;

        Ok(ctx.take())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::{TestWallet, WalletOptionMint};

    use chia_wallet_sdk::driver::OptionType;
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
