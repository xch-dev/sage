use anyhow::Result;
use bip39::Mnemonic;
use chia_wallet_sdk::{driver::P2Singleton, prelude::*, utils::Address};
use sage::Error;
use sage_api::{
    DeleteWallet, GetDerivations, GetKey, GetKeys, GetSyncStatus, GetWallet, GetWalletAddress,
    GetWallets, ImportKey, IncreaseDerivationIndex, Login, RenameWallet, Resync, SetWalletEmoji,
    WalletKind,
};
use sage_wallet::SyncEvent;
use serde_json::{Value, json};

use super::TestApp;

const LAUNCHER_ID: Bytes32 = Bytes32::new([1; 32]);
const WATCHED: [Bytes32; 2] = [Bytes32::new([2; 32]), Bytes32::new([3; 32])];

fn txch(p2_puzzle_hash: Bytes32) -> String {
    Address::new(p2_puzzle_hash, "txch".to_string())
        .encode()
        .expect("valid address")
}

async fn add_vault(app: &TestApp) -> Result<u32> {
    Ok(app
        .sage
        .lock()
        .await
        .add_vault_wallet(LAUNCHER_ID, "Vault".to_string(), None)?)
}

async fn add_watch(app: &TestApp) -> Result<u32> {
    Ok(app
        .sage
        .lock()
        .await
        .add_watch_wallet(&WATCHED, "Watch".to_string(), None)?)
}

async fn login(app: &mut TestApp, fingerprint: u32) -> Result<()> {
    app.login(Login { fingerprint }).await?;
    app.consume_until(|event| matches!(event, SyncEvent::Subscribed))
        .await;
    Ok(())
}

#[tokio::test]
async fn lists_every_wallet_kind() -> Result<()> {
    let mut app = TestApp::new().await?;

    let bls = app.setup_bls(0).await?;
    let vault = add_vault(&app).await?;
    let watch = add_watch(&app).await?;

    let wallets = app.get_wallets(GetWallets {}).await?.wallets;
    let kinds: Vec<(u32, WalletKind)> = wallets
        .iter()
        .map(|wallet| (wallet.fingerprint, wallet.kind.clone()))
        .collect();

    assert_eq!(
        kinds,
        [
            (bls, WalletKind::Bls),
            (
                vault,
                WalletKind::Vault {
                    launcher_id: hex::encode(LAUNCHER_ID)
                },
            ),
            (
                watch,
                WalletKind::Watch {
                    addresses: WATCHED.into_iter().map(txch).collect()
                },
            ),
        ]
    );

    // Only the BLS wallet has a key in the keychain.
    let keys = app.get_keys(GetKeys {}).await?.keys;
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].fingerprint, bls);
    assert!(keys[0].has_secrets);
    for fingerprint in [vault, watch] {
        assert!(app.get_key(GetKey { fingerprint }).await?.key.is_none());
    }

    // BLS wallets keep serializing their kind as a plain string.
    let raw: Value = app.call_rpc("/get_wallets", GetWallets {}).await?;
    assert_eq!(raw["wallets"][0]["kind"], json!("bls"));
    assert_eq!(
        raw["wallets"][1]["kind"],
        json!({ "vault": { "launcher_id": hex::encode(LAUNCHER_ID) } })
    );

    // Looking up a wallet other than the open one describes that wallet.
    let wallet = app
        .get_wallet(GetWallet {
            fingerprint: Some(vault),
        })
        .await?
        .wallet
        .expect("vault is missing");
    assert_eq!(wallet.name, "Vault");
    assert!(matches!(wallet.kind, WalletKind::Vault { .. }));

    Ok(())
}

#[tokio::test]
async fn opens_vault_and_watch_wallets() -> Result<()> {
    let mut app = TestApp::new().await?;

    let vault = add_vault(&app).await?;
    let watch = add_watch(&app).await?;

    let vault_address = txch(P2Singleton::new(LAUNCHER_ID, 0).tree_hash().into());

    for (fingerprint, receive_address) in [(vault, vault_address), (watch, txch(WATCHED[0]))] {
        login(&mut app, fingerprint).await?;

        let wallet = app
            .get_wallet(GetWallet { fingerprint: None })
            .await?
            .wallet
            .expect("should be logged in");
        assert_eq!(wallet.fingerprint, fingerprint);

        let status = app.get_sync_status(GetSyncStatus {}).await?;
        assert_eq!(status.receive_address, receive_address);
        assert_eq!(status.unhardened_derivation_index, 0);
        assert_eq!(status.hardened_derivation_index, 0);

        let derivations = app
            .get_derivations(GetDerivations {
                hardened: false,
                offset: 0,
                limit: 10,
            })
            .await?;
        assert_eq!(derivations.total, 0);

        let error = app
            .increase_derivation_index(IncreaseDerivationIndex {
                hardened: None,
                unhardened: None,
                index: 10,
            })
            .await
            .expect_err("derivations should be rejected");
        assert!(
            error.to_string().contains("doesn't derive addresses"),
            "{error}"
        );

        let address = app
            .get_wallet_address(GetWalletAddress {
                fingerprint,
                network_id: "testnet11".to_string(),
            })
            .await?
            .address;
        assert_eq!(address, receive_address);
    }

    Ok(())
}

