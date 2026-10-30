use std::{slice, time::Duration};

use anyhow::{Result, bail};
use bip39::Mnemonic;
use chia_wallet_sdk::{
    driver::{InnerPuzzleSpend, MipsSpend, P2Singleton, Vault, VaultVersion},
    prelude::*,
    test::BlsPair,
    types::{MessageFlags, MessageSide, puzzles::BlsMember},
    utils::Address,
};
use rand::Rng;
use sage_api::{
    AttachVaultSigner, DeleteWallet, GetKeys, GetSyncStatus, GetVault, GetWallet, GetWalletAddress,
    GetWallets, ImportVault, Login, RebuildVaultConfig, SendXch, VaultConfigSource,
    VaultMemberKind, VaultRecord, VaultRecoveryKey, VaultState, WalletKind,
};
use sage_database::{VaultConfig, VaultMember, VaultSigners};
use sage_wallet::{build_vault, vault_hashes};
use tokio::time::sleep;

use super::TestApp;

fn vault_config(custody: PublicKey, recovery: PublicKey) -> VaultConfig {
    VaultConfig {
        custody: VaultSigners::new(vec![VaultMember::bls(custody)], 1),
        recovery: VaultSigners::new(vec![VaultMember::bls(recovery)], 1),
        recovery_timelock: 60,
        version: VaultVersion::ForceSingletonRecreation,
    }
}

/// Mints a vault, with its memo in the launcher solution unless `memo` is false.
async fn mint_vault(app: &TestApp, config: &VaultConfig, memo: bool) -> Result<Vault> {
    let mut sim = app.sim.lock().await;
    let funder = sim.bls(1);

    let mut ctx = SpendContext::new();
    let built = build_vault(&mut ctx, config)?;
    let memos = if memo { built.memo_list } else { NodePtr::NIL };

    let (conditions, vault) = Launcher::new(funder.coin.coin_id(), 1).mint_vault(
        &mut ctx,
        built.hashes.custody_hash,
        memos,
    )?;
    StandardLayer::new(funder.pk).spend(&mut ctx, funder.coin, conditions)?;
    sim.spend_coins(ctx.take(), &[funder.sk])?;

    Ok(vault)
}

