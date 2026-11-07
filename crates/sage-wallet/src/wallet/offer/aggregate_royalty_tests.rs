//! Royalty permutations for aggregated offers.
//!
//! When a maker locks an NFT into settlement, its royalty transfer program commits to a list of
//! trade prices. For each trade price it asserts a settlement puzzle announcement for exactly one
//! notarized payment: nonce = launcher id, paying `floor(price * basis_points / 10000)` to the
//! royalty address. The taker has to reproduce every one of those payments exactly, per NFT and
//! per asset. Buyers who request an NFT pay its royalty inside their own offer, so the taker's NFT
//! has to reveal trade prices whose royalties match those payments exactly.
//!
//! Each test asserts that exact outcome for one combination of offers.

use chia_wallet_sdk::prelude::*;
use indexmap::{IndexMap, indexmap};
use sage_database::SerializedNft;
use test_log::test;

use crate::{Offered, Requested, RequestedCat, TestWallet};

use super::{
    aggregate_offers,
    tests::{mint_nft_with_royalty, nft_offer_info, sign_taken_offer},
};

const ROYALTY_A: Bytes32 = Bytes32::new([1; 32]);
const ROYALTY_B: Bytes32 = Bytes32::new([2; 32]);
const ROYALTY_C: Bytes32 = Bytes32::new([3; 32]);

async fn make_signed_offer(
    maker: &TestWallet,
    offered: Offered,
    requested: Requested,
) -> anyhow::Result<SpendBundle> {
    let offer = maker.wallet.make_offer(offered, requested, None).await?;
    Ok(maker
        .wallet
        .sign_transaction(offer, &maker.agg_sig, maker.master_sk.clone(), true)
        .await?)
}

async fn sell_for_xch(
    maker: &TestWallet,
    nfts: &[&SerializedNft],
    xch: u64,
) -> anyhow::Result<SpendBundle> {
    make_signed_offer(
        maker,
        Offered {
            nfts: nfts.iter().map(|nft| nft.info.launcher_id).collect(),
            ..Default::default()
        },
        Requested {
            xch,
            ..Default::default()
        },
    )
    .await
}

async fn bid_xch(maker: &TestWallet, nft: &SerializedNft, xch: u64) -> anyhow::Result<SpendBundle> {
    make_signed_offer(
        maker,
        Offered {
            xch,
            ..Default::default()
        },
        Requested {
            nfts: indexmap! { nft.info.launcher_id => nft_offer_info(nft) },
            ..Default::default()
        },
    )
    .await
}

/// Takes the aggregate of `offers`, returning the parsed offer so tests can check the royalty
/// amounts Sage displays for it. The spend is validated against a copy of the simulator first, so
/// a rejected take fails with the validation error instead of a sync timeout.
async fn take_aggregate(taker: &TestWallet, offers: Vec<SpendBundle>) -> anyhow::Result<Offer> {
    let taken = taker.wallet.take_offer(aggregate_offers(offers), 0).await?;
    let offer = taken.offer.clone();
    let spend_bundle = sign_taken_offer(taker, taken).await?;
    taker
        .sim
        .lock()
        .await
        .clone()
        .new_transaction(spend_bundle.clone())?;
    taker.push_bundle(spend_bundle).await?;
    Ok(offer)
}

fn xch_amounts(xch: u64) -> OfferAmounts {
    OfferAmounts {
        xch: xch.into(),
        cats: IndexMap::new(),
    }
}

/// Returns the (XCH, CAT) amounts paid to a royalty address.
async fn royalties_paid(wallet: &TestWallet, royalty_puzzle_hash: Bytes32) -> (u64, u64) {
    let sim = wallet.sim.lock().await;
    let mut paid = (0, 0);

    for coin in sim.unspent_coins(royalty_puzzle_hash, true) {
        if coin.puzzle_hash == royalty_puzzle_hash {
            paid.0 += coin.amount;
        } else {
            paid.1 += coin.amount;
        }
    }

    paid
}

async fn owns_nft(wallet: &TestWallet, nft: &SerializedNft) -> anyhow::Result<bool> {
    Ok(wallet.wallet.db.nft(nft.info.launcher_id).await?.is_some())
}

