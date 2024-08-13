use sage_client::Peer;

use super::sync_status::SyncStatus;

#[derive(Debug)]
pub struct WalletPeer {
    pub peer: Peer,
    pub sync_status: SyncStatus,
}
