use chia_wallet_sdk::prelude::*;
use sqlx::{SqliteExecutor, SqlitePool};

use crate::{Convert, Database, DatabaseError, DatabaseTx, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum XchandleStatus {
    /// The values were verified against the chain by the most recent lookup.
    Verified = 0,
    /// The most recent lookup was rejected, so the stored values can't be relied on.
    Unverified = 1,
    /// The values were verified, but must be verified again before they're used, for
    /// example because a reorg may have undone the coins they were verified against.
    Stale = 2,
}

impl TryFrom<i64> for XchandleStatus {
    type Error = DatabaseError;

    fn try_from(value: i64) -> Result<Self> {
        Ok(match value {
            0 => Self::Verified,
            1 => Self::Unverified,
            2 => Self::Stale,
            _ => return Err(DatabaseError::InvalidEnumVariant),
        })
    }
}

/// Which stored handle mappings to list.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum XchandleFilter {
    #[default]
    All = 0,
    /// Handles whose owner singleton is an NFT or DID owned by this wallet.
    Owned = 1,
    /// Handles saved in the address book.
    Contacts = 2,
}

/// A handle mapping, holding the values from its last successful verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XchandleRow {
    pub registry_launcher_id: Bytes32,
    pub handle: String,
    pub handle_hash: Bytes32,
    pub owner_launcher_id: Bytes32,
    pub resolved_launcher_id: Bytes32,
    pub resolved_puzzle_hash: Bytes32,
    pub expiration: u64,
    pub slot_coin_id: Bytes32,
    pub slot_counter: u64,
    pub slot_left_handle_hash: Bytes32,
    pub slot_right_handle_hash: Bytes32,
    pub slot_parent_id: Bytes32,
    pub slot_parent_parent_id: Bytes32,
    pub slot_parent_inner_puzzle_hash: Bytes32,
    pub slot_confirmation_height: u32,
    pub resolved_coin_id: Bytes32,
    pub resolved_confirmation_height: u32,
    pub indexed_height: u32,
    pub verified_height: u32,
    pub verified_timestamp: u64,
    pub last_verified_at: u64,
    pub status: XchandleStatus,
    pub status_reason: Option<String>,
    pub previous_resolved_launcher_id: Option<Bytes32>,
    pub previous_resolved_puzzle_hash: Option<Bytes32>,
    pub last_sent_puzzle_hash: Option<Bytes32>,
}

impl Database {
    pub async fn xchandle(
        &self,
        registry_launcher_id: Bytes32,
        handle: &str,
    ) -> Result<Option<XchandleRow>> {
        xchandle(&self.pool, registry_launcher_id, handle).await
    }

    /// Verified handles which resolve to the given puzzle hash.
    pub async fn xchandles_by_puzzle_hash(
        &self,
        registry_launcher_id: Bytes32,
        puzzle_hash: Bytes32,
    ) -> Result<Vec<XchandleRow>> {
        xchandles_by_puzzle_hash(&self.pool, registry_launcher_id, puzzle_hash).await
    }

    /// A page of stored handle mappings, sorted by handle, and the total number matching.
    ///
    /// Unless `include_unverified` is set, only verified mappings which haven't expired by
    /// `now` or the peak they were verified at are included.
    pub async fn xchandles_page(
        &self,
        registry_launcher_id: Bytes32,
        filter: XchandleFilter,
        include_unverified: bool,
        now: u64,
        limit: u32,
        offset: u32,
    ) -> Result<(Vec<XchandleRow>, u32)> {
        xchandles_page(
            &self.pool,
            registry_launcher_id,
            filter,
            include_unverified,
            now,
            limit,
            offset,
        )
        .await
    }

    /// The display names of NFTs owned by this wallet which use the `XCHandles` metadata
    /// updater. These are only hints about which handles the wallet may own.
    pub async fn owned_handle_nft_display_names(&self) -> Result<Vec<String>> {
        owned_handle_nft_display_names(&self.pool).await
    }

