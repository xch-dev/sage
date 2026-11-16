use chia_wallet_sdk::{driver::VaultVersion, prelude::*};
use indexmap::indexmap;
use test_log::test;

use crate::{Offered, PreSplitOffer, Requested, RequestedCat, TakenOffer, TestWallet, WalletError};

use super::tests::{config, mint, vault_transact};

const VERSION: VaultVersion = VaultVersion::ForceSingletonRecreation;

/// Signs and takes an offer with the wallet, which may be a vault.
async fn take(taker: &TestWallet, offer: SpendBundle, fee: u64) -> anyhow::Result<SpendBundle> {
    let TakenOffer {
        offer,
        spend_bundle,
    } = taker.wallet.take_offer(offer, fee).await?;

    let signed = taker.sign(spend_bundle.coin_spends).await?;

    Ok(offer.take(signed))
}

/// Issues a CAT from the wallet, and waits for it to sync.
async fn issue_cat(test: &mut TestWallet, amount: u64) -> anyhow::Result<Bytes32> {
    let (coin_spends, asset_id) = test.wallet.issue_cat(amount, 0, None).await?;
    test.transact(coin_spends).await?;
    test.wait_until(|wallet| async move {
        Ok(wallet.db.selectable_cat_balance(asset_id).await? == u128::from(amount))
    })
    .await?;
    Ok(asset_id)
}

/// Makes a vault offer, and confirms the transaction that pre-splits its coins.
async fn make_vault_offer(
    vault: &mut TestWallet,
    offered: Offered,
    requested: Requested,
) -> anyhow::Result<SpendBundle> {
    let PreSplitOffer {
        offer,
        split_coin_spends,
    } = vault
        .wallet
        .make_pre_split_offer(offered, requested, None, 0)
        .await?;

    let split_coin_spends = split_coin_spends.expect("a vault's offer is always pre-split");

    // The vault's transaction is clear signed, and creates an offer for the pre-split coins.
    let summary = vault
        .wallet
        .parse_vault_spend(&split_coin_spends)
        .await?
        .expect("the vault is spent");
    assert!(!summary.changes_custody());

    vault_transact(vault, split_coin_spends).await?;

    Ok(offer)
}

#[test(tokio::test)]
async fn test_vault_offer_xch_for_cat() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(2000).await?;
    let mut vault = mint(&mut alice, &config(VERSION), 1000).await?;
    let asset_id = issue_cat(&mut alice, 500).await?;

    let offer = make_vault_offer(
        &mut vault,
        Offered {
            xch: 300,
            fee: 20,
            ..Default::default()
        },
        Requested {
            cats: indexmap! { asset_id => RequestedCat { amount: 500, hidden_puzzle_hash: None } },
            ..Default::default()
        },
    )
    .await?;

    // The pre-split coin is still the vault's until the offer is taken.
    assert_eq!(vault.wallet.db.xch_balance().await?, 1000);

    // The offer can't be taken without the requested payment.
    let mut allocator = Allocator::new();
    let parsed = Offer::from_spend_bundle(&mut allocator, &offer)?;
    assert_eq!(parsed.cancellable_coin_spends()?.len(), 1);

    let spend_bundle = take(&alice, offer, 0).await?;
    alice.push_bundle(spend_bundle).await?;

    vault
        .wait_until(|wallet| async move {
            Ok(wallet.db.cat_balance(asset_id).await? == 500
                && wallet.db.xch_balance().await? == 680)
        })
        .await?;

    alice
        .wait_until(|wallet| async move { Ok(wallet.db.cat_balance(asset_id).await? == 0) })
        .await?;

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_offer_cat_for_xch() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(2000).await?;
    let mut vault = mint(&mut alice, &config(VERSION), 1000).await?;

    let (coin_spends, asset_id) = vault.wallet.issue_cat(800, 0, None).await?;
    vault_transact(&mut vault, coin_spends).await?;
    vault
        .wait_until(|wallet| async move { Ok(wallet.db.cat_balance(asset_id).await? == 800) })
        .await?;

    let offer = make_vault_offer(
        &mut vault,
        Offered {
            cats: indexmap! { asset_id => 300 },
            ..Default::default()
        },
        Requested {
            xch: 700,
            ..Default::default()
        },
    )
    .await?;

    let spend_bundle = take(&alice, offer, 0).await?;
    alice.push_bundle(spend_bundle).await?;

    vault
        .wait_until(|wallet| async move {
            Ok(wallet.db.cat_balance(asset_id).await? == 500
                && wallet.db.xch_balance().await? == 900)
        })
        .await?;

    alice
        .wait_until(|wallet| async move { Ok(wallet.db.cat_balance(asset_id).await? == 300) })
        .await?;

    Ok(())
}

