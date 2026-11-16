use anyhow::Result;
use sage_api::{
    Amount, GetOffers, GetPendingTransactions, GetSyncStatus, Login, MakeOffer, OfferAmount,
    SendXch,
};

use super::TestApp;

fn send(address: &str, amount: u64) -> SendXch {
    SendXch {
        confirmed_handles: Vec::new(),
        address: address.to_string(),
        amount: Amount::u64(amount),
        fee: Amount::u64(0),
        memos: Vec::new(),
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

/// Two sends that each need the wallet's only coin, made at the same time, must not both
/// select it. One is submitted, and the other finds no coin left to spend.
#[tokio::test]
async fn concurrent_sends_dont_select_the_same_coin() -> Result<()> {
    let mut app = TestApp::new().await?;

    let fingerprint = app.setup_bls(1000).await?;
    app.login(Login { fingerprint }).await?;
    app.wait_for_coins().await;

    let address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;

    let (first, second) = {
        let sage = app.sage.lock().await;

        // The endpoints are called on the same `Sage` directly, so that nothing above them
        // serializes the two requests.
        tokio::join!(
            sage.send_xch(send(&address, 600)),
            sage.send_xch(send(&address, 600)),
        )
    };

    assert!(
        first.is_ok() != second.is_ok(),
        "exactly one send should succeed: {:?}, {:?}",
        first.as_ref().err(),
        second.as_ref().err()
    );

    let pending = app
        .get_pending_transactions(GetPendingTransactions {})
        .await?
        .transactions;
    assert_eq!(pending.len(), 1);

    Ok(())
}

/// A send made while an offer is being created must not spend the coin the offer locks.
#[tokio::test]
async fn concurrent_send_doesnt_spend_a_coin_locked_by_a_new_offer() -> Result<()> {
    let mut app = TestApp::new().await?;

    let fingerprint = app.setup_bls(1000).await?;
    app.login(Login { fingerprint }).await?;
    app.wait_for_coins().await;

    let address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;

    let (offer, sent) = {
        let sage = app.sage.lock().await;

        tokio::join!(
            sage.make_offer(MakeOffer {
                confirmed_handles: Vec::new(),
                requested_assets: vec![xch(100)],
                offered_assets: vec![xch(300)],
                fee: Amount::u64(0),
                receive_address: None,
                expires_at_second: None,
                auto_import: true,
                coin_ids: None,
                pre_split_fee: None,
            }),
            sage.send_xch(send(&address, 600)),
        )
    };

    assert!(
        offer.is_ok() != sent.is_ok(),
        "exactly one of the offer and the send should succeed: {:?}, {:?}",
        offer.as_ref().err(),
        sent.as_ref().err()
    );

    let offers = app.get_offers(GetOffers::default()).await?.offers;
    let pending = app
        .get_pending_transactions(GetPendingTransactions {})
        .await?
        .transactions;

    if offer.is_ok() {
        assert_eq!(offers.len(), 1);
        assert!(pending.is_empty());
    } else {
        assert!(offers.is_empty());
        assert_eq!(pending.len(), 1);
    }

    Ok(())
}
