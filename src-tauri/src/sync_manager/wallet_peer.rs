use chia_wallet_sdk::Peer;

use super::sync_status::SyncStatus;

#[derive(Debug)]
pub struct WalletPeer {
    pub peer: Peer,
    pub sync_status: SyncStatus,
}
