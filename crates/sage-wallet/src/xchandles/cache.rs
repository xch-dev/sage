//! The verified handle store.
//!
//! Policy:
//!
//! - Every lookup is verified against the chain when the API and peers are reachable. A
//!   stored mapping never stands in for a lookup that succeeded or was rejected. A handle
//!   counts as expired once either the peak or the local clock reaches its expiration.
//! - When the API or peers are unavailable, a mapping that was verified within
//!   [`OFFLINE_MAX_AGE_SECONDS`] is returned instead, marked as cached, as long as it has
//!   not expired by the later of the local clock and the peak it was verified at. This
//!   includes rate limiting, server errors, unparseable responses, and an API whose view
//!   of the chain is behind.
//! - A lookup whose answer is rejected, or which shows the handle is no longer registered,
//!   marks the stored mapping unverified, and a reorg below anything it was verified against
//!   marks it stale. Neither can be used offline until verified again. Lookups which fail
//!   for any other reason leave the stored mapping untouched.
//! - Mappings are stored per registry launcher ID, and each wallet database belongs to a
//!   single network, so mappings never cross deployments or networks.
//! - If a verification finds a different resolved address than the last verified one, the
//!   old address is kept as a pending change until it is explicitly acknowledged.

use std::future::Future;

use chia_wallet_sdk::prelude::*;
use sage_database::{Database, XchandleRow, XchandleStatus};
use tracing::warn;

use super::{Handle, VerifiedHandle, XchandlesError};

/// How long a verified mapping may be used while the API or peers are unavailable.
pub const OFFLINE_MAX_AGE_SECONDS: u64 = 24 * 60 * 60;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandleResolution {
    pub row: XchandleRow,
    /// Whether the mapping comes from an earlier verification because the handle couldn't
    /// be verified right now.
    pub cached: bool,
}

impl HandleResolution {
    /// The last verified address, if it differs from the current one and the change
    /// hasn't been acknowledged yet.
    pub fn changed_from(&self) -> Option<Bytes32> {
        self.row.previous_resolved_puzzle_hash
    }
}

/// Builds the stored mapping for a successful verification, carrying over the pending
/// change and last sent address from the previous mapping.
pub fn verified_row(
    previous: Option<&XchandleRow>,
    verified: &VerifiedHandle,
    now: u64,
) -> XchandleRow {
    let new_puzzle_hash = verified.resolved_puzzle_hash;

    let (previous_resolved_launcher_id, previous_resolved_puzzle_hash) = match previous {
        // An unacknowledged change keeps the address the user last knew about, unless the
        // handle has gone back to resolving to it.
        Some(XchandleRow {
            previous_resolved_launcher_id,
            previous_resolved_puzzle_hash: Some(pending),
            ..
        }) => {
            if *pending == new_puzzle_hash {
                (None, None)
            } else {
                (*previous_resolved_launcher_id, Some(*pending))
            }
        }
        Some(previous) if previous.resolved_puzzle_hash != new_puzzle_hash => (
            Some(previous.resolved_launcher_id),
            Some(previous.resolved_puzzle_hash),
        ),
        _ => (None, None),
    };

    XchandleRow {
        registry_launcher_id: verified.registry_launcher_id,
        handle: verified.handle.as_str().to_string(),
        handle_hash: verified.handle_hash(),
        owner_launcher_id: verified.owner_launcher_id(),
        resolved_launcher_id: verified.resolved_launcher_id(),
        resolved_puzzle_hash: new_puzzle_hash,
        expiration: verified.expiration(),
        slot_coin_id: verified.slot_coin_id,
        slot_counter: verified.slot_value.counter,
        slot_left_handle_hash: verified.slot_value.neighbors.left_value,
        slot_right_handle_hash: verified.slot_value.neighbors.right_value,
        slot_parent_id: verified.slot_parent_id,
        slot_parent_parent_id: verified.slot_proof.parent_parent_coin_info,
        slot_parent_inner_puzzle_hash: verified.slot_proof.parent_inner_puzzle_hash,
        slot_confirmation_height: verified.slot_confirmation_height,
        resolved_coin_id: verified.resolved_coin_id,
        resolved_confirmation_height: verified.resolved_confirmation_height,
        indexed_height: verified.indexed_height,
        verified_height: verified.tip.height,
        verified_timestamp: verified.tip.timestamp,
        last_verified_at: now,
        status: XchandleStatus::Verified,
        status_reason: None,
        previous_resolved_launcher_id,
        previous_resolved_puzzle_hash,
        last_sent_puzzle_hash: previous.and_then(|previous| previous.last_sent_puzzle_hash),
    }
}

/// Whether a stored mapping may be used in place of a lookup that couldn't be performed.
pub fn usable_offline(row: &XchandleRow, now: u64) -> bool {
    row.status == XchandleStatus::Verified
        && now.saturating_sub(row.last_verified_at) <= OFFLINE_MAX_AGE_SECONDS
        && row.expiration > now.max(row.verified_timestamp)
}

/// Runs a verified lookup and records its outcome in the store, falling back to the
/// stored mapping only if the lookup couldn't be performed.
///
/// The stored mapping is read after the lookup, in the same transaction as the write, so
/// changes made while the lookup was in flight (such as recording the address last sent
/// to, or acknowledging a change) aren't overwritten.
pub async fn resolve_and_store(
    db: &Database,
    registry_launcher_id: Bytes32,
    handle: &Handle,
    now: u64,
    lookup: impl Future<Output = Result<VerifiedHandle, XchandlesError>>,
) -> Result<HandleResolution, XchandlesError> {
    // The peak timestamp comes from a peer which may be behind, so the handle must not have
    // expired by the local clock either.
    let result = lookup.await.and_then(|verified| {
        if verified.expiration() <= now {
            Err(XchandlesError::Expired {
                expiration: verified.expiration(),
                timestamp: now,
            })
        } else {
            Ok(verified)
        }
    });

    let error = match result {
        Ok(verified) => {
            let mut tx = db.tx().await?;
            let previous = tx.xchandle(registry_launcher_id, handle.as_str()).await?;
            let row = verified_row(previous.as_ref(), &verified, now);
            tx.upsert_xchandle(&row).await?;
            tx.commit().await?;
            return Ok(HandleResolution { row, cached: false });
        }
        Err(error) => error,
    };

    if error.is_unavailable() {
        if let Some(row) = db.xchandle(registry_launcher_id, handle.as_str()).await?
            && usable_offline(&row, now)
        {
            return Ok(HandleResolution { row, cached: true });
        }
    } else if error.is_rejection() {
        // Only affects an existing mapping, keeping its last verified values. Failing to
        // record this must not hide why the lookup was rejected.
        if let Err(db_error) = db
            .set_xchandle_status(
                registry_launcher_id,
                handle.as_str(),
                XchandleStatus::Unverified,
                Some(&error.to_string()),
            )
            .await
        {
            warn!("Failed to mark the mapping for {handle} as unverified: {db_error}");
        }
    }

    Err(error)
}
