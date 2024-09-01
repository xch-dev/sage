use chia::protocol::Bytes32;
use chia_wallet_sdk::Peer;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub struct PeerInfo {
    pub peer: Peer,
    pub claimed_peak: u32,
    pub header_hash: Bytes32,
    pub receive_message_task: JoinHandle<()>,
}

impl Drop for PeerInfo {
    fn drop(&mut self) {
        self.receive_message_task.abort();
    }
}
