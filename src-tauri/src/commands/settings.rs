use sage_api::{
    AddPeer, AddPeerResponse, GetPeers, GetPeersResponse, RemovePeer, RemovePeerResponse,
    SetDerivationBatchSize, SetDerivationBatchSizeResponse, SetDeriveAutomatically,
    SetDeriveAutomaticallyResponse, SetDiscoverPeers, SetDiscoverPeersResponse, SetNetworkId,
    SetNetworkIdResponse, SetTargetPeers, SetTargetPeersResponse,
};
use sage_config::{NetworkConfig, WalletConfig};
use specta::specta;
use tauri::{command, State};

use crate::{app_state::AppState, error::Result};

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
