use std::time::Duration;

use anyhow::{Result, bail};
use sage_api::{
    Amount, CancelOffer, GenerateMnemonic, GetCats, GetOffer, GetSyncStatus, GetVault, IssueCat,
    IssueMoreCat, Login, MakeOffer, MintVault, OfferAmount, OfferRecordStatus, SendXch, TakeOffer,
    VaultState,
};
use tokio::time::sleep;

use super::TestApp;

async fn wait_for_balance(app: &mut TestApp, balance: u64) -> Result<()> {
    for _ in 0..100 {
        while app.events.try_recv().is_ok() {}

        let status = app.get_sync_status(GetSyncStatus {}).await?;

        if status.selectable_balance.to_u64() == Some(balance) {
            return Ok(());
        }

        sleep(Duration::from_millis(100)).await;
    }

    let status = app.get_sync_status(GetSyncStatus {}).await?;
    bail!("timed out waiting for a balance of {balance}: {status:?}");
}

async fn wait_for_cat_balance(app: &mut TestApp, asset_id: &str, balance: u64) -> Result<()> {
    for _ in 0..100 {
        while app.events.try_recv().is_ok() {}

        let wallet = app.sage.lock().await.wallet()?;
        let asset_id = hex::decode(asset_id)?
            .try_into()
            .expect("a 32 byte asset id");

        if wallet.db.selectable_cat_balance(asset_id).await? == u128::from(balance) {
            return Ok(());
        }

        sleep(Duration::from_millis(100)).await;
    }

    bail!("timed out waiting for a CAT balance of {balance}");
}

async fn wait_for_offer_status(
    app: &mut TestApp,
    offer_id: &str,
    matches: impl Fn(&OfferRecordStatus) -> bool,
) -> Result<()> {
    let mut status = None;

    for _ in 0..100 {
        while app.events.try_recv().is_ok() {}

        let offer = app
            .get_offer(GetOffer {
                offer_id: offer_id.to_string(),
            })
            .await?
            .offer;

        if matches(&offer.status) {
            return Ok(());
        }

        status = Some(offer.status);
        sleep(Duration::from_millis(100)).await;
    }

    bail!("timed out waiting for the offer's status, which is {status:?}");
}

/// The asset id of the open wallet's first CAT, once it's synced.
async fn first_cat(app: &mut TestApp) -> Result<String> {
    for _ in 0..100 {
        while app.events.try_recv().is_ok() {}

        if let Some(asset_id) = app
            .get_cats(GetCats {})
            .await?
            .cats
            .first()
            .and_then(|cat| cat.asset_id.clone())
        {
            return Ok(asset_id);
        }

        sleep(Duration::from_millis(100)).await;
    }

    bail!("timed out waiting for a CAT");
}

fn send(address: String, amount: u64) -> SendXch {
    SendXch {
        confirmed_handles: Vec::new(),
        address,
        amount: Amount::u64(amount),
        fee: Amount::u64(0),
        memos: vec![],
        clawback: None,
        auto_submit: true,
        coin_ids: None,
    }
}

fn xch(amount: u64) -> OfferAmount {
    OfferAmount {
        asset_id: None,
        hidden_puzzle_hash: None,
        amount: Amount::u64(amount),
    }
}

fn cat(asset_id: &str, amount: u64) -> OfferAmount {
    OfferAmount {
        asset_id: Some(asset_id.to_string()),
        hidden_puzzle_hash: None,
        amount: Amount::u64(amount),
    }
}

/// Mints a vault from alice's wallet, which has `alice_balance`, and sends it `balance`. Returns
/// the vault's fingerprint.
async fn setup_vault(
    app: &mut TestApp,
    alice: u32,
    alice_balance: u64,
    balance: u64,
) -> Result<u32> {
    let mnemonic = app
        .generate_mnemonic(GenerateMnemonic {
            use_24_words: false,
        })
        .await?
        .mnemonic;

    let response = app
        .mint_vault(MintVault {
            name: "Vault".to_string(),
            emoji: None,
            custody_fingerprint: None,
            recovery_mnemonic: Some(mnemonic),
            recovery_fingerprint: None,
            recovery_timelock: 12 * 60 * 60,
            fee: Amount::u64(0),
            auto_submit: true,
        })
        .await?;
    let fingerprint = response.fingerprint.expect("the mint was submitted");

    // The mint pays the vault's mojo, and the change has to confirm before alice sends more.
    wait_for_balance(app, alice_balance - 1).await?;

    app.login(Login { fingerprint }).await?;

    for _ in 0..100 {
        let vault = app.get_vault(GetVault {}).await?.vault;

        if vault.state == VaultState::Active && vault.config.is_some() && !vault.minting {
            break;
        }

        sleep(Duration::from_millis(100)).await;
    }

    let vault_address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;

    app.login(Login { fingerprint: alice }).await?;
    app.sim.lock().await.create_block();
    app.send_xch(send(vault_address, balance)).await?;
    wait_for_balance(app, alice_balance - 1 - balance).await?;

    app.login(Login { fingerprint }).await?;
    wait_for_balance(app, balance).await?;

    Ok(fingerprint)
}

