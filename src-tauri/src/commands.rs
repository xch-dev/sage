use std::time::Duration;

use chia_wallet_sdk::decode_address;
use sage_api::{wallet_connect::*, *};
use sage_api_macro::impl_endpoints;
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

impl_endpoints! {
    (repeat
        #[command]
        #[specta]
        pub async fn endpoint(state: State<'_, AppState>, req: Endpoint) -> Result<EndpointResponse> {
            Ok(state.lock().await.endpoint(req) maybe_await?)
        }
    )
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
pub async fn network_config(state: State<'_, AppState>) -> Result<NetworkConfig> {
    let state = state.lock().await;
    Ok(state.config.network.clone())
}

#[command]
#[specta]
pub async fn wallet_config(state: State<'_, AppState>, fingerprint: u32) -> Result<WalletConfig> {
    let mut state = state.lock().await;
    Ok(state.try_wallet_config(fingerprint).clone())
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
pub async fn sign_message_by_address(
    state: State<'_, AppState>,
    req: SignMessageByAddress,
) -> Result<SignMessageByAddressResponse> {
    Ok(state.lock().await.sign_message_by_address(req).await?)
}

#[command]
#[specta]
pub async fn send_transaction_immediately(
    state: State<'_, AppState>,
    req: SendTransactionImmediately,
) -> Result<SendTransactionImmediatelyResponse> {
    Ok(state.lock().await.send_transaction_immediately(req).await?)
}
