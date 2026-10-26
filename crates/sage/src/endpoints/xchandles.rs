use std::time::{SystemTime, UNIX_EPOCH};

use chia_wallet_sdk::{prelude::*, utils::Address};
use futures_util::future::join_all;
use sage_api::{
    HandleRecord, HandleStatus, OfferHandleNft, OfferHandleStatus, ResolveHandle,
    ResolveHandleResponse,
};
use sage_database::{NftUpdaterKindExt, ParsedNftMetadata};
use sage_wallet::{
    Handle, HandleResolution, RESOLVE_TIMEOUT, XchandlesClient, XchandlesError, chain_tip,
    resolve_and_store, resolve_handle,
};
use tokio::time::timeout;

use crate::{Error, Result, Sage, unconfirmed_change};

/// The most handle NFTs in an offer which are checked against the registry.
const MAX_OFFER_HANDLE_NFTS: usize = 8;

impl Sage {
    pub async fn resolve_handle(&self, req: ResolveHandle) -> Result<ResolveHandleResponse> {
        let handle = Handle::parse(&req.handle).map_err(XchandlesError::from)?;
        let resolution = self.resolve_handle_mapping(&handle).await?;

        Ok(ResolveHandleResponse {
            handle: self.handle_record(&resolution)?,
        })
    }

    /// Verifies a handle and records the outcome in the store, falling back to a recently
    /// verified mapping if it can't be verified right now.
    pub(crate) async fn resolve_handle_mapping(&self, handle: &Handle) -> Result<HandleResolution> {
        let wallet = self.wallet()?;
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

            let client = XchandlesClient::new(
                self.http_client.clone(),
                &config.api_url,
                config.registry_launcher_id,
            )?;

            resolve_handle(
                &client,
                &peer,
                wallet.genesis_challenge,
                config.registry_launcher_id,
                handle,
                async { Ok(chain_tip(&peer, peak_height).await?) },
            )
            .await
        };

        let lookup = async {
            timeout(RESOLVE_TIMEOUT, Box::pin(lookup))
                .await
                .unwrap_or(Err(XchandlesError::Timeout))
        };

        Ok(resolve_and_store(&wallet.db, config.registry_launcher_id, handle, now, lookup).await?)
    }

    pub(crate) fn handle_record(&self, resolution: &HandleResolution) -> Result<HandleRecord> {
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
            changed_from: unconfirmed_change(resolution, None)
                .map(encode)
                .transpose()?,
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

    /// Checks offered NFTs which claim to be handle NFTs against the registry. Their
    /// display name is set by their owner, so it's only used to find which handle to check,
    /// and the registry must confirm that the NFT owns that handle. Only display names
    /// written as `@handle` are treated as claims, so that ordinary NFT names aren't.
    ///
    /// A handle whose owner or resolved singleton is about to be updated could be taken
    /// back by the seller after the trade, so that is reported as a pending transfer.
    ///
    /// The claims are checked concurrently, and at most [`MAX_OFFER_HANDLE_NFTS`] of them,
    /// so that an offer can't hold up the wallet with many lookups. Any beyond that are
    /// reported as unavailable, which blocks taking the offer.
    pub(crate) async fn check_offer_handle_nfts(
        &self,
        spend_bundle: &SpendBundle,
    ) -> Result<Vec<OfferHandleNft>> {
        let Some(config) = self.network().xchandles() else {
            return Ok(Vec::new());
        };

        let mut ctx = SpendContext::new();
        let offer = Offer::from_spend_bundle(&mut ctx, spend_bundle)?;

        let mut claimed = Vec::new();

        for (&launcher_id, nft) in &offer.offered_coins().nfts {
            let Some(metadata) = ParsedNftMetadata::parse(
                &ctx,
                nft.info.metadata.ptr(),
                nft.info.metadata_updater_kind(),
            ) else {
                continue;
            };

            let Some(Ok(handle)) = metadata
                .display_name
                .as_deref()
                .and_then(|name| Handle::from_input(name.trim()))
            else {
                continue;
            };

            claimed.push((launcher_id, handle));
        }

        let client = XchandlesClient::new(
            self.http_client.clone(),
            &config.api_url,
            config.registry_launcher_id,
        )
        .map_err(XchandlesError::from)?;

        let checked = claimed.len().min(MAX_OFFER_HANDLE_NFTS);

        let results = join_all(claimed[..checked].iter().map(|(launcher_id, handle)| {
            self.check_offer_handle_nft(&client, *launcher_id, handle)
        }))
        .await;

        let mut handle_nfts = Vec::with_capacity(claimed.len());

        for ((launcher_id, handle), result) in claimed[..checked].iter().zip(results) {
            let Some((status, reason)) = result? else {
                continue;
            };

            handle_nfts.push(OfferHandleNft {
                nft_id: Address::new(*launcher_id, "nft".to_string()).encode()?,
                handle: handle.as_str().to_string(),
                status,
                reason,
            });
        }

        for (launcher_id, handle) in &claimed[checked..] {
            handle_nfts.push(OfferHandleNft {
                nft_id: Address::new(*launcher_id, "nft".to_string()).encode()?,
                handle: handle.as_str().to_string(),
                status: OfferHandleStatus::Unavailable,
                reason: Some(format!(
                    "This offer claims more than {MAX_OFFER_HANDLE_NFTS} handles, so {handle} \
                    wasn't checked"
                )),
            });
        }

        Ok(handle_nfts)
    }

    /// Checks a single offered NFT's handle claim. Returns `None` if handles aren't
    /// supported on this network.
    async fn check_offer_handle_nft(
        &self,
        client: &XchandlesClient,
        launcher_id: Bytes32,
        handle: &Handle,
    ) -> Result<Option<(OfferHandleStatus, Option<String>)>> {
        Ok(Some(match self.resolve_handle_mapping(handle).await {
            Ok(resolution) if resolution.row.owner_launcher_id != launcher_id => (
                OfferHandleStatus::NotAssociated,
                Some(format!(
                    "The registry doesn't list this NFT as the owner of {handle}"
                )),
            ),
            Ok(_) => match timeout(RESOLVE_TIMEOUT, client.pending_transfer(handle)).await {
                Ok(Ok(None)) => (OfferHandleStatus::Verified, None),
                Ok(Ok(Some(transfer))) => (
                    OfferHandleStatus::PendingTransfer,
                    Some(format!(
                        "An update to {handle} is pending, which moves it to the owner {} and \
                        can be completed at height {}",
                        Address::new(transfer.new_owner_launcher_id, "nft".to_string()).encode()?,
                        transfer.minimum_execution_height
                    )),
                ),
                Ok(Err(error)) => (
                    OfferHandleStatus::Unavailable,
                    Some(format!(
                        "Couldn't check {handle} for a pending transfer: {error}"
                    )),
                ),
                Err(_) => (
                    OfferHandleStatus::Unavailable,
                    Some(format!(
                        "Timed out checking {handle} for a pending transfer"
                    )),
                ),
            },
            Err(Error::Xchandles(XchandlesError::UnsupportedNetwork)) => return Ok(None),
            Err(Error::Xchandles(error)) if error.is_not_registered() || error.is_rejection() => (
                OfferHandleStatus::NotAssociated,
                Some(format!("{handle} couldn't be verified: {error}")),
            ),
            Err(Error::Xchandles(error)) if error.is_unavailable() => (
                OfferHandleStatus::Unavailable,
                Some(format!("{handle} couldn't be verified right now: {error}")),
            ),
            Err(error) => return Err(error),
        }))
    }
}
