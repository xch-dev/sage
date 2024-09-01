use std::net::IpAddr;

use chia::protocol::Bytes32;

#[derive(Debug, Clone)]
pub enum SyncEvent {
    Start(IpAddr),
    Stop,
    Subscribed,
    CoinUpdate(Vec<Bytes32>),
}
