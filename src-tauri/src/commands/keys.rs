use sage_api::{
    DeleteKey, DeleteKeyResponse, GenerateMnemonic, GenerateMnemonicResponse, GetKey,
    GetKeyResponse, GetKeys, GetKeysResponse, GetSecretKey, GetSecretKeyResponse, ImportKey,
    ImportKeyResponse, Login, LoginResponse, Logout, LogoutResponse, Resync, ResyncResponse,
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
pub async fn generate_mnemonic(
    state: State<'_, AppState>,
    req: GenerateMnemonic,
) -> Result<GenerateMnemonicResponse> {
    Ok(state.lock().await.generate_mnemonic(req)?)
}
