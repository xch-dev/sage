use chia_wallet_sdk::decode_address;
use sage_api::{
    AddNftUri, AssignNftsToDid, BulkMintNfts, CombineCat, CombineXch, CreateDid, IssueCat, SendCat,
    SendXch, SignCoinSpends, SignCoinSpendsResponse, SplitCat, SplitXch, SubmitTransaction,
    SubmitTransactionResponse, TransactionResponse, TransferDids, TransferNfts,
};
use specta::specta;
use tauri::{command, State};

use crate::{app_state::AppState, error::Result};

#[command]
#[specta]
pub async fn validate_address(state: State<'_, AppState>, address: String) -> Result<bool> {
    let state = state.lock().await;
    let Some((_puzzle_hash, prefix)) = decode_address(&address).ok() else {
        return Ok(false);
    };
    Ok(prefix == state.network().address_prefix)
}

#[command]
#[specta]
pub async fn send_xch(state: State<'_, AppState>, req: SendXch) -> Result<TransactionResponse> {
    Ok(state.lock().await.send_xch(req).await?)
}

#[command]
#[specta]
pub async fn combine_xch(
    state: State<'_, AppState>,
    req: CombineXch,
) -> Result<TransactionResponse> {
    Ok(state.lock().await.combine_xch(req).await?)
}

#[command]
#[specta]
pub async fn split_xch(state: State<'_, AppState>, req: SplitXch) -> Result<TransactionResponse> {
    Ok(state.lock().await.split_xch(req).await?)
}

#[command]
#[specta]
pub async fn combine_cat(
    state: State<'_, AppState>,
    req: CombineCat,
) -> Result<TransactionResponse> {
    Ok(state.lock().await.combine_cat(req).await?)
}

#[command]
#[specta]
pub async fn split_cat(state: State<'_, AppState>, req: SplitCat) -> Result<TransactionResponse> {
    Ok(state.lock().await.split_cat(req).await?)
}

#[command]
#[specta]
pub async fn issue_cat(state: State<'_, AppState>, req: IssueCat) -> Result<TransactionResponse> {
    Ok(state.lock().await.issue_cat(req).await?)
}

#[command]
#[specta]
pub async fn send_cat(state: State<'_, AppState>, req: SendCat) -> Result<TransactionResponse> {
    Ok(state.lock().await.send_cat(req).await?)
}

#[command]
#[specta]
pub async fn create_did(state: State<'_, AppState>, req: CreateDid) -> Result<TransactionResponse> {
    Ok(state.lock().await.create_did(req).await?)
}

#[command]
#[specta]
pub async fn bulk_mint_nfts(
    state: State<'_, AppState>,
    req: BulkMintNfts,
) -> Result<TransactionResponse> {
    Ok(state.lock().await.bulk_mint_nfts(req).await?)
}

#[command]
#[specta]
pub async fn transfer_nfts(
    state: State<'_, AppState>,
    req: TransferNfts,
) -> Result<TransactionResponse> {
    Ok(state.lock().await.transfer_nfts(req).await?)
}

#[command]
#[specta]
pub async fn add_nft_uri(
    state: State<'_, AppState>,
    req: AddNftUri,
) -> Result<TransactionResponse> {
    Ok(state.lock().await.add_nft_uri(req).await?)
}

#[command]
#[specta]
pub async fn assign_nfts_to_did(
    state: State<'_, AppState>,
    req: AssignNftsToDid,
) -> Result<TransactionResponse> {
    Ok(state.lock().await.assign_nfts_to_did(req).await?)
}

#[command]
#[specta]
pub async fn transfer_dids(
    state: State<'_, AppState>,
    req: TransferDids,
) -> Result<TransactionResponse> {
    Ok(state.lock().await.transfer_dids(req).await?)
}

#[command]
#[specta]
pub async fn sign_coin_spends(
    state: State<'_, AppState>,
    req: SignCoinSpends,
) -> Result<SignCoinSpendsResponse> {
    Ok(state.lock().await.sign_coin_spends(req).await?)
}

#[command]
#[specta]
pub async fn submit_transaction(
    state: State<'_, AppState>,
    req: SubmitTransaction,
) -> Result<SubmitTransactionResponse> {
    Ok(state.lock().await.submit_transaction(req).await?)
}