    /// The stored handles whose owner singleton is an NFT or DID owned by this wallet,
    /// whatever their status.
    pub async fn owned_xchandle_names(&self, registry_launcher_id: Bytes32) -> Result<Vec<String>> {
        owned_xchandle_names(&self.pool, registry_launcher_id).await
    }

    pub async fn upsert_xchandle(&self, row: &XchandleRow) -> Result<()> {
        upsert_xchandle(&self.pool, row).await
    }

    pub async fn set_xchandle_status(
        &self,
        registry_launcher_id: Bytes32,
        handle: &str,
        status: XchandleStatus,
        reason: Option<&str>,
    ) -> Result<()> {
        set_xchandle_status(&self.pool, registry_launcher_id, handle, status, reason).await
    }

    /// Marks verified handles as stale if anything they were verified against is in a block
    /// above the fork height, up to the peak height, which a reorg from that peak back to the
    /// fork height may have undone. Blocks above the peak weren't part of the reorged chain.
    pub async fn invalidate_xchandles_between(
        &self,
        fork_height: u32,
        peak_height: u32,
    ) -> Result<()> {
        invalidate_xchandles_between(&self.pool, fork_height, peak_height).await
    }

    pub async fn acknowledge_xchandle_change(
        &self,
        registry_launcher_id: Bytes32,
        handle: &str,
    ) -> Result<()> {
        acknowledge_xchandle_change(&self.pool, registry_launcher_id, handle).await
    }

    pub async fn set_xchandle_last_sent(
        &self,
        registry_launcher_id: Bytes32,
        handle: &str,
        puzzle_hash: Bytes32,
    ) -> Result<()> {
        set_xchandle_last_sent(&self.pool, registry_launcher_id, handle, puzzle_hash).await
    }
}

impl DatabaseTx<'_> {
    pub async fn xchandle(
        &mut self,
        registry_launcher_id: Bytes32,
        handle: &str,
    ) -> Result<Option<XchandleRow>> {
        xchandle(&mut *self.tx, registry_launcher_id, handle).await
    }

    pub async fn upsert_xchandle(&mut self, row: &XchandleRow) -> Result<()> {
        upsert_xchandle(&mut *self.tx, row).await
    }

    pub async fn acknowledge_xchandle_change(
        &mut self,
        registry_launcher_id: Bytes32,
        handle: &str,
    ) -> Result<()> {
        acknowledge_xchandle_change(&mut *self.tx, registry_launcher_id, handle).await
    }

    pub async fn set_xchandle_last_sent(
        &mut self,
        registry_launcher_id: Bytes32,
        handle: &str,
        puzzle_hash: Bytes32,
    ) -> Result<()> {
        set_xchandle_last_sent(&mut *self.tx, registry_launcher_id, handle, puzzle_hash).await
    }
}

macro_rules! xchandle_row {
    ($row:expr) => {{
        let row = $row;
        Ok(XchandleRow {
            registry_launcher_id: row.registry_launcher_id.convert()?,
            handle: row.handle,
            handle_hash: row.handle_hash.convert()?,
            owner_launcher_id: row.owner_launcher_id.convert()?,
            resolved_launcher_id: row.resolved_launcher_id.convert()?,
            resolved_puzzle_hash: row.resolved_puzzle_hash.convert()?,
            expiration: row.expiration.convert()?,
            slot_coin_id: row.slot_coin_id.convert()?,
            slot_counter: row.slot_counter.convert()?,
            slot_left_handle_hash: row.slot_left_handle_hash.convert()?,
            slot_right_handle_hash: row.slot_right_handle_hash.convert()?,
            slot_parent_id: row.slot_parent_id.convert()?,
            slot_parent_parent_id: row.slot_parent_parent_id.convert()?,
            slot_parent_inner_puzzle_hash: row.slot_parent_inner_puzzle_hash.convert()?,
            slot_confirmation_height: row.slot_confirmation_height.convert()?,
            resolved_coin_id: row.resolved_coin_id.convert()?,
            resolved_confirmation_height: row.resolved_confirmation_height.convert()?,
            indexed_height: row.indexed_height.convert()?,
            verified_height: row.verified_height.convert()?,
            verified_timestamp: row.verified_timestamp.convert()?,
            last_verified_at: row.last_verified_at.convert()?,
            status: row.status.try_into()?,
            status_reason: row.status_reason,
            previous_resolved_launcher_id: row.previous_resolved_launcher_id.convert()?,
            previous_resolved_puzzle_hash: row.previous_resolved_puzzle_hash.convert()?,
            last_sent_puzzle_hash: row.last_sent_puzzle_hash.convert()?,
        })
    }};
}

