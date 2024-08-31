use bigdecimal::BigDecimal;
use chia_wallet_sdk::encode_address;
use tauri::{command, State};

use crate::{
    app_state::AppState,
    error::{Error, Result},
    models::{DidData, NftData, SyncInfo},
};

#[command]
pub async fn sync_info(state: State<'_, AppState>) -> Result<SyncInfo> {
    let state = state.lock().await;
    let wallet = state.wallet.as_ref().ok_or(Error::NoActiveWallet)?;

    let mut tx = wallet.db.tx().await?;

    let balance = tx.p2_balance().await?;
    let total_coins = tx.total_coin_count().await?;
    let synced_coins = tx.synced_coin_count().await?;

    tx.commit().await?;

    Ok(SyncInfo {
        xch_balance: (BigDecimal::from(balance) / BigDecimal::from(1_000_000_000_000u128))
            .to_string(),
        total_coins,
        synced_coins,
    })
}

#[command]
pub async fn did_list(state: State<'_, AppState>) -> Result<Vec<DidData>> {
    let state = state.lock().await;
    let wallet = state.wallet.as_ref().ok_or(Error::NoActiveWallet)?;

    let mut did_data = Vec::new();

    let mut tx = wallet.db.tx().await?;

    let did_ids = tx.did_list().await?;

    for did_id in did_ids {
        let did = tx.did_coin(did_id).await?.ok_or(Error::CoinStateNotFound)?;
        did_data.push(DidData {
            encoded_id: encode_address(did.info.launcher_id.to_bytes(), "did:chia:")?,
            launcher_id: did.info.launcher_id,
            address: encode_address(did.info.p2_puzzle_hash.to_bytes(), "xch")?,
        });
    }

    tx.commit().await?;

    Ok(did_data)
}

#[command]
pub async fn nft_list(state: State<'_, AppState>) -> Result<Vec<NftData>> {
    let state = state.lock().await;
    let wallet = state.wallet.as_ref().ok_or(Error::NoActiveWallet)?;

    let mut nft_data = Vec::new();

    let mut tx = wallet.db.tx().await?;

    let nft_ids = tx.nft_list().await?;

    for nft_id in nft_ids {
        let nft = tx.nft_coin(nft_id).await?.ok_or(Error::CoinStateNotFound)?;
        nft_data.push(NftData {
            encoded_id: encode_address(nft.info.launcher_id.to_bytes(), "nft")?,
            launcher_id: nft.info.launcher_id,
            address: encode_address(nft.info.p2_puzzle_hash.to_bytes(), "xch")?,
        });
    }

    tx.commit().await?;

    Ok(nft_data)
}
