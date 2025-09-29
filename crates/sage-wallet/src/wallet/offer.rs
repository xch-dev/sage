mod aggregate_offer;
mod cancel_offer;
mod make_offer;
mod offer_assets;
mod take_offer;

pub use aggregate_offer::*;
pub use make_offer::*;

#[cfg(test)]
mod tests {
    use chia::{
        clvm_traits::{FromClvm, ToClvm},
        protocol::{Bytes32, Program},
        puzzles::nft::NftMetadata,
    };
    use clvmr::Allocator;
    use indexmap::indexmap;
    use sage_database::NftOfferInfo;
    use test_log::test;

    use crate::{Offered, Requested, RequestedCat, TestWallet, WalletNftMint};

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
                Offered {
                    xch: 750,
                    fee: 250,
                    ..Default::default()
                },
                Requested {
                    cats: indexmap! { asset_id => RequestedCat { amount: 1000, hidden_puzzle_hash: None } },
                    ..Default::default()
                },
                None,
            )
            .await?;
        let offer = alice
            .wallet
            .sign_transaction(offer, &alice.agg_sig, alice.master_sk.clone(), true)
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0).await?;
        let spend_bundle = bob
            .wallet
            .sign_transaction(offer, &bob.agg_sig, bob.master_sk.clone(), true)
            .await?;
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        bob.wait_for_coins().await;
        alice.wait_for_puzzles().await;

        // Check balances
        assert_eq!(alice.wallet.db.cat_balance(asset_id).await?, 1000);
        assert_eq!(bob.wallet.db.xch_balance().await?, 750);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_offer_xch_for_nft() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1030).await?;
        let mut bob = alice.next(2).await?;

        let (coin_spends, did) = bob.wallet.create_did(0).await?;
        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;

        let (coin_spends, mut nfts) = bob
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
                Offered {
                    xch: 1000,
                    ..Default::default()
                },
                Requested {
                    nfts: indexmap! {
                        nft.info.launcher_id => NftOfferInfo {
                            metadata,
                            metadata_updater_puzzle_hash: nft.info.metadata_updater_puzzle_hash,
                            royalty_puzzle_hash: nft.info.royalty_puzzle_hash,
                            royalty_basis_points: nft.info.royalty_basis_points,
                        },
                    },
                    ..Default::default()
                },
                None,
            )
            .await?;
        let offer = alice
            .wallet
            .sign_transaction(offer, &alice.agg_sig, alice.master_sk.clone(), true)
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0).await?;
        let spend_bundle = bob
            .wallet
            .sign_transaction(offer, &bob.agg_sig, bob.master_sk.clone(), true)
            .await?;
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        bob.wait_for_coins().await;
        alice.wait_for_puzzles().await;

        // Check balances
        assert_ne!(alice.wallet.db.nft(nft.info.launcher_id).await?, None);
        assert_eq!(bob.wallet.db.xch_balance().await?, 1000);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_offer_nft_for_xch() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(2).await?;
        let mut bob = alice.next(1030).await?;

        let (coin_spends, did) = alice.wallet.create_did(0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, mut nfts) = alice
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
            )
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let nft = nfts.remove(0);

        // Create offer
        let offer = alice
            .wallet
            .make_offer(
                Offered {
                    nfts: vec![nft.info.launcher_id],
                    ..Default::default()
                },
                Requested {
                    xch: 1000,
                    ..Default::default()
                },
                None,
            )
            .await?;
        let offer = alice
            .wallet
            .sign_transaction(offer, &alice.agg_sig, alice.master_sk.clone(), true)
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0).await?;
        let spend_bundle = bob
            .wallet
            .sign_transaction(offer, &bob.agg_sig, bob.master_sk.clone(), true)
            .await?;
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        alice.wait_for_coins().await;
        bob.wait_for_coins().await;

        // Check balances
        assert_eq!(alice.wallet.db.xch_balance().await?, 1000);
        assert_ne!(bob.wallet.db.nft(nft.info.launcher_id).await?, None);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_offer_nft_same_royalties_for_xch() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(3).await?;
        let mut bob = alice.next(1030).await?;

        let (coin_spends, did) = alice.wallet.create_did(0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, mut nfts) = alice
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
                Offered {
                    nfts: vec![
                        nft_id_first.info.launcher_id,
                        nft_id_second.info.launcher_id,
                    ],
                    ..Default::default()
                },
                Requested {
                    xch: 1000,
                    ..Default::default()
                },
                None,
            )
            .await?;
        let offer = alice
            .wallet
            .sign_transaction(offer, &alice.agg_sig, alice.master_sk.clone(), true)
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0).await?;
        let spend_bundle = bob
            .wallet
            .sign_transaction(offer, &bob.agg_sig, bob.master_sk.clone(), true)
            .await?;
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        alice.wait_for_coins().await;
        bob.wait_for_coins().await;

        // Check balances
        assert_eq!(alice.wallet.db.xch_balance().await?, 1000);
        assert_ne!(
            bob.wallet.db.nft(nft_id_first.info.launcher_id).await?,
            None
        );
        assert_ne!(
            bob.wallet.db.nft(nft_id_second.info.launcher_id).await?,
            None
        );

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_offer_nft_same_royalties_for_cat() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(3).await?;
        let mut bob = alice.next(1030).await?;

        let (coin_spends, did) = alice.wallet.create_did(0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, mut nfts) = alice
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
                Offered {
                    nfts: vec![
                        nft_id_first.info.launcher_id,
                        nft_id_second.info.launcher_id,
                    ],
                    ..Default::default()
                },
                Requested {
                    cats: indexmap! { asset_id => RequestedCat { amount: 1000, hidden_puzzle_hash: None } },
                    ..Default::default()
                },
                None,
            )
            .await?;
        let offer = alice
            .wallet
            .sign_transaction(offer, &alice.agg_sig, alice.master_sk.clone(), true)
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0).await?;
        let spend_bundle = bob
            .wallet
            .sign_transaction(offer, &bob.agg_sig, bob.master_sk.clone(), true)
            .await?;
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        alice.wait_for_puzzles().await;
        bob.wait_for_coins().await;

        // Check balances
        assert_eq!(alice.wallet.db.cat_balance(asset_id).await?, 1000);
        assert_ne!(
            bob.wallet.db.nft(nft_id_first.info.launcher_id).await?,
            None
        );
        assert_ne!(
            bob.wallet.db.nft(nft_id_second.info.launcher_id).await?,
            None
        );

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_offer_nft_mixed_royalties_for_xch() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(3).await?;
        let mut bob = alice.next(1030).await?;

        let (coin_spends, did) = alice.wallet.create_did(0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, mut nfts) = alice
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
                Offered {
                    nfts: vec![
                        nft_id_first.info.launcher_id,
                        nft_id_second.info.launcher_id,
                    ],
                    ..Default::default()
                },
                Requested {
                    xch: 1000,
                    ..Default::default()
                },
                None,
            )
            .await?;
        let offer = alice
            .wallet
            .sign_transaction(offer, &alice.agg_sig, alice.master_sk.clone(), true)
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0).await?;
        let spend_bundle = bob
            .wallet
            .sign_transaction(offer, &bob.agg_sig, bob.master_sk.clone(), true)
            .await?;
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        alice.wait_for_coins().await;
        bob.wait_for_coins().await;

        // Check balances
        assert_eq!(alice.wallet.db.xch_balance().await?, 1000);
        assert_ne!(
            bob.wallet.db.nft(nft_id_first.info.launcher_id).await?,
            None
        );
        assert_ne!(
            bob.wallet.db.nft(nft_id_second.info.launcher_id).await?,
            None
        );

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_offer_nft_for_xch_aggregate() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(3).await?;
        let mut bob = alice.next(1030).await?;

        let (coin_spends, did) = alice.wallet.create_did(0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, mut nfts) = alice
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
                Offered {
                    nfts: vec![nft_first.info.launcher_id],
                    ..Default::default()
                },
                Requested {
                    xch: 500,
                    ..Default::default()
                },
                None,
            )
            .await?;
        let first_offer = alice
            .wallet
            .sign_transaction(first_offer, &alice.agg_sig, alice.master_sk.clone(), true)
            .await?;

        // Create second offer
        let second_offer = alice
            .wallet
            .make_offer(
                Offered {
                    nfts: vec![nft_second.info.launcher_id],
                    ..Default::default()
                },
                Requested {
                    xch: 500,
                    ..Default::default()
                },
                None,
            )
            .await?;
        let second_offer = alice
            .wallet
            .sign_transaction(second_offer, &alice.agg_sig, alice.master_sk.clone(), true)
            .await?;

        // Aggregate offers
        let offer = aggregate_offers(vec![first_offer, second_offer]);

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0).await?;
        let spend_bundle = bob
            .wallet
            .sign_transaction(offer, &bob.agg_sig, bob.master_sk.clone(), true)
            .await?;
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        alice.wait_for_coins().await;
        bob.wait_for_coins().await;

        // Check balances
        assert_eq!(alice.wallet.db.xch_balance().await?, 1000);
        assert_ne!(bob.wallet.db.nft(nft_first.info.launcher_id).await?, None);
        assert_ne!(bob.wallet.db.nft(nft_second.info.launcher_id).await?, None);

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
                Offered {
                    xch: 1000,
                    ..Default::default()
                },
                Requested::default(),
                None,
            )
            .await?;
        let offer = alice
            .wallet
            .sign_transaction(offer, &alice.agg_sig, alice.master_sk.clone(), true)
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0).await?;
        let spend_bundle = bob
            .wallet
            .sign_transaction(offer, &bob.agg_sig, bob.master_sk.clone(), true)
            .await?;
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        bob.wait_for_coins().await;

        // Check balances
        assert_eq!(bob.wallet.db.xch_balance().await?, 1000);

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
                Offered {
                    cats: indexmap! { asset_id => 1000 },
                    ..Default::default()
                },
                Requested::default(),
                None,
            )
            .await?;
        let offer = alice
            .wallet
            .sign_transaction(offer, &alice.agg_sig, alice.master_sk.clone(), true)
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0).await?;
        let spend_bundle = bob
            .wallet
            .sign_transaction(offer, &bob.agg_sig, bob.master_sk.clone(), true)
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
        let mut alice = TestWallet::new(2).await?;
        let mut bob = alice.next(0).await?;

        let (coin_spends, did) = alice.wallet.create_did(0).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let (coin_spends, mut nfts) = alice
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
            )
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let nft = nfts.remove(0);

        // Create offer
        let offer = alice
            .wallet
            .make_offer(
                Offered {
                    nfts: vec![nft.info.launcher_id],
                    ..Default::default()
                },
                Requested::default(),
                None,
            )
            .await?;
        let offer = alice
            .wallet
            .sign_transaction(offer, &alice.agg_sig, alice.master_sk.clone(), true)
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0).await?;
        let spend_bundle = bob
            .wallet
            .sign_transaction(offer, &bob.agg_sig, bob.master_sk.clone(), true)
            .await?;
        assert_eq!(spend_bundle.coin_spends.len(), 3);
        bob.push_bundle(spend_bundle).await?;

        // We need to wait for both wallets to sync in this case
        bob.wait_for_coins().await;

        let nft = bob
            .wallet
            .db
            .spendable_nft(nft.info.launcher_id)
            .await?
            .expect("NFT should be spendable");

        let is_spent = bob
            .sim
            .lock()
            .await
            .coin_state(nft.coin.coin_id())
            .expect("coin should exist")
            .spent_height
            .is_some();

        assert!(!is_spent);

        // Resync to make sure the NFT is spendable even if it wasn't pending previously
        bob.resync().await?;
        bob.wait_for_puzzles().await;

        // Check balances
        let nft = bob
            .wallet
            .db
            .spendable_nft(nft.info.launcher_id)
            .await?
            .expect("NFT should be spendable");

        let is_spent = bob
            .sim
            .lock()
            .await
            .coin_state(nft.coin.coin_id())
            .expect("coin should exist")
            .spent_height
            .is_some();

        assert!(!is_spent);

        Ok(())
    }
}
