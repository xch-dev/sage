use chia::{
    clvm_utils::ToTreeHash,
    protocol::{Bytes32, CoinSpend},
};
use chia_wallet_sdk::driver::{Action, ClawbackV2, Id, OptionContract, OptionType, SpendContext};

use crate::{
    wallet::memos::{calculate_memos, Hint},
    WalletError,
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
        let creator_puzzle_hash = self.p2_puzzle_hash(false, true).await?;

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
        let sender_puzzle_hash = self.p2_puzzle_hash(false, true).await?;

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
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::{TestWallet, WalletOptionMint};

    use chia_wallet_sdk::driver::OptionType;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_mint_option() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1001).await?;

        let (coin_spends, asset_id) = test.wallet.issue_cat(1000, 0, None).await?;
        assert_eq!(coin_spends.len(), 2);

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

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
}
