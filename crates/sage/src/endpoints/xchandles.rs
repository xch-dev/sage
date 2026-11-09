use std::{
    collections::{BTreeSet, HashMap, HashSet},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chia_wallet_sdk::{prelude::*, utils::Address};
use futures_util::{StreamExt, future::join_all, stream};
use rand::seq::SliceRandom;
use sage_api::{
    AddressHandlesRecord, GetHandle, GetHandleResponse, GetHandles, GetHandlesResponse,
    HandleFilterMode, HandleRecord, HandleSlotRecord, HandleStatus, HandleVerification,
    LookupAddressHandles, LookupAddressHandlesResponse, OfferHandleNft, OfferHandleStatus,
    PendingTransferRecord, PendingTransferStatus, ResolveHandle, ResolveHandleResponse,
};
use sage_config::XchandlesConfig;
use sage_database::{
    NftUpdaterKindExt, ParsedNftMetadata, XchandleFilter, XchandleQuery, XchandleRow,
    XchandleStatus,
};
use sage_wallet::{
    Handle, HandleResolution, OFFLINE_MAX_AGE_SECONDS, PendingTransferResponse, RESOLVE_TIMEOUT,
    WalletPeer, XchandlesClient, XchandlesError, chain_tip, resolve_and_store, resolve_handle,
    verify_pending_transfer,
};
use tokio::time::timeout;
use tracing::{debug, warn};

use crate::{Error, Result, Sage, parse_hash, unconfirmed_change};

/// The most addresses `lookup_address_handles` accepts at once.
pub const MAX_LOOKUP_ADDRESSES: usize = 1000;

/// The most handle NFTs `get_handles` verifies when refreshing owned handles. Anyone can send
/// NFTs to a wallet, so this bounds how many lookups they can cause.
const MAX_REFRESH_HANDLES: usize = 50;

/// How many handles are verified at the same time while refreshing.
const REFRESH_CONCURRENCY: usize = 4;

/// The longest refreshing owned handles may take, since the Sage lock is held throughout.
/// Lookups which haven't finished by then are abandoned.
const REFRESH_TIMEOUT: Duration = Duration::from_secs(30);

/// The most handle NFTs in an offer which are checked against the registry.
const MAX_OFFER_HANDLE_NFTS: usize = 8;

type LookupResult<T> = std::result::Result<T, XchandlesError>;

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time before epoch")
        .as_secs()
}

fn parse_handle(input: &str) -> Result<Handle> {
    Ok(Handle::parse(input).map_err(XchandlesError::from)?)
}

impl Sage {
    pub async fn resolve_handle(&self, req: ResolveHandle) -> Result<ResolveHandleResponse> {
        let handle = parse_handle(&req.handle)?;
        let resolution = self.resolve_handle_mapping(&handle).await?;

        Ok(ResolveHandleResponse {
            handle: self.handle_record(&resolution)?,
        })
    }

    /// Verifies a handle and records the outcome in the store, falling back to a recently
    /// verified mapping if it can't be verified right now.
    pub(crate) async fn resolve_handle_mapping(&self, handle: &Handle) -> Result<HandleResolution> {
        let config = self.xchandles_config()?;
        self.resolve_verified(&config, handle, unix_now()).await
    }

