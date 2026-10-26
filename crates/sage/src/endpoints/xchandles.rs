use std::{
    collections::{BTreeSet, HashMap, HashSet},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use chia_wallet_sdk::{prelude::*, utils::Address};
use futures_util::{StreamExt, stream};
use rand::seq::SliceRandom;
use sage_api::{
    AddressHandlesRecord, GetHandle, GetHandleResponse, GetHandles, GetHandlesResponse,
    HandleFilterMode, HandleRecord, HandleSlotRecord, HandleStatus, HandleVerification,
    LookupAddressHandles, LookupAddressHandlesResponse, PendingTransferRecord,
    PendingTransferStatus, ResolveHandle, ResolveHandleResponse,
};
use sage_config::XchandlesConfig;
use sage_database::{XchandleRow, XchandleStatus};
use sage_wallet::{
    Handle, HandleResolution, OFFLINE_MAX_AGE_SECONDS, PendingTransferResponse, RESOLVE_TIMEOUT,
    WalletPeer, XchandlesClient, XchandlesError, chain_tip, resolve_and_store, resolve_handle,
    verify_pending_transfer,
};
use tokio::time::timeout;
use tracing::{debug, warn};

use crate::{Error, Result, Sage, parse_hash};

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
        let config = self.xchandles_config()?;
        let now = unix_now();
        let resolution = self.resolve_verified(&config, &handle, now).await?;

        Ok(ResolveHandleResponse {
            handle: self.handle_record(&resolution.row, resolution.cached, now)?,
        })
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
            handle: self.handle_record(row, resolution.cached, now)?,
            slot: HandleSlotRecord {
                registry_launcher_id: hex::encode(row.registry_launcher_id),
                counter: row.slot_counter,
                left_handle_hash: hex::encode(row.slot_left_handle_hash),
                right_handle_hash: hex::encode(row.slot_right_handle_hash),
                parent_id: hex::encode(row.slot_parent_id),
                parent_parent_id: hex::encode(row.slot_parent_parent_id),
                parent_inner_puzzle_hash: hex::encode(row.slot_parent_inner_puzzle_hash),
                resolved_confirmation_height: row.resolved_confirmation_height,
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

        let (rows, total) = wallet
            .db
            .xchandles_page(
                config.registry_launcher_id,
                req.filter == HandleFilterMode::Owned,
                req.include_unverified,
                now,
                req.limit,
                req.offset,
            )
            .await?;

        Ok(GetHandlesResponse {
            handles: rows
                .iter()
                .map(|row| self.handle_record(row, true, now))
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
                .db
                .xchandles_by_puzzle_hash(config.registry_launcher_id, puzzle_hash)
                .await?
                .iter()
                .filter(|row| {
                    matches!(
                        verification(row, now),
                        HandleVerification::Verified | HandleVerification::Outdated
                    )
                })
                .map(|row| self.handle_record(row, true, now))
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
    fn parse_lookup_address(&self, input: &str) -> Result<Bytes32> {
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

        Ok(resolve_and_store(&wallet.db, config.registry_launcher_id, handle, now, lookup).await?)
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

        let owned: HashSet<String> = wallet
            .db
            .owned_xchandle_names(config.registry_launcher_id)
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

    fn handle_record(&self, row: &XchandleRow, cached: bool, now: u64) -> Result<HandleRecord> {
        let prefix = self.network().prefix();
        let encode = |puzzle_hash| Address::new(puzzle_hash, prefix.clone()).encode();

        Ok(HandleRecord {
            handle: row.handle.clone(),
            handle_hash: hex::encode(row.handle_hash),
            address: encode(row.resolved_puzzle_hash)?,
            status: if row.previous_resolved_puzzle_hash.is_some() {
                HandleStatus::Changed
            } else {
                HandleStatus::Verified
            },
            previous_address: row.previous_resolved_puzzle_hash.map(encode).transpose()?,
            last_sent_address: row.last_sent_puzzle_hash.map(encode).transpose()?,
            cached,
            verification: verification(row, now),
            verification_reason: row.status_reason.clone(),
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
