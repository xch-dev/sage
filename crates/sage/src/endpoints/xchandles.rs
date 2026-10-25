use std::time::{SystemTime, UNIX_EPOCH};

use chia_wallet_sdk::utils::Address;
use sage_api::{HandleRecord, HandleStatus, ResolveHandle, ResolveHandleResponse};
use sage_wallet::{
    Handle, HandleResolution, XchandlesClient, XchandlesError, chain_tip, resolve_and_store,
    resolve_handle,
};

use crate::{Result, Sage};

impl Sage {
    pub async fn resolve_handle(&self, req: ResolveHandle) -> Result<ResolveHandleResponse> {
        let wallet = self.wallet()?;
        let handle = Handle::parse(&req.handle).map_err(XchandlesError::from)?;
        let config = self
            .network()
            .xchandles()
            .ok_or(XchandlesError::UnsupportedNetwork)?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before epoch")
            .as_secs();

        let lookup = async {
            let (peer, peak_height) = {
                let state = self.peer_state.lock().await;
                let peer = state.acquire_peer().ok_or(XchandlesError::NoPeers)?;
                let (peak_height, _) = state
                    .peak_of(peer.socket_addr().ip())
                    .ok_or(XchandlesError::NoPeers)?;
                (peer, peak_height)
            };

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
            handle: self.handle_record(&resolution)?,
        })
    }

    fn handle_record(&self, resolution: &HandleResolution) -> Result<HandleRecord> {
        let row = &resolution.row;
        let prefix = self.network().prefix();
        let encode = |puzzle_hash| Address::new(puzzle_hash, prefix.clone()).encode();

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
}