    pub async fn get_handle(&self, req: GetHandle) -> Result<GetHandleResponse> {
        let wallet = self.wallet()?;
        let handle = parse_handle(&req.handle)?;
        let config = self.xchandles_config()?;
        let now = unix_now();

        // Neither depends on the other, so the pending transfer is checked while the handle
        // is being resolved.
        let (resolution, pending) = tokio::join!(
            self.resolve_verified(&config, &handle, now),
            self.pending_transfer(&config, wallet.genesis_challenge, &handle),
        );
        let resolution = resolution?;
        let row = &resolution.row;

        let (pending_transfer_status, pending_transfer) = match pending {
            Ok(None) => (PendingTransferStatus::None, None),
            Ok(Some((transfer, update_slot_coin_id))) => (
                if update_slot_coin_id.is_some() {
                    PendingTransferStatus::Verified
                } else {
                    PendingTransferStatus::Unverified
                },
                Some(PendingTransferRecord {
                    new_owner_launcher_id: hex::encode(transfer.new_owner_launcher_id),
                    new_resolved_launcher_id: hex::encode(transfer.new_resolved_launcher_id),
                    minimum_execution_height: transfer.minimum_execution_height,
                    initiator_coin_id: hex::encode(transfer.initiator_coin_id),
                    executor_coin_id: hex::encode(transfer.executor_coin_id),
                    update_confirmation_height: transfer.update_confirmation_height,
                    update_slot_coin_id: update_slot_coin_id.map(hex::encode),
                }),
            ),
            Err(error) => {
                warn!("Failed to check for a pending transfer of {handle}: {error}");
                (PendingTransferStatus::Unavailable, None)
            }
        };

        Ok(GetHandleResponse {
            handle: self.handle_record_at(&resolution, now)?,
            slot: HandleSlotRecord {
                registry_launcher_id: hex::encode(row.registry_launcher_id),
                counter: row.slot_counter,
                left_handle_hash: hex::encode(row.slot_left_handle_hash),
                right_handle_hash: hex::encode(row.slot_right_handle_hash),
                parent_id: hex::encode(row.slot_parent_id),
                parent_parent_id: hex::encode(row.slot_parent_parent_id),
                parent_inner_puzzle_hash: hex::encode(row.slot_parent_inner_puzzle_hash),
                indexed_height: row.indexed_height,
            },
            pending_transfer_status,
            pending_transfer,
        })
    }

    pub async fn get_handles(&self, req: GetHandles) -> Result<GetHandlesResponse> {
        let wallet = self.wallet()?;
        let config = self.xchandles_config()?;

        if req.refresh {
            self.refresh_owned_handles(&config).await?;
        }

        let now = unix_now();

        let filter = match req.filter {
            HandleFilterMode::All => XchandleFilter::All,
            HandleFilterMode::Owned => XchandleFilter::Owned,
            HandleFilterMode::Contacts => XchandleFilter::Contacts,
        };

        // Handle mappings are shared by every wallet on the network, but which handles are
        // owned depends on the NFTs and DIDs in this wallet.
        let owned_launcher_ids = if filter == XchandleFilter::Owned {
            wallet.db.owned_singleton_launcher_ids().await?
        } else {
            Vec::new()
        };

        // An address or puzzle hash matches the handles which resolve to it, and `@handle`
        // matches by the handle.
        let search = req
            .search
            .as_deref()
            .map(str::trim)
            .filter(|search| !search.is_empty());
        let puzzle_hash = search.and_then(|search| self.parse_lookup_address(search).ok());
        let search = search.map(|search| search.strip_prefix('@').unwrap_or(search));

        let (rows, total) = wallet
            .shared_db
            .xchandles_page(
                config.registry_launcher_id,
                XchandleQuery {
                    filter,
                    owned_launcher_ids: &owned_launcher_ids,
                    include_unverified: req.include_unverified,
                    search,
                    puzzle_hash,
                },
                now,
                req.limit,
                req.offset,
            )
            .await?;

        Ok(GetHandlesResponse {
            handles: rows
                .into_iter()
                .map(|row| self.handle_record_at(&HandleResolution { row, cached: true }, now))
                .collect::<Result<_>>()?,
            total,
        })
    }