#[test(tokio::test)]
async fn test_aggregate_sells_same_price_different_royalties() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(2).await?;
    let mut bob = alice.next(2).await?;
    let mut carol = bob.next(21_500).await?;

    let nft_a = mint_nft_with_royalty(&mut alice, ROYALTY_A, 500).await?;
    let nft_b = mint_nft_with_royalty(&mut bob, ROYALTY_B, 1000).await?;

    let offers = vec![
        sell_for_xch(&alice, &[&nft_a], 10_000).await?,
        sell_for_xch(&bob, &[&nft_b], 10_000).await?,
    ];
    let offer = take_aggregate(&carol, offers).await?;
    assert_eq!(offer.requested_royalty_amounts()?, xch_amounts(1500));

    carol.wait_for_coins().await;
    alice.wait_for_coins().await;
    bob.wait_for_coins().await;

    assert_eq!(alice.wallet.db.xch_balance().await?, 10_000);
    assert_eq!(bob.wallet.db.xch_balance().await?, 10_000);
    assert_eq!(carol.wallet.db.xch_balance().await?, 0);
    assert!(owns_nft(&carol, &nft_a).await?);
    assert!(owns_nft(&carol, &nft_b).await?);
    assert_eq!(royalties_paid(&carol, ROYALTY_A).await, (500, 0));
    assert_eq!(royalties_paid(&carol, ROYALTY_B).await, (1000, 0));

    Ok(())
}

#[test(tokio::test)]
async fn test_aggregate_sells_different_prices_different_royalties() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(2).await?;
    let mut bob = alice.next(2).await?;
    let mut carol = bob.next(43_500).await?;

    let nft_a = mint_nft_with_royalty(&mut alice, ROYALTY_A, 500).await?;
    let nft_b = mint_nft_with_royalty(&mut bob, ROYALTY_B, 1000).await?;

    let offers = vec![
        sell_for_xch(&alice, &[&nft_a], 10_000).await?,
        sell_for_xch(&bob, &[&nft_b], 30_000).await?,
    ];
    let offer = take_aggregate(&carol, offers).await?;
    assert_eq!(offer.requested_royalty_amounts()?, xch_amounts(3500));

    carol.wait_for_coins().await;
    alice.wait_for_coins().await;
    bob.wait_for_coins().await;

    assert_eq!(alice.wallet.db.xch_balance().await?, 10_000);
    assert_eq!(bob.wallet.db.xch_balance().await?, 30_000);
    assert_eq!(carol.wallet.db.xch_balance().await?, 0);
    assert_eq!(royalties_paid(&carol, ROYALTY_A).await, (500, 0));
    assert_eq!(royalties_paid(&carol, ROYALTY_B).await, (3000, 0));

    Ok(())
}

#[test(tokio::test)]
async fn test_aggregate_sells_different_prices_same_royalty() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(2).await?;
    let mut bob = alice.next(2).await?;
    let mut carol = bob.next(42_000).await?;

    let nft_a = mint_nft_with_royalty(&mut alice, ROYALTY_A, 500).await?;
    let nft_b = mint_nft_with_royalty(&mut bob, ROYALTY_B, 500).await?;

    let offers = vec![
        sell_for_xch(&alice, &[&nft_a], 10_000).await?,
        sell_for_xch(&bob, &[&nft_b], 30_000).await?,
    ];
    let offer = take_aggregate(&carol, offers).await?;
    assert_eq!(offer.requested_royalty_amounts()?, xch_amounts(2000));

    carol.wait_for_coins().await;
    alice.wait_for_coins().await;
    bob.wait_for_coins().await;

    // The total royalty (2000) is the same as an even split, but each NFT asserts its own amount
    assert_eq!(carol.wallet.db.xch_balance().await?, 0);
    assert_eq!(royalties_paid(&carol, ROYALTY_A).await, (500, 0));
    assert_eq!(royalties_paid(&carol, ROYALTY_B).await, (1500, 0));

    Ok(())
}