/// Spends a coin from the vault's first address, which the vault approves with a message. This
/// reveals the address's puzzle, so the vault can be found from it.
async fn spend_from_address(
    app: &TestApp,
    vault: &Vault,
    config: &VaultConfig,
    custody: &SecretKey,
) -> Result<Vault> {
    let mut sim = app.sim.lock().await;
    let mut ctx = SpendContext::new();

    let p2_singleton = P2Singleton::new(vault.info.launcher_id, 0);
    let coin = sim.new_coin(p2_singleton.tree_hash().into(), 10);

    let delegated_spend =
        ctx.delegated_spend(Conditions::new().create_coin(Bytes32::default(), 10, Memos::None))?;
    let delegated_puzzle_hash = ctx.tree_hash(delegated_spend.puzzle);
    let spend = p2_singleton.spend(
        &mut ctx,
        vault.info.custody_hash.into(),
        vault.coin.amount,
        delegated_spend,
    )?;
    ctx.spend(coin, spend)?;

    let coin_id = ctx.alloc(&coin.coin_id())?;
    let hashes = vault_hashes(config)?;
    let conditions = Conditions::new()
        .create_coin(
            vault.info.custody_hash.into(),
            vault.coin.amount,
            Memos::None,
        )
        .send_message(
            MessageFlags::PUZZLE.encode(MessageSide::Sender)
                | MessageFlags::COIN.encode(MessageSide::Receiver),
            delegated_puzzle_hash.to_vec().into(),
            vec![coin_id],
        );

    let mut spend = MipsSpend::new(ctx.delegated_spend(conditions)?);
    spend.members.insert(
        hashes.custody_hash,
        InnerPuzzleSpend::m_of_n(
            0,
            vec![],
            1,
            vec![hashes.custody_member_hash, hashes.recovery_member_hash],
        ),
    );
    let member = ctx.curry(BlsMember::new(custody.public_key()))?;
    spend.members.insert(
        hashes.custody_member_hash,
        InnerPuzzleSpend::new(0, vec![], Spend::new(member, NodePtr::NIL)),
    );
    vault.spend(&mut ctx, &spend)?;

    sim.spend_coins(ctx.take(), slice::from_ref(custody))?;

    Ok(vault.child(vault.info.custody_hash, vault.coin.amount))
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

fn vault_id(launcher_id: Bytes32) -> String {
    Address::new(launcher_id, "vault".to_string())
        .encode()
        .expect("should encode")
}

fn import(
    launcher_id: Option<Bytes32>,
    address: Option<String>,
    recovery: Option<VaultRecoveryKey>,
) -> ImportVault {
    ImportVault {
        name: "Vault".to_string(),
        launcher_id: launcher_id.map(vault_id),
        address,
        recovery,
        login: true,
        emoji: None,
    }
}

#[tokio::test]
async fn test_import_vault_by_launcher_id() -> Result<()> {
    let mut app = TestApp::new().await?;

    let custody = BlsPair::new(1);
    let recovery = BlsPair::new(2);
    let config = vault_config(custody.pk, recovery.pk);
    let vault = mint_vault(&app, &config, true).await?;
    let launcher_id = vault.info.launcher_id;

    // XCH which was sent to the vault's address before it's imported.
    let address = P2Singleton::new(launcher_id, 0).tree_hash().into();
    app.sim.lock().await.new_coin(address, 500);

    wait_for_peer(&app).await?;

    let response = app
        .import_vault(import(Some(launcher_id), None, None))
        .await?;
    assert_eq!(response.launcher_id, vault_id(launcher_id));

    // The vault's config is decoded from the memo in its launcher's solution.
    let record = wait_for_vault(&mut app, |vault| vault.state == VaultState::Active).await?;
    let record_config = record.config.expect("config should be known");
    assert_eq!(record_config.source, VaultConfigSource::Memo);
    assert_eq!(record_config.custody.members[0].kind, VaultMemberKind::Bls);
    assert_eq!(
        record_config.custody.members[0].key,
        hex::encode(custody.pk.to_bytes())
    );
    assert!(record.signers.is_empty());

    // It receives at its first address, and holds what was sent there.
    let status = app.get_sync_status(GetSyncStatus {}).await?;
    let expected = Address::new(address, "txch".to_string()).encode()?;
    assert_eq!(status.receive_address, expected);
    assert_eq!(
        app.get_wallet_address(GetWalletAddress {
            fingerprint: response.fingerprint,
            network_id: "testnet11".to_string(),
        })
        .await?
        .address,
        expected
    );

    for _ in 0..100 {
        if app
            .get_sync_status(GetSyncStatus {})
            .await?
            .selectable_balance
            .to_u64()
            == Some(500)
        {
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(
        app.get_sync_status(GetSyncStatus {})
            .await?
            .selectable_balance
            .to_u64(),
        Some(500)
    );

    let wallet = app
        .get_wallet(GetWallet { fingerprint: None })
        .await?
        .wallet
        .expect("should be logged in");
    assert_eq!(
        wallet.kind,
        WalletKind::Vault {
            launcher_id: vault_id(launcher_id),
            signers: Vec::new(),
        }
    );

    // Vaults can't spend yet.
    let error = app
        .send_xch(SendXch {
            address: expected,
            amount: sage_api::Amount::u64(100),
            fee: sage_api::Amount::u64(0),
            memos: Vec::new(),
            clawback: None,
            auto_submit: true,
            coin_ids: None,
            confirmed_handles: Vec::new(),
        })
        .await
        .expect_err("vaults can't spend yet");
    assert!(error.to_string().contains("vault"), "{error}");

    // The same vault can't be imported twice.
    let error = app
        .import_vault(import(Some(launcher_id), None, None))
        .await
        .expect_err("the vault is already a wallet");
    assert!(error.to_string().contains("already exists"), "{error}");

    // A launcher id that isn't a vault isn't found.
    let error = app
        .import_vault(import(Some(Bytes32::new([9; 32])), None, None))
        .await
        .expect_err("there's no such vault");
    assert!(error.to_string().contains("No vault"), "{error}");

    Ok(())
}

#[tokio::test]
async fn test_import_vault_by_address() -> Result<()> {
    let mut app = TestApp::new().await?;

    let custody = BlsPair::new(1);
    let config = vault_config(custody.pk, BlsPair::new(2).pk);
    let vault = mint_vault(&app, &config, true).await?;
    let launcher_id = vault.info.launcher_id;

    wait_for_peer(&app).await?;

    let address = Address::new(
        P2Singleton::new(launcher_id, 0).tree_hash().into(),
        "txch".to_string(),
    )
    .encode()?;

    // Until the vault has spent from the address, its puzzle isn't known.
    let error = app
        .import_vault(import(None, Some(address.clone()), None))
        .await
        .expect_err("the address hasn't been spent from");
    assert!(error.to_string().contains("watch the address"), "{error}");

    let vault = spend_from_address(&app, &vault, &config, &custody.sk).await?;

    let response = app.import_vault(import(None, Some(address), None)).await?;
    assert_eq!(response.launcher_id, vault_id(launcher_id));

    let record = wait_for_vault(&mut app, |record| {
        record.coin_id == Some(hex::encode(vault.coin.coin_id()))
    })
    .await?;
    assert_eq!(record.state, VaultState::Active);
    assert!(record.config.is_some());

    Ok(())
}

fn recovery_key(key: String, timelock: Option<u64>) -> VaultRecoveryKey {
    VaultRecoveryKey { key, timelock }
}

#[tokio::test]
async fn test_import_vault_by_recovery_key() -> Result<()> {
    let mut app = TestApp::new().await?;

    let custody = BlsPair::new(1);
    let recovery = BlsPair::new(2);
    let config = vault_config(custody.pk, recovery.pk);
    let recovery_pk = hex::encode(recovery.pk.to_bytes());

    // Without memos on chain, the config is rebuilt from the vault's spends and its recovery key.
    let vault = mint_vault(&app, &config, false).await?;
    let launcher_id = vault.info.launcher_id;

    wait_for_peer(&app).await?;

    // Until the vault is spent through its custody, its custody isn't revealed.
    let error = app
        .import_vault(import(
            Some(launcher_id),
            None,
            Some(recovery_key(recovery_pk.clone(), None)),
        ))
        .await
        .expect_err("the custody hasn't been revealed");
    assert!(error.to_string().contains("hasn't been spent"), "{error}");

    let vault = spend_from_address(&app, &vault, &config, &custody.sk).await?;

    // A key which isn't the recovery member doesn't rebuild the vault.
    let error = app
        .import_vault(import(
            Some(launcher_id),
            None,
            Some(recovery_key(
                hex::encode(BlsPair::new(3).pk.to_bytes()),
                Some(60),
            )),
        ))
        .await
        .expect_err("the recovery key doesn't match");
    assert!(error.to_string().contains("doesn't rebuild"), "{error}");

    let error = app
        .import_vault(import(
            Some(launcher_id),
            None,
            Some(recovery_key("not a key".to_string(), None)),
        ))
        .await
        .expect_err("the recovery key is invalid");
    assert!(
        error.to_string().contains("Invalid recovery key"),
        "{error}"
    );

    // The recovery timelock is found among the common ones.
    app.import_vault(import(
        Some(launcher_id),
        None,
        Some(recovery_key(recovery_pk, None)),
    ))
    .await?;

    let record = wait_for_vault(&mut app, |record| {
        record.coin_id == Some(hex::encode(vault.coin.coin_id()))
    })
    .await?;
    let record_config = record.config.expect("config should be known");
    assert_eq!(record_config.source, VaultConfigSource::Reconstructed);
    assert_eq!(record_config.recovery_timelock, config.recovery_timelock);
    assert_eq!(
        record_config.custody.members[0].key,
        hex::encode(custody.pk.to_bytes())
    );
    assert_eq!(
        record_config
            .recovery
            .expect("recovery should be known")
            .members[0]
            .key,
        hex::encode(recovery.pk.to_bytes())
    );

    Ok(())
}

#[tokio::test]
async fn test_vault_without_config() -> Result<()> {
    let mut app = TestApp::new().await?;

    let custody = BlsPair::new(1);
    let recovery = BlsPair::new(2);
    let config = vault_config(custody.pk, recovery.pk);
    let vault = mint_vault(&app, &config, false).await?;
    let launcher_id = vault.info.launcher_id;

    // A spend through the vault's custody, which reveals it.
    let _vault = spend_from_address(&app, &vault, &config, &custody.sk).await?;

    let address = P2Singleton::new(launcher_id, 0).tree_hash().into();
    app.sim.lock().await.new_coin(address, 250);

    wait_for_peer(&app).await?;

    // A vault without a memo is imported read-only, with its balances.
    let fingerprint = app
        .import_vault(import(Some(launcher_id), None, None))
        .await?
        .fingerprint;

    let record = wait_for_vault(&mut app, |record| record.coin_id.is_some()).await?;
    assert_eq!(record.state, VaultState::Active);
    assert!(record.config.is_none());

    for _ in 0..100 {
        if app
            .get_sync_status(GetSyncStatus {})
            .await?
            .selectable_balance
            .to_u64()
            == Some(250)
        {
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(
        app.get_sync_status(GetSyncStatus {})
            .await?
            .selectable_balance
            .to_u64(),
        Some(250)
    );

    // Signers can't be checked until the config is known.
    let error = app
        .attach_vault_signer(AttachVaultSigner {
            fingerprint,
            key: hex::encode(custody.sk.to_bytes()),
        })
        .await
        .expect_err("the config isn't known");
    assert!(error.to_string().contains("isn't known"), "{error}");

    // Rebuilding it from the vault's spends and its recovery key afterwards fills it in.
    let rebuilt = app
        .rebuild_vault_config(RebuildVaultConfig {
            recovery: VaultRecoveryKey {
                key: hex::encode(recovery.sk.to_bytes()),
                timelock: Some(config.recovery_timelock),
            },
        })
        .await?;
    assert_eq!(rebuilt.recovery_timelock, config.recovery_timelock);

    let record = app.get_vault(GetVault {}).await?.vault;
    assert_eq!(
        record.config.map(|config| config.source),
        Some(VaultConfigSource::Reconstructed)
    );

    Ok(())
}

#[tokio::test]
async fn test_attach_vault_signers() -> Result<()> {
    let mut app = TestApp::new().await?;

    let mnemonic = Mnemonic::from_entropy(&app.rng.r#gen::<[u8; 16]>())?;
    let custody = SecretKey::from_seed(&mnemonic.to_seed(""));
    let recovery = BlsPair::new(2);
    let config = vault_config(custody.public_key(), recovery.pk);
    let vault = mint_vault(&app, &config, true).await?;
    let launcher_id = vault.info.launcher_id;

    wait_for_peer(&app).await?;

    let fingerprint = app
        .import_vault(import(Some(launcher_id), None, None))
        .await?
        .fingerprint;
    wait_for_vault(&mut app, |record| record.config.is_some()).await?;

    // A key which isn't a member of the vault is rejected.
    let error = app
        .attach_vault_signer(AttachVaultSigner {
            fingerprint,
            key: hex::encode(BlsPair::new(3).sk.to_bytes()),
        })
        .await
        .expect_err("the key isn't a member");
    assert!(
        error
            .to_string()
            .contains("isn't a custody or recovery member"),
        "{error}"
    );

    // So is a public key, which can't sign.
    let error = app
        .attach_vault_signer(AttachVaultSigner {
            fingerprint,
            key: hex::encode(custody.public_key().to_bytes()),
        })
        .await
        .expect_err("a public key can't sign");
    assert!(error.to_string().contains("cold"), "{error}");

    // The custody key can be attached from its mnemonic, as Cloud Wallet derives it.
    let attached = app
        .attach_vault_signer(AttachVaultSigner {
            fingerprint,
            key: mnemonic.to_string(),
        })
        .await?;
    assert_eq!(attached.fingerprint, custody.public_key().get_fingerprint());
    assert!(attached.custody);
    assert!(!attached.recovery);

    let attached = app
        .attach_vault_signer(AttachVaultSigner {
            fingerprint,
            key: hex::encode(recovery.sk.to_bytes()),
        })
        .await?;
    assert!(!attached.custody);
    assert!(attached.recovery);

    let record = app.get_vault(GetVault {}).await?.vault;
    assert_eq!(record.signers.len(), 2);
    let record_config = record.config.expect("config should be known");
    assert_eq!(
        record_config.custody.members[0].signer_fingerprint,
        Some(custody.public_key().get_fingerprint())
    );
    assert_eq!(
        record_config
            .recovery
            .expect("recovery should be known")
            .members[0]
            .signer_fingerprint,
        Some(recovery.pk.get_fingerprint())
    );

    let wallets = app.get_wallets(GetWallets {}).await?.wallets;
    let wallet = wallets
        .iter()
        .find(|wallet| wallet.fingerprint == fingerprint)
        .expect("the vault should be listed");
    assert_eq!(
        wallet.kind,
        WalletKind::Vault {
            launcher_id: vault_id(launcher_id),
            signers: vec![
                custody.public_key().get_fingerprint(),
                recovery.pk.get_fingerprint()
            ],
        }
    );

    // Signers aren't wallets of their own.
    assert_eq!(app.get_keys(GetKeys {}).await?.keys.len(), 2);
    assert!(
        app.login(Login {
            fingerprint: recovery.pk.get_fingerprint(),
        })
        .await
        .is_err()
    );

    // Deleting the vault deletes its signers' keys, since nothing else uses them.
    app.delete_wallet(DeleteWallet {
        fingerprint,
        delete_keys: true,
    })
    .await?;
    assert!(app.get_keys(GetKeys {}).await?.keys.is_empty());

    Ok(())
}