    pub async fn lookup_address_handles(
        &self,
        req: LookupAddressHandles,
    ) -> Result<LookupAddressHandlesResponse> {
        let wallet = self.wallet()?;
        let config = self.xchandles_config()?;

        if req.addresses.len() > MAX_LOOKUP_ADDRESSES {
            return Err(Error::TooManyAddresses(MAX_LOOKUP_ADDRESSES));
        }

        let puzzle_hashes = req
            .addresses
            .iter()
            .map(|address| self.parse_lookup_address(address))
            .collect::<Result<Vec<_>>>()?;

        let now = unix_now();
        let mut handles = HashMap::new();

        for &puzzle_hash in &puzzle_hashes {
            if handles.contains_key(&puzzle_hash) {
                continue;
            }

            let records = wallet
                .shared_db
                .xchandles_by_puzzle_hash(config.registry_launcher_id, puzzle_hash)
                .await?
                .into_iter()
                .filter(|row| {
                    matches!(
                        verification(row, now),
                        HandleVerification::Verified | HandleVerification::Outdated
                    )
                })
                .map(|row| self.handle_record_at(&HandleResolution { row, cached: true }, now))
                .collect::<Result<Vec<_>>>()?;

            handles.insert(puzzle_hash, records);
        }

        Ok(LookupAddressHandlesResponse {
            results: req
                .addresses
                .into_iter()
                .zip(puzzle_hashes)
                .map(|(address, puzzle_hash)| AddressHandlesRecord {
                    address,
                    handles: handles[&puzzle_hash].clone(),
                })
                .collect(),
        })
    }

    /// Parses an address on the current network or a hex puzzle hash. This deliberately
    /// doesn't accept handles, since looking up a handle would make network requests.
    pub(crate) fn parse_lookup_address(&self, input: &str) -> Result<Bytes32> {
        let hex = input.strip_prefix("0x").unwrap_or(input);

        if hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return parse_hash(input.to_string());
        }

        let address = Address::decode(input)?;

        if address.prefix != self.network().prefix() {
            return Err(Error::AddressPrefix(address.prefix));
        }