#[test(tokio::test)]
async fn test_aggregate_sells_multi_nft_leg_and_single_nft_leg() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(4).await?;
    let mut bob = alice.next(2).await?;
    let mut carol = bob.next(42_000).await?;

    let nft_a1 = mint_nft_with_royalty(&mut alice, ROYALTY_A, 500).await?;
    let nft_a2 = mint_nft_with_royalty(&mut alice, ROYALTY_B, 500).await?;
    let nft_b = mint_nft_with_royalty(&mut bob, ROYALTY_C, 500).await?;

    // Alice's two NFTs each commit to half of her 30,000 price
    let offers = vec![
        sell_for_xch(&alice, &[&nft_a1, &nft_a2], 30_000).await?,
        sell_for_xch(&bob, &[&nft_b], 10_000).await?,
    ];
    let offer = take_aggregate(&carol, offers).await?;
    assert_eq!(offer.requested_royalty_amounts()?, xch_amounts(2000));

    carol.wait_for_coins().await;
    alice.wait_for_coins().await;
    bob.wait_for_coins().await;

    assert_eq!(carol.wallet.db.xch_balance().await?, 0);
    assert_eq!(royalties_paid(&carol, ROYALTY_A).await, (750, 0));
    assert_eq!(royalties_paid(&carol, ROYALTY_B).await, (750, 0));
    assert_eq!(royalties_paid(&carol, ROYALTY_C).await, (500, 0));

    Ok(())
}

#[test(tokio::test)]
async fn test_aggregate_sells_for_different_assets() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(2).await?;
    let mut bob = alice.next(2).await?;
    let mut carol = bob.next(21_000).await?;

    let nft_a = mint_nft_with_royalty(&mut alice, ROYALTY_A, 500).await?;
    let nft_b = mint_nft_with_royalty(&mut bob, ROYALTY_B, 500).await?;

    let (coin_spends, asset_id) = carol.wallet.issue_cat(10_500, 0, None).await?;
    carol.transact(coin_spends).await?;
    carol.wait_for_coins().await;

    let cat_offer = make_signed_offer(
        &bob,
        Offered {
            nfts: vec![nft_b.info.launcher_id],
            ..Default::default()
        },
        Requested {
            cats: indexmap! { asset_id => RequestedCat { amount: 10_000, hidden_puzzle_hash: None } },
            ..Default::default()
        },
    )
    .await?;

    let offers = vec![sell_for_xch(&alice, &[&nft_a], 10_000).await?, cat_offer];
    let offer = take_aggregate(&carol, offers).await?;
    assert_eq!(
        offer.requested_royalty_amounts()?,
        OfferAmounts {
            xch: 500,
            cats: indexmap! { asset_id => 500 },
        }
    );

    carol.wait_for_coins().await;
    alice.wait_for_coins().await;
    bob.wait_for_puzzles().await;

    // Each NFT owes royalties only in the asset its own maker requested
    assert_eq!(alice.wallet.db.xch_balance().await?, 10_000);
    assert_eq!(bob.wallet.db.cat_balance(asset_id).await?, 10_000);
    assert_eq!(carol.wallet.db.xch_balance().await?, 0);
    assert_eq!(carol.wallet.db.cat_balance(asset_id).await?, 0);
    assert_eq!(royalties_paid(&carol, ROYALTY_A).await, (500, 0));
    assert_eq!(royalties_paid(&carol, ROYALTY_B).await, (0, 500));

    Ok(())
}

#[test(tokio::test)]
async fn test_aggregate_nft_sell_with_unrelated_cat_trade() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(2).await?;
    let mut bob = alice.next(5_000).await?;
    let mut carol = bob.next(26_500).await?;

    let nft_a = mint_nft_with_royalty(&mut alice, ROYALTY_A, 500).await?;

    // Carol holds 1,000 more CAT than the trade needs, so an overpaid royalty shows up in balances
    let (coin_spends, asset_id) = carol.wallet.issue_cat(21_000, 0, None).await?;
    carol.transact(coin_spends).await?;
    carol.wait_for_coins().await;

    let cat_offer = make_signed_offer(
        &bob,
        Offered {
            xch: 5_000,
            ..Default::default()
        },
        Requested {
            cats: indexmap! { asset_id => RequestedCat { amount: 20_000, hidden_puzzle_hash: None } },
            ..Default::default()
        },
    )
    .await?;

    let offers = vec![sell_for_xch(&alice, &[&nft_a], 10_000).await?, cat_offer];
    let offer = take_aggregate(&carol, offers).await?;
    assert_eq!(offer.requested_royalty_amounts()?, xch_amounts(500));

    carol.wait_for_coins().await;
    alice.wait_for_coins().await;
    bob.wait_for_puzzles().await;

    // Alice's NFT only commits to her 10,000 XCH, so no CAT royalty is owed
    assert_eq!(royalties_paid(&carol, ROYALTY_A).await, (500, 0));
    assert_eq!(alice.wallet.db.xch_balance().await?, 10_000);
    assert_eq!(bob.wallet.db.cat_balance(asset_id).await?, 20_000);
    assert_eq!(carol.wallet.db.xch_balance().await?, 0);
    assert_eq!(carol.wallet.db.cat_balance(asset_id).await?, 1_000);

    Ok(())
}

