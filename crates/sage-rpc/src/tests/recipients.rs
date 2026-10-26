use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use axum::http::StatusCode;
use chia_wallet_sdk::{prelude::*, utils::Address};
use sage_api::{
    Amount, BulkSendXch, HandleConfirmation, Login, MakeOffer, OfferAmount, OfferHandleStatus,
    SendXch, SignCoinSpends, TakeOffer, TransactionResponse, ViewOffer,
};
use sage_database::{XchandleRow, XchandleStatus};
use serde_json::Value;

use super::{TestApp, mint_mutable_nft, mint_mutable_nft_named};

fn puzzle_hash(byte: u8) -> Bytes32 {
    Bytes32::new([byte; 32])
}

fn address(byte: u8) -> String {
    Address::new(puzzle_hash(byte), "txch".to_string())
        .encode()
        .expect("valid address")
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before epoch")
        .as_secs()
}

fn confirm(handle: &str, address: String) -> HandleConfirmation {
    HandleConfirmation {
        handle: handle.to_string(),
        address,
    }
}

fn send(address: &str, confirmed_handles: Vec<HandleConfirmation>, auto_submit: bool) -> SendXch {
    SendXch {
        address: address.to_string(),
        amount: Amount::u64(100),
        fee: Amount::u64(0),
        memos: vec![],
        clawback: None,
        auto_submit,
        coin_ids: None,
        confirmed_handles,
    }
}

async fn registry_launcher_id(app: &TestApp) -> Bytes32 {
    app.sage
        .lock()
        .await
        .network()
        .xchandles()
        .expect("missing XCHandles deployment")
        .registry_launcher_id
}

/// Stores a mapping as if it had been verified a minute ago. The test API is unavailable
/// unless a response is configured, so lookups fall back to it.
async fn store_handle(
    app: &TestApp,
    handle: &str,
    resolved_puzzle_hash: Bytes32,
    edit: impl FnOnce(&mut XchandleRow),
) -> Result<()> {
    let now = now();

    let mut row = XchandleRow {
        registry_launcher_id: registry_launcher_id(app).await,
        handle: handle.to_string(),
        handle_hash: Bytes32::default(),
        owner_launcher_id: Bytes32::default(),
        resolved_launcher_id: Bytes32::default(),
        resolved_puzzle_hash,
        expiration: now + 365 * 24 * 60 * 60,
        slot_coin_id: Bytes32::default(),
        slot_counter: 0,
        slot_left_handle_hash: Bytes32::default(),
        slot_right_handle_hash: Bytes32::default(),
        slot_parent_id: Bytes32::default(),
        slot_parent_parent_id: Bytes32::default(),
        slot_parent_inner_puzzle_hash: Bytes32::default(),
        slot_confirmation_height: 10,
        resolved_coin_id: Bytes32::default(),
        resolved_confirmation_height: 20,
        indexed_height: 0,
        verified_height: 0,
        verified_timestamp: now - 60,
        last_verified_at: now - 60,
        status: XchandleStatus::Verified,
        status_reason: None,
        previous_resolved_launcher_id: None,
        previous_resolved_puzzle_hash: None,
        last_sent_puzzle_hash: None,
    };

    edit(&mut row);

    let wallet = app.sage.lock().await.wallet()?;
    wallet.db.upsert_xchandle(&row).await?;

    Ok(())
}

async fn stored_handle(app: &TestApp, handle: &str) -> Result<XchandleRow> {
    let registry_launcher_id = registry_launcher_id(app).await;
    let wallet = app.sage.lock().await.wallet()?;

    Ok(wallet
        .db
        .xchandle(registry_launcher_id, handle)
        .await?
        .expect("missing handle mapping"))
}

/// The status and JSON body of a failed request.
fn error_body(error: &anyhow::Error) -> (StatusCode, Value) {
    let message = error.to_string();
    let status = if message.contains("status 409") {
        StatusCode::CONFLICT
    } else if message.contains("status 400") {
        StatusCode::BAD_REQUEST
    } else {
        panic!("unexpected error: {message}");
    };

    let body = &message[message.find('{').expect("missing JSON body")..];
    (
        status,
        serde_json::from_str(body).expect("invalid JSON body"),
    )
}

fn recipient_kinds(body: &Value) -> Vec<&str> {
    body["recipients"]
        .as_array()
        .expect("missing recipient errors")
        .iter()
        .map(|recipient| recipient["kind"].as_str().expect("missing kind"))
        .collect()
}

fn pays(response: &TransactionResponse, address: &str) -> bool {
    response
        .summary
        .inputs
        .iter()
        .flat_map(|input| &input.outputs)
        .any(|output| output.address == address)
}

