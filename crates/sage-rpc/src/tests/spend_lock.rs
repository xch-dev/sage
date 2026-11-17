//! Requests that select coins run at the same time here, by calling the endpoints on one
//! `Sage` directly, so that nothing above them serializes the requests. Each pair needs the
//! wallet's only coin, so exactly one must succeed, and the other must find no coin left.
//!
//! `tokio::join!` polls the first request first, so it's the one that gets the spend lock,
//! and the outcome doesn't depend on timing. Without the lock, both requests select the coin
//! before either records it, and both succeed.

use std::{future::Future, time::Duration};

use anyhow::Result;
use sage_api::{
    Amount, GetOffers, GetPendingTransactions, GetSyncStatus, ImportOffer, Login, MakeOffer,
    OfferAmount, SendXch, SignCoinSpends, SubmitTransaction, TakeOffer,
};
use tokio::time::timeout;

use super::TestApp;

fn send(address: &str, auto_submit: bool) -> SendXch {
    SendXch {
        confirmed_handles: Vec::new(),
        address: address.to_string(),
        amount: Amount::u64(600),
        fee: Amount::u64(0),
        memos: Vec::new(),
        clawback: None,
        auto_submit,
        coin_ids: None,
        fee_per_cost: None,
    }
}

fn offer(auto_import: bool) -> MakeOffer {
    let xch = |amount| OfferAmount {
        asset_id: None,
        hidden_puzzle_hash: None,
        amount: Amount::u64(amount),
    };

    MakeOffer {
        confirmed_handles: Vec::new(),
        requested_assets: vec![xch(600)],
        offered_assets: vec![xch(300)],
        fee: Amount::u64(0),
        receive_address: None,
        expires_at_second: None,
        auto_import,
        coin_ids: None,
        pre_split_fee: None,
        pre_split_fee_per_cost: None,
    }
}

/// Runs both requests at once, failing rather than hanging if they deadlock. The requests
/// are boxed by the caller, since the endpoint futures are large.
async fn concurrently<A, B>(first: A, second: B) -> (A::Output, B::Output)
where
    A: Future,
    B: Future,
{
    timeout(Duration::from_secs(30), async {
        tokio::join!(first, second)
    })
    .await
    .expect("the requests should finish rather than deadlock")
}

/// Checks that the first request succeeded, and that the second failed because the coin was
/// already taken, rather than for some other reason.
fn assert_first_wins<A, B>(first: &sage::Result<A>, second: &sage::Result<B>) {
    if let Err(error) = first {
        panic!("the first request should succeed: {error}");
    }

    match second {
        Ok(_) => panic!("the second request should fail, since the coin is taken"),
        Err(error) => assert!(
            error.to_string().contains("no spendable coins"),
            "the second request should find no coin left, but failed with: {error}"
        ),
    }
}

/// Logs into a new wallet with a single coin of 1000 mojos, and returns its fingerprint and
/// address.
async fn wallet_with_one_coin(app: &mut TestApp) -> Result<(u32, String)> {
    let fingerprint = app.setup_bls(1000).await?;
    app.login(Login { fingerprint }).await?;
    app.wait_for_coins().await;

    let address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;

    Ok((fingerprint, address))
}

async fn pending_transactions(app: &TestApp) -> Result<usize> {
    Ok(app
        .get_pending_transactions(GetPendingTransactions {})
        .await?
        .transactions
        .len())
}

async fn saved_offers(app: &TestApp) -> Result<usize> {
    Ok(app.get_offers(GetOffers::default()).await?.offers.len())
}

#[tokio::test]
async fn concurrent_sends_dont_select_the_same_coin() -> Result<()> {
    let mut app = TestApp::new().await?;
    let (_, address) = wallet_with_one_coin(&mut app).await?;

    let (first, second) = {
        let sage = app.sage.read().await;
        concurrently(
            Box::pin(sage.send_xch(send(&address, true))),
            Box::pin(sage.send_xch(send(&address, true))),
        )
        .await
    };

    assert_first_wins(&first, &second);
    assert_eq!(pending_transactions(&app).await?, 1);

    Ok(())
}

#[tokio::test]
async fn send_doesnt_spend_a_coin_locked_by_a_concurrent_offer() -> Result<()> {
    let mut app = TestApp::new().await?;
    let (_, address) = wallet_with_one_coin(&mut app).await?;

    let (made, sent) = {
        let sage = app.sage.read().await;
        concurrently(
            Box::pin(sage.make_offer(offer(true))),
            Box::pin(sage.send_xch(send(&address, true))),
        )
        .await
    };

    assert_first_wins(&made, &sent);
    assert_eq!(saved_offers(&app).await?, 1);
    assert_eq!(pending_transactions(&app).await?, 0);

    Ok(())
}

#[tokio::test]
async fn send_doesnt_select_a_coin_a_concurrent_submission_spends() -> Result<()> {
    let mut app = TestApp::new().await?;
    let (_, address) = wallet_with_one_coin(&mut app).await?;

    let coin_spends = app.send_xch(send(&address, false)).await?.coin_spends;
    let spend_bundle = app
        .sign_coin_spends(SignCoinSpends {
            coin_spends,
            auto_submit: false,
            partial: false,
            aggregated_signature: None,
        })
        .await?
        .spend_bundle;

    let (submitted, sent) = {
        let sage = app.sage.read().await;
        concurrently(
            Box::pin(sage.submit_transaction(SubmitTransaction { spend_bundle })),
            Box::pin(sage.send_xch(send(&address, true))),
        )
        .await
    };

    assert_first_wins(&submitted, &sent);
    assert_eq!(pending_transactions(&app).await?, 1);

    Ok(())
}

#[tokio::test]
async fn send_doesnt_spend_a_coin_locked_by_a_concurrent_import() -> Result<()> {
    let mut app = TestApp::new().await?;
    let (_, address) = wallet_with_one_coin(&mut app).await?;

    let encoded = app.make_offer(offer(false)).await?.offer;

    let (imported, sent) = {
        let sage = app.sage.read().await;
        concurrently(
            Box::pin(sage.import_offer(ImportOffer { offer: encoded })),
            Box::pin(sage.send_xch(send(&address, true))),
        )
        .await
    };

    assert_first_wins(&imported, &sent);
    assert_eq!(saved_offers(&app).await?, 1);
    assert_eq!(pending_transactions(&app).await?, 0);

    Ok(())
}

#[tokio::test]
async fn send_doesnt_select_a_coin_a_concurrent_take_spends() -> Result<()> {
    let mut app = TestApp::new().await?;

    // The maker asks for 600, which the taker can only pay with its 1000 mojo coin.
    let (maker, _) = wallet_with_one_coin(&mut app).await?;
    let (taker, address) = wallet_with_one_coin(&mut app).await?;

    app.login(Login { fingerprint: maker }).await?;
    app.wait_for_coins().await;
    let encoded = app.make_offer(offer(false)).await?.offer;

    app.login(Login { fingerprint: taker }).await?;
    app.wait_for_coins().await;

    let (taken, sent) = {
        let sage = app.sage.read().await;
        concurrently(
            Box::pin(sage.take_offer(TakeOffer {
                offer: encoded,
                fee: Amount::u64(0),
                auto_submit: true,
                fee_per_cost: None,
            })),
            Box::pin(sage.send_xch(send(&address, true))),
        )
        .await
    };

    assert_first_wins(&taken, &sent);
    assert_eq!(pending_transactions(&app).await?, 1);

    Ok(())
}
