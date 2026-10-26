use std::{future::Future, time::Duration};

use chia_wallet_sdk::prelude::*;
use tracing::debug;

use crate::{WalletError, WalletPeer};

use super::{ChainTip, Handle, VerifiedHandle, XchandlesError, XchandlesIndex, verify_handle};

/// How many times to fetch a handle from the API when its slot has been spent in the
/// meantime, which happens whenever a neighboring handle is registered or changed.
pub const MAX_RESOLVE_ATTEMPTS: usize = 3;

/// How far below the peak to look for a transaction block, which carries a timestamp.
/// Transaction blocks are usually only a few blocks apart.
const MAX_TIP_LOOKBACK: u32 = 16;

/// The longest a complete lookup may take. Sage endpoints hold the Sage lock while they
/// run, so a slow API or peer must not stall the rest of the wallet for long. A lookup that
/// times out is treated as unavailable, and falls back to a recently verified mapping.
pub const RESOLVE_TIMEOUT: Duration = Duration::from_secs(15);

/// Finds the latest transaction block at or below the peak, whose timestamp is what
/// on-chain time locks such as handle expiration are checked against.
pub async fn chain_tip(peer: &WalletPeer, peak_height: u32) -> Result<ChainTip, WalletError> {
    for height in (peak_height.saturating_sub(MAX_TIP_LOOKBACK)..=peak_height).rev() {
        if let Some(timestamp) = peer.block_timestamp_if_transaction_block(height).await? {
            return Ok(ChainTip { height, timestamp });
        }
    }

    Err(WalletError::PeerMisbehaved)
}

/// Fetches a handle from the index and verifies it against the chain.
///
/// A spent slot causes the handle to be fetched again rather than rejected, since slots
/// are recreated by unrelated activity on neighboring handles.
///
/// The chain tip doesn't depend on the API, so it's fetched at the same time as the first
/// response.
pub async fn resolve_handle(
    index: &impl XchandlesIndex,
    peer: &WalletPeer,
    genesis_challenge: Bytes32,
    registry_launcher_id: Bytes32,
    handle: &Handle,
    tip: impl Future<Output = Result<ChainTip, XchandlesError>>,
) -> Result<VerifiedHandle, XchandlesError> {
    let (tip, mut response) = tokio::try_join!(tip, async {
        Ok::<_, XchandlesError>(index.handle(handle).await?)
    })?;

    let mut attempt = 1;

    loop {
        match verify_handle(
            peer,
            genesis_challenge,
            registry_launcher_id,
            handle,
            &response,
            tip,
        )
        .await
        {
            Err(XchandlesError::SlotSpent(coin_id)) if attempt < MAX_RESOLVE_ATTEMPTS => {
                debug!("Slot {coin_id} for {handle} was spent, fetching it again");
            }
            result => return result,
        }

        attempt += 1;
        response = index.handle(handle).await?;
    }
}
