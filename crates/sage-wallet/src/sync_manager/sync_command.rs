use std::net::IpAddr;

use chia::protocol::{Bytes32, Message};

#[derive(Debug)]
pub enum SyncCommand {
    Reset { remove_peers: bool },
    HandleMessage { ip: IpAddr, message: Message },
    ConnectPeer { ip: IpAddr, user_managed: bool },
    SubscribeCoins { coin_ids: Vec<Bytes32> },
    SubscribePuzzles { puzzle_hashes: Vec<Bytes32> },
    ConnectionClosed(IpAddr),
}