/// A vault takes an offer made by a BLS wallet.
#[test(tokio::test)]
async fn test_vault_takes_offer() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(2000).await?;
    let mut vault = mint(&mut alice, &config(VERSION), 1000).await?;
    let asset_id = issue_cat(&mut alice, 500).await?;

    let offer = alice
        .wallet
        .make_offer(
            Offered {
                cats: indexmap! { asset_id => 500 },
                ..Default::default()
            },
            Requested {
                xch: 400,
                ..Default::default()
            },
            None,
        )
        .await?;
    let offer = alice
        .wallet
        .sign_transaction(offer, &alice.agg_sig, alice.master_sk.clone(), true)
        .await?;

    let spend_bundle = take(&vault, offer, 10).await?;
    vault.push_bundle(spend_bundle).await?;

    vault
        .wait_until(|wallet| async move {
            Ok(wallet.db.cat_balance(asset_id).await? == 500
                && wallet.db.xch_balance().await? == 590)
        })
        .await?;

    Ok(())
}

/// One vault takes another vault's offer.
#[test(tokio::test)]
async fn test_vault_offer_between_vaults() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(4000).await?;
    let mut maker = mint(&mut alice, &config(VERSION), 1000).await?;
    let mut taker = mint(&mut alice, &config(VERSION), 1000).await?;

    let (coin_spends, asset_id) = taker.wallet.issue_cat(600, 0, None).await?;
    vault_transact(&mut taker, coin_spends).await?;
    taker
        .wait_until(|wallet| async move { Ok(wallet.db.cat_balance(asset_id).await? == 600) })
        .await?;

    let offer = make_vault_offer(
        &mut maker,
        Offered {
            xch: 250,
            ..Default::default()
        },
        Requested {
            cats: indexmap! { asset_id => RequestedCat { amount: 600, hidden_puzzle_hash: None } },
            ..Default::default()
        },
    )
    .await?;

    let spend_bundle = take(&taker, offer, 0).await?;
    taker.push_bundle(spend_bundle).await?;

    maker
        .wait_until(|wallet| async move {
            Ok(wallet.db.cat_balance(asset_id).await? == 600
                && wallet.db.xch_balance().await? == 750)
        })
        .await?;

    taker
        .wait_until(|wallet| async move {
            Ok(
                wallet.db.cat_balance(asset_id).await? == 0
                    && wallet.db.xch_balance().await? == 650,
            )
        })
        .await?;

    Ok(())
}

/// Cancelling a vault's offer takes its pre-split coins back through the vault.
#[test(tokio::test)]
async fn test_vault_cancel_offer() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(2000).await?;
    let mut vault = mint(&mut alice, &config(VERSION), 1000).await?;
    let asset_id = issue_cat(&mut alice, 500).await?;

    let requested = || Requested {
        cats: indexmap! { asset_id => RequestedCat { amount: 500, hidden_puzzle_hash: None } },
        ..Default::default()
    };

    let first = make_vault_offer(
        &mut vault,
        Offered {
            xch: 300,
            ..Default::default()
        },
        requested(),
    )
    .await?;

    // Sage locks the first offer's pre-split coin once it saves the offer, which the wallet
    // doesn't do on its own, so the second offer is made from the vault's change.
    let change = vault
        .wallet
        .db
        .selectable_xch_coins()
        .await?
        .into_iter()
        .find(|coin| coin.puzzle_hash == vault.puzzle_hash)
        .expect("the vault has change");

    let second = make_vault_offer(
        &mut vault,
        Offered {
            xch: 200,
            selected_coin_ids: vec![change.coin_id()],
            ..Default::default()
        },
        requested(),
    )
    .await?;

    let coin_spends = vault
        .wallet
        .cancel_offers(vec![first.clone(), second.clone()], 10, vec![])
        .await?;
    vault_transact(&mut vault, coin_spends).await?;

    vault
        .wait_until(|wallet| async move { Ok(wallet.db.xch_balance().await? == 990) })
        .await?;

    // The pre-split coins were spent back to the vault's own address.
    let coins = vault.wallet.db.selectable_xch_coins().await?;
    assert!(
        coins
            .iter()
            .all(|coin| coin.puzzle_hash == vault.puzzle_hash)
    );

    let mut allocator = Allocator::new();
    for offer in [first, second] {
        for coin_spend in
            Offer::from_spend_bundle(&mut allocator, &offer)?.cancellable_coin_spends()?
        {
            assert!(
                coins
                    .iter()
                    .all(|coin| coin.coin_id() != coin_spend.coin.coin_id())
            );
        }
    }

    Ok(())
}

#[test(tokio::test)]
async fn test_vault_offer_unsupported() -> anyhow::Result<()> {
    let mut alice = TestWallet::new(2000).await?;
    let vault = mint(&mut alice, &config(VERSION), 1000).await?;

    // An offer that spends the vault directly would lock it until it's taken.
    let result = vault
        .wallet
        .make_offer(
            Offered {
                xch: 100,
                ..Default::default()
            },
            Requested {
                xch: 200,
                ..Default::default()
            },
            None,
        )
        .await;
    assert!(
        matches!(result, Err(WalletError::VaultDirectOfferNotSupported)),
        "{result:?}"
    );

    let result = vault
        .wallet
        .make_pre_split_offer(
            Offered {
                nfts: vec![Bytes32::new([1; 32])],
                ..Default::default()
            },
            Requested {
                xch: 200,
                ..Default::default()
            },
            None,
            0,
        )
        .await;
    assert!(
        matches!(result, Err(WalletError::VaultOfferAssetNotSupported)),
        "{result:?}"
    );

    Ok(())
}