async fn funded_app() -> Result<TestApp> {
    let mut app = TestApp::new().await?;
    let alice = app.setup_bls(1000).await?;
    app.login(Login { fingerprint: alice }).await?;
    app.wait_for_coins().await;
    Ok(app)
}

#[tokio::test]
async fn test_send_to_handle_records_last_sent_address() -> Result<()> {
    let mut app = funded_app().await?;
    store_handle(&app, "alice", puzzle_hash(1), |_| {}).await?;

    let response = app.send_xch(send("@alice", vec![], false)).await?;

    // The summary shows the handle with the address it resolved to, which is what's paid.
    assert_eq!(response.summary.handles.len(), 1);
    let handle = &response.summary.handles[0];
    assert_eq!(handle.handle, "alice");
    assert_eq!(handle.address, address(1));
    assert!(handle.cached);
    assert_eq!(handle.changed_from, None);
    // The resolved singleton moved after the slot was last updated.
    assert_eq!(handle.updated_height, 20);
    assert!(pays(&response, &address(1)));

    // Nothing is recorded until the transaction is submitted.
    assert_eq!(
        stored_handle(&app, "alice").await?.last_sent_puzzle_hash,
        None
    );

    app.sign_coin_spends(SignCoinSpends {
        coin_spends: response.coin_spends,
        auto_submit: true,
        partial: false,
        aggregated_signature: None,
    })
    .await?;
    app.wait_for_coins().await;

    assert_eq!(
        stored_handle(&app, "alice").await?.last_sent_puzzle_hash,
        Some(puzzle_hash(1))
    );

    // Sending to the same address again doesn't need to be confirmed, and surrounding
    // whitespace is ignored.
    app.send_xch(send("@alice", vec![], false)).await?;
    app.send_xch(send(" @alice ", vec![], false)).await?;

    Ok(())
}

#[tokio::test]
async fn test_changed_handle_requires_confirmation() -> Result<()> {
    let mut app = funded_app().await?;

    // The handle was verified as resolving to one address, and now resolves to another.
    store_handle(&app, "alice", puzzle_hash(2), |row| {
        row.previous_resolved_launcher_id = Some(Bytes32::default());
        row.previous_resolved_puzzle_hash = Some(puzzle_hash(1));
    })
    .await?;

    let error = app
        .send_xch(send("@alice", vec![], true))
        .await
        .unwrap_err();
    let (status, body) = error_body(&error);
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["kind"], "recipient");
    assert_eq!(recipient_kinds(&body), ["changed"]);
    assert_eq!(body["recipients"][0]["handle"], "alice");
    assert_eq!(body["recipients"][0]["address"], address(2));
    assert_eq!(body["recipients"][0]["previous_address"], address(1));

    // Confirming the address it used to resolve to doesn't help.
    let error = app
        .send_xch(send("@alice", vec![confirm("alice", address(1))], true))
        .await
        .unwrap_err();
    assert_eq!(error_body(&error).0, StatusCode::CONFLICT);

    // Confirming the new address does, and acknowledges the change once it's submitted.
    app.send_xch(send("@alice", vec![confirm("@alice", address(2))], true))
        .await?;
    app.wait_for_coins().await;

    let row = stored_handle(&app, "alice").await?;
    assert_eq!(row.previous_resolved_puzzle_hash, None);
    assert_eq!(row.last_sent_puzzle_hash, Some(puzzle_hash(2)));

    app.send_xch(send("@alice", vec![], false)).await?;

    // A handle which resolves to a different address than the one last sent to also needs
    // to be confirmed, even if the change was already acknowledged.
    store_handle(&app, "bob", puzzle_hash(3), |row| {
        row.last_sent_puzzle_hash = Some(puzzle_hash(4));
    })
    .await?;

    let error = app.send_xch(send("@bob", vec![], false)).await.unwrap_err();
    let (status, body) = error_body(&error);
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["recipients"][0]["previous_address"], address(4));
    assert_eq!(body["recipients"][0]["address"], address(3));

    Ok(())
}

