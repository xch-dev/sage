use std::{slice, str::FromStr, time::Duration};

use anyhow::{Result, bail};
use bip39::Mnemonic;
use chia_wallet_sdk::{prelude::*, test::sign_transaction};
use sage_api::{
    Amount, CancelVaultRecovery, FinishVaultRecovery, GenerateMnemonic, GetSyncStatus, GetVault,
    Login, MintVault, RekeyVault, SendXch, StartVaultRecovery, VaultKeySource, VaultRecord,
    VaultRecoveryFinishBlocker, VaultState,
};
use sage_database::VaultMember;
use sage_wallet::spend_start_recovery;
use tokio::time::sleep;

use super::TestApp;

/// An hour, the shortest recovery timelock Sage mints a vault with.
const TIMELOCK: u64 = 60 * 60;

async fn new_mnemonic(app: &TestApp) -> Result<String> {
    Ok(app
        .generate_mnemonic(GenerateMnemonic {
            use_24_words: false,
        })
        .await?
        .mnemonic)
}

fn secret_key(mnemonic: &str) -> Result<SecretKey> {
    Ok(SecretKey::from_seed(
        &Mnemonic::from_str(mnemonic)?.to_seed(""),
    ))
}

async fn wait_for_vault(
    app: &mut TestApp,
    f: impl Fn(&VaultRecord) -> bool,
) -> Result<VaultRecord> {
    let mut last = None;

    for _ in 0..100 {
        while app.events.try_recv().is_ok() {}

        let vault = app.get_vault(GetVault {}).await?.vault;

        if f(&vault) {
            return Ok(vault);
        }

        last = Some(vault);
        sleep(Duration::from_millis(100)).await;
    }

    bail!("timed out waiting for the vault, which is {last:?}");
}

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

/// Starts a recovery of the open vault from elsewhere, with the recovery key, to a custody key that
/// Sage doesn't have. It's sent through the peer, since spending coins in the simulator directly
/// doesn't notify the wallets that subscribed to them.
async fn start_recovery_elsewhere(
    app: &TestApp,
    recovery_key: &SecretKey,
    reveal_memo: bool,
) -> Result<()> {
    let attacker = SecretKey::from_seed(&[42; 32]);

    let (wallet, peer) = {
        let sage = app.sage.lock().await;
        let peer = sage.peer_state.lock().await.acquire_peer();
        (sage.wallet()?, peer.expect("a peer is connected"))
    };

    let (vault, config) = wallet.active_vault().await?;
    let mut post_recovery = config.clone();
    post_recovery.custody.members = vec![VaultMember::bls(attacker.public_key())];

    let mut ctx = SpendContext::new();
    spend_start_recovery(
        &mut ctx,
        &vault,
        &config,
        &post_recovery,
        &[recovery_key.public_key()],
        reveal_memo,
    )?;

    let coin_spends = ctx.take();
    let signature = sign_transaction(&coin_spends, slice::from_ref(recovery_key))?;

    app.sim.lock().await.create_block();

    let ack = peer
        .send_transaction(SpendBundle::new(coin_spends, signature))
        .await?;
    assert_eq!(ack.status, 1, "{:?}", ack.error);

    Ok(())
}

fn is_active(vault: &VaultRecord) -> bool {
    vault.state == VaultState::Active && vault.config.is_some()
}

fn is_recovering(vault: &VaultRecord) -> bool {
    vault.state == VaultState::RecoveryPending && vault.recovery.is_some()
}

