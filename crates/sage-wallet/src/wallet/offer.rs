mod aggregate_offer;
mod cancel_offer;
mod make_offer;
mod offer_assets;
mod take_offer;

pub use aggregate_offer::*;
pub use make_offer::*;

#[cfg(test)]
mod tests {
    use chia_wallet_sdk::{chia::puzzle_types::nft::NftMetadata, prelude::*};
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
    async fn test_offer_xch_for_cat_with_selected_coins() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(1000).await?;

        // Split Alice's XCH into multiple coins
        let coins = alice.wallet.db.selectable_xch_coins().await?;
        let coin_spends = alice
            .wallet
            .split(coins.iter().map(Coin::coin_id).collect(), 3, 0)
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let coins = alice.wallet.db.selectable_xch_coins().await?;
        assert_eq!(coins.len(), 3);

        // Issue CAT for Bob
        let (coin_spends, asset_id) = bob.wallet.issue_cat(1000, 0, None).await?;
        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;

        // Select only the first coin for the offer
        let selected_coin = coins[0];
        let selected_amount = selected_coin.amount;

        let unsigned_offer = alice
            .wallet
            .make_offer(
                Offered {
                    xch: selected_amount,
                    selected_coin_ids: vec![selected_coin.coin_id()],
                    ..Default::default()
                },
                Requested {
                    cats: indexmap! { asset_id => RequestedCat { amount: 1000, hidden_puzzle_hash: None } },
                    ..Default::default()
                },
                None,
            )
            .await?;

        // Assert the selected coin is actually used in the offer
        assert!(
            unsigned_offer
                .coin_spends
                .iter()
                .any(|cs| cs.coin.coin_id() == selected_coin.coin_id()),
            "Selected coin should be included in the offer's coin spends"
        );

        let offer = alice
            .wallet
            .sign_transaction(
                unsigned_offer,
                &alice.agg_sig,
                alice.master_sk.clone(),
                true,
            )
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0).await?;
        let spend_bundle = bob
            .wallet
            .sign_transaction(offer, &bob.agg_sig, bob.master_sk.clone(), true)
            .await?;
        bob.push_bundle(spend_bundle).await?;

        bob.wait_for_coins().await;
        alice.wait_for_puzzles().await;

        // Check balances
        assert_eq!(alice.wallet.db.cat_balance(asset_id).await?, 1000);
        assert_eq!(bob.wallet.db.xch_balance().await?, selected_amount as u128);

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_offer_cat_with_selected_coins() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(2000).await?;
        let mut bob = alice.next(1000).await?;

        // Issue CAT for Alice
        let (coin_spends, asset_id) = alice.wallet.issue_cat(1000, 0, None).await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        // Split Alice's CAT into multiple coins
        let cats = alice.wallet.db.selectable_cat_coins(asset_id).await?;
        assert_eq!(cats.len(), 1);
        let coin_spends = alice
            .wallet
            .split(vec![cats[0].coin.coin_id()], 3, 0)
            .await?;
        alice.transact(coin_spends).await?;
        alice.wait_for_coins().await;

        let cats = alice.wallet.db.selectable_cat_coins(asset_id).await?;
        assert_eq!(cats.len(), 3);

        // Select only the first CAT coin for the offer
        let selected_cat = &cats[0];
        let selected_amount = selected_cat.coin.amount;

        let unsigned_offer = alice
            .wallet
            .make_offer(
                Offered {
                    cats: indexmap! { asset_id => selected_amount },
                    selected_coin_ids: vec![selected_cat.coin.coin_id()],
                    ..Default::default()
                },
                Requested {
                    xch: 500,
                    ..Default::default()
                },
                None,
            )
            .await?;

        // Assert the selected CAT coin is actually used in the offer
        assert!(
            unsigned_offer
                .coin_spends
                .iter()
                .any(|cs| cs.coin.coin_id() == selected_cat.coin.coin_id()),
            "Selected CAT coin should be included in the offer's coin spends"
        );

        let offer = alice
            .wallet
            .sign_transaction(
                unsigned_offer,
                &alice.agg_sig,
                alice.master_sk.clone(),
                true,
            )
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0).await?;
        let spend_bundle = bob
            .wallet
            .sign_transaction(offer, &bob.agg_sig, bob.master_sk.clone(), true)
            .await?;
        bob.push_bundle(spend_bundle).await?;

        bob.wait_for_coins().await;
        alice.wait_for_puzzles().await;

        // Check balances: Alice started with 2000, spent 1000 on CAT issuance, received 500 from offer
        assert_eq!(alice.wallet.db.xch_balance().await?, 1500);
        assert_eq!(
            bob.wallet.db.cat_balance(asset_id).await?,
            selected_amount as u128
        );

        Ok(())
    }

    #[test(tokio::test)]
    async fn test_offer_xch_with_fee_and_selected_coins() -> anyhow::Result<()> {
        let mut alice = TestWallet::new(1000).await?;
        let mut bob = alice.next(1000).await?;

        // Issue CAT for Bob
        let (coin_spends, asset_id) = bob.wallet.issue_cat(1000, 0, None).await?;
        bob.transact(coin_spends).await?;
        bob.wait_for_coins().await;

        // Select Alice's single coin explicitly for the offer with fee
        let coins = alice.wallet.db.selectable_xch_coins().await?;
        assert_eq!(coins.len(), 1);

        let selected_coin_id = coins[0].coin_id();
        let unsigned_offer = alice
            .wallet
            .make_offer(
                Offered {
                    xch: 750,
                    fee: 250,
                    selected_coin_ids: vec![selected_coin_id],
                    ..Default::default()
                },
                Requested {
                    cats: indexmap! { asset_id => RequestedCat { amount: 1000, hidden_puzzle_hash: None } },
                    ..Default::default()
                },
                None,
            )
            .await?;

        // Assert the selected coin is actually used in the offer
        assert!(
            unsigned_offer
                .coin_spends
                .iter()
                .any(|cs| cs.coin.coin_id() == selected_coin_id),
            "Selected coin should be included in the offer's coin spends"
        );

        let offer = alice
            .wallet
            .sign_transaction(
                unsigned_offer,
                &alice.agg_sig,
                alice.master_sk.clone(),
                true,
            )
            .await?;

        // Take offer
        let offer = bob.wallet.take_offer(offer, 0).await?;
        let spend_bundle = bob
            .wallet
            .sign_transaction(offer, &bob.agg_sig, bob.master_sk.clone(), true)
            .await?;
        bob.push_bundle(spend_bundle).await?;

        bob.wait_for_coins().await;
        alice.wait_for_puzzles().await;

        // Check balances
        assert_eq!(alice.wallet.db.cat_balance(asset_id).await?, 1000);
        assert_eq!(bob.wallet.db.xch_balance().await?, 750);

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
