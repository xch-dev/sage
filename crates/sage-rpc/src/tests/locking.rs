//! Requests that only read `Sage` share it, while those that change it get it to themselves.

use std::time::Duration;

use anyhow::Result;
use sage_api::{Amount, GetPendingTransactions, GetSyncStatus, GetVersion, Login, SendXch};
use tokio::time::timeout;

use super::TestApp;

/// Long enough for a request that isn't waiting on the lock to finish.
const WAIT: Duration = Duration::from_secs(5);

#[tokio::test]
async fn shared_requests_run_while_another_request_holds_sage() -> Result<()> {
    let mut app = TestApp::new().await?;
    app.setup_bls(0).await?;

    // This stands in for a slow request, such as one waiting on the network.
    let held = app.sage.clone().read_owned().await;

    let status = timeout(WAIT, app.get_sync_status(GetSyncStatus {}))
        .await
        .expect("a shared request shouldn't wait for another one")?;
    assert!(!status.receive_address.is_empty());

    timeout(WAIT, app.get_version(GetVersion {}))
        .await
        .expect("a shared request shouldn't wait for another one")?;

    drop(held);

    Ok(())
}

#[tokio::test]
async fn exclusive_requests_wait_for_requests_in_progress() -> Result<()> {
    let mut app = TestApp::new().await?;
    let fingerprint = app.setup_bls(0).await?;

    let held = app.sage.clone().read_owned().await;

    // Logging in replaces the wallet, so it mustn't happen while a request is using it.
    let login = app.login(Login { fingerprint });
    tokio::pin!(login);

    assert!(
        timeout(Duration::from_millis(500), &mut login)
            .await
            .is_err(),
        "an exclusive request should wait for the request in progress"
    );

    // A queued writer keeps new readers out, so this shows that the login is waiting for the
    // lock, rather than just being slow.
    assert!(
        app.sage.try_read().is_err(),
        "the login should be waiting for exclusive access"
    );

    drop(held);

    timeout(WAIT, login)
        .await
        .expect("an exclusive request should run once the request in progress is done")?;

    Ok(())
}

/// Sends made through the RPC at the same time run concurrently now, so it's the spend lock
/// that keeps them from selecting the same coin.
#[tokio::test]
async fn concurrent_rpc_sends_dont_select_the_same_coin() -> Result<()> {
    let mut app = TestApp::new().await?;

    let fingerprint = app.setup_bls(1000).await?;
    app.login(Login { fingerprint }).await?;
    app.wait_for_coins().await;

    let address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;

    let send = || SendXch {
        confirmed_handles: Vec::new(),
        address: address.clone(),
        amount: Amount::u64(600),
        fee: Amount::u64(0),
        memos: Vec::new(),
        clawback: None,
        auto_submit: true,
        coin_ids: None,
        fee_per_cost: None,
        deduct_fee: false,
    };

    let (first, second) = timeout(WAIT, async {
        tokio::join!(app.send_xch(send()), app.send_xch(send()))
    })
    .await
    .expect("the sends should finish rather than deadlock");

    assert!(
        first.is_ok() != second.is_ok(),
        "exactly one send should succeed: {:?}, {:?}",
        first.as_ref().err(),
        second.as_ref().err()
    );

    let error = first.err().or(second.err()).expect("one send should fail");
    assert!(
        error.to_string().contains("no spendable coins"),
        "the other send should find no coin left, but failed with: {error}"
    );

    let pending = app
        .get_pending_transactions(GetPendingTransactions {})
        .await?
        .transactions;
    assert_eq!(pending.len(), 1);

    Ok(())
}
