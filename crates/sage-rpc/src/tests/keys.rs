use anyhow::Result;
use bip39::Mnemonic;
use chia_wallet_sdk::prelude::*;
use sage_api::{
    AddKey, DeleteKey, DeleteWallet, GetKey, GetKeys, GetWallets, ImportWallet, KeyKind,
};

use super::TestApp;

fn mnemonic(seed: u8) -> Mnemonic {
    Mnemonic::from_entropy(&[seed; 16]).expect("valid entropy")
}

fn import(mnemonic: &Mnemonic) -> ImportWallet {
    ImportWallet {
        name: "Wallet".to_string(),
        key: mnemonic.to_string(),
        derivation_index: 0,
        hardened: None,
        unhardened: None,
        save_secrets: true,
        login: false,
        emoji: None,
    }
}

#[tokio::test]
async fn adds_and_deletes_keys_without_wallets() -> Result<()> {
    let app = TestApp::new().await?;

    let mnemonic = mnemonic(1);
    let master_pk = SecretKey::from_seed(&mnemonic.to_seed("")).public_key();

    // A public key can be added on its own, and its secrets added later.
    let fingerprint = app
        .add_key(AddKey {
            key: hex::encode(master_pk.to_bytes()),
        })
        .await?
        .fingerprint;
    assert_eq!(fingerprint, master_pk.get_fingerprint());

    let key = app
        .get_key(GetKey { fingerprint })
        .await?
        .key
        .expect("key is missing");
    assert_eq!(key.kind, KeyKind::Bls);
    assert_eq!(key.public_key, hex::encode(master_pk.to_bytes()));
    assert!(!key.has_secrets);

    // It's only a key, not a wallet.
    assert!(app.get_wallets(GetWallets {}).await?.wallets.is_empty());

    app.add_key(AddKey {
        key: mnemonic.to_string(),
    })
    .await?;
    assert!(
        app.get_key(GetKey { fingerprint })
            .await?
            .key
            .expect("key is missing")
            .has_secrets
    );

    let error = app
        .add_key(AddKey {
            key: mnemonic.to_string(),
        })
        .await
        .expect_err("the key already exists");
    assert!(error.to_string().contains("Key already exists"), "{error}");

    app.delete_key(DeleteKey { fingerprint }).await?;
    assert!(app.get_keys(GetKeys {}).await?.keys.is_empty());

    Ok(())
}

#[tokio::test]
async fn wallet_keys_follow_their_wallets() -> Result<()> {
    let app = TestApp::new().await?;

    // Importing a wallet adds its key.
    let fingerprint = app.import_wallet(import(&mnemonic(2))).await?.fingerprint;

    let keys = app.get_keys(GetKeys {}).await?.keys;
    assert_eq!(keys.len(), 1);
    assert_eq!(keys[0].fingerprint, fingerprint);
    assert!(keys[0].has_secrets);

    // The key can't be removed while the wallet uses it.
    let error = app
        .delete_key(DeleteKey { fingerprint })
        .await
        .expect_err("the key is in use");
    assert!(error.to_string().contains("used by a wallet"), "{error}");

    // Deleting the wallet removes its key.
    app.delete_wallet(DeleteWallet {
        fingerprint,
        delete_keys: true,
    })
    .await?;
    assert!(app.get_keys(GetKeys {}).await?.keys.is_empty());

    let error = app
        .delete_key(DeleteKey { fingerprint })
        .await
        .expect_err("the key is gone");
    assert!(error.to_string().contains("Unknown fingerprint"), "{error}");

    Ok(())
}