/// Mints a vault from a new BLS wallet with the recovery mnemonic, funds it, and logs in to it.
/// Returns the BLS wallet's fingerprint.
async fn mint_vault(app: &mut TestApp, recovery_mnemonic: String) -> Result<u32> {
    let alice = app.setup_bls(10_000).await?;
    wait_for_balance(app, 10_000).await?;

    let response = app
        .mint_vault(MintVault {
            name: "Vault".to_string(),
            emoji: None,
            custody_fingerprint: None,
            recovery_mnemonic: Some(recovery_mnemonic),
            recovery_fingerprint: None,
            recovery_timelock: TIMELOCK,
            fee: Amount::u64(0),
            auto_submit: true,
        })
        .await?;
    let fingerprint = response.fingerprint.expect("the mint was submitted");
    wait_for_balance(app, 10_000 - 1).await?;

    app.login(Login { fingerprint }).await?;
    wait_for_vault(app, is_active).await?;
    let vault_address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;

    app.login(Login { fingerprint: alice }).await?;

    // The simulator confirms transactions at the height of its last block, which the vault wallet
    // has already synced up to, so a new block is made first, as a real chain would.
    app.sim.lock().await.create_block();
    app.send_xch(send(vault_address, 5000)).await?;
    wait_for_balance(app, 10_000 - 1 - 5000).await?;

    app.login(Login { fingerprint }).await?;
    wait_for_balance(app, 5000).await?;

    Ok(alice)
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

fn start(custody_mnemonic: String) -> StartVaultRecovery {
    StartVaultRecovery {
        recovery_mnemonic: None,
        custody: VaultKeySource::Mnemonic {
            mnemonic: custody_mnemonic,
        },
        fee: Amount::u64(0),
        fee_fingerprint: None,
        auto_submit: true,
    }
}

/// The peer simulator doesn't serve block headers, so Sage never syncs block timestamps. The
/// timestamps of the block the recovery was confirmed in, and of the latest block, are stored as if
/// it had, since the countdown only uses block timestamps.
async fn store_timestamps(app: &TestApp, recovery_timestamp: u64) -> Result<()> {
    let vault = app.get_vault(GetVault {}).await?.vault;
    let coin_id = vault.coin_id.expect("the vault is synced");

    let wallet = app.sage.lock().await.wallet()?;
    let coin_id = Bytes32::new(hex::decode(coin_id)?.try_into().expect("a coin id"));
    let height = wallet
        .db
        .vault_coins(wallet.vault_launcher_id().expect("a vault wallet"))
        .await?
        .into_iter()
        .find(|row| row.coin.coin_id() == coin_id)
        .and_then(|row| row.created_height)
        .expect("the vault coin is confirmed");

    let (header_hash, peak, peak_hash, peak_timestamp) = {
        let sim = app.sim.lock().await;
        let peak = sim.height() - 1;
        (
            sim.header_hash_of(height).expect("the block exists"),
            peak,
            sim.header_hash_of(peak).expect("the block exists"),
            sim.next_timestamp() - 1,
        )
    };

    wallet
        .db
        .insert_block(
            height,
            header_hash,
            Some(recovery_timestamp.try_into()?),
            false,
        )
        .await?;

    if peak != height {
        wallet
            .db
            .insert_block(peak, peak_hash, Some(peak_timestamp.try_into()?), false)
            .await?;
    }

    Ok(())
}

#[tokio::test]
async fn test_vault_recovery_cancel_and_finish() -> Result<()> {
    let mut app = TestApp::new().await?;

    let recovery_mnemonic = new_mnemonic(&app).await?;
    mint_vault(&mut app, recovery_mnemonic).await?;
    let original = app.get_vault(GetVault {}).await?.vault;

    // Without submitting, nothing happens.
    let custody_mnemonic = new_mnemonic(&app).await?;
    let preview = app
        .start_vault_recovery(StartVaultRecovery {
            auto_submit: false,
            ..start(custody_mnemonic.clone())
        })
        .await?;
    assert_eq!(preview.recovery_timelock, TIMELOCK);
    let vault_spend = preview.summary.vault.expect("the vault is spent");
    assert!(vault_spend.custody_changed, "{vault_spend:?}");
    assert_eq!(
        app.get_vault(GetVault {}).await?.vault.state,
        VaultState::Active
    );

    // The recovery key was attached when the vault was minted.
    app.start_vault_recovery(start(custody_mnemonic.clone()))
        .await?;

    let vault = wait_for_vault(&mut app, is_recovering).await?;
    let recovery = vault.recovery.expect("recovery is pending");
    assert!(recovery.started_locally);
    assert!(recovery.can_cancel);
    assert_eq!(
        recovery.finish_blocker,
        Some(VaultRecoveryFinishBlocker::Timelock)
    );
    assert!(recovery.post_recovery.is_some());

    // The vault can't be spent while recovery is pending.
    let address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;
    assert!(app.send_xch(send(address, 100)).await.is_err());

    // It can't be finished before the timelock expires.
    let result = app
        .finish_vault_recovery(FinishVaultRecovery {
            fee: Amount::u64(0),
            fee_fingerprint: None,
            auto_submit: true,
        })
        .await;
    assert!(result.is_err(), "{result:?}");

    // The custody cancels it, paying the fee from the vault.
    app.cancel_vault_recovery(CancelVaultRecovery {
        fee: Amount::u64(100),
        auto_submit: true,
    })
    .await?;

    let vault = wait_for_vault(&mut app, |vault| {
        is_active(vault) && vault.custody_hash == original.custody_hash
    })
    .await?;
    assert!(vault.recovery.is_none());
    wait_for_balance(&mut app, 4900).await?;

    // A recovery that's allowed to finish moves the vault to the new custody key.
    app.sim.lock().await.create_block();
    let recovery_timestamp = app.sim.lock().await.next_timestamp();
    app.start_vault_recovery(start(custody_mnemonic.clone()))
        .await?;
    wait_for_vault(&mut app, is_recovering).await?;

    {
        let mut sim = app.sim.lock().await;
        sim.pass_time(TIMELOCK);
        sim.create_block();
    }
    store_timestamps(&app, recovery_timestamp).await?;

    let vault = wait_for_vault(&mut app, |vault| {
        vault
            .recovery
            .as_ref()
            .is_some_and(|recovery| recovery.finish_blocker.is_none())
    })
    .await?;
    assert!(vault.recovery_finishes_at.is_some());

    app.finish_vault_recovery(FinishVaultRecovery {
        fee: Amount::u64(0),
        fee_fingerprint: None,
        auto_submit: true,
    })
    .await?;

    let vault = wait_for_vault(&mut app, |vault| {
        is_active(vault) && vault.custody_hash != original.custody_hash
    })
    .await?;

    // The new custody key is attached, and spends the vault.
    let custody_fingerprint = secret_key(&custody_mnemonic)?
        .public_key()
        .get_fingerprint();
    assert!(
        vault
            .signers
            .iter()
            .any(|signer| signer.fingerprint == custody_fingerprint && signer.custody)
    );

    app.sim.lock().await.create_block();
    let address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;
    app.send_xch(send(address, 100)).await?;
    wait_for_vault(&mut app, |vault| {
        is_active(vault) && vault.coin_id != original.coin_id
    })
    .await?;

    Ok(())
}

/// A recovery started elsewhere is flagged, and can be cancelled.
#[tokio::test]
async fn test_unexpected_vault_recovery() -> Result<()> {
    let mut app = TestApp::new().await?;

    let recovery_mnemonic = new_mnemonic(&app).await?;
    mint_vault(&mut app, recovery_mnemonic.clone()).await?;
    let original = app.get_vault(GetVault {}).await?.vault;

    // Someone with the recovery key starts a recovery to their own key, without a memo.
    start_recovery_elsewhere(&app, &secret_key(&recovery_mnemonic)?, false).await?;

    let vault = wait_for_vault(&mut app, is_recovering).await?;
    let recovery = vault.recovery.expect("recovery is pending");
    assert!(!recovery.started_locally);
    assert!(recovery.can_cancel);
    assert_eq!(
        recovery.finish_blocker,
        Some(VaultRecoveryFinishBlocker::UnknownCustody)
    );

    // It can't be finished, even once the timelock expires.
    let result = app
        .finish_vault_recovery(FinishVaultRecovery {
            fee: Amount::u64(0),
            fee_fingerprint: None,
            auto_submit: true,
        })
        .await;
    assert!(result.is_err(), "{result:?}");

    app.sim.lock().await.create_block();
    app.cancel_vault_recovery(CancelVaultRecovery {
        fee: Amount::u64(0),
        auto_submit: true,
    })
    .await?;

    wait_for_vault(&mut app, |vault| {
        is_active(vault) && vault.custody_hash == original.custody_hash
    })
    .await?;

    Ok(())
}

/// A recovery started elsewhere to a custody that Sage doesn't hold isn't finished, even if its
/// memo reveals it.
#[tokio::test]
async fn test_foreign_vault_recovery_not_finished() -> Result<()> {
    let mut app = TestApp::new().await?;

    let recovery_mnemonic = new_mnemonic(&app).await?;
    mint_vault(&mut app, recovery_mnemonic.clone()).await?;

    start_recovery_elsewhere(&app, &secret_key(&recovery_mnemonic)?, true).await?;

    let vault = wait_for_vault(&mut app, is_recovering).await?;
    let recovery = vault.recovery.expect("recovery is pending");
    assert!(!recovery.started_locally);
    assert_eq!(
        recovery.finish_blocker,
        Some(VaultRecoveryFinishBlocker::NotOurs)
    );

    let result = app
        .finish_vault_recovery(FinishVaultRecovery {
            fee: Amount::u64(0),
            fee_fingerprint: None,
            auto_submit: true,
        })
        .await;
    assert!(result.is_err(), "{result:?}");

    Ok(())
}

#[tokio::test]
async fn test_rekey_vault() -> Result<()> {
    let mut app = TestApp::new().await?;

    let recovery_mnemonic = new_mnemonic(&app).await?;
    mint_vault(&mut app, recovery_mnemonic).await?;
    let original = app.get_vault(GetVault {}).await?.vault;

    // Nothing to change is refused.
    let result = app
        .rekey_vault(RekeyVault {
            custody: None,
            recovery: None,
            recovery_timelock: None,
            fee: Amount::u64(0),
            auto_submit: true,
        })
        .await;
    assert!(result.is_err(), "{result:?}");

    // A timelock that's too short is refused.
    let result = app
        .rekey_vault(RekeyVault {
            custody: None,
            recovery: None,
            recovery_timelock: Some(60),
            fee: Amount::u64(0),
            auto_submit: true,
        })
        .await;
    assert!(result.is_err(), "{result:?}");

    let custody_mnemonic = new_mnemonic(&app).await?;
    let response = app
        .rekey_vault(RekeyVault {
            custody: Some(VaultKeySource::Mnemonic {
                mnemonic: custody_mnemonic.clone(),
            }),
            recovery: None,
            recovery_timelock: Some(2 * TIMELOCK),
            fee: Amount::u64(100),
            auto_submit: true,
        })
        .await?;
    assert!(
        response
            .summary
            .vault
            .is_some_and(|vault| vault.custody_changed)
    );

    let vault = wait_for_vault(&mut app, |vault| {
        is_active(vault) && vault.custody_hash != original.custody_hash
    })
    .await?;
    let config = vault.config.expect("the memo reveals the config");
    assert_eq!(config.recovery_timelock, Some(2 * TIMELOCK));

    let custody_fingerprint = secret_key(&custody_mnemonic)?
        .public_key()
        .get_fingerprint();
    assert_eq!(response.custody_fingerprint, Some(custody_fingerprint));
    assert_eq!(
        config.custody.members[0].signer_fingerprint,
        Some(custody_fingerprint)
    );

    wait_for_balance(&mut app, 4900).await?;

    // The new custody key spends the vault.
    app.sim.lock().await.create_block();
    let address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;
    app.send_xch(send(address, 100)).await?;
    wait_for_vault(&mut app, |vault| {
        is_active(vault) && vault.coin_id != original.coin_id
    })
    .await?;

    Ok(())
}