#[tokio::test]
async fn manages_vault_and_watch_wallets() -> Result<()> {
    let mut app = TestApp::new().await?;

    let vault = add_vault(&app).await?;
    let watch = add_watch(&app).await?;

    for fingerprint in [vault, watch] {
        login(&mut app, fingerprint).await?;

        app.rename_wallet(RenameWallet {
            fingerprint,
            name: "Renamed".to_string(),
        })
        .await?;
        app.set_wallet_emoji(SetWalletEmoji {
            fingerprint,
            emoji: Some("🔒".to_string()),
        })
        .await?;

        let wallet = app
            .get_wallet(GetWallet {
                fingerprint: Some(fingerprint),
            })
            .await?
            .wallet
            .expect("wallet is missing");
        assert_eq!(wallet.name, "Renamed");
        assert_eq!(wallet.emoji.as_deref(), Some("🔒"));

        app.resync(Resync {
            fingerprint,
            delete_coins: true,
            delete_assets: true,
            delete_files: true,
            delete_offers: true,
            delete_addresses: true,
            delete_blocks: true,
        })
        .await?;
        app.consume_until(|event| matches!(event, SyncEvent::Subscribed))
            .await;

        app.delete_wallet(DeleteWallet {
            fingerprint,
            delete_key: true,
        })
        .await?;
    }

    assert!(app.get_wallets(GetWallets {}).await?.wallets.is_empty());

    // A deleted vault can be added again.
    add_vault(&app).await?;

    Ok(())
}

#[tokio::test]
async fn rejects_duplicate_wallets() -> Result<()> {
    let app = TestApp::new().await?;

    let vault = add_vault(&app).await?;
    let watch = add_watch(&app).await?;

    let mut sage = app.sage.lock().await;

    assert!(matches!(
        sage.add_vault_wallet(LAUNCHER_ID, "Again".to_string(), None),
        Err(Error::WalletExists(existing)) if existing == vault
    ));

    // The same set of addresses, in a different order.
    assert!(matches!(
        sage.add_watch_wallet(&[WATCHED[1], WATCHED[0], WATCHED[1]], "Again".to_string(), None),
        Err(Error::WalletExists(existing)) if existing == watch
    ));
    assert!(matches!(
        sage.add_watch_wallet(&[], "Empty".to_string(), None),
        Err(Error::NoWatchAddresses)
    ));

    // A vault whose preferred fingerprint is taken gets a different one.
    let mut launcher_id = LAUNCHER_ID.to_bytes();
    launcher_id[31] = 9;
    let other = sage.add_vault_wallet(Bytes32::new(launcher_id), "Other".to_string(), None)?;
    assert_eq!(vault, 0x0101_0101);
    assert_ne!(other, vault);

    Ok(())
}

#[tokio::test]
async fn imports_a_key_that_is_not_a_wallet_yet() -> Result<()> {
    let app = TestApp::new().await?;

    let mnemonic = Mnemonic::from_entropy(&[4; 16])?;
    let master_pk = SecretKey::from_seed(&mnemonic.to_seed("")).public_key();

    // The keychain can hold keys without wallets, such as a public key added on its own.
    let fingerprint = app.sage.lock().await.keychain.add_public_key(&master_pk)?;

    let import = |save_secrets| ImportKey {
        name: "Imported".to_string(),
        key: mnemonic.to_string(),
        derivation_index: 0,
        hardened: None,
        unhardened: None,
        save_secrets,
        login: false,
        emoji: None,
    };

    // Until then, it's only a key.
    assert!(
        app.get_wallet(GetWallet {
            fingerprint: Some(fingerprint),
        })
        .await?
        .wallet
        .is_none()
    );
    let key = app
        .get_key(GetKey { fingerprint })
        .await?
        .key
        .expect("key is missing");
    assert!(!key.has_secrets);

    assert_eq!(app.import_key(import(true)).await?.fingerprint, fingerprint);

    let wallet = app
        .get_wallet(GetWallet {
            fingerprint: Some(fingerprint),
        })
        .await?
        .wallet
        .expect("wallet is missing");
    assert_eq!(wallet.name, "Imported");

    let key = app
        .get_key(GetKey { fingerprint })
        .await?
        .key
        .expect("key is missing");
    assert!(key.has_secrets, "the import should add the missing secrets");

    let error = app
        .import_key(import(true))
        .await
        .expect_err("the wallet already exists");
    assert!(error.to_string().contains("already exists"), "{error}");

    Ok(())
}

#[tokio::test]
async fn deleting_a_wallet_can_keep_its_key() -> Result<()> {
    let app = TestApp::new().await?;

    let import = |seed: u8| ImportKey {
        name: "Wallet".to_string(),
        key: Mnemonic::from_entropy(&[seed; 16])
            .expect("valid entropy")
            .to_string(),
        derivation_index: 0,
        hardened: None,
        unhardened: None,
        save_secrets: true,
        login: false,
        emoji: None,
    };

    let kept = app.import_key(import(5)).await?.fingerprint;
    let removed = app.import_key(import(6)).await?.fingerprint;

    app.delete_wallet(DeleteWallet {
        fingerprint: kept,
        delete_key: false,
    })
    .await?;

    // The key is no longer a wallet, but it's still in the keychain with its secrets.
    assert!(
        app.get_wallet(GetWallet {
            fingerprint: Some(kept),
        })
        .await?
        .wallet
        .is_none()
    );
    let key = app
        .get_key(GetKey { fingerprint: kept })
        .await?
        .key
        .expect("key should be kept");
    assert!(key.has_secrets);

    // By default, the key is removed along with its wallet.
    let response: Value = app
        .call_rpc("/delete_wallet", json!({ "fingerprint": removed }))
        .await?;
    assert_eq!(response, json!({}));
    assert!(
        app.get_key(GetKey {
            fingerprint: removed
        })
        .await?
        .key
        .is_none()
    );

    // The kept key can become a wallet again.
    assert_eq!(app.import_key(import(5)).await?.fingerprint, kept);

    Ok(())
}