async fn xchandle(
    conn: impl SqliteExecutor<'_>,
    registry_launcher_id: Bytes32,
    handle: &str,
) -> Result<Option<XchandleRow>> {
    let registry_launcher_id = registry_launcher_id.as_ref();

    sqlx::query!(
        "SELECT * FROM xchandles WHERE registry_launcher_id = ? AND handle = ?",
        registry_launcher_id,
        handle
    )
    .fetch_optional(conn)
    .await?
    .map(|row| xchandle_row!(row))
    .transpose()
}

async fn xchandles_by_puzzle_hash(
    conn: impl SqliteExecutor<'_>,
    registry_launcher_id: Bytes32,
    puzzle_hash: Bytes32,
) -> Result<Vec<XchandleRow>> {
    let registry_launcher_id = registry_launcher_id.as_ref();
    let puzzle_hash = puzzle_hash.as_ref();

    sqlx::query!(
        "
        SELECT * FROM xchandles
        WHERE registry_launcher_id = ? AND resolved_puzzle_hash = ? AND status = 0
        ORDER BY handle ASC
        ",
        registry_launcher_id,
        puzzle_hash
    )
    .fetch_all(conn)
    .await?
    .into_iter()
    .map(|row| xchandle_row!(row))
    .collect()
}

async fn xchandles_page(
    pool: &SqlitePool,
    registry_launcher_id: Bytes32,
    filter: XchandleFilter,
    include_unverified: bool,
    now: u64,
    limit: u32,
    offset: u32,
) -> Result<(Vec<XchandleRow>, u32)> {
    let registry_launcher_id = registry_launcher_id.as_ref();
    let now: i64 = now.try_into()?;
    let filter = filter as u8;

    // The total is counted separately, so that it's still correct for a page past the end.
    let total = sqlx::query_scalar!(
        "
        SELECT COUNT(*) FROM xchandles
        WHERE registry_launcher_id = ?
            AND (? OR (status = 0 AND expiration > MAX(?, verified_timestamp)))
            AND (
                ? != 1 OR owner_launcher_id IN (
                    SELECT asset_hash FROM owned_coins WHERE asset_kind IN (1, 2)
                )
            )
            AND (? != 2 OR handle IN (SELECT handle FROM contacts WHERE kind = 1))
        ",
        registry_launcher_id,
        include_unverified,
        now,
        filter,
        filter
    )
    .fetch_one(pool)
    .await?;

    let rows = sqlx::query!(
        "
        SELECT * FROM xchandles
        WHERE registry_launcher_id = ?
            AND (? OR (status = 0 AND expiration > MAX(?, verified_timestamp)))
            AND (
                ? != 1 OR owner_launcher_id IN (
                    SELECT asset_hash FROM owned_coins WHERE asset_kind IN (1, 2)
                )
            )
            AND (? != 2 OR handle IN (SELECT handle FROM contacts WHERE kind = 1))
        ORDER BY handle ASC
        LIMIT ? OFFSET ?
        ",
        registry_launcher_id,
        include_unverified,
        now,
        filter,
        filter,
        limit,
        offset
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|row| xchandle_row!(row))
    .collect::<Result<_>>()?;

    Ok((rows, total.try_into()?))
}

