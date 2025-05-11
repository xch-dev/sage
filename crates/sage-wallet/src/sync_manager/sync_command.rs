use std::{net::IpAddr, sync::Arc};

use chia::protocol::{Bytes32, Message};
use sage_config::Network;

use crate::Wallet;

#[derive(Debug)]
pub enum SyncCommand {
    SwitchWallet { wallet: Option<Arc<Wallet>> },
    SwitchNetwork(Network),
    HandleMessage { ip: IpAddr, message: Message },
    ConnectPeer { ip: IpAddr, user_managed: bool },
    SubscribeCoins { coin_ids: Vec<Bytes32> },
    SubscribePuzzles { puzzle_hashes: Vec<Bytes32> },
    ConnectionClosed(IpAddr),
    SetTargetPeers(usize),
    SetDiscoverPeers(bool),
}
