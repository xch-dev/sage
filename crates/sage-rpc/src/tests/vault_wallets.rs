use std::{slice, time::Duration};

use anyhow::{Result, bail};
use bip39::Mnemonic;
use chia_wallet_sdk::{
    driver::{InnerPuzzleSpend, MipsSpend, P2Singleton, Vault, VaultVersion},
    prelude::*,
    test::BlsPair,
    types::{
        MessageFlags, MessageSide, Mod,
        puzzles::{
            BlsMember, Force1of2RestrictedVariable, Force1of2RestrictedVariableSolution,
            ForceSingletonRecreationMod, PreventConditionOpcode, PreventMultipleCreateCoinsMod,
            Timelock,
        },
    },
    utils::Address,
};
use rand::Rng;
use sage_api::{
    AttachVaultSigner, DeleteKey, DeleteWallet, ExportVaultConfig, GetKeys, GetSyncStatus,
    GetVault, GetWallet, GetWalletAddress, GetWallets, ImportVault, ImportVaultConfig, Login,
    RebuildVaultConfig, SendXch, VaultConfigSource, VaultMemberKind, VaultRecord,
    VaultRecoveryKeys, VaultState, WalletKind,
};
use sage_database::{VaultConfig, VaultMember, VaultRecoveryConfig, VaultSigners};
use sage_wallet::{
    build_vault, build_vault_recovery, parse_vault_config_file, recovery_restrictions,
    vault_config_file, vault_hashes,
};
use tokio::time::sleep;

use super::TestApp;

fn vault_config(custody: PublicKey, recovery: PublicKey) -> VaultConfig {
    VaultConfig::with_recovery(
        VaultSigners::new(vec![VaultMember::bls(custody)], 1),
        VaultSigners::new(vec![VaultMember::bls(recovery)], 1),
        60,
        VaultVersion::ForceSingletonRecreation,
    )
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
    spend_vault_to(app, vault, config, custody, vault.info.custody_hash.into()).await
}

/// Like [`spend_from_address`], but recreates the vault with a new custody hash, without a memo.
async fn spend_vault_to(
    app: &TestApp,
    vault: &Vault,
    config: &VaultConfig,
    custody: &SecretKey,
    new_custody_hash: Bytes32,
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
        .create_coin(new_custody_hash, vault.coin.amount, Memos::None)
        .send_message(
            MessageFlags::PUZZLE.encode(MessageSide::Sender)
                | MessageFlags::COIN.encode(MessageSide::Receiver),
            delegated_puzzle_hash.to_vec().into(),
            vec![coin_id],
        );

    let mut spend = MipsSpend::new(ctx.delegated_spend(conditions)?);
    let member = ctx.curry(BlsMember::new(custody.public_key()))?;
    let member = InnerPuzzleSpend::new(0, vec![], Spend::new(member, NodePtr::NIL));

    // A vault without recovery is spent through its custody at the top level.
    if let Some(recovery_member_hash) = hashes.recovery_member_hash {
        spend.members.insert(
            hashes.custody_hash,
            InnerPuzzleSpend::m_of_n(
                0,
                vec![],
                1,
                vec![hashes.custody_member_hash, recovery_member_hash],
            ),
        );
        spend.members.insert(hashes.custody_member_hash, member);
    } else {
        spend.members.insert(hashes.custody_hash, member);
    }
    vault.spend(&mut ctx, &spend)?;

    sim.spend_coins(ctx.take(), slice::from_ref(custody))?;

    Ok(vault.child(new_custody_hash.into(), vault.coin.amount))
}

