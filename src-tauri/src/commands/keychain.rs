use bip39::Mnemonic;
use chia::bls::{PublicKey, SecretKey};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use std::str::FromStr;
use tauri::{command, State};

use crate::error::Error;
use crate::{app_state::AppState, error::Result, models::WalletInfo};

#[command]
pub async fn active_wallet(state: State<'_, AppState>) -> Result<Option<WalletInfo>> {
    let state = state.lock().await;
    state.active_wallet()
}

#[command]
pub async fn wallet_list(state: State<'_, AppState>) -> Result<Vec<WalletInfo>> {
    let state = state.lock().await;
    state.wallets()
}

#[command]
pub async fn login_wallet(state: State<'_, AppState>, fingerprint: u32) -> Result<()> {
    let state = state.lock().await;
    state.login_wallet(fingerprint)
}

#[command]
pub async fn logout_wallet(state: State<'_, AppState>) -> Result<()> {
    let state = state.lock().await;
    state.logout_wallet()
}

#[command]
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
pub async fn create_wallet(
    state: State<'_, AppState>,
    name: String,
    mnemonic: String,
    save_mnemonic: bool,
) -> Result<()> {
    let mut state = state.lock().await;
    let mnemonic = Mnemonic::from_str(&mnemonic)?;

    let fingerprint = if save_mnemonic {
        state.import_mnemonic(&mnemonic)?
    } else {
        let seed = mnemonic.to_seed("");
        let master_sk = SecretKey::from_seed(&seed);
        let master_pk = master_sk.public_key();
        state.import_public_key(&master_pk)?
    };

    state.rename_wallet(fingerprint, name)?;
    state.login_wallet(fingerprint)?;

    Ok(())
}

#[command]
pub async fn import_wallet(state: State<'_, AppState>, name: String, key: String) -> Result<()> {
    let mut state = state.lock().await;

    let mut key_hex = key.as_str();

    if key_hex.starts_with("0x") || key_hex.starts_with("0X") {
        key_hex = &key_hex[2..];
    }

    let fingerprint = if let Ok(bytes) = hex::decode(key_hex) {
        if let Ok(master_pk) = bytes.clone().try_into() {
            let master_pk = PublicKey::from_bytes(&master_pk)?;
            state.import_public_key(&master_pk)?
        } else if let Ok(master_sk) = bytes.try_into() {
            let master_sk = SecretKey::from_bytes(&master_sk)?;
            state.import_secret_key(&master_sk)?
        } else {
            return Err(Error::InvalidKeySize);
        }
    } else {
        let mnemonic = Mnemonic::from_str(&key)?;
        state.import_mnemonic(&mnemonic)?
    };

    state.rename_wallet(fingerprint, name)?;
    state.login_wallet(fingerprint)?;

    Ok(())
}

#[command]
pub async fn delete_wallet(state: State<'_, AppState>, fingerprint: u32) -> Result<()> {
    let state = state.lock().await;
    state.delete_wallet(fingerprint)?;
    Ok(())
}

#[command]
pub async fn rename_wallet(
    state: State<'_, AppState>,
    fingerprint: u32,
    name: String,
) -> Result<()> {
    let state = state.lock().await;
    state.rename_wallet(fingerprint, name)?;
    Ok(())
}
