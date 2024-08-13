use std::sync::Arc;

use sage_client::Peer;
use tracing::instrument;

use crate::{error::Result, wallet::Wallet};

#[instrument(skip(peer, wallet))]
pub async fn initial_sync(peer: Peer, wallet: Arc<Wallet>) -> Result<()> {
    wallet.sync_against(&peer, 500).await?;
    Ok(())
}