/// Starts recovery with the recovery key, moving the vault to the recovery state, which moves it to
/// `post_recovery` once the timelock expires.
async fn start_recovery(
    app: &TestApp,
    vault: &Vault,
    config: &VaultConfig,
    recovery: &SecretKey,
    post_recovery: &VaultConfig,
) -> Result<()> {
    let mut sim = app.sim.lock().await;
    let mut ctx = SpendContext::new();
    let hashes = vault_hashes(config)?;
    let recovery_side = config.recovery.as_ref().expect("the vault has recovery");
    let recovery_member_hash = hashes.recovery_member_hash.expect("the vault has recovery");

    let built = build_vault_recovery(
        &mut ctx,
        &VaultRecoveryConfig {
            custody: config.custody.clone(),
            recovery_timelock: recovery_side.timelock,
            version: recovery_side.version,
            recovery_custody_hash: vault_hashes(post_recovery)?.custody_hash.into(),
            amount: vault.coin.amount,
        },
        post_recovery,
    )?;

    let conditions = Conditions::new()
        .create_coin(
            built.hashes.custody_hash.into(),
            vault.coin.amount,
            Memos::Some(built.memo_list),
        )
        .assert_my_amount(vault.coin.amount);
    let mut spend = MipsSpend::new(ctx.delegated_spend(conditions)?);

    spend.members.insert(
        hashes.custody_hash,
        InnerPuzzleSpend::m_of_n(
            0,
            vec![],
            1,
            vec![hashes.custody_member_hash, recovery_member_hash],
        ),
    );
    let member = ctx.curry(BlsMember::new(recovery.public_key()))?;
    spend.members.insert(
        recovery_member_hash,
        InnerPuzzleSpend::new(
            0,
            recovery_restrictions(
                hashes.custody_member_hash,
                recovery_side.timelock,
                recovery_side.version,
            ),
            Spend::new(member, NodePtr::NIL),
        ),
    );

    // The recovery path's delegated puzzle wrappers, in order.
    let force_1_of_2 = Force1of2RestrictedVariable::new(
        hashes.custody_member_hash.into(),
        0,
        vec![Timelock::new(recovery_side.timelock).curry_tree_hash()]
            .tree_hash()
            .into(),
        ().tree_hash().into(),
    );
    let puzzle = ctx.curry(force_1_of_2)?;
    let finish_puzzle = built
        .recovery_finish_puzzle
        .expect("a recovery state has a finish puzzle");
    let finish_puzzle_hash = ctx.tree_hash(finish_puzzle).into();
    let solution = ctx.alloc(&Force1of2RestrictedVariableSolution::new(
        finish_puzzle_hash,
    ))?;
    spend
        .restrictions
        .insert(force_1_of_2.curry_tree_hash(), Spend::new(puzzle, solution));

    for opcode in [60, 62, 66, 67] {
        let wrapper = PreventConditionOpcode::new(opcode);
        let puzzle = ctx.curry(wrapper)?;
        spend
            .restrictions
            .insert(wrapper.curry_tree_hash(), Spend::new(puzzle, NodePtr::NIL));
    }

    let puzzle = ctx.alloc_mod::<PreventMultipleCreateCoinsMod>()?;
    spend.restrictions.insert(
        PreventMultipleCreateCoinsMod::mod_hash(),
        Spend::new(puzzle, NodePtr::NIL),
    );

    if recovery_side.version == VaultVersion::ForceSingletonRecreation {
        let puzzle = ctx.alloc_mod::<ForceSingletonRecreationMod>()?;
        spend.restrictions.insert(
            ForceSingletonRecreationMod::mod_hash(),
            Spend::new(puzzle, NodePtr::NIL),
        );
    }

    vault.spend(&mut ctx, &spend)?;
    sim.spend_coins(ctx.take(), slice::from_ref(recovery))?;

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
    recovery: Option<VaultRecoveryKeys>,
) -> ImportVault {
    ImportVault {
        name: "Vault".to_string(),
        launcher_id: launcher_id.map(vault_id),
        address,
        config: None,
        recovery,
        login: true,
        emoji: None,
    }
}

