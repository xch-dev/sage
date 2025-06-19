mod aggregate_offer;
mod cancel_offer;
mod lock_assets;
mod make_offer;
mod offer_coins;
mod parse_offer;
mod payments;
mod royalties;
mod take_offer;
mod unlock_assets;

pub use aggregate_offer::*;
pub use lock_assets::*;
pub use make_offer::*;
pub use offer_coins::*;
pub use parse_offer::*;
pub use payments::*;
pub use royalties::*;
pub use take_offer::*;
pub use unlock_assets::*;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use chia::{
        clvm_traits::{FromClvm, ToClvm},
        protocol::{Bytes32, Program},
        puzzles::nft::NftMetadata,
    };
    use clvmr::Allocator;
    use indexmap::indexmap;
    use test_log::test;

    use crate::{
        default_test_options, MakerSide, RequestedNft, SyncOptions, TakerSide, TestWallet,
        Timeouts, WalletNftMint,
    };

    use super::aggregate_offers;

    #[test(tokio::test)]
    async fn test_offer_xch_for_cat() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(1000).await?;

        // Issue CAT
        let (coin_spends, asset_id) = bob.wallet.issue_cat(1000, 0, None).await?;
        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;

        // Create offer
        let offer = alice
            .wallet
            .make_offer(
                MakerSide {
                    xch: 750,
                    fee: 250,
                    ..Default::default()
                },
                TakerSide {
                    cats: indexmap! { asset_id => 1000 },
                    ..Default::default()
                },
                None,
                false,
                true,
            )
            .await?;
        let offer = alice
            .wallet
            .sign_make_offer(offer, &alice.agg_sig, alice.master_sk.clone())
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0, false, true).await?;
        let spend_bundle = bob
            .wallet
            .sign_take_offer(offer, &bob.agg_sig, bob.master_sk.clone())
            .await?;
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        bob.wait_for_coins().await;
        alice.wait_for_puzzles().await;

        // Check balances
        assert_eq!(alice.wallet.db.cat_balance(asset_id).await?, 1000);
        assert_eq!(bob.wallet.db.balance().await?, 750);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_offer_xch_for_nft() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1030).await?;
        let mut bob = alice.next(2).await?;

        let (coin_spends, did) = bob.wallet.create_did(0, false, true).await?;
        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;

        let (coin_spends, mut nfts, _did) = bob
            .wallet
            .bulk_mint_nfts(
                0,
                did.info.launcher_id,
                vec![WalletNftMint {
                    metadata: NftMetadata::default(),
                    p2_puzzle_hash: None,
                    royalty_puzzle_hash: Some(Bytes32::default()),
                    royalty_basis_points: 300,
                }],
                false,
                true,
            )
            .await?;
        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;

        let nft = nfts.remove(0);

        let mut allocator = Allocator::new();
        let metadata = nft.info.metadata.to_clvm(&mut allocator)?;
        let metadata = Program::from_clvm(&allocator, metadata)?;

        // Create offer
        let offer = alice
            .wallet
            .make_offer(
                MakerSide {
                    xch: 1000,
                    ..Default::default()
                },
                TakerSide {
                    nfts: indexmap! {
                        nft.info.launcher_id => RequestedNft {
                            metadata,
                            metadata_updater_puzzle_hash: nft.info.metadata_updater_puzzle_hash,
                            royalty_puzzle_hash: nft.info.royalty_puzzle_hash,
                            royalty_basis_points: nft.info.royalty_basis_points,
                        },
                    },
                    ..Default::default()
                },
                None,
                false,
                true,
            )
            .await?;
        let offer = alice
            .wallet
            .sign_make_offer(offer, &alice.agg_sig, alice.master_sk.clone())
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0, false, true).await?;
        let spend_bundle = bob
            .wallet
            .sign_take_offer(offer, &bob.agg_sig, bob.master_sk.clone())
            .await?;
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        bob.wait_for_coins().await;
        alice.wait_for_puzzles().await;

        // Check balances
        assert_ne!(
            alice.wallet.db.spendable_nft(nft.info.launcher_id).await?,
            None
        );
        assert_eq!(bob.wallet.db.balance().await?, 1000);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_offer_nft_for_xch() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(2).await?;
        let mut bob = alice.next(1030).await?;

        let (coin_spends, did) = alice.wallet.create_did(0, false, true).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, mut nfts, _did) = alice
            .wallet
            .bulk_mint_nfts(
                0,
                did.info.launcher_id,
                vec![WalletNftMint {
                    metadata: NftMetadata::default(),
                    p2_puzzle_hash: None,
                    royalty_puzzle_hash: Some(Bytes32::default()),
                    royalty_basis_points: 300,
                }],
                false,
                true,
            )
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let nft = nfts.remove(0);

        // Create offer
        let offer = alice
            .wallet
            .make_offer(
                MakerSide {
                    nfts: vec![nft.info.launcher_id],
                    ..Default::default()
                },
                TakerSide {
                    xch: 1000,
                    ..Default::default()
                },
                None,
                false,
                true,
            )
            .await?;
        let offer = alice
            .wallet
            .sign_make_offer(offer, &alice.agg_sig, alice.master_sk.clone())
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0, false, true).await?;
        let spend_bundle = bob
            .wallet
            .sign_take_offer(offer, &bob.agg_sig, bob.master_sk.clone())
            .await?;
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        alice.wait_for_coins().await;
        bob.wait_for_coins().await;

        // Check balances
        assert_eq!(alice.wallet.db.balance().await?, 1000);
        assert_ne!(
            bob.wallet.db.spendable_nft(nft.info.launcher_id).await?,
            None
        );

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_offer_nft_same_royalties_for_xch() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(3).await?;
        let mut bob = alice.next(1030).await?;

        let (coin_spends, did) = alice.wallet.create_did(0, false, true).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, mut nfts, _did) = alice
            .wallet
            .bulk_mint_nfts(
                0,
                did.info.launcher_id,
                vec![
                    WalletNftMint {
                        metadata: NftMetadata::default(),
                        p2_puzzle_hash: None,
                        royalty_puzzle_hash: Some(Bytes32::default()),
                        royalty_basis_points: 300,
                    },
                    WalletNftMint {
                        metadata: NftMetadata::default(),
                        p2_puzzle_hash: None,
                        royalty_puzzle_hash: Some(Bytes32::default()),
                        royalty_basis_points: 300,
                    },
                ],
                false,
                true,
            )
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let nft_id_first = nfts.remove(0);
        let nft_id_second = nfts.remove(0);

        // Create offer
        let offer = alice
            .wallet
            .make_offer(
                MakerSide {
                    nfts: vec![
                        nft_id_first.info.launcher_id,
                        nft_id_second.info.launcher_id,
                    ],
                    ..Default::default()
                },
                TakerSide {
                    xch: 1000,
                    ..Default::default()
                },
                None,
                false,
                true,
            )
            .await?;
        let offer = alice
            .wallet
            .sign_make_offer(offer, &alice.agg_sig, alice.master_sk.clone())
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0, false, true).await?;
        let spend_bundle = bob
            .wallet
            .sign_take_offer(offer, &bob.agg_sig, bob.master_sk.clone())
            .await?;
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        alice.wait_for_coins().await;
        bob.wait_for_coins().await;

        // Check balances
        assert_eq!(alice.wallet.db.balance().await?, 1000);
        assert_ne!(
            bob.wallet
                .db
                .spendable_nft(nft_id_first.info.launcher_id)
                .await?,
            None
        );
        assert_ne!(
            bob.wallet
                .db
                .spendable_nft(nft_id_second.info.launcher_id)
                .await?,
            None
        );

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_offer_nft_same_royalties_for_cat() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(3).await?;
        let mut bob = alice.next(1030).await?;

        let (coin_spends, did) = alice.wallet.create_did(0, false, true).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, mut nfts, _did) = alice
            .wallet
            .bulk_mint_nfts(
                0,
                did.info.launcher_id,
                vec![
                    WalletNftMint {
                        metadata: NftMetadata::default(),
                        p2_puzzle_hash: None,
                        royalty_puzzle_hash: Some(Bytes32::default()),
                        royalty_basis_points: 300,
                    },
                    WalletNftMint {
                        metadata: NftMetadata::default(),
                        p2_puzzle_hash: None,
                        royalty_puzzle_hash: Some(Bytes32::default()),
                        royalty_basis_points: 300,
                    },
                ],
                false,
                true,
            )
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let nft_id_first = nfts.remove(0);
        let nft_id_second = nfts.remove(0);

        // Issue CAT
        let (coin_spends, asset_id) = bob.wallet.issue_cat(1030, 0, None).await?;
        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;

        // Create offer
        let offer = alice
            .wallet
            .make_offer(
                MakerSide {
                    nfts: vec![
                        nft_id_first.info.launcher_id,
                        nft_id_second.info.launcher_id,
                    ],
                    ..Default::default()
                },
                TakerSide {
                    cats: indexmap! { asset_id => 1000 },
                    ..Default::default()
                },
                None,
                false,
                true,
            )
            .await?;
        let offer = alice
            .wallet
            .sign_make_offer(offer, &alice.agg_sig, alice.master_sk.clone())
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0, false, true).await?;
        let spend_bundle = bob
            .wallet
            .sign_take_offer(offer, &bob.agg_sig, bob.master_sk.clone())
            .await?;
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        alice.wait_for_puzzles().await;
        bob.wait_for_coins().await;

        // Check balances
        assert_eq!(alice.wallet.db.cat_balance(asset_id).await?, 1000);
        assert_ne!(
            bob.wallet
                .db
                .spendable_nft(nft_id_first.info.launcher_id)
                .await?,
            None
        );
        assert_ne!(
            bob.wallet
                .db
                .spendable_nft(nft_id_second.info.launcher_id)
                .await?,
            None
        );

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_offer_nft_mixed_royalties_for_xch() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(3).await?;
        let mut bob = alice.next(1030).await?;

        let (coin_spends, did) = alice.wallet.create_did(0, false, true).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, mut nfts, _did) = alice
            .wallet
            .bulk_mint_nfts(
                0,
                did.info.launcher_id,
                vec![
                    WalletNftMint {
                        metadata: NftMetadata::default(),
                        p2_puzzle_hash: None,
                        royalty_puzzle_hash: Some(Bytes32::default()),
                        royalty_basis_points: 300,
                    },
                    WalletNftMint {
                        metadata: NftMetadata::default(),
                        p2_puzzle_hash: None,
                        royalty_puzzle_hash: Some(Bytes32::default()),
                        royalty_basis_points: 0,
                    },
                ],
                false,
                true,
            )
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let nft_id_first = nfts.remove(0);
        let nft_id_second = nfts.remove(0);

        // Create offer
        let offer = alice
            .wallet
            .make_offer(
                MakerSide {
                    nfts: vec![
                        nft_id_first.info.launcher_id,
                        nft_id_second.info.launcher_id,
                    ],
                    ..Default::default()
                },
                TakerSide {
                    xch: 1000,
                    ..Default::default()
                },
                None,
                false,
                true,
            )
            .await?;
        let offer = alice
            .wallet
            .sign_make_offer(offer, &alice.agg_sig, alice.master_sk.clone())
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0, false, true).await?;
        let spend_bundle = bob
            .wallet
            .sign_take_offer(offer, &bob.agg_sig, bob.master_sk.clone())
            .await?;
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        alice.wait_for_coins().await;
        bob.wait_for_coins().await;

        // Check balances
        assert_eq!(alice.wallet.db.balance().await?, 1000);
        assert_ne!(
            bob.wallet
                .db
                .spendable_nft(nft_id_first.info.launcher_id)
                .await?,
            None
        );
        assert_ne!(
            bob.wallet
                .db
                .spendable_nft(nft_id_second.info.launcher_id)
                .await?,
            None
        );

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_offer_nft_for_xch_aggregate() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(3).await?;
        let mut bob = alice.next(1030).await?;

        let (coin_spends, did) = alice.wallet.create_did(0, false, true).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, mut nfts, _did) = alice
            .wallet
            .bulk_mint_nfts(
                0,
                did.info.launcher_id,
                vec![
                    WalletNftMint {
                        metadata: NftMetadata::default(),
                        p2_puzzle_hash: None,
                        royalty_puzzle_hash: Some(Bytes32::default()),
                        royalty_basis_points: 300,
                    },
                    WalletNftMint {
                        metadata: NftMetadata::default(),
                        p2_puzzle_hash: None,
                        royalty_puzzle_hash: Some(Bytes32::default()),
                        royalty_basis_points: 300,
                    },
                ],
                false,
                true,
            )
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let nft_first = nfts.remove(0);
        let nft_second = nfts.remove(0);

        // Create first offer
        let first_offer = alice
            .wallet
            .make_offer(
                MakerSide {
                    nfts: vec![nft_first.info.launcher_id],
                    ..Default::default()
                },
                TakerSide {
                    xch: 500,
                    ..Default::default()
                },
                None,
                false,
                true,
            )
            .await?;
        let first_offer = alice
            .wallet
            .sign_make_offer(first_offer, &alice.agg_sig, alice.master_sk.clone())
            .await?;

        // Create second offer
        let second_offer = alice
            .wallet
            .make_offer(
                MakerSide {
                    nfts: vec![nft_second.info.launcher_id],
                    ..Default::default()
                },
                TakerSide {
                    xch: 500,
                    ..Default::default()
                },
                None,
                false,
                true,
            )
            .await?;
        let second_offer = alice
            .wallet
            .sign_make_offer(second_offer, &alice.agg_sig, alice.master_sk.clone())
            .await?;

        // Aggregate offers
        let offer = aggregate_offers(vec![first_offer, second_offer]);

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0, false, true).await?;
        let spend_bundle = bob
            .wallet
            .sign_take_offer(offer, &bob.agg_sig, bob.master_sk.clone())
            .await?;
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        alice.wait_for_coins().await;
        bob.wait_for_coins().await;

        // Check balances
        assert_eq!(alice.wallet.db.balance().await?, 1000);
        assert_ne!(
            bob.wallet
                .db
                .spendable_nft(nft_first.info.launcher_id)
                .await?,
            None
        );
        assert_ne!(
            bob.wallet
                .db
                .spendable_nft(nft_second.info.launcher_id)
                .await?,
            None
        );

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_offer_xch_single_sided() -> anyhow::Result<()> {
        let alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        // Create offer
        let offer = alice
            .wallet
            .make_offer(
                MakerSide {
                    xch: 1000,
                    ..Default::default()
                },
                TakerSide::default(),
                None,
                false,
                true,
            )
            .await?;
        let offer = alice
            .wallet
            .sign_make_offer(offer, &alice.agg_sig, alice.master_sk.clone())
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0, false, true).await?;
        let spend_bundle = bob
            .wallet
            .sign_take_offer(offer, &bob.agg_sig, bob.master_sk.clone())
            .await?;
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        bob.wait_for_coins().await;

        // Check balances
        assert_eq!(bob.wallet.db.balance().await?, 1000);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_offer_cat_single_sided() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        // Create offer
        let offer = alice
            .wallet
            .make_offer(
                MakerSide {
                    cats: indexmap! { asset_id => 1000 },
                    ..Default::default()
                },
                TakerSide::default(),
                None,
                false,
                true,
            )
            .await?;
        let offer = alice
            .wallet
            .sign_make_offer(offer, &alice.agg_sig, alice.master_sk.clone())
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0, false, true).await?;
        let spend_bundle = bob
            .wallet
            .sign_take_offer(offer, &bob.agg_sig, bob.master_sk.clone())
            .await?;
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        bob.wait_for_coins().await;

        // Check balances
        assert_eq!(bob.wallet.db.cat_balance(asset_id).await?, 1000);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_offer_nft_single_sided() -> anyhow::Result<()> {
        let options = default_test_options();

        let mut alice = TestWallet::new(2).await?;
        let mut bob = alice
            .next_with_options(
                0,
                SyncOptions {
                    puzzle_batch_size_per_peer: 1,
                    timeouts: Timeouts {
                        puzzle_delay: Duration::from_secs(1),
                        ..options.timeouts
                    },
                    ..options
                },
            )
            .await?;

        let (coin_spends, did) = alice.wallet.create_did(0, false, true).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, mut nfts, _did) = alice
            .wallet
            .bulk_mint_nfts(
                0,
                did.info.launcher_id,
                vec![WalletNftMint {
                    metadata: NftMetadata::default(),
                    p2_puzzle_hash: None,
                    royalty_puzzle_hash: Some(Bytes32::default()),
                    royalty_basis_points: 300,
                }],
                false,
                true,
            )
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let nft = nfts.remove(0);

        // Create offer
        let offer = alice
            .wallet
            .make_offer(
                MakerSide {
                    nfts: vec![nft.info.launcher_id],
                    ..Default::default()
                },
                TakerSide::default(),
                None,
                false,
                true,
            )
            .await?;
        let offer = alice
            .wallet
            .sign_make_offer(offer, &alice.agg_sig, alice.master_sk.clone())
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0, false, true).await?;
        let spend_bundle = bob
            .wallet
            .sign_take_offer(offer, &bob.agg_sig, bob.master_sk.clone())
            .await?;
        assert_eq!(spend_bundle.coin_spends.len(), 3);
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        bob.wait_for_coins().await;

        // Resync to make sure the NFT is spendable even if it wasn't pending previously
        bob.resync().await?;
        bob.wait_for_puzzles().await;
        bob.wait_for_puzzles().await;

        // Check balances
        let nft = bob
            .wallet
            .db
            .spendable_nft(nft.info.launcher_id)
            .await?
            .expect("NFT should be spendable");

        let coin_id = bob
            .wallet
            .db
            .nft_row(nft.info.launcher_id)
            .await?
            .expect("NFT should exist")
            .coin_id;

        let is_spent = bob
            .wallet
            .db
            .coin_state(coin_id)
            .await?
            .expect("coin should exist")
            .spent_height
            .is_some();

        assert!(!is_spent);

        Ok(())
    }
}