        Ok(address.puzzle_hash)
    }

    fn xchandles_config(&self) -> Result<XchandlesConfig> {
        Ok(self
            .network()
            .xchandles()
            .ok_or(XchandlesError::UnsupportedNetwork)?)
    }

    async fn verification_peer(&self) -> LookupResult<(WalletPeer, u32)> {
        let state = self.peer_state.lock().await;
        let peer = state.acquire_peer().ok_or(XchandlesError::NoPeers)?;
        let (peak_height, _) = state
            .peak_of(peer.socket_addr().ip())
            .ok_or(XchandlesError::NoPeers)?;
        Ok((peer, peak_height))
    }

    fn xchandles_client(&self, config: &XchandlesConfig) -> LookupResult<XchandlesClient> {
        Ok(XchandlesClient::new(
            self.http_client.clone(),
            &config.api_url,
            config.registry_launcher_id,
        )?)
    }

    /// Verifies a handle against the chain and records the outcome in the store, falling
    /// back to a recently verified mapping only if the handle can't be verified right now.
    async fn resolve_verified(
        &self,
        config: &XchandlesConfig,
        handle: &Handle,
        now: u64,
    ) -> Result<HandleResolution> {
        let wallet = self.wallet()?;

        let lookup = async {
            let (peer, peak_height) = self.verification_peer().await?;
            let client = self.xchandles_client(config)?;

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

        Ok(resolve_and_store(
            &wallet.shared_db,
            config.registry_launcher_id,
            handle,
            now,
            lookup,
        )
        .await?)
    }

    /// Fetches the pending transfer the API reports for a handle, if any, along with the
    /// update slot which verifies it on chain.
    async fn pending_transfer(
        &self,
        config: &XchandlesConfig,
        genesis_challenge: Bytes32,
        handle: &Handle,
    ) -> LookupResult<Option<(PendingTransferResponse, Option<Bytes32>)>> {
        let check = async {
            let client = self.xchandles_client(config)?;

            let Some(transfer) = client.pending_transfer(handle).await? else {
                return Ok(None);
            };

            let (peer, _) = self.verification_peer().await?;

            let update_slot_coin_id = verify_pending_transfer(
                &peer,
                genesis_challenge,
                config.registry_launcher_id,
                handle,
                &transfer,
            )
            .await?;

            Ok(Some((transfer, update_slot_coin_id)))
        };

        timeout(RESOLVE_TIMEOUT, Box::pin(check))
            .await
            .unwrap_or(Err(XchandlesError::Timeout))
    }

    /// Verifies the handles named by `XCHandles` NFTs in this wallet, so that the handles it
    /// owns are in the store. Display names are owner-editable, so they're only used to
    /// decide which handles to look up. Whether the wallet owns a handle is decided by the
    /// verified registry slot.
    async fn refresh_owned_handles(&self, config: &XchandlesConfig) -> Result<()> {
        let wallet = self.wallet()?;

        let names: BTreeSet<Handle> = wallet
            .db
            .owned_handle_nft_display_names()
            .await?
            .iter()
            .filter_map(|name| Handle::parse(name).ok())
            .collect();

        let owned_launcher_ids = wallet.db.owned_singleton_launcher_ids().await?;

        let owned: HashSet<String> = wallet
            .shared_db
            .owned_xchandle_names(config.registry_launcher_id, &owned_launcher_ids)
            .await?
            .into_iter()
            .collect();

        // Anyone can send NFTs to a wallet, and pick their names. Handles already verified as
        // owned by this wallet are refreshed first, and the rest are picked at random, so that
        // NFTs with names which sort first can't crowd out the wallet's real handles.
        let (mut handles, mut others): (Vec<Handle>, Vec<Handle>) = names
            .into_iter()
            .partition(|handle| owned.contains(handle.as_str()));
        others.shuffle(&mut rand::thread_rng());
        handles.extend(others);

        if handles.len() > MAX_REFRESH_HANDLES {
            warn!(
                "Only refreshing {MAX_REFRESH_HANDLES} of {} possible owned handles",
                handles.len()
            );
        }

        let now = unix_now();

        let refresh = stream::iter(handles.into_iter().take(MAX_REFRESH_HANDLES))
            .map(|handle| async move {
                if let Err(error) = self.resolve_verified(config, &handle, now).await {
                    debug!("Couldn't verify possibly owned handle {handle}: {error}");
                }
            })
            .buffer_unordered(REFRESH_CONCURRENCY)
            .collect::<Vec<()>>();

        if timeout(REFRESH_TIMEOUT, refresh).await.is_err() {
            warn!("Timed out while refreshing owned handles");
        }

        Ok(())
    }

    pub(crate) fn handle_record(&self, resolution: &HandleResolution) -> Result<HandleRecord> {
        self.handle_record_at(resolution, unix_now())
    }

    pub(crate) fn handle_record_at(
        &self,
        resolution: &HandleResolution,
        now: u64,
    ) -> Result<HandleRecord> {
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
            verification: verification(row, now),
            verification_reason: row.status_reason.clone(),
            owner_launcher_id: hex::encode(row.owner_launcher_id),
            resolved_launcher_id: hex::encode(row.resolved_launcher_id),
            expiration: row.expiration,
            slot_coin_id: hex::encode(row.slot_coin_id),
            slot_confirmation_height: row.slot_confirmation_height,
            resolved_coin_id: hex::encode(row.resolved_coin_id),
            resolved_confirmation_height: row.resolved_confirmation_height,
            updated_height: row
                .slot_confirmation_height
                .max(row.resolved_confirmation_height),
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

/// Whether a stored mapping can be relied on. A handle counts as expired once either the
/// local clock or the peak it was verified at reaches its expiration, and as outdated once
/// it's too old to be used in place of a lookup.
fn verification(row: &XchandleRow, now: u64) -> HandleVerification {
    match row.status {
        XchandleStatus::Verified if row.expiration <= now.max(row.verified_timestamp) => {
            HandleVerification::Expired
        }
        XchandleStatus::Verified
            if now.saturating_sub(row.last_verified_at) > OFFLINE_MAX_AGE_SECONDS =>
        {
            HandleVerification::Outdated
        }
        XchandleStatus::Verified => HandleVerification::Verified,
        XchandleStatus::Unverified => HandleVerification::Unverified,
        XchandleStatus::Stale => HandleVerification::Stale,
    }
}
