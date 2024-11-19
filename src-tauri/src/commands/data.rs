use sage_api::{
    GetAddresses, GetAddressesResponse, GetCat, GetCatCoins, GetCatCoinsResponse, GetCatResponse,
    GetCats, GetCatsResponse, GetDids, GetDidsResponse, GetNft, GetNftCollection,
    GetNftCollectionResponse, GetNftCollections, GetNftCollectionsResponse, GetNftResponse,
    GetNftStatus, GetNftStatusResponse, GetNfts, GetNftsResponse, GetPendingTransactions,
    GetPendingTransactionsResponse, GetSyncStatus, GetSyncStatusResponse, GetXchCoins,
    GetXchCoinsResponse,
};
use specta::specta;
use tauri::{command, State};

use crate::{app_state::AppState, error::Result};

#[command]
#[specta]
pub async fn get_sync_status(
    state: State<'_, AppState>,
    req: GetSyncStatus,
) -> Result<GetSyncStatusResponse> {
    Ok(state.lock().await.get_sync_status(req).await?)
}

#[command]
#[specta]
pub async fn get_addresses(
    state: State<'_, AppState>,
    req: GetAddresses,
) -> Result<GetAddressesResponse> {
    Ok(state.lock().await.get_addresses(req).await?)
}

#[command]
#[specta]
pub async fn get_xch_coins(
    state: State<'_, AppState>,
    req: GetXchCoins,
) -> Result<GetXchCoinsResponse> {
    Ok(state.lock().await.get_xch_coins(req).await?)
}

#[command]
#[specta]
pub async fn get_cat_coins(
    state: State<'_, AppState>,
    req: GetCatCoins,
) -> Result<GetCatCoinsResponse> {
    Ok(state.lock().await.get_cat_coins(req).await?)
}

#[command]
#[specta]
pub async fn get_cats(state: State<'_, AppState>, req: GetCats) -> Result<GetCatsResponse> {
    Ok(state.lock().await.get_cats(req).await?)
}

#[command]
#[specta]
pub async fn get_cat(state: State<'_, AppState>, req: GetCat) -> Result<GetCatResponse> {
    Ok(state.lock().await.get_cat(req).await?)
}

#[command]
#[specta]
pub async fn get_dids(state: State<'_, AppState>, req: GetDids) -> Result<GetDidsResponse> {
    Ok(state.lock().await.get_dids(req).await?)
}

#[command]
#[specta]
pub async fn get_pending_transactions(
    state: State<'_, AppState>,
    req: GetPendingTransactions,
) -> Result<GetPendingTransactionsResponse> {
    Ok(state.lock().await.get_pending_transactions(req).await?)
}

#[command]
#[specta]
pub async fn get_nft_status(
    state: State<'_, AppState>,
    req: GetNftStatus,
) -> Result<GetNftStatusResponse> {
    Ok(state.lock().await.get_nft_status(req).await?)
}

#[command]
#[specta]
pub async fn get_nft_collections(
    state: State<'_, AppState>,
    req: GetNftCollections,
) -> Result<GetNftCollectionsResponse> {
    Ok(state.lock().await.get_nft_collections(req).await?)
}

#[command]
#[specta]
pub async fn get_nft_collection(
    state: State<'_, AppState>,
    req: GetNftCollection,
) -> Result<GetNftCollectionResponse> {
    Ok(state.lock().await.get_nft_collection(req).await?)
}

#[command]
#[specta]
pub async fn get_nfts(state: State<'_, AppState>, req: GetNfts) -> Result<GetNftsResponse> {
    Ok(state.lock().await.get_nfts(req).await?)
}

#[command]
#[specta]
pub async fn get_nft(state: State<'_, AppState>, req: GetNft) -> Result<GetNftResponse> {
    Ok(state.lock().await.get_nft(req).await?)
}