#[tokio::test]
async fn test_handle_changed_before_submission_is_not_submitted() -> Result<()> {
    let app = funded_app().await?;
    store_handle(&app, "alice", puzzle_hash(1), |_| {}).await?;

    let response = app.send_xch(send("@alice", vec![], false)).await?;

    // The handle changes hands between the transaction being reviewed and approved.
    store_handle(&app, "alice", puzzle_hash(2), |row| {
        row.previous_resolved_launcher_id = Some(Bytes32::default());
        row.previous_resolved_puzzle_hash = Some(puzzle_hash(1));
    })
    .await?;

    let sign = || SignCoinSpends {
        coin_spends: response.coin_spends.clone(),
        auto_submit: true,
        partial: false,
        aggregated_signature: None,
    };

    let error = app.sign_coin_spends(sign()).await.unwrap_err();
    let (status, body) = error_body(&error);
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(recipient_kinds(&body), ["changed"]);
    assert_eq!(body["recipients"][0]["previous_address"], address(1));
    assert_eq!(body["recipients"][0]["address"], address(2));
    let reason = body["recipients"][0]["reason"].as_str().unwrap_or_default();
    assert!(reason.contains("Create the transaction again"), "{reason}");

    // Retrying is checked again, rather than submitting it the second time.
    let error = app.sign_coin_spends(sign()).await.unwrap_err();
    assert_eq!(error_body(&error).0, StatusCode::CONFLICT);

    assert_eq!(
        stored_handle(&app, "alice").await?.last_sent_puzzle_hash,
        None
    );

    Ok(())
}

#[tokio::test]
async fn test_bulk_send_reports_each_recipient() -> Result<()> {
    let app = funded_app().await?;

    store_handle(&app, "alice", puzzle_hash(1), |_| {}).await?;

    // Mappings which can't be used offline.
    store_handle(&app, "stale", puzzle_hash(2), |row| {
        row.status = XchandleStatus::Stale;
    })
    .await?;
    store_handle(&app, "old", puzzle_hash(3), |row| {
        row.last_verified_at = now() - 2 * 24 * 60 * 60;
    })
    .await?;

    app.xchandles.respond(
        "/handle/nobody",
        StatusCode::NOT_FOUND,
        r#"{"code":"handle_not_found","message":"Handle not found"}"#,
    );

    let mainnet_address = Address::new(puzzle_hash(5), "xch".to_string()).encode()?;

    let bulk = |addresses: Vec<String>| BulkSendXch {
        addresses,
        amount: Amount::u64(10),
        fee: Amount::u64(0),
        memos: vec![],
        auto_submit: false,
        confirmed_handles: vec![],
    };

    let error = app
        .bulk_send_xch(bulk(vec![
            "@alice".to_string(),
            "@nobody".to_string(),
            "@uncached".to_string(),
            "@Al".to_string(),
            mainnet_address.clone(),
            "@stale".to_string(),
            "@old".to_string(),
            address(6),
        ]))
        .await
        .unwrap_err();

    let (status, body) = error_body(&error);
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        recipient_kinds(&body),
        [
            "not_registered",
            "unavailable",
            "invalid",
            "invalid",
            "unavailable",
            "unavailable",
        ]
    );
    let recipients = body["recipients"].as_array().expect("missing recipients");
    assert_eq!(recipients[0]["recipient"], "@nobody");
    assert_eq!(recipients[3]["recipient"], mainnet_address);

    // Handles and addresses can be mixed.
    let response = app
        .bulk_send_xch(bulk(vec!["@alice".to_string(), address(6)]))
        .await?;
    assert_eq!(response.summary.handles.len(), 1);
    assert!(pays(&response, &address(1)));
    assert!(pays(&response, &address(6)));

    Ok(())
}

#[tokio::test]
async fn test_raw_address_errors_are_unchanged() -> Result<()> {
    let app = funded_app().await?;

    let mainnet_address = Address::new(puzzle_hash(5), "xch".to_string()).encode()?;
    let error = app
        .send_xch(send(&mainnet_address, vec![], false))
        .await
        .unwrap_err()
        .to_string();

    assert!(error.contains("status 400"), "{error}");
    assert!(error.contains("Wrong address prefix"), "{error}");
    assert!(!error.contains("recipients"), "{error}");

    let response = app.send_xch(send(&address(1), vec![], false)).await?;
    assert!(response.summary.handles.is_empty());
    assert!(pays(&response, &address(1)));

    Ok(())
}

#[tokio::test]
async fn test_handle_recipients_on_unsupported_networks() -> Result<()> {
    let app = funded_app().await?;

    for network in &mut app.sage.lock().await.network_list.networks {
        network.inherit = None;
        network.xchandles = None;
    }

    let error = app
        .send_xch(send("@alice", vec![], false))
        .await
        .unwrap_err();
    let (status, body) = error_body(&error);
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(recipient_kinds(&body), ["unsupported_network"]);

    Ok(())
}