#[test(tokio::test)]
async fn test_aggregate_bids_same_price_different_royalties() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(10_500).await?;
    let mut bob = alice.next(11_000).await?;
    let mut carol = bob.next(4).await?;

    let nft_a = mint_nft_with_royalty(&mut carol, ROYALTY_A, 500).await?;
    let nft_b = mint_nft_with_royalty(&mut carol, ROYALTY_B, 1000).await?;

    let offers = vec![
        bid_xch(&alice, &nft_a, 10_000).await?,
        bid_xch(&bob, &nft_b, 10_000).await?,
    ];
    let offer = take_aggregate(&carol, offers).await?;
    assert_eq!(offer.offered_royalty_amounts()?, xch_amounts(1500));

    carol.wait_for_coins().await;
    alice.wait_for_puzzles().await;
    bob.wait_for_puzzles().await;

    assert_eq!(carol.wallet.db.xch_balance().await?, 20_000);
    assert_eq!(alice.wallet.db.xch_balance().await?, 0);
    assert_eq!(bob.wallet.db.xch_balance().await?, 0);
    assert!(owns_nft(&alice, &nft_a).await?);
    assert!(owns_nft(&bob, &nft_b).await?);
    assert_eq!(royalties_paid(&carol, ROYALTY_A).await, (500, 0));
    assert_eq!(royalties_paid(&carol, ROYALTY_B).await, (1000, 0));

    Ok(())
}

#[test(tokio::test)]
async fn test_aggregate_bids_different_prices() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(10_500).await?;
    let mut bob = alice.next(33_000).await?;
    let mut carol = bob.next(4).await?;

    let nft_a = mint_nft_with_royalty(&mut carol, ROYALTY_A, 500).await?;
    let nft_b = mint_nft_with_royalty(&mut carol, ROYALTY_B, 1000).await?;

    let offers = vec![
        bid_xch(&alice, &nft_a, 10_000).await?,
        bid_xch(&bob, &nft_b, 30_000).await?,
    ];
    let offer = take_aggregate(&carol, offers).await?;
    assert_eq!(offer.offered_royalty_amounts()?, xch_amounts(3500));

    carol.wait_for_coins().await;
    alice.wait_for_puzzles().await;
    bob.wait_for_puzzles().await;

    // Each bidder already paid its NFT's royalty; Carol's NFTs have to reveal matching prices
    assert_eq!(carol.wallet.db.xch_balance().await?, 40_000);
    assert!(owns_nft(&alice, &nft_a).await?);
    assert!(owns_nft(&bob, &nft_b).await?);
    assert_eq!(royalties_paid(&carol, ROYALTY_A).await, (500, 0));
    assert_eq!(royalties_paid(&carol, ROYALTY_B).await, (3000, 0));

    Ok(())
}

#[test(tokio::test)]
async fn test_sell_price_below_royalty_granularity() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(2).await?;
    let mut carol = alice.next(10).await?;

    let nft_a = mint_nft_with_royalty(&mut alice, ROYALTY_A, 300).await?;

    // 3% of 10 mojos rounds down to 0. Settlement can't make a 0 payment, so the maker must not
    // commit that trade price.
    let offer = take_aggregate(&carol, vec![sell_for_xch(&alice, &[&nft_a], 10).await?]).await?;
    assert_eq!(offer.requested_royalty_amounts()?, OfferAmounts::new());

    carol.wait_for_coins().await;
    alice.wait_for_coins().await;

    assert_eq!(alice.wallet.db.xch_balance().await?, 10);
    assert_eq!(carol.wallet.db.xch_balance().await?, 0);
    assert!(owns_nft(&carol, &nft_a).await?);

    Ok(())
}