fn import_config(config: String) -> ImportVault {
    ImportVault {
        config: Some(config),
        ..import(None, None, None)
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

fn recovery_key(key: String, timelock: Option<u64>) -> VaultRecoveryKeys {
    VaultRecoveryKeys {
        keys: vec![key],
        threshold: None,
        timelock,
    }
}

#[tokio::test]
async fn test_import_vault_by_config() -> Result<()> {
    let mut app = TestApp::new().await?;

    let custody = BlsPair::new(1);
    let config = vault_config(custody.pk, BlsPair::new(2).pk);

    // Without a memo on chain, the config file is one way to know the vault's config.
    let vault = mint_vault(&app, &config, false).await?;
    let launcher_id = vault.info.launcher_id;

    wait_for_peer(&app).await?;

    // A config which doesn't rebuild the vault on chain is rejected.
    let wrong = vault_config(BlsPair::new(3).pk, BlsPair::new(2).pk);
    let error = app
        .import_vault(import_config(vault_config_file(launcher_id, &wrong)?))
        .await
        .expect_err("the config doesn't match");
    assert!(error.to_string().contains("doesn't match"), "{error}");

    let mut file: serde_json::Value =
        serde_json::from_str(&vault_config_file(launcher_id, &config)?)?;

    // Files without a puzzle version match either version.
    file.as_object_mut()
        .expect("the file is an object")
        .remove("vaultPuzzleVersion");

    // A recovery key can't be given with a config file.
    let error = app
        .import_vault(ImportVault {
            recovery: Some(recovery_key(
                hex::encode(BlsPair::new(2).pk.to_bytes()),
                None,
            )),
            ..import_config(file.to_string())
        })
        .await
        .expect_err("a recovery key can't be given with a config file");
    assert!(error.to_string().contains("config file"), "{error}");

    app.import_vault(import_config(file.to_string())).await?;

    let record = wait_for_vault(&mut app, |record| record.coin_id.is_some()).await?;
    let record_config = record.config.expect("config should be known");
    assert_eq!(record_config.source, VaultConfigSource::ConfigFile);

    // The config can be exported again, in the same format.
    let exported = app.export_vault_config(ExportVaultConfig {}).await?.config;
    let parsed = parse_vault_config_file(&exported)?;
    assert_eq!(parsed.launcher_id, launcher_id);
    assert_eq!(parsed.candidates, [config]);

    Ok(())
}

#[tokio::test]
async fn test_import_vault_config_later() -> Result<()> {
    let mut app = TestApp::new().await?;

    let config = vault_config(BlsPair::new(1).pk, BlsPair::new(2).pk);
    let vault = mint_vault(&app, &config, false).await?;
    let launcher_id = vault.info.launcher_id;

    wait_for_peer(&app).await?;

    app.import_vault(import(Some(launcher_id), None, None))
        .await?;
    let record = wait_for_vault(&mut app, |record| record.coin_id.is_some()).await?;
    assert!(record.config.is_none());

    // A config file for another vault is rejected.
    let error = app
        .import_vault_config(ImportVaultConfig {
            config: vault_config_file(Bytes32::new([9; 32]), &config)?,
        })
        .await
        .expect_err("the config is for another vault");
    assert!(error.to_string().contains("doesn't match"), "{error}");

    // Importing its config file afterwards fills it in.
    app.import_vault_config(ImportVaultConfig {
        config: vault_config_file(launcher_id, &config)?,
    })
    .await?;

    let record = app.get_vault(GetVault {}).await?.vault;
    assert_eq!(
        record.config.map(|config| config.source),
        Some(VaultConfigSource::ConfigFile)
    );

    Ok(())
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
    assert!(error.to_string().contains("don't rebuild"), "{error}");

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
    assert_eq!(record_config.recovery_timelock, Some(60));
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
async fn test_import_vault_with_recovery_keys() -> Result<()> {
    let mut app = TestApp::new().await?;

    let custody = BlsPair::new(1);
    let recovery = [BlsPair::new(2), BlsPair::new(3)];
    let config = VaultConfig::with_recovery(
        VaultSigners::new(vec![VaultMember::bls(custody.pk)], 1),
        VaultSigners::new(
            recovery
                .iter()
                .map(|key| VaultMember::bls(key.pk))
                .collect(),
            2,
        ),
        3600,
        VaultVersion::Legacy,
    );

    let vault = mint_vault(&app, &config, false).await?;
    let launcher_id = vault.info.launcher_id;
    let _vault = spend_from_address(&app, &vault, &config, &custody.sk).await?;

    wait_for_peer(&app).await?;

    let keys = vec![
        hex::encode(recovery[1].pk.to_bytes()),
        hex::encode(recovery[0].sk.to_bytes()),
    ];

    // A threshold that's more than the number of keys is rejected.
    let error = app
        .import_vault(import(
            Some(launcher_id),
            None,
            Some(VaultRecoveryKeys {
                keys: keys.clone(),
                threshold: Some(3),
                timelock: None,
            }),
        ))
        .await
        .expect_err("the threshold is invalid");
    assert!(error.to_string().contains("threshold"), "{error}");

    // The threshold and timelock are both found.
    app.import_vault(import(
        Some(launcher_id),
        None,
        Some(VaultRecoveryKeys {
            keys,
            threshold: None,
            timelock: None,
        }),
    ))
    .await?;

    let record = wait_for_vault(&mut app, |record| record.coin_id.is_some()).await?;
    let record_config = record.config.expect("config should be known");
    assert_eq!(record_config.recovery_timelock, Some(3600));
    assert_eq!(
        record_config.version,
        Some(sage_api::VaultPuzzleVersion::Legacy)
    );

    let recovery_record = record_config.recovery.expect("recovery should be known");
    assert_eq!(recovery_record.threshold, 2);
    assert_eq!(recovery_record.members.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_import_vault_without_recovery() -> Result<()> {
    let mut app = TestApp::new().await?;

    let custody = BlsPair::new(1);
    let config =
        VaultConfig::without_recovery(VaultSigners::new(vec![VaultMember::bls(custody.pk)], 1));

    let vault = mint_vault(&app, &config, false).await?;
    let launcher_id = vault.info.launcher_id;
    let _vault = spend_from_address(&app, &vault, &config, &custody.sk).await?;

    wait_for_peer(&app).await?;

    // A vault without recovery is rebuilt from its spends alone, without memos or recovery keys.
    let fingerprint = app
        .import_vault(import(Some(launcher_id), None, None))
        .await?
        .fingerprint;

    let record = wait_for_vault(&mut app, |record| record.coin_id.is_some()).await?;
    let record_config = record.config.expect("config should be known");
    assert_eq!(record_config.source, VaultConfigSource::Reconstructed);
    assert!(record_config.recovery.is_none());
    assert_eq!(record_config.recovery_timelock, None);
    assert_eq!(record_config.version, None);

    // It has no recovery signers, only custody ones.
    let attached = app
        .attach_vault_signer(AttachVaultSigner {
            fingerprint,
            key: hex::encode(custody.sk.to_bytes()),
        })
        .await?;
    assert!(attached.custody);
    assert!(!attached.recovery);

    // Rebuilding it again doesn't need recovery keys either.
    let rebuilt = app
        .rebuild_vault_config(RebuildVaultConfig { recovery: None })
        .await?;
    assert_eq!(rebuilt.recovery_timelock, None);
    assert_eq!(rebuilt.recovery_threshold, None);

    // It's exported without recovery.
    let exported = app.export_vault_config(ExportVaultConfig {}).await?.config;
    assert_eq!(parse_vault_config_file(&exported)?.candidates, [config]);

    Ok(())
}

/// After a rekey without memos, the old config no longer applies, so it isn't used for the new
/// custody.
#[tokio::test]
async fn test_rebuild_after_rekey() -> Result<()> {
    let mut app = TestApp::new().await?;

    let old_custody = BlsPair::new(1);
    let new_custody = BlsPair::new(3);
    let recovery = BlsPair::new(2);
    let old_config = vault_config(old_custody.pk, recovery.pk);
    let new_config = vault_config(new_custody.pk, recovery.pk);

    let vault = mint_vault(&app, &old_config, false).await?;
    let launcher_id = vault.info.launcher_id;
    let vault = spend_from_address(&app, &vault, &old_config, &old_custody.sk).await?;
    let vault = spend_vault_to(
        &app,
        &vault,
        &old_config,
        &old_custody.sk,
        vault_hashes(&new_config)?.custody_hash.into(),
    )
    .await?;

    wait_for_peer(&app).await?;

    // Only the old custody has been spent through, so only its config can be rebuilt. The keys
    // don't rebuild the current one, so importing with them says why.
    let recovery_keys = || Some(recovery_key(hex::encode(recovery.pk.to_bytes()), Some(60)));
    let error = app
        .import_vault(import(Some(launcher_id), None, recovery_keys()))
        .await
        .expect_err("the current custody hasn't been spent through");
    assert!(error.to_string().contains("hasn't been spent"), "{error}");

    // Without them, the vault is imported with its current config unknown.
    let fingerprint = app
        .import_vault(import(Some(launcher_id), None, None))
        .await?
        .fingerprint;

    let record = wait_for_vault(&mut app, |record| {
        record.coin_id == Some(hex::encode(vault.coin.coin_id()))
    })
    .await?;
    assert!(record.config.is_none());

    // The old custody key isn't a member anymore.
    let error = app
        .attach_vault_signer(AttachVaultSigner {
            fingerprint,
            key: hex::encode(old_custody.sk.to_bytes()),
        })
        .await
        .expect_err("the current config isn't known");
    assert!(error.to_string().contains("isn't known"), "{error}");

    let error = app
        .export_vault_config(ExportVaultConfig {})
        .await
        .expect_err("the current config isn't known");
    assert!(error.to_string().contains("isn't known"), "{error}");

    let error = app
        .rebuild_vault_config(RebuildVaultConfig {
            recovery: recovery_keys(),
        })
        .await
        .expect_err("the current custody hasn't been spent through");
    assert!(error.to_string().contains("hasn't been spent"), "{error}");

    // Once the new custody has been spent through, it's rebuilt.
    let _vault = spend_from_address(&app, &vault, &new_config, &new_custody.sk).await?;

    let rebuilt = app
        .rebuild_vault_config(RebuildVaultConfig {
            recovery: recovery_keys(),
        })
        .await?;
    assert_eq!(rebuilt.recovery_timelock, Some(60));

    let record = app.get_vault(GetVault {}).await?.vault;
    assert_eq!(
        record
            .config
            .map(|config| config.custody.members[0].key.clone()),
        Some(hex::encode(new_custody.pk.to_bytes()))
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

    let error = app
        .export_vault_config(ExportVaultConfig {})
        .await
        .expect_err("the config isn't known");
    assert!(error.to_string().contains("isn't known"), "{error}");

    // Its spends alone don't rebuild a vault with recovery.
    let error = app
        .rebuild_vault_config(RebuildVaultConfig { recovery: None })
        .await
        .expect_err("the recovery keys are needed");
    assert!(error.to_string().contains("recovery keys"), "{error}");

    // Rebuilding it from the vault's spends and its recovery key afterwards fills it in.
    let rebuilt = app
        .rebuild_vault_config(RebuildVaultConfig {
            recovery: Some(VaultRecoveryKeys {
                keys: vec![hex::encode(recovery.sk.to_bytes())],
                threshold: None,
                timelock: Some(60),
            }),
        })
        .await?;
    assert_eq!(rebuilt.recovery_timelock, Some(60));
    assert_eq!(rebuilt.recovery_threshold, Some(1));

    let record = app.get_vault(GetVault {}).await?.vault;
    assert_eq!(
        record.config.map(|config| config.source),
        Some(VaultConfigSource::Reconstructed)
    );

    // The rebuilt config can be exported as a config file.
    let exported = app.export_vault_config(ExportVaultConfig {}).await?.config;
    assert_eq!(parse_vault_config_file(&exported)?.candidates, [config]);

    Ok(())
}

/// While recovery is pending, the vault's config is the one from before it started, whose custody
/// can cancel it and whose recovery members started it.
#[tokio::test]
async fn test_vault_recovery_pending() -> Result<()> {
    let mut app = TestApp::new().await?;

    let custody = BlsPair::new(1);
    let recovery = BlsPair::new(2);
    let config = vault_config(custody.pk, recovery.pk);
    let vault = mint_vault(&app, &config, true).await?;
    let launcher_id = vault.info.launcher_id;

    wait_for_peer(&app).await?;

    let fingerprint = app
        .import_vault(import(Some(launcher_id), None, None))
        .await?
        .fingerprint;
    wait_for_vault(&mut app, |record| record.config.is_some()).await?;
    let exported = app.export_vault_config(ExportVaultConfig {}).await?.config;

    let post_recovery = vault_config(recovery.pk, custody.pk);
    start_recovery(&app, &vault, &config, &recovery.sk, &post_recovery).await?;

    let record = wait_for_vault(&mut app, |record| {
        record.state == VaultState::RecoveryPending
    })
    .await?;
    let record_config = record
        .config
        .expect("the config from before recovery is known");
    assert_eq!(
        record_config.custody.members[0].key,
        hex::encode(custody.pk.to_bytes())
    );
    if let (Some(started_at), Some(finishes_at)) =
        (record.recovery_started_at, record.recovery_finishes_at)
    {
        assert_eq!(finishes_at, started_at + 60);
    }

    // The config from before recovery is exported, and its keys can be attached in their roles.
    assert_eq!(
        app.export_vault_config(ExportVaultConfig {}).await?.config,
        exported
    );

    let attached = app
        .attach_vault_signer(AttachVaultSigner {
            fingerprint,
            key: hex::encode(custody.sk.to_bytes()),
        })
        .await?;
    assert!(attached.custody && !attached.recovery);

    let attached = app
        .attach_vault_signer(AttachVaultSigner {
            fingerprint,
            key: hex::encode(recovery.sk.to_bytes()),
        })
        .await?;
    assert!(!attached.custody && attached.recovery);

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

    // A signer's key is in use, so it can't be deleted on its own.
    let error = app
        .delete_key(DeleteKey {
            fingerprint: custody.public_key().get_fingerprint(),
        })
        .await
        .expect_err("the key is used by the vault");
    assert!(error.to_string().contains("used by a wallet"), "{error}");

    // Another vault with the same recovery key, which is attached to it too.
    let other_config = vault_config(BlsPair::new(3).pk, recovery.pk);
    let other = mint_vault(&app, &other_config, true).await?;
    let other_fingerprint = app
        .import_vault(import(Some(other.info.launcher_id), None, None))
        .await?
        .fingerprint;
    wait_for_vault(&mut app, |record| record.config.is_some()).await?;
    app.attach_vault_signer(AttachVaultSigner {
        fingerprint: other_fingerprint,
        key: hex::encode(recovery.sk.to_bytes()),
    })
    .await?;

    // Deleting the first vault deletes the keys nothing else uses, and keeps the shared one.
    app.delete_wallet(DeleteWallet {
        fingerprint,
        delete_keys: true,
    })
    .await?;
    let keys = app.get_keys(GetKeys {}).await?.keys;
    assert_eq!(
        keys.iter().map(|key| key.fingerprint).collect::<Vec<_>>(),
        [recovery.pk.get_fingerprint()]
    );

    // Once nothing uses it, it goes too.
    app.delete_wallet(DeleteWallet {
        fingerprint: other_fingerprint,
        delete_keys: true,
    })
    .await?;
    assert!(app.get_keys(GetKeys {}).await?.keys.is_empty());

    Ok(())
}
