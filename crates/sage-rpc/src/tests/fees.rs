//! Transactions priced at a fee rate pay a fee that covers their own cost, including the coins
//! that pay it.

use anyhow::Result;
use sage_api::{Amount, GetFeeRate, GetSyncStatus, Login, SendXch};

use super::TestApp;

/// One XCH, so that the fee at the recommended rate fits in the balance.
const BALANCE: u64 = 1_000_000_000_000;

fn send(address: String, amount: u64, fee: u64, fee_per_cost: Option<f64>) -> SendXch {
    SendXch {
        confirmed_handles: Vec::new(),
        address,
        amount: Amount::u64(amount),
        fee: Amount::u64(fee),
        fee_per_cost,
        memos: vec![],
        clawback: None,
        auto_submit: false,
        coin_ids: None,
    }
}

#[tokio::test]
async fn fee_covers_cost_at_rate() -> Result<()> {
    let mut app = TestApp::new().await?;
    let alice = app.setup_bls(BALANCE).await?;
    app.login(Login { fingerprint: alice }).await?;
    app.wait_for_coins().await;

    let address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;
    let rates = app.get_fee_rate(GetFeeRate {}).await?;
    let rate = rates.fee_per_cost;
    assert!((rate - 5.0).abs() < f64::EPSILON);
    assert!(rates.minimum_fee_per_cost <= rate);

    let mut request = send(address, 1_000, 0, Some(rate));
    request.auto_submit = true;
    let summary = app.send_xch(request).await?.summary;

    let cost = summary.cost.expect("a complete transaction has a cost");
    let fee = summary.fee.to_u64().expect("fee fits in u64");
    let needed = cost * 5;

    assert!(fee >= needed, "fee {fee} doesn't cover cost {cost}");
    // The fee is raised by what's missing, so it overshoots by at most the cost the higher fee
    // added, which is a few bytes for a larger `RESERVE_FEE`, not another coin.
    assert!(fee - needed < 100_000, "fee {fee} overshoots {needed}");

    app.wait_for_coins().await;

    let balance = app
        .get_sync_status(GetSyncStatus {})
        .await?
        .selectable_balance
        .to_u64();
    assert_eq!(balance, Some(BALANCE - fee));

    Ok(())
}

#[tokio::test]
async fn fee_is_at_least_the_given_fee() -> Result<()> {
    let mut app = TestApp::new().await?;
    let alice = app.setup_bls(BALANCE).await?;
    app.login(Login { fingerprint: alice }).await?;
    app.wait_for_coins().await;

    let address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;

    // The given fee already covers the cost, so it's paid as is.
    let summary = app
        .send_xch(send(address.clone(), 1_000, 1_000_000_000, Some(5.0)))
        .await?
        .summary;
    assert_eq!(summary.fee.to_u64(), Some(1_000_000_000));

    // Without a rate, the given fee is paid even when it doesn't cover the cost.
    let summary = app.send_xch(send(address, 1_000, 0, None)).await?.summary;
    assert_eq!(summary.fee.to_u64(), Some(0));
    assert!(summary.cost.is_some());

    Ok(())
}

#[tokio::test]
async fn fee_must_fit_in_balance() -> Result<()> {
    let mut app = TestApp::new().await?;
    let alice = app.setup_bls(BALANCE).await?;
    app.login(Login { fingerprint: alice }).await?;
    app.wait_for_coins().await;

    let address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;

    // Sending the whole balance leaves nothing for the fee.
    assert!(
        app.send_xch(send(address.clone(), BALANCE, 0, Some(5.0)))
            .await
            .is_err()
    );

    // A negative rate is rejected rather than ignored.
    assert!(
        app.send_xch(send(address, 1_000, 0, Some(-1.0)))
            .await
            .is_err()
    );

    Ok(())
}
