use chia::protocol::{Bytes32, CoinSpend};
use chia_wallet_sdk::driver::{OptionContract, OptionType};

use crate::WalletError;

use super::{Id, MintOptionAction, SpendAction, TransferOptionAction, Wallet};

impl Wallet {
    pub async fn mint_option(
        &self,
        seconds: u64,
        underlying_type: OptionType,
        strike_type: OptionType,
        fee: u64,
    ) -> Result<(Vec<CoinSpend>, OptionContract), WalletError> {
        let creator_puzzle_hash = self.p2_puzzle_hash(false, true).await?;
        let owner_puzzle_hash = self.p2_puzzle_hash(false, true).await?;

        let result = self
            .transact(
                vec![SpendAction::MintOption(MintOptionAction::new(
                    creator_puzzle_hash,
                    owner_puzzle_hash,
                    seconds,
                    underlying_type.into(),
                    strike_type.into(),
                ))],
                fee,
            )
            .await?;

        Ok((
            result.coin_spends,
            result
                .unspent_assets
                .options
                .into_values()
                .next()
                .expect("no option"),
        ))
    }

    pub async fn transfer_options(
        &self,
        option_ids: Vec<Bytes32>,
        puzzle_hash: Bytes32,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        Ok(self
            .transact(
                option_ids
                    .into_iter()
                    .map(|option_id| {
                        SpendAction::TransferOption(TransferOptionAction::new(
                            Id::Existing(option_id),
                            puzzle_hash,
                        ))
                    })
                    .collect(),
                fee,
            )
            .await?
            .coin_spends)
    }
}

#[cfg(test)]
mod tests {
    use crate::TestWallet;

    use chia::protocol::Bytes32;
    use chia_wallet_sdk::driver::OptionType;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_mint_option() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1001).await?;

        let (coin_spends, option) = test
            .wallet
            .mint_option(
                0,
                OptionType::Xch { amount: 1000 },
                OptionType::Cat {
                    asset_id: Bytes32::default(),
                    amount: 1000,
                },
                0,
            )
            .await?;

        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        for _ in 0..2 {
            let coin_spends = test
                .wallet
                .transfer_options(vec![option.info.launcher_id], test.puzzle_hash, 0)
                .await?;
            test.transact(coin_spends).await?;
            test.wait_for_coins().await;
        }

        assert_eq!(test.wallet.db.balance().await?, 0);

        assert_ne!(
            test.wallet
                .db
                .spendable_option(option.info.launcher_id)
                .await?,
            None
        );

        Ok(())
    }
}
