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
    use chia::{
        clvm_traits::{FromClvm, ToClvm},
        protocol::{Bytes32, Program},
        puzzles::nft::NftMetadata,
    };
    use chia_wallet_sdk::{AggSigConstants, TESTNET11_CONSTANTS};
    use clvmr::Allocator;
    use indexmap::{indexmap, IndexMap};
    use sqlx::SqlitePool;
    use test_log::test;

    use crate::{MakerSide, RequestedNft, SyncEvent, TakerSide, TestWallet, WalletNftMint};

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

    #[test(sqlx::test)]
    async fn test_xch_for_nft(pool: SqlitePool) -> anyhow::Result<()> {
        let mut test = TestWallet::new(pool, 1032).await?;

        let (coin_spends, did) = test.wallet.create_did(0, false, true).await?;
        test.transact(coin_spends).await?;
        test.consume_until(SyncEvent::CoinState).await;

        let (coin_spends, mut nfts, _did) = test
            .wallet
            .bulk_mint_nfts(
                0,
                did.info.launcher_id,
                vec![WalletNftMint {
                    metadata: NftMetadata::default(),
                    royalty_puzzle_hash: Some(Bytes32::default()),
                    royalty_ten_thousandths: 300,
                }],
                false,
                true,
            )
            .await?;
        test.transact(coin_spends).await?;
        test.consume_until(SyncEvent::PuzzleBatchSynced).await;

        let nft = nfts.remove(0);

        let mut allocator = Allocator::new();
        let metadata = nft.info.metadata.to_clvm(&mut allocator)?;
        let metadata = Program::from_clvm(&allocator, metadata)?;

        // Create offer
        let offer = test
            .wallet
            .make_offer(
                MakerSide {
                    xch: 1000,
                    cats: IndexMap::new(),
                    nfts: Vec::new(),
                    fee: 0,
                },
                TakerSide {
                    xch: 0,
                    cats: IndexMap::new(),
                    nfts: indexmap! {
                        nft.info.launcher_id => RequestedNft {
                            metadata,
                            metadata_updater_puzzle_hash: nft.info.metadata_updater_puzzle_hash,
                            royalty_puzzle_hash: nft.info.royalty_puzzle_hash,
                            royalty_ten_thousandths: nft.info.royalty_ten_thousandths,
                        },
                    },
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

    #[test(sqlx::test)]
    async fn test_nft_for_xch(pool: SqlitePool) -> anyhow::Result<()> {
        let mut test = TestWallet::new(pool, 1032).await?;

        let (coin_spends, did) = test.wallet.create_did(0, false, true).await?;
        test.transact(coin_spends).await?;
        test.consume_until(SyncEvent::CoinState).await;

        let (coin_spends, mut nfts, _did) = test
            .wallet
            .bulk_mint_nfts(
                0,
                did.info.launcher_id,
                vec![WalletNftMint {
                    metadata: NftMetadata::default(),
                    royalty_puzzle_hash: Some(Bytes32::default()),
                    royalty_ten_thousandths: 300,
                }],
                false,
                true,
            )
            .await?;
        test.transact(coin_spends).await?;
        test.consume_until(SyncEvent::PuzzleBatchSynced).await;

        let nft = nfts.remove(0);

        // Create offer
        let offer = test
            .wallet
            .make_offer(
                MakerSide {
                    xch: 0,
                    cats: IndexMap::new(),
                    nfts: vec![nft.info.launcher_id],
                    fee: 0,
                },
                TakerSide {
                    xch: 1000,
                    cats: IndexMap::new(),
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
