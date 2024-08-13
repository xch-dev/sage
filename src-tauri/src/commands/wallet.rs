use tauri::{command, State};

use crate::{
    app_state::AppState,
    error::{Error, Result},
    models::SyncInfo,
};

#[command]
pub async fn sync_info(state: State<'_, AppState>) -> Result<SyncInfo> {
    let state = state.lock().await;
    let wallet = state.wallet.as_ref().ok_or(Error::NoActiveWallet)?;

    let mut tx = wallet.db.tx().await?;

    let total_coins = tx.total_coin_count().await?;
    let synced_coins = tx.synced_coin_count().await?;

    tx.commit().await?;

    Ok(SyncInfo {
        total_coins,
        synced_coins,
    })
}
