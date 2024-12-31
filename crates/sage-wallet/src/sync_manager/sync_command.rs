use std::{net::IpAddr, sync::Arc};

use chia::protocol::{Bytes32, Message};
use chia_wallet_sdk::Network;

use crate::Wallet;

#[derive(Debug)]
pub enum SyncCommand {
    SwitchWallet {
        wallet: Option<Arc<Wallet>>,
    },
    SwitchNetwork {
        network_id: String,
        network: Network,
    },
    HandleMessage {
        ip: IpAddr,
        message: Message,
    },
    ConnectPeer {
        ip: IpAddr,
        trusted: bool,
    },
    SubscribeCoins {
        coin_ids: Vec<Bytes32>,
    },
    ConnectionClosed(IpAddr),
    SetTargetPeers(usize),
    SetDiscoverPeers(bool),
}