async fn owned_xchandle_names(
    conn: impl SqliteExecutor<'_>,
    registry_launcher_id: Bytes32,
) -> Result<Vec<String>> {
    let registry_launcher_id = registry_launcher_id.as_ref();

    Ok(sqlx::query_scalar!(
        "
        SELECT handle FROM xchandles
        WHERE registry_launcher_id = ? AND owner_launcher_id IN (
            SELECT asset_hash FROM owned_coins WHERE asset_kind IN (1, 2)
        )
        ",
        registry_launcher_id
    )
    .fetch_all(conn)
    .await?)
}

async fn owned_handle_nft_display_names(conn: impl SqliteExecutor<'_>) -> Result<Vec<String>> {
    Ok(sqlx::query_scalar!(
        "
        SELECT DISTINCT display_name AS \"display_name!\" FROM owned_nfts
        WHERE metadata_updater_kind = 1 AND display_name IS NOT NULL
        ORDER BY display_name ASC
        "
    )
    .fetch_all(conn)
    .await?)
}

async fn upsert_xchandle(conn: impl SqliteExecutor<'_>, row: &XchandleRow) -> Result<()> {
    let registry_launcher_id = row.registry_launcher_id.as_ref();
    let handle_hash = row.handle_hash.as_ref();
    let owner_launcher_id = row.owner_launcher_id.as_ref();
    let resolved_launcher_id = row.resolved_launcher_id.as_ref();
    let resolved_puzzle_hash = row.resolved_puzzle_hash.as_ref();
    let expiration: i64 = row.expiration.try_into()?;
    let slot_coin_id = row.slot_coin_id.as_ref();
    let slot_counter: i64 = row.slot_counter.try_into()?;
    let slot_left_handle_hash = row.slot_left_handle_hash.as_ref();
    let slot_right_handle_hash = row.slot_right_handle_hash.as_ref();
    let slot_parent_id = row.slot_parent_id.as_ref();
    let slot_parent_parent_id = row.slot_parent_parent_id.as_ref();
    let slot_parent_inner_puzzle_hash = row.slot_parent_inner_puzzle_hash.as_ref();
    let resolved_coin_id = row.resolved_coin_id.as_ref();
    let verified_timestamp: i64 = row.verified_timestamp.try_into()?;
    let last_verified_at: i64 = row.last_verified_at.try_into()?;
    let status = row.status as u8;
    let previous_resolved_launcher_id = row.previous_resolved_launcher_id.as_deref();
    let previous_resolved_puzzle_hash = row.previous_resolved_puzzle_hash.as_deref();
    let last_sent_puzzle_hash = row.last_sent_puzzle_hash.as_deref();

    sqlx::query!(
        "
        INSERT INTO xchandles (
            registry_launcher_id, handle, handle_hash, owner_launcher_id, resolved_launcher_id,
            resolved_puzzle_hash, expiration, slot_coin_id, slot_counter, slot_left_handle_hash,
            slot_right_handle_hash, slot_parent_id, slot_parent_parent_id,
            slot_parent_inner_puzzle_hash, slot_confirmation_height, resolved_coin_id,
            resolved_confirmation_height, indexed_height, verified_height, verified_timestamp,
            last_verified_at, status, status_reason, previous_resolved_launcher_id,
            previous_resolved_puzzle_hash, last_sent_puzzle_hash
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT (registry_launcher_id, handle) DO UPDATE SET
            handle_hash = excluded.handle_hash,
            owner_launcher_id = excluded.owner_launcher_id,
            resolved_launcher_id = excluded.resolved_launcher_id,
            resolved_puzzle_hash = excluded.resolved_puzzle_hash,
            expiration = excluded.expiration,
            slot_coin_id = excluded.slot_coin_id,
            slot_counter = excluded.slot_counter,
            slot_left_handle_hash = excluded.slot_left_handle_hash,
            slot_right_handle_hash = excluded.slot_right_handle_hash,
            slot_parent_id = excluded.slot_parent_id,
            slot_parent_parent_id = excluded.slot_parent_parent_id,
            slot_parent_inner_puzzle_hash = excluded.slot_parent_inner_puzzle_hash,
            slot_confirmation_height = excluded.slot_confirmation_height,
            resolved_coin_id = excluded.resolved_coin_id,
            resolved_confirmation_height = excluded.resolved_confirmation_height,
            indexed_height = excluded.indexed_height,
            verified_height = excluded.verified_height,
            verified_timestamp = excluded.verified_timestamp,
            last_verified_at = excluded.last_verified_at,
            status = excluded.status,
            status_reason = excluded.status_reason,
            previous_resolved_launcher_id = excluded.previous_resolved_launcher_id,
            previous_resolved_puzzle_hash = excluded.previous_resolved_puzzle_hash,
            last_sent_puzzle_hash = excluded.last_sent_puzzle_hash
        ",
        registry_launcher_id,
        row.handle,
        handle_hash,
        owner_launcher_id,
        resolved_launcher_id,
        resolved_puzzle_hash,
        expiration,
        slot_coin_id,
        slot_counter,
        slot_left_handle_hash,
        slot_right_handle_hash,
        slot_parent_id,
        slot_parent_parent_id,
        slot_parent_inner_puzzle_hash,
        row.slot_confirmation_height,
        resolved_coin_id,
        row.resolved_confirmation_height,
        row.indexed_height,
        row.verified_height,
        verified_timestamp,
        last_verified_at,
        status,
        row.status_reason,
        previous_resolved_launcher_id,
        previous_resolved_puzzle_hash,
        last_sent_puzzle_hash
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn set_xchandle_status(
    conn: impl SqliteExecutor<'_>,
    registry_launcher_id: Bytes32,
    handle: &str,
    status: XchandleStatus,
    reason: Option<&str>,
) -> Result<()> {
    let registry_launcher_id = registry_launcher_id.as_ref();
    let status = status as u8;

    sqlx::query!(
        "
        UPDATE xchandles SET status = ?, status_reason = ?
        WHERE registry_launcher_id = ? AND handle = ?
        ",
        status,
        reason,
        registry_launcher_id,
        handle
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn invalidate_xchandles_between(
    conn: impl SqliteExecutor<'_>,
    fork_height: u32,
    peak_height: u32,
) -> Result<()> {
    sqlx::query!(
        "
        UPDATE xchandles SET status = 2, status_reason = 'reorg'
        WHERE status = 0 AND (
            slot_confirmation_height BETWEEN ? + 1 AND ?
            OR resolved_confirmation_height BETWEEN ? + 1 AND ?
            OR verified_height BETWEEN ? + 1 AND ?
        )
        ",
        fork_height,
        peak_height,
        fork_height,
        peak_height,
        fork_height,
        peak_height
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn acknowledge_xchandle_change(
    conn: impl SqliteExecutor<'_>,
    registry_launcher_id: Bytes32,
    handle: &str,
) -> Result<()> {
    let registry_launcher_id = registry_launcher_id.as_ref();

    sqlx::query!(
        "
        UPDATE xchandles
        SET previous_resolved_launcher_id = NULL, previous_resolved_puzzle_hash = NULL
        WHERE registry_launcher_id = ? AND handle = ?
        ",
        registry_launcher_id,
        handle
    )
    .execute(conn)
    .await?;

    Ok(())
}

async fn set_xchandle_last_sent(
    conn: impl SqliteExecutor<'_>,
    registry_launcher_id: Bytes32,
    handle: &str,
    puzzle_hash: Bytes32,
) -> Result<()> {
    let registry_launcher_id = registry_launcher_id.as_ref();
    let puzzle_hash = puzzle_hash.as_ref();

    sqlx::query!(
        "
        UPDATE xchandles SET last_sent_puzzle_hash = ?
        WHERE registry_launcher_id = ? AND handle = ?
        ",
        puzzle_hash,
        registry_launcher_id,
        handle
    )
    .execute(conn)
    .await?;

    Ok(())
}
