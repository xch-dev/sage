use std::time::Duration;

use anyhow::{Result, bail};
use chia_wallet_sdk::prelude::*;
use sage_api::{
    Amount, GenerateMnemonic, GetKeys, GetSyncStatus, GetVault, GetWallets, Login, MintVault,
    SendXch, VaultRecord, VaultState, WalletKind,
};
use sage_config::{PendingVaultMint, Wallet as WalletEntry, WalletKind as ConfigWalletKind};
use tokio::time::sleep;

use super::TestApp;

/// Twelve hours, the default recovery timelock.
const TIMELOCK: u64 = 12 * 60 * 60;

async fn recovery_mnemonic(app: &TestApp) -> Result<String> {
    Ok(app
        .generate_mnemonic(GenerateMnemonic {
            use_24_words: false,
        })
        .await?
        .mnemonic)
}

fn mint(recovery_mnemonic: String, auto_submit: bool) -> MintVault {
    MintVault {
        name: "Vault".to_string(),
        emoji: None,
        custody_fingerprint: None,
        recovery_mnemonic: Some(recovery_mnemonic),
        recovery_fingerprint: None,
        recovery_timelock: TIMELOCK,
        fee: Amount::u64(10),
        auto_submit,
    }
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

#[tokio::test]
async fn test_mint_vault() -> Result<()> {
    let mut app = TestApp::new().await?;

    let alice = app.setup_bls(10_000).await?;
    wait_for_balance(&mut app, 10_000).await?;
    let alice_address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;

    let mnemonic = recovery_mnemonic(&app).await?;

    // Without submitting, nothing is stored.
    let preview = app.mint_vault(mint(mnemonic.clone(), false)).await?;
    assert_eq!(preview.fingerprint, None);
    assert_eq!(preview.summary.fee.to_u64(), Some(10));
    assert_eq!(app.get_wallets(GetWallets {}).await?.wallets.len(), 1);
    assert_eq!(app.get_keys(GetKeys {}).await?.keys.len(), 1);

    let response = app.mint_vault(mint(mnemonic.clone(), true)).await?;
    let fingerprint = response.fingerprint.expect("the mint was submitted");

    // The vault wallet is added with its custody and recovery keys as signers.
    let wallets = app.get_wallets(GetWallets {}).await?.wallets;
    let vault_wallet = wallets
        .iter()
        .find(|wallet| wallet.fingerprint == fingerprint)
        .expect("the vault wallet is added");
    let WalletKind::Vault {
        launcher_id,
        signers,
    } = &vault_wallet.kind
    else {
        bail!("expected a vault wallet, got {:?}", vault_wallet.kind);
    };
    assert_eq!(
        signers,
        &[response.custody_fingerprint, response.recovery_fingerprint]
    );
    assert_eq!(launcher_id, &response.launcher_id);
    assert_eq!(app.get_keys(GetKeys {}).await?.keys.len(), 3);

    wait_for_balance(&mut app, 10_000 - 1 - 10).await?;

    // The custody key is derived from alice's key, so minting again uses a different one.
    let second = app.mint_vault(mint(mnemonic, false)).await?;
    assert_ne!(second.custody_fingerprint, response.custody_fingerprint);

    app.login(Login { fingerprint }).await?;

    let vault = wait_for_vault(&mut app, |vault| {
        vault.state == VaultState::Active && vault.config.is_some() && !vault.minting
    })
    .await?;
    assert!(vault.signers.iter().any(|signer| signer.custody));
    assert!(vault.signers.iter().any(|signer| signer.recovery));
    assert_eq!(
        vault.config.and_then(|config| config.recovery_timelock),
        Some(TIMELOCK)
    );

    let vault_address = app.get_sync_status(GetSyncStatus {}).await?.receive_address;

    app.login(Login { fingerprint: alice }).await?;

    // The simulator confirms transactions at the height of its last block, which the vault wallet
    // has already synced up to, so a new block is made first, as a real chain would.
    app.sim.lock().await.create_block();

    app.send_xch(send(vault_address, 5000)).await?;
    wait_for_balance(&mut app, 10_000 - 1 - 10 - 5000).await?;

    app.login(Login { fingerprint }).await?;
    wait_for_balance(&mut app, 5000).await?;

    let response = app.send_xch(send(alice_address, 2000)).await?;

    // The summary shows the vault's spend, which doesn't change its custody.
    let vault_spend = response.summary.vault.expect("the vault is spent");
    assert!(vault_spend.verified);
    assert!(!vault_spend.custody_changed);
    assert!(!vault_spend.melted);

    wait_for_balance(&mut app, 3000).await?;

    Ok(())
}

#[tokio::test]
async fn test_mint_vault_rejects_bad_keys() -> Result<()> {
    let mut app = TestApp::new().await?;

    let alice = app.setup_bls(10_000).await?;
    wait_for_balance(&mut app, 10_000).await?;

    let mnemonic = recovery_mnemonic(&app).await?;

    let too_short = MintVault {
        recovery_timelock: 60,
        ..mint(mnemonic.clone(), true)
    };
    assert!(app.mint_vault(too_short).await.is_err());

    let no_recovery = MintVault {
        recovery_mnemonic: None,
        ..mint(mnemonic.clone(), true)
    };
    assert!(app.mint_vault(no_recovery).await.is_err());

    // Alice's own key can't be both the custody and the recovery key.
    let same_key = MintVault {
        custody_fingerprint: Some(alice),
        recovery_mnemonic: None,
        recovery_fingerprint: Some(alice),
        ..mint(mnemonic, true)
    };
    assert!(app.mint_vault(same_key).await.is_err());

    // Nothing was stored by the failed mints.
    assert_eq!(app.get_wallets(GetWallets {}).await?.wallets.len(), 1);
    assert_eq!(app.get_keys(GetKeys {}).await?.keys.len(), 1);

    Ok(())
}

/// Adds a vault wallet whose mint hasn't been found on chain, as if Sage had closed after
/// submitting it.
async fn add_pending_mint(
    app: &TestApp,
    funding_fingerprint: u32,
    fingerprint: u32,
    expires_at: u64,
) -> Result<()> {
    let mut sage = app.sage.lock().await;

    sage.wallet_config.wallets.push(WalletEntry {
        name: "Pending".to_string(),
        fingerprint,
        kind: ConfigWalletKind::Vault {
            launcher_id: Bytes32::new([fingerprint.to_be_bytes()[3]; 32]),
            signers: Vec::new(),
        },
        vault_mint: Some(PendingVaultMint {
            funding_fingerprint,
            transaction_id: Bytes32::new([9; 32]),
            expires_at: Some(expires_at),
        }),
        ..Default::default()
    });

    sage.save_config()?;

    Ok(())
}

async fn wait_for_peer(app: &TestApp) -> Result<()> {
    for _ in 0..100 {
        let peer_state = app.sage.lock().await.peer_state.clone();

        if peer_state.lock().await.acquire_peer().is_some() {
            return Ok(());
        }

        sleep(Duration::from_millis(100)).await;
    }

    bail!("no peer connected");
}

#[tokio::test]
async fn test_pending_vault_mint() -> Result<()> {
    let mut app = TestApp::new().await?;

    let alice = app.setup_bls(0).await?;
    wait_for_peer(&app).await?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();

    // A mint that can still be confirmed is kept, and shown as minting.
    add_pending_mint(&app, alice, 100, now + 3600).await?;
    app.login(Login { fingerprint: 100 }).await?;

    let vault = app.get_vault(GetVault {}).await?.vault;
    assert!(vault.minting);
    assert_eq!(vault.state, VaultState::Unsynced);

    // A mint whose transaction expired without the vault appearing on chain failed, so the
    // wallet is removed when it's opened.
    add_pending_mint(&app, alice, 101, 1).await?;
    app.login(Login { fingerprint: 101 }).await?;

    let wallets = app.get_wallets(GetWallets {}).await?.wallets;
    assert!(wallets.iter().any(|wallet| wallet.fingerprint == 100));
    assert!(!wallets.iter().any(|wallet| wallet.fingerprint == 101));

    Ok(())
}