#[tokio::test]
async fn test_offer_with_pending_handle_transfer() -> Result<()> {
    let mut app = funded_app().await?;

    // The NFT's display name claims it's the handle NFT for @alice.
    let nft_id = mint_mutable_nft(&mut app).await?;
    let launcher_id = Address::decode(&nft_id)?.puzzle_hash;

    let offer = app
        .make_offer(MakeOffer {
            requested_assets: vec![OfferAmount {
                asset_id: None,
                hidden_puzzle_hash: None,
                amount: Amount::u64(1),
            }],
            offered_assets: vec![OfferAmount {
                asset_id: Some(nft_id.clone()),
                hidden_puzzle_hash: None,
                amount: Amount::u64(1),
            }],
            fee: Amount::u64(0),
            receive_address: None,
            expires_at_second: None,
            auto_import: false,
            coin_ids: None,
            confirmed_handles: vec![],
        })
        .await?
        .offer;

    let status = async |app: &TestApp| -> Result<OfferHandleStatus> {
        let handle_nfts = app
            .view_offer(ViewOffer {
                offer: offer.clone(),
            })
            .await?
            .handle_nfts;
        assert_eq!(handle_nfts.len(), 1);
        assert_eq!(handle_nfts[0].nft_id, nft_id);
        assert_eq!(handle_nfts[0].handle, "alice");
        Ok(handle_nfts[0].status)
    };

    let take = || TakeOffer {
        offer: offer.clone(),
        fee: Amount::u64(0),
        auto_submit: false,
    };

    // The handle can't be checked, so the offer can't be taken.
    assert_eq!(status(&app).await?, OfferHandleStatus::Unavailable);
    let error = app.take_offer(take()).await.unwrap_err().to_string();
    assert!(error.contains("can't be taken"), "{error}");

    // The registry says the handle is owned by a different NFT, so the display name lied.
    store_handle(&app, "alice", puzzle_hash(1), |row| {
        row.owner_launcher_id = puzzle_hash(9);
    })
    .await?;
    assert_eq!(status(&app).await?, OfferHandleStatus::NotAssociated);

    // The NFT owns the handle, and no update is pending.
    store_handle(&app, "alice", puzzle_hash(1), |row| {
        row.owner_launcher_id = launcher_id;
    })
    .await?;
    app.xchandles
        .respond("/handle/alice/pending-transfer", StatusCode::NO_CONTENT, "");
    assert_eq!(status(&app).await?, OfferHandleStatus::Verified);

    // An update is pending, which would let the seller take the handle back.
    app.xchandles.respond(
        "/handle/alice/pending-transfer",
        StatusCode::OK,
        &serde_json::json!({
            "new_owner_launcher_id": hex::encode(puzzle_hash(7)),
            "new_resolved_launcher_id": hex::encode(puzzle_hash(7)),
            "update_confirmation_height": 100,
            "minimum_execution_height": 132,
            "initiator_coin_id": hex::encode(puzzle_hash(3)),
            "executor_coin_id": hex::encode(puzzle_hash(4)),
        })
        .to_string(),
    );
    assert_eq!(status(&app).await?, OfferHandleStatus::PendingTransfer);

    let error = app.take_offer(take()).await.unwrap_err().to_string();
    assert!(error.contains("can't be taken"), "{error}");
    assert!(error.contains("pending"), "{error}");

    Ok(())
}

#[tokio::test]
async fn test_offer_nft_named_like_a_handle_is_not_a_claim() -> Result<()> {
    let mut app = funded_app().await?;

    // Only names written as `@handle` claim to be a handle, so an NFT which happens to be
    // named like one doesn't block the offer while handles can't be checked.
    let nft_id = mint_mutable_nft_named(&mut app, "alice").await?;

    let offer = app
        .make_offer(MakeOffer {
            requested_assets: vec![OfferAmount {
                asset_id: None,
                hidden_puzzle_hash: None,
                amount: Amount::u64(1),
            }],
            offered_assets: vec![OfferAmount {
                asset_id: Some(nft_id),
                hidden_puzzle_hash: None,
                amount: Amount::u64(1),
            }],
            fee: Amount::u64(0),
            receive_address: None,
            expires_at_second: None,
            auto_import: false,
            coin_ids: None,
            confirmed_handles: vec![],
        })
        .await?
        .offer;

    let handle_nfts = app
        .view_offer(ViewOffer {
            offer: offer.clone(),
        })
        .await?
        .handle_nfts;
    assert!(handle_nfts.is_empty());

    app.take_offer(TakeOffer {
        offer,
        fee: Amount::u64(0),
        auto_submit: false,
    })
    .await?;

    Ok(())
}
