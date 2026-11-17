//! Transactions priced at a fee rate pay a fee that covers their own cost, including the coins
//! that pay it.

use anyhow::Result;
use chia_wallet_sdk::{driver::spend_bundle_cost, prelude::*};
use sage::{rust_bundle, rust_spend};
use sage_api::{
    Action as ApiAction, Amount, CoinSpendJson, CreateTransaction, FeeAction, GetCats, GetFeeRate,
    GetSyncStatus, Id as ApiId, IssueCat, Login, MakeOffer, OfferAmount, SendAction, SendXch,
    TakeOffer, TransactionSummary,
};

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

fn xch(amount: u64) -> OfferAmount {
    OfferAmount {
        asset_id: None,
        hidden_puzzle_hash: None,
        amount: Amount::u64(amount),
    }
}

/// The cost of the coin spends as the mempool charges it, worked out here rather than taken
/// from the wallet, so that a wrong cost in the wallet's pricing can't hide itself. The tests
/// run on testnet11's rules, at the wallet's current peak.
async fn independent_cost(app: &TestApp, coin_spends: &[CoinSpend]) -> Result<u64> {
    let height = app
        .sage
        .read()
        .await
        .wallet()?
        .db
        .latest_peak()
        .await?
        .map(|(height, _)| height);

    Ok(spend_bundle_cost(
        coin_spends,
        &TESTNET11_CONSTANTS,
        height,
    )?)
}

fn fee(summary: &TransactionSummary) -> u64 {
    summary.fee.to_u64().expect("fee fits in u64")
}

/// Checks that the fee covers the cost at 5 mojos per cost, without overshooting by more than
/// the few bytes a larger `RESERVE_FEE` adds, and that the summary reports the same cost.
async fn assert_priced(
    app: &TestApp,
    summary: &TransactionSummary,
    coin_spends: &[CoinSpendJson],
) -> Result<u64> {
    let coin_spends = coin_spends
        .iter()
        .cloned()
        .map(rust_spend)
        .collect::<Result<Vec<_>, _>>()?;
    let cost = independent_cost(app, &coin_spends).await?;
    let needed = cost * 5;

    assert_eq!(summary.cost, Some(cost));
    assert!(
        fee(summary) >= needed,
        "fee {} under {needed}",
        fee(summary)
    );
    assert!(
        fee(summary) - needed < 100_000,
        "fee {} overshoots {needed}",
        fee(summary)
    );

    Ok(cost)
}

async fn selectable_balance(app: &TestApp) -> Result<u64> {
    Ok(app
        .get_sync_status(GetSyncStatus {})
        .await?
        .selectable_balance
        .to_u64()
        .expect("balance fits in u64"))
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
    assert!((rates.minimum_fee_per_cost - 5.0).abs() < f64::EPSILON);

    let mut request = send(address, 1_000, 0, Some(rate));
    request.auto_submit = true;
    let response = app.send_xch(request).await?;
    let cost = assert_priced(&app, &response.summary, &response.coin_spends).await?;

    // A standard spend costs millions, so a cost this low means it wasn't priced properly.
    assert!(cost > 1_000_000, "cost {cost}");

    app.wait_for_coins().await;
    assert_eq!(
        selectable_balance(&app).await?,
        BALANCE - fee(&response.summary)
    );

    Ok(())
}

