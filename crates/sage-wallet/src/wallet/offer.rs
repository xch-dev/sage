mod lock_assets;
mod make_offer;
mod offer_coins;
mod parse_offer;
mod royalties;
mod take_offer;
mod unlock_assets;

pub use lock_assets::*;
pub use make_offer::*;
pub use offer_coins::*;
pub use parse_offer::*;
pub use royalties::*;
pub use take_offer::*;
pub use unlock_assets::*;

#[cfg(test)]
mod tests {
    use chia_wallet_sdk::{AggSigConstants, TESTNET11_CONSTANTS};
    use indexmap::{indexmap, IndexMap};
    use sqlx::SqlitePool;
    use test_log::test;

    use crate::{MakerSide, SyncEvent, TakerSide, TestWallet};

    #[test(sqlx::test)]
    async fn test_offer_xch_for_cat(pool: SqlitePool) -> anyhow::Result<()> {
        let mut test = TestWallet::new(pool, 2000).await?;

        // Issue CAT
        let (coin_spends, asset_id) = test.wallet.issue_cat(1000, 0, None, false, true).await?;
        test.transact(coin_spends).await?;
        test.consume_until(SyncEvent::CoinState).await;

        // Create offer
        let offer = test
            .wallet
            .make_offer(
                MakerSide {
                    xch: 750,
                    cats: IndexMap::new(),
                    nfts: Vec::new(),
                    fee: 250,
                },
                TakerSide {
                    xch: 0,
                    cats: indexmap! { asset_id => 1000 },
                    nfts: IndexMap::new(),
                },
                false,
                true,
            )
            .await?;
        let offer = test
            .wallet
            .sign_make_offer(
                offer,
                &AggSigConstants::new(TESTNET11_CONSTANTS.agg_sig_me_additional_data),
                test.master_sk.clone(),
            )
            .await?;

        // Take offer
        let offer = test.wallet.take_offer(offer, 0, false, true).await?;
        let spend_bundle = test
            .wallet
            .sign_take_offer(
                offer,
                &AggSigConstants::new(TESTNET11_CONSTANTS.agg_sig_me_additional_data),
                test.master_sk.clone(),
            )
            .await?;
        test.push_bundle(spend_bundle).await?;
        test.consume_until(SyncEvent::CoinState).await;

        Ok(())
    }
}
