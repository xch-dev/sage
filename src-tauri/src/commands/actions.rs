use chia_wallet_sdk::decode_address;
use sage_api::CatRecord;
use specta::specta;
use tauri::{command, State};

use crate::{AppState, Error, Result};

#[command]
#[specta]
pub async fn remove_cat_info(state: State<'_, AppState>, asset_id: String) -> Result<()> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let asset_id: [u8; 32] = hex::decode(asset_id)?
        .try_into()
        .map_err(|_| Error::invalid_asset_id())?;

    let mut assets = wallet.assets.lock().await;
    assets.tokens.shift_remove(&hex::encode(asset_id));
    assets.save(&wallet.assets_path)?;

    Ok(())
}

#[command]
#[specta]
pub async fn update_cat_info(state: State<'_, AppState>, record: CatRecord) -> Result<()> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let asset_id: [u8; 32] = hex::decode(record.asset_id)?
        .try_into()
        .map_err(|_| Error::invalid_asset_id())?;

    let mut assets = wallet.assets.lock().await;
    let asset = assets.tokens.entry(hex::encode(asset_id)).or_default();

    asset.name = record.name;
    asset.description = record.description;
    asset.ticker = record.ticker;
    asset.icon_url = record.icon_url;
    asset.hidden = !record.visible;

    assets.save(&wallet.assets_path)?;

    Ok(())
}

#[command]
#[specta]
pub async fn update_did(
    state: State<'_, AppState>,
    did_id: String,
    name: Option<String>,
    visible: bool,
) -> Result<()> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let (_launcher_id, prefix) = decode_address(&did_id)?;

    if prefix != "did:chia:" {
        return Err(Error::invalid_prefix(&prefix));
    }

    let mut assets = wallet.assets.lock().await;
    let asset = assets.profiles.entry(did_id).or_default();

    asset.name = name;
    asset.hidden = !visible;

    assets.save(&wallet.assets_path)?;

    Ok(())
}

#[command]
#[specta]
pub async fn update_nft(state: State<'_, AppState>, nft_id: String, visible: bool) -> Result<()> {
    let state = state.lock().await;
    let wallet = state.wallet()?;

    let (_launcher_id, prefix) = decode_address(&nft_id)?;

    if prefix != "nft" {
        return Err(Error::invalid_prefix(&prefix));
    }

    let mut assets = wallet.assets.lock().await;
    let asset = assets.nfts.entry(nft_id).or_default();

    asset.hidden = !visible;

    assets.save(&wallet.assets_path)?;

    Ok(())
}