#[tokio::test]
async fn fee_coin_is_selected_and_priced() -> Result<()> {
    let mut app = TestApp::new().await?;

    // Each coin covers the payment on its own, but not the payment and the fee, so paying the
    // fee needs a second coin, which raises the cost.
    let coin = 1_000_000_000;
    let alice = app.setup_bls_coins(&[coin, coin, coin]).await?;
    app.login(Login { fingerprint: alice }).await?;
    app.wait_for_coins().await;

    let address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;
    let amount = coin - 1_000_000;

    let mut request = send(address, amount, 0, Some(5.0));
    request.auto_submit = true;
    let response = app.send_xch(request).await?;

    assert_eq!(response.coin_spends.len(), 2);
    assert_priced(&app, &response.summary, &response.coin_spends).await?;

    app.wait_for_coins().await;
    assert_eq!(
        selectable_balance(&app).await?,
        3 * coin - fee(&response.summary)
    );

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
    assert_eq!(fee(&summary), 1_000_000_000);

    // Without a rate, the given fee is paid even when it doesn't cover the cost.
    let summary = app.send_xch(send(address, 1_000, 0, None)).await?.summary;
    assert_eq!(fee(&summary), 0);
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

async fn make_offer(app: &mut TestApp, maker_fee: u64) -> Result<String> {
    Ok(app
        .make_offer(MakeOffer {
            confirmed_handles: Vec::new(),
            requested_assets: vec![xch(100)],
            offered_assets: vec![xch(300)],
            fee: Amount::u64(maker_fee),
            receive_address: None,
            expires_at_second: None,
            auto_import: false,
            coin_ids: None,
            pre_split_fee: None,
            pre_split_fee_per_cost: None,
        })
        .await?
        .offer)
}

#[tokio::test]
async fn taking_an_offer_counts_the_maker_fee() -> Result<()> {
    let mut app = TestApp::new().await?;
    let alice = app.setup_bls(BALANCE).await?;
    let bob = app.setup_bls(BALANCE).await?;

    app.login(Login { fingerprint: alice }).await?;
    app.wait_for_coins().await;

    // The maker's fee already covers the whole transaction, so the taker adds nothing.
    let generous = make_offer(&mut app, 1_000_000_000).await?;
    // The maker's fee covers only a little, so the taker pays the rest.
    let stingy = make_offer(&mut app, 50).await?;

    app.login(Login { fingerprint: bob }).await?;
    app.wait_for_coins().await;

    let take = |offer: String| TakeOffer {
        offer,
        fee: Amount::u64(0),
        auto_submit: false,
        fee_per_cost: Some(5.0),
    };

    let summary = app.take_offer(take(generous)).await?.summary;
    assert_eq!(fee(&summary), 1_000_000_000);

    let response = app.take_offer(take(stingy)).await?;
    let coin_spends = rust_bundle(response.spend_bundle)?.coin_spends;
    let cost = independent_cost(&app, &coin_spends).await?;
    let paid = fee(&response.summary);
    assert_eq!(response.summary.cost, Some(cost));
    assert!(paid >= cost * 5, "fee {paid} under {}", cost * 5);
    // The fee never goes down between attempts, and the offer's surplus can change which outputs
    // the taker needs, so an earlier attempt can cost a little more than the final one. The fee
    // stays close to the rate, though.
    assert!(paid < cost * 6, "fee {paid} overshoots {}", cost * 5);

    Ok(())
}

#[tokio::test]
async fn create_transaction_adds_a_fee_action() -> Result<()> {
    let mut app = TestApp::new().await?;
    let alice = app.setup_bls(BALANCE).await?;
    app.login(Login { fingerprint: alice }).await?;
    app.wait_for_coins().await;

    let address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;

    let request = |fee: Option<u64>| {
        let mut actions = vec![ApiAction::Send(SendAction {
            id: ApiId::Xch,
            address: address.clone(),
            amount: Amount::u64(1_000),
            clawback: None,
            memos: vec![],
        })];

        if let Some(fee) = fee {
            actions.push(ApiAction::Fee(FeeAction {
                amount: Amount::u64(fee),
            }));
        }

        CreateTransaction {
            confirmed_handles: Vec::new(),
            actions,
            selected_coin_ids: vec![],
            auto_submit: false,
            fee_per_cost: Some(5.0),
        }
    };

    let response = app.create_transaction(request(None)).await?;
    assert_eq!(response.coin_spends.len(), 1);
    assert_priced(&app, &response.summary, &response.coin_spends).await?;

    // A fee action that already covers the cost is paid as is, with nothing added.
    let summary = app
        .create_transaction(request(Some(1_000_000_000)))
        .await?
        .summary;
    assert_eq!(fee(&summary), 1_000_000_000);

    Ok(())
}

#[tokio::test]
async fn issued_cat_is_recorded_from_the_priced_transaction() -> Result<()> {
    let mut app = TestApp::new().await?;

    // The first attempt, without a fee, spends one coin, and the priced one needs both. The
    // asset id comes from the coin that's spent, so it has to be taken from the final attempt.
    let coin = 1_000_000_000;
    let alice = app.setup_bls_coins(&[coin, coin]).await?;
    app.login(Login { fingerprint: alice }).await?;
    app.wait_for_coins().await;

    let response = app
        .issue_cat(IssueCat {
            name: "Priced".to_string(),
            ticker: "PRC".to_string(),
            amount: Amount::u64(coin - 1_000_000),
            revocable: false,
            fee: Amount::u64(0),
            fee_per_cost: Some(5.0),
            auto_submit: true,
        })
        .await?;
    assert_priced(&app, &response.summary, &response.coin_spends).await?;
    app.wait_for_coins().await;

    let cats = app.get_cats(GetCats {}).await?.cats;
    assert_eq!(cats.len(), 1);
    assert_eq!(cats[0].name.as_deref(), Some("Priced"));
    assert_eq!(cats[0].balance.to_u64(), Some(coin - 1_000_000));

    Ok(())
}

#[tokio::test]
async fn pre_split_transaction_is_priced() -> Result<()> {
    let mut app = TestApp::new().await?;
    let alice = app.setup_bls(BALANCE).await?;
    app.login(Login { fingerprint: alice }).await?;
    app.wait_for_coins().await;

    let response = app
        .make_offer(MakeOffer {
            confirmed_handles: Vec::new(),
            requested_assets: vec![xch(1)],
            offered_assets: vec![xch(300)],
            fee: Amount::u64(50),
            receive_address: None,
            expires_at_second: None,
            auto_import: false,
            coin_ids: None,
            pre_split_fee: Some(Amount::u64(0)),
            pre_split_fee_per_cost: Some(5.0),
        })
        .await?;
    assert!(response.pre_split_transaction_id.is_some());

    // The split's change is spendable once it confirms. Everything else is the offered amount,
    // the offer's fee, and the split's fee.
    let mut change = 0;

    for _ in 0..100 {
        change = selectable_balance(&app).await?;

        if change > 0 {
            break;
        }

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    assert!(change > 0, "the split never confirmed");
    let split_fee = BALANCE - 300 - 50 - change;

    // A split is one standard spend creating a few coins, which costs a few million to a few
    // tens of millions, so at 5 mojos per cost its fee is within these bounds. Ignoring the
    // rate would leave it at 0, and a much higher rate would go over.
    assert!(
        (5 * 1_000_000..5 * 50_000_000).contains(&split_fee),
        "split fee {split_fee}"
    );

    Ok(())
}
