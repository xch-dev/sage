use bip39::Mnemonic;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use sage_api::{
    DeleteKey, DeleteKeyResponse, GetKey, GetKeyResponse, GetKeys, GetKeysResponse, GetSecretKey,
    GetSecretKeyResponse, ImportKey, ImportKeyResponse, Login, LoginResponse, Logout,
    LogoutResponse, Resync, ResyncResponse,
};
use specta::specta;
use tauri::{command, State};

use crate::{app_state::AppState, error::Result};

#[command]
#[specta]
pub async fn login(state: State<'_, AppState>, req: Login) -> Result<LoginResponse> {
    Ok(state.lock().await.login(req).await?)
}

#[command]
#[specta]
pub async fn logout(state: State<'_, AppState>, req: Logout) -> Result<LogoutResponse> {
    Ok(state.lock().await.logout(req).await?)
}

#[command]
#[specta]
pub async fn resync(state: State<'_, AppState>, req: Resync) -> Result<ResyncResponse> {
    Ok(state.lock().await.resync(req).await?)
}

#[command]
#[specta]
pub async fn import_key(state: State<'_, AppState>, req: ImportKey) -> Result<ImportKeyResponse> {
    Ok(state.lock().await.import_key(req).await?)
}

#[command]
#[specta]
pub async fn delete_key(state: State<'_, AppState>, req: DeleteKey) -> Result<DeleteKeyResponse> {
    Ok(state.lock().await.delete_key(req)?)
}

#[command]
#[specta]
pub async fn get_keys(state: State<'_, AppState>, req: GetKeys) -> Result<GetKeysResponse> {
    Ok(state.lock().await.get_keys(req)?)
}

#[command]
#[specta]
pub async fn get_key(state: State<'_, AppState>, req: GetKey) -> Result<GetKeyResponse> {
    Ok(state.lock().await.get_key(req)?)
}

#[command]
#[specta]
pub async fn get_secret_key(
    state: State<'_, AppState>,
    req: GetSecretKey,
) -> Result<GetSecretKeyResponse> {
    Ok(state.lock().await.get_secret_key(req)?)
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
