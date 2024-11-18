use bip39::Mnemonic;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use sage_api::{
    DeleteKey, ImportKey, ImportKeyResponse, Login, Logout, Resync, WalletInfo, WalletKind,
    WalletSecrets,
};
use specta::specta;
use tauri::{command, State};

use crate::{app_state::AppState, error::Result};

#[command]
#[specta]
pub async fn login(state: State<'_, AppState>, req: Login) -> Result<()> {
    Ok(state.lock().await.login(req).await?)
}

#[command]
#[specta]
pub async fn logout(state: State<'_, AppState>, req: Logout) -> Result<()> {
    Ok(state.lock().await.logout(req).await?)
}

#[command]
#[specta]
pub async fn resync(state: State<'_, AppState>, req: Resync) -> Result<()> {
    Ok(state.lock().await.resync(req).await?)
}

#[command]
#[specta]
pub async fn import_key(state: State<'_, AppState>, req: ImportKey) -> Result<ImportKeyResponse> {
    Ok(state.lock().await.import_key(req).await?)
}

#[command]
#[specta]
pub async fn delete_key(state: State<'_, AppState>, req: DeleteKey) -> Result<()> {
    Ok(state.lock().await.delete_key(req)?)
}

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
