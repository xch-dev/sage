use chia::protocol::{Bytes32, CoinSpend};
use chia_wallet_sdk::driver::{Did, HashedPtr};

use crate::WalletError;

use super::{CreateDidAction, Id, NormalizeDidAction, SpendAction, TransferDidAction, Wallet};

impl Wallet {
    pub async fn create_did(
        &self,
        fee: u64,
    ) -> Result<(Vec<CoinSpend>, Did<HashedPtr>), WalletError> {
        let result = self
            .transact(vec![SpendAction::CreateDid(CreateDidAction)], fee)
            .await?;

        Ok((
            result.coin_spends,
            result
                .new_assets
                .dids
                .values()
                .copied()
                .next()
                .expect("no did"),
        ))
    }

    pub async fn transfer_dids(
        &self,
        did_ids: Vec<Bytes32>,
        puzzle_hash: Bytes32,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        Ok(self
            .transact(
                did_ids
                    .into_iter()
                    .map(|did_id| {
                        SpendAction::TransferDid(TransferDidAction::new(
                            Id::Existing(did_id),
                            puzzle_hash,
                        ))
                    })
                    .collect(),
                fee,
            )
            .await?
            .coin_spends)
    }

    pub async fn normalize_dids(
        &self,
        did_ids: Vec<Bytes32>,
        fee: u64,
    ) -> Result<Vec<CoinSpend>, WalletError> {
        Ok(self
            .transact(
                did_ids
                    .into_iter()
                    .map(|did_id| {
                        SpendAction::NormalizeDid(NormalizeDidAction::new(Id::Existing(did_id)))
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

    use test_log::test;

    #[test(tokio::test)]
    async fn test_create_did() -> anyhow::Result<()> {
        let mut test = TestWallet::new(1).await?;

        let (coin_spends, did) = test.wallet.create_did(0).await?;
        test.transact(coin_spends).await?;
        test.wait_for_coins().await;

        for _ in 0..2 {
            let coin_spends = test
                .wallet
                .transfer_dids(vec![did.info.launcher_id], test.puzzle_hash, 0)
                .await?;
            test.transact(coin_spends).await?;
            test.wait_for_coins().await;
        }

        assert_ne!(
            test.wallet.db.spendable_did(did.info.launcher_id).await?,
            None
        );

        Ok(())
    }
}
