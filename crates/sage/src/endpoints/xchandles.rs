use std::time::{SystemTime, UNIX_EPOCH};

use chia_wallet_sdk::utils::Address;
use sage_api::{HandleRecord, HandleStatus, ResolveHandle, ResolveHandleResponse};
use sage_wallet::{
    Handle, HandleResolution, XchandlesClient, XchandlesError, chain_tip, resolve_and_store,
    resolve_handle,
};
use tokio::sync::Mutex;

use crate::{Result, Sage};

impl Sage {
    /// Resolves a handle without holding the Sage lock while waiting on the API and peers,
    /// which can take several seconds. Everything the lookup needs is captured up front, so
    /// switching wallets or networks in the meantime can't mix their state.
    pub async fn resolve_handle(sage: &Mutex<Self>, req: ResolveHandle) -> Result<ResolveHandleResponse> {
        let handle = Handle::parse(&req.handle).map_err(XchandlesError::from)?;

        let (wallet, config, prefix, peer) = {
            let sage = sage.lock().await;
            let wallet = sage.wallet()?;
            let network = sage.network();
            let config = network
                .xchandles()
                .ok_or(XchandlesError::UnsupportedNetwork)?;

            let peer = {
                let state = sage.peer_state.lock().await;
                state.acquire_peer().and_then(|peer| {
                    let (peak_height, _) = state.peak_of(peer.socket_addr().ip())?;
                    Some((peer, peak_height))
                })
            };

            (wallet, config, network.prefix(), peer)
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before epoch")
            .as_secs();

        let lookup = async {
            let (peer, peak_height) = peer.ok_or(XchandlesError::NoPeers)?;
            let tip = chain_tip(&peer, peak_height).await?;
            let client = XchandlesClient::new(&config.api_url, config.registry_launcher_id)?;

            resolve_handle(
                &client,
                &peer,
                wallet.genesis_challenge,
                config.registry_launcher_id,
                &handle,
                tip,
            )
            .await
        };

        let resolution = resolve_and_store(
            &wallet.db,
            config.registry_launcher_id,
            &handle,
            now,
            lookup,
        )
        .await?;

        Ok(ResolveHandleResponse {
            handle: handle_record(&resolution, &prefix)?,
        })
    }
}

fn handle_record(resolution: &HandleResolution, prefix: &str) -> Result<HandleRecord> {
    let row = &resolution.row;
    let encode = |puzzle_hash| Address::new(puzzle_hash, prefix.to_string()).encode();

    Ok(HandleRecord {
        handle: row.handle.clone(),
        handle_hash: hex::encode(row.handle_hash),
        address: encode(row.resolved_puzzle_hash)?,
        status: if resolution.changed_from().is_some() {
            HandleStatus::Changed
        } else {
            HandleStatus::Verified
        },
        previous_address: resolution.changed_from().map(encode).transpose()?,
        last_sent_address: row.last_sent_puzzle_hash.map(encode).transpose()?,
        cached: resolution.cached,
        owner_launcher_id: hex::encode(row.owner_launcher_id),
        resolved_launcher_id: hex::encode(row.resolved_launcher_id),
        expiration: row.expiration,
        slot_coin_id: hex::encode(row.slot_coin_id),
        slot_confirmation_height: row.slot_confirmation_height,
        resolved_coin_id: hex::encode(row.resolved_coin_id),
        verified_height: row.verified_height,
        verified_timestamp: row.verified_timestamp,
        last_verified_at: row.last_verified_at,
    })
}
