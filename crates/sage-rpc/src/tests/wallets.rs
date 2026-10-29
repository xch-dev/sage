use anyhow::Result;
use bip39::Mnemonic;
use chia_wallet_sdk::prelude::*;
use sage_api::{
    DeleteWallet, GetKey, GetKeys, GetWallet, GetWallets, ImportWallet, Login, WalletKind,
};
use serde_json::{Value, json};

use super::TestApp;

#[tokio::test]
async fn lists_wallets_and_their_keys() -> Result<()> {
    let mut app = TestApp::new().await?;

    let fingerprint = app.setup_bls(0).await?;

    let wallets = app.get_wallets(GetWallets {}).await?.wallets;
    assert_eq!(wallets.len(), 1);
    assert_eq!(wallets[0].fingerprint, fingerprint);
    assert_eq!(wallets[0].kind, WalletKind::Bls);

    let keys = app.get_keys(GetKeys {}).await?.keys;
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].fingerprint, fingerprint);
    assert!(keys[0].has_secrets);

    let raw: Value = app.call_rpc("/get_wallets", GetWallets {}).await?;
    assert_eq!(raw["wallets"][0]["kind"], json!("bls"));

    Ok(())
}

#[tokio::test]
async fn imports_a_key_that_is_not_a_wallet_yet() -> Result<()> {
    let app = TestApp::new().await?;

    let mnemonic = Mnemonic::from_entropy(&[4; 16])?;
    let master_pk = SecretKey::from_seed(&mnemonic.to_seed("")).public_key();

    // The keychain can hold keys without wallets, such as a public key added on its own.
    let fingerprint = app.sage.lock().await.keychain.add_public_key(&master_pk)?;

    let import = |save_secrets| ImportWallet {
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

    assert_eq!(
        app.import_wallet(import(true)).await?.fingerprint,
        fingerprint
    );

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
        .import_wallet(import(true))
        .await
        .expect_err("the wallet already exists");
    assert!(error.to_string().contains("already exists"), "{error}");

    Ok(())
}

#[tokio::test]
async fn deleting_a_wallet_can_keep_its_key() -> Result<()> {
    let app = TestApp::new().await?;

    let import = |seed: u8| ImportWallet {
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

    let kept = app.import_wallet(import(5)).await?.fingerprint;
    let removed = app.import_wallet(import(6)).await?.fingerprint;

    app.delete_wallet(DeleteWallet {
        fingerprint: kept,
        delete_keys: false,
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
    assert_eq!(app.import_wallet(import(5)).await?.fingerprint, kept);

    Ok(())
}

#[tokio::test]
async fn reimporting_restores_a_wallet_whose_key_is_missing() -> Result<()> {
    let app = TestApp::new().await?;

    let mnemonic = Mnemonic::from_entropy(&[7; 16])?;
    let import = |name: &str| ImportWallet {
        name: name.to_string(),
        key: mnemonic.to_string(),
        derivation_index: 0,
        hardened: None,
        unhardened: None,
        save_secrets: true,
        login: false,
        emoji: None,
    };

    let fingerprint = app.import_wallet(import("Original")).await?.fingerprint;

    // Without its key, such as after restoring an older keychain, the wallet is hidden.
    app.sage.lock().await.keychain.remove(fingerprint);
    assert!(app.get_wallets(GetWallets {}).await?.wallets.is_empty());

    assert_eq!(
        app.import_wallet(import("Again")).await?.fingerprint,
        fingerprint
    );

    let wallets = app.get_wallets(GetWallets {}).await?.wallets;
    assert_eq!(wallets.len(), 1);
    assert_eq!(wallets[0].name, "Original");
    assert!(
        app.get_key(GetKey { fingerprint })
            .await?
            .key
            .expect("key is missing")
            .has_secrets
    );

    Ok(())
}

#[tokio::test]
async fn logging_into_a_key_that_is_not_a_wallet_fails() -> Result<()> {
    let app = TestApp::new().await?;

    let master_pk = SecretKey::from_seed(&[8; 32]).public_key();
    let fingerprint = app.sage.lock().await.keychain.add_public_key(&master_pk)?;

    let error = app
        .login(Login { fingerprint })
        .await
        .expect_err("a key that isn't a wallet can't be logged into");
    assert!(error.to_string().contains("Unknown fingerprint"), "{error}");

    // Nothing is saved, so the next start doesn't try to open it.
    assert_eq!(app.sage.lock().await.config.global.fingerprint, None);

    Ok(())
}