/// A vault's offer is pre-split, so only the offered amount is locked while it's open. It's
/// completed once it's taken through the pre-split coin's fixed conditions, and cancelled once the
/// vault takes the coin back.
#[tokio::test]
async fn test_vault_offers() -> Result<()> {
    let mut app = TestApp::new().await?;

    let alice = app.setup_bls(10_000).await?;
    wait_for_balance(&mut app, 10_000).await?;

    app.issue_cat(IssueCat {
        name: "Token".to_string(),
        ticker: "TKN".to_string(),
        amount: Amount::u64(1000),
        revocable: false,
        multi_issuance: false,
        fee: Amount::u64(0),
        auto_submit: true,
    })
    .await?;
    let asset_id = first_cat(&mut app).await?;
    wait_for_cat_balance(&mut app, &asset_id, 1000).await?;
    wait_for_balance(&mut app, 9000).await?;

    let vault = setup_vault(&mut app, alice, 9000, 5000).await?;

    let make = |offered, requested| MakeOffer {
        confirmed_handles: Vec::new(),
        requested_assets: vec![requested],
        offered_assets: vec![offered],
        fee: Amount::u64(0),
        receive_address: None,
        expires_at_second: None,
        auto_import: false,
        coin_ids: None,
        pre_split_fee: Some(Amount::u64(0)),
    };

    // A vault offer is pre-split even when it isn't asked for, as by a WalletConnect dApp, since
    // one that isn't would lock the vault.
    let mut taken = make(xch(1000), cat(&asset_id, 500));
    taken.pre_split_fee = None;
    let taken = app.make_offer(taken).await?;
    assert!(taken.pre_split_transaction_id.is_some());

    wait_for_offer_status(&mut app, &taken.offer_id, |status| {
        matches!(status, OfferRecordStatus::Active)
    })
    .await?;

    // Only the offered amount is locked.
    wait_for_balance(&mut app, 4000).await?;

    // The offer is still open once the wallet is loaded again.
    app.login(Login { fingerprint: alice }).await?;
    app.login(Login { fingerprint: vault }).await?;
    wait_for_offer_status(&mut app, &taken.offer_id, |status| {
        matches!(status, OfferRecordStatus::Active)
    })
    .await?;
    wait_for_balance(&mut app, 4000).await?;

    // The next offer is made while the first is open, from the vault's other coins.
    let cancelled = app.make_offer(make(xch(500), cat(&asset_id, 100))).await?;
    wait_for_offer_status(&mut app, &cancelled.offer_id, |status| {
        matches!(status, OfferRecordStatus::Active)
    })
    .await?;
    wait_for_balance(&mut app, 3500).await?;

    app.login(Login { fingerprint: alice }).await?;
    app.take_offer(TakeOffer {
        offer: taken.offer.clone(),
        fee: Amount::u64(0),
        auto_submit: true,
    })
    .await?;
    wait_for_cat_balance(&mut app, &asset_id, 500).await?;

    app.login(Login { fingerprint: vault }).await?;
    wait_for_offer_status(&mut app, &taken.offer_id, |status| {
        matches!(status, OfferRecordStatus::Completed)
    })
    .await?;
    wait_for_cat_balance(&mut app, &asset_id, 500).await?;

    app.cancel_offer(CancelOffer {
        offer_id: cancelled.offer_id.clone(),
        fee: Amount::u64(0),
        auto_submit: true,
        coin_ids: None,
    })
    .await?;

    wait_for_offer_status(&mut app, &cancelled.offer_id, |status| {
        matches!(status, OfferRecordStatus::Cancelled)
    })
    .await?;
    wait_for_balance(&mut app, 4000).await?;

    Ok(())
}

/// A vault can issue a CAT that only it can issue more of.
#[tokio::test]
async fn test_vault_multi_issuance() -> Result<()> {
    let mut app = TestApp::new().await?;

    let alice = app.setup_bls(10_000).await?;
    wait_for_balance(&mut app, 10_000).await?;

    // A BLS wallet can't, since the TAIL is controlled by a vault.
    let issue = |amount| IssueCat {
        name: "Vault token".to_string(),
        ticker: "VLT".to_string(),
        amount: Amount::u64(amount),
        revocable: false,
        multi_issuance: true,
        fee: Amount::u64(0),
        auto_submit: true,
    };
    assert!(app.issue_cat(issue(100)).await.is_err());

    setup_vault(&mut app, alice, 10_000, 5000).await?;

    app.issue_cat(issue(300)).await?;
    let asset_id = first_cat(&mut app).await?;
    wait_for_cat_balance(&mut app, &asset_id, 300).await?;

    app.issue_more_cat(IssueMoreCat {
        asset_id: asset_id.clone(),
        amount: Amount::u64(200),
        fee: Amount::u64(0),
        auto_submit: true,
    })
    .await?;
    wait_for_cat_balance(&mut app, &asset_id, 500).await?;

    Ok(())
}
