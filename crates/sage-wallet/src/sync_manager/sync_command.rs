use std::{net::IpAddr, sync::Arc};

use chia_wallet_sdk::{chia::protocol::Message, prelude::*};
use sage_config::Network;
use tokio::sync::mpsc;

use crate::Wallet;

#[derive(Debug)]
pub enum SyncCommand {
    SwitchWallet {
        wallet: Option<Arc<Wallet>>,
        delta_sync: bool,
    },
    SwitchNetwork(Network),
    HandleMessage {
        ip: IpAddr,
        message: Message,
    },
    ConnectPeer {
        ip: IpAddr,
        user_managed: bool,
    },
    AddPeer {
        peer: Peer,
        receiver: mpsc::Receiver<Message>,
    },
    SubscribeCoins {
        coin_ids: Vec<Bytes32>,
    },
    SubscribePuzzles {
        puzzle_hashes: Vec<Bytes32>,
    },
    ConnectionClosed(IpAddr),
    SetTargetPeers(usize),
    SetDiscoverPeers(bool),
}
