use std::time::Duration;

use chia_wallet_sdk::decode_address;
use sage_api::{wallet_connect::*, *};
use sage_config::{NetworkConfig, WalletConfig};
use specta::specta;
use tauri::{command, State};
use tokio::time::sleep;
use tracing::error;

use crate::{app_state::AppState, error::Result};

#[command]
#[specta]
pub async fn initialize(state: State<'_, AppState>) -> Result<()> {
    if state.lock().await.initialize().await? {
        return Ok(());
    }

    let app_state = (*state).clone();

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(3)).await;

            let app_state = app_state.lock().await;

            if let Err(error) = app_state.sage.save_peers().await {
                error!("Error while saving peers: {error:?}");
            }

            drop(app_state);
        }
    });

    Ok(())
}

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
pub async fn rename_key(state: State<'_, AppState>, req: RenameKey) -> Result<RenameKeyResponse> {
    Ok(state.lock().await.rename_key(req)?)
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
pub async fn view_coin_spends(
    state: State<'_, AppState>,
    req: ViewCoinSpends,
) -> Result<ViewCoinSpendsResponse> {
    Ok(state.lock().await.view_coin_spends(req).await?)
}

#[command]
#[specta]
pub async fn submit_transaction(
    state: State<'_, AppState>,
    req: SubmitTransaction,
) -> Result<SubmitTransactionResponse> {
    Ok(state.lock().await.submit_transaction(req).await?)
}

#[command]
#[specta]
pub async fn make_offer(state: State<'_, AppState>, req: MakeOffer) -> Result<MakeOfferResponse> {
    Ok(state.lock().await.make_offer(req).await?)
}

#[command]
#[specta]
pub async fn take_offer(state: State<'_, AppState>, req: TakeOffer) -> Result<TakeOfferResponse> {
    Ok(state.lock().await.take_offer(req).await?)
}

#[command]
#[specta]
pub async fn view_offer(state: State<'_, AppState>, req: ViewOffer) -> Result<ViewOfferResponse> {
    Ok(state.lock().await.view_offer(req).await?)
}

#[command]
#[specta]
pub async fn import_offer(
    state: State<'_, AppState>,
    req: ImportOffer,
) -> Result<ImportOfferResponse> {
    Ok(state.lock().await.import_offer(req).await?)
}

#[command]
#[specta]
pub async fn get_offers(state: State<'_, AppState>, req: GetOffers) -> Result<GetOffersResponse> {
    Ok(state.lock().await.get_offers(req).await?)
}

#[command]
#[specta]
pub async fn get_offer(state: State<'_, AppState>, req: GetOffer) -> Result<GetOfferResponse> {
    Ok(state.lock().await.get_offer(req).await?)
}

#[command]
#[specta]
pub async fn delete_offer(
    state: State<'_, AppState>,
    req: DeleteOffer,
) -> Result<DeleteOfferResponse> {
    Ok(state.lock().await.delete_offer(req).await?)
}

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
pub async fn get_derivations(
    state: State<'_, AppState>,
    req: GetDerivations,
) -> Result<GetDerivationsResponse> {
    Ok(state.lock().await.get_derivations(req).await?)
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

#[command]
#[specta]
pub async fn remove_cat(state: State<'_, AppState>, req: RemoveCat) -> Result<RemoveCatResponse> {
    Ok(state.lock().await.remove_cat(req).await?)
}

#[command]
#[specta]
pub async fn update_cat(state: State<'_, AppState>, req: UpdateCat) -> Result<UpdateCatResponse> {
    Ok(state.lock().await.update_cat(req).await?)
}

#[command]
#[specta]
pub async fn update_did(state: State<'_, AppState>, req: UpdateDid) -> Result<UpdateDidResponse> {
    Ok(state.lock().await.update_did(req).await?)
}

#[command]
#[specta]
pub async fn update_nft(state: State<'_, AppState>, req: UpdateNft) -> Result<UpdateNftResponse> {
    Ok(state.lock().await.update_nft(req).await?)
}

#[command]
#[specta]
pub async fn get_peers(state: State<'_, AppState>, req: GetPeers) -> Result<GetPeersResponse> {
    Ok(state.lock().await.get_peers(req).await?)
}

#[command]
#[specta]
pub async fn remove_peer(
    state: State<'_, AppState>,
    req: RemovePeer,
) -> Result<RemovePeerResponse> {
    Ok(state.lock().await.remove_peer(req).await?)
}

#[command]
#[specta]
pub async fn add_peer(state: State<'_, AppState>, req: AddPeer) -> Result<AddPeerResponse> {
    Ok(state.lock().await.add_peer(req).await?)
}

#[command]
#[specta]
pub async fn network_config(state: State<'_, AppState>) -> Result<NetworkConfig> {
    let state = state.lock().await;
    Ok(state.config.network.clone())
}

#[command]
#[specta]
pub async fn set_discover_peers(
    state: State<'_, AppState>,
    req: SetDiscoverPeers,
) -> Result<SetDiscoverPeersResponse> {
    Ok(state.lock().await.set_discover_peers(req).await?)
}

#[command]
#[specta]
pub async fn set_target_peers(
    state: State<'_, AppState>,
    req: SetTargetPeers,
) -> Result<SetTargetPeersResponse> {
    Ok(state.lock().await.set_target_peers(req).await?)
}

#[command]
#[specta]
pub async fn set_network_id(
    state: State<'_, AppState>,
    req: SetNetworkId,
) -> Result<SetNetworkIdResponse> {
    Ok(state.lock().await.set_network_id(req).await?)
}

#[command]
#[specta]
pub async fn wallet_config(state: State<'_, AppState>, fingerprint: u32) -> Result<WalletConfig> {
    let state = state.lock().await;
    Ok(state.try_wallet_config(fingerprint).cloned()?)
}

#[command]
#[specta]
pub async fn set_derive_automatically(
    state: State<'_, AppState>,
    req: SetDeriveAutomatically,
) -> Result<SetDeriveAutomaticallyResponse> {
    Ok(state.lock().await.set_derive_automatically(req)?)
}

#[command]
#[specta]
pub async fn set_derivation_batch_size(
    state: State<'_, AppState>,
    req: SetDerivationBatchSize,
) -> Result<SetDerivationBatchSizeResponse> {
    Ok(state.lock().await.set_derivation_batch_size(req)?)
}

#[command]
#[specta]
pub async fn get_networks(
    state: State<'_, AppState>,
    req: GetNetworks,
) -> Result<GetNetworksResponse> {
    Ok(state.lock().await.get_networks(req)?)
}

#[command]
#[specta]
pub async fn filter_unlocked_coins(
    state: State<'_, AppState>,
    req: FilterUnlockedCoins,
) -> Result<FilterUnlockedCoinsResponse> {
    Ok(state.lock().await.filter_unlocked_coins(req).await?)
}

#[command]
#[specta]
pub async fn get_asset_coins(
    state: State<'_, AppState>,
    req: GetAssetCoins,
) -> Result<GetAssetCoinsResponse> {
    Ok(state.lock().await.get_asset_coins(req).await?)
}

#[command]
#[specta]
pub async fn sign_message_with_public_key(
    state: State<'_, AppState>,
    req: SignMessageWithPublicKey,
) -> Result<SignMessageWithPublicKeyResponse> {
    Ok(state.lock().await.sign_message_with_public_key(req).await?)
}

#[command]
#[specta]
pub async fn send_transaction_immediately(
    state: State<'_, AppState>,
    req: SendTransactionImmediately,
) -> Result<SendTransactionImmediatelyResponse> {
    Ok(state.lock().await.send_transaction_immediately(req).await?)
}
