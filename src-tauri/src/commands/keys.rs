use std::str::FromStr;

use bip39::Mnemonic;
use chia::bls::{PublicKey, SecretKey};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use specta::specta;
use tauri::{command, State};

use crate::{AppState, Error, Result, WalletInfo, WalletKind, WalletSecrets};

#[command]
#[specta]
pub async fn wallet_list(state: State<'_, AppState>) -> Result<Vec<WalletInfo>> {
    let state = state.lock().await;
    state.wallets()
}

#[command]
#[specta]
pub async fn active_wallet(state: State<'_, AppState>) -> Result<Option<WalletInfo>> {
    let state = state.lock().await;

    let Some(fingerprint) = state.config.app.active_fingerprint else {
        return Ok(None);
    };

    let name = state
        .config
        .wallets
        .get(&fingerprint.to_string())
        .map_or_else(
            || "Unnamed Wallet".to_string(),
            |config| config.name.clone(),
        );

    let Some(master_pk) = state.keychain.extract_public_key(fingerprint)? else {
        return Ok(None);
    };

    Ok(Some(WalletInfo {
        name,
        fingerprint,
        public_key: hex::encode(master_pk.to_bytes()),
        kind: if state.keychain.has_secret_key(fingerprint) {
            WalletKind::Hot
        } else {
            WalletKind::Cold
        },
    }))
}

#[command]
#[specta]
pub async fn get_wallet_secrets(
    state: State<'_, AppState>,
    fingerprint: u32,
) -> Result<Option<WalletSecrets>> {
    let state = state.lock().await;

    let (mnemonic, Some(secret_key)) = state.keychain.extract_secrets(fingerprint, b"")? else {
        return Ok(None);
    };

    Ok(Some(WalletSecrets {
        mnemonic: mnemonic.map(|m| m.to_string()),
        secret_key: hex::encode(secret_key.to_bytes()),
    }))
}

#[command]
#[specta]
pub async fn login_wallet(state: State<'_, AppState>, fingerprint: u32) -> Result<()> {
    let mut state = state.lock().await;
    state.config.app.active_fingerprint = Some(fingerprint);
    state.save_config()?;
    state.switch_wallet().await
}

#[command]
#[specta]
pub async fn resync_wallet(state: State<'_, AppState>, fingerprint: u32) -> Result<()> {
    let mut state = state.lock().await;

    let login = state.config.app.active_fingerprint == Some(fingerprint);

    if login {
        state.config.app.active_fingerprint = None;
        state.switch_wallet().await?;
    }

    state.delete_wallet_db(fingerprint)?;

    if login {
        state.config.app.active_fingerprint = Some(fingerprint);
        state.save_config()?;
        state.switch_wallet().await?;
    }

    Ok(())
}

#[command]
#[specta]
pub async fn logout_wallet(state: State<'_, AppState>) -> Result<()> {
    let mut state = state.lock().await;
    state.config.app.active_fingerprint = None;
    state.save_config()?;
    state.switch_wallet().await
}

#[command]
#[specta]
pub fn generate_mnemonic(use_24_words: bool) -> Result<String> {
    let mut rng = ChaCha20Rng::from_entropy();
    let mnemonic = if use_24_words {
        let entropy: [u8; 32] = rng.gen();
        Mnemonic::from_entropy(&entropy)?
    } else {
        let entropy: [u8; 16] = rng.gen();
        Mnemonic::from_entropy(&entropy)?
    };
    Ok(mnemonic.to_string())
}

#[command]
#[specta]
pub async fn create_wallet(
    state: State<'_, AppState>,
    name: String,
    mnemonic: String,
    save_mnemonic: bool,
) -> Result<()> {
    let mut state = state.lock().await;
    let mnemonic = Mnemonic::from_str(&mnemonic)?;

    let fingerprint = if save_mnemonic {
        state.keychain.add_mnemonic(&mnemonic, b"")?
    } else {
        let seed = mnemonic.to_seed("");
        let master_sk = SecretKey::from_seed(&seed);
        let master_pk = master_sk.public_key();
        state.keychain.add_public_key(&master_pk)?
    };
    state.save_keychain()?;

    let config = state.wallet_config_mut(fingerprint);
    config.name = name;
    state.config.app.active_fingerprint = Some(fingerprint);
    state.save_config()?;

    state.switch_wallet().await
}

#[command]
#[specta]
pub async fn import_wallet(state: State<'_, AppState>, name: String, key: String) -> Result<()> {
    let mut state = state.lock().await;

    let mut key_hex = key.as_str();

    if key_hex.starts_with("0x") || key_hex.starts_with("0X") {
        key_hex = &key_hex[2..];
    }

    let fingerprint = if let Ok(bytes) = hex::decode(key_hex) {
        if let Ok(master_pk) = bytes.clone().try_into() {
            let master_pk = PublicKey::from_bytes(&master_pk)?;
            state.keychain.add_public_key(&master_pk)?
        } else if let Ok(master_sk) = bytes.try_into() {
            let master_sk = SecretKey::from_bytes(&master_sk)?;
            state.keychain.add_secret_key(&master_sk, b"")?
        } else {
            return Err(Error::invalid_key("Must be 32 or 48 bytes"));
        }
    } else {
        let mnemonic = Mnemonic::from_str(&key)?;
        state.keychain.add_mnemonic(&mnemonic, b"")?
    };
    state.save_keychain()?;

    let config = state.wallet_config_mut(fingerprint);
    config.name = name;
    state.config.app.active_fingerprint = Some(fingerprint);
    state.save_config()?;

    state.switch_wallet().await
}

#[command]
#[specta]
pub async fn delete_wallet(state: State<'_, AppState>, fingerprint: u32) -> Result<()> {
    let mut state = state.lock().await;
    state.delete_wallet(fingerprint)?;
    Ok(())
}
